use std::{collections::HashMap, fs, io, path::Path, process::{Child, Command}, sync::{Arc, Mutex}};

use walkdir::WalkDir;

use crate::{cfg::ServicesList, log::{LogLevel, log}, service::{Service, ServiceState}};

#[derive(Debug, Default)]
pub struct ServiceManager {
    services: HashMap<String, Service>,
}

impl ServiceManager {
    pub fn setup_from_cfg(&mut self, dir: &str) 
        -> Result<(), Box<dyn std::error::Error>> {
        
        let mut files = Vec::new();

        for entry in WalkDir::new(dir) {
            match entry {
                Ok(de) => {
                    let path = de.path().to_owned();

                    if path.is_file() && path.extension() == Some("toml".as_ref()) {
                        files.push(path);
                    }
                }
                Err(e) => {
                    log(LogLevel::Error, &format!(
                        "Error reading directory {}: {}",
                        dir, e
                    ));
                }
            }
        }

        let mut all_services = HashMap::new();
        for cfg_fname in &files {
            let cfg_str = fs::read_to_string(cfg_fname)?;

            let sd_list: ServicesList = toml::from_str(&cfg_str)?;

            let services: HashMap<String, Service> = sd_list.services
                .iter().map(|sd| {
                    let c = sd.clone();
                    (c.0.to_owned(), c.1.clone().to_service())
            }).collect(); 
            all_services.extend(services);
        }
        
        self.services = all_services;

        Ok(()) 
    }

    pub fn startup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while !self.services.iter().all(|s| {
            s.1.state.is_active()
        }) { 
            let mut to_activate: HashMap<String, Option<Child>> = HashMap::new();

            for (snm, srvc) in &self.services {
                if srvc.state.is_active() {
                    continue;
                }

                let mut can_proceed = true;
                for depnm in &srvc.data.depends {
                    let dep = match self.services.get(depnm) {
                        Some(v) => v,
                        None => {
                            log(
                                LogLevel::Error, &format!( 
                                    "Unknown service in {}'s dependency list: {}",
                                    snm, depnm
                                )
                            );
                            can_proceed = false;
                            break;
                        }
                    };
                    if !dep.state.is_active() {
                        can_proceed = false;
                    }
                }
                if !can_proceed {
                    continue;
                }

                let mut parts = srvc.data.command.split_whitespace();

                let mut child: Option<Child> = None;
                if let Some(program) = parts.next() {
                    child = match Command::new(program)
                        .args(parts)
                        .spawn()
                    {
                        Ok(c) => Some(c),
                        Err(e) => {
                            log(LogLevel::Error, &format!(
                                "Failed to spawn {}: {}", program, e)
                            );
                            None
                        }
                    };
                }
                to_activate.insert(snm.clone(), child);
            }

            if to_activate.len() == 0 {
                log(
                    LogLevel::Error, &format!(
                        "ERROR: Seems like there's a deadlock in services \
                        dependencies which causes startup to fail.\n\
                        Unstarted services: {:#?}",
                        self.services.values()
                            .filter(|s| !s.state.is_active())
                            .cloned()
                            .collect::<Vec<Service>>()
                    )
                );
                break;
            }

            for (snm, chld) in to_activate {
                let s   = self.services.get_mut(&snm).unwrap();
                s.state = ServiceState::Starting;
                
                if let Some(child) = chld {
                    s.proc = Some(Arc::new(Mutex::new(child)));
                };
            }
            
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.services.iter().any(|(_, s)| s.state.is_active() ) {
            let mut updated: HashMap<String, Service> = HashMap::new();
            for (snm, s) in &self.services {
                if let Some(ch) = &s.proc {
                    let mut guard = ch.lock().unwrap();
                    let mut cl = s.clone();

                    match guard.try_wait()? {
                        Some(status) => { 
                            if status.success() {
                                cl.state = ServiceState::Exited;
                            } else {
                                cl.state = ServiceState::Failed(status.code());
                            }
                        }
                        None => {
                            cl.state = ServiceState::Running; 
                        }
                    }
                    updated.insert(snm.clone(), cl);
                }; 
            }

            for (snm, s) in updated {
                if self.services.get(&snm).is_some() {
                    self.services.insert(snm.clone(), s);
                }
            }
        }

        Ok(())
    }
}
