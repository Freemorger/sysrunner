use std::{collections::HashMap, fs, io::{self, ErrorKind, Read, Write}, net::Shutdown, os::unix::net::{UnixListener, UnixStream}, path::Path, process::{self, Child, Command}, sync::{Arc, Mutex}, thread::sleep, time::Duration};

use walkdir::WalkDir;

use crate::{cfg::ServicesList, log::{LogLevel, log}, service::{Service, ServiceState}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcCommand {
    None,
    
    Response(String),

    Ping,
    
    Start(String),
    Restart(String),
    Enable(String),
    Disable(String),

    Status(String),
    Pid(String),
    
    Ps,
    
    Shutdown,
}

#[derive(Debug)]
pub struct ServiceManager {
    services:        HashMap<String, Service>,
    ipc_socket:      UnixListener,
    pending_clients: Vec<UnixStream>
}

pub const SYSRUNNER_IPC_FILEPATH: &str = "/tmp/sysrunner_ipc.sock";

impl ServiceManager {
    /// Creates and returns default unix socket for allat 
    pub fn def_sock() -> Result<UnixListener, Box<dyn std::error::Error>> {
        let listnr = UnixListener::bind(SYSRUNNER_IPC_FILEPATH)?;
        listnr.set_nonblocking(true)?;
        Ok(listnr)
    }

    pub fn new(ipc_socket: UnixListener) -> ServiceManager {
        return ServiceManager { 
            services: HashMap::new(), 
            ipc_socket,
            pending_clients: Vec::new()
        }
    }

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

            self.ipc_tick();
            sleep(Duration::new(0, 20 * 1_000_000)); // 20 ms
        }

        Ok(())
    }

    fn ipc_tick(&mut self) {
        loop {
            match self.ipc_socket.accept() {
                Ok((stream, _addr)) => {
                    if let Err(e) = stream.set_nonblocking(true) {
                        log(LogLevel::Error, &format!("Failed to set stream to non-blocking: {}", e));
                        continue;
                    }
                    self.pending_clients.push(stream);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) => {
                    log(LogLevel::Error, &format!("IPC client accept error: {}", e));
                    break;
                }
            }
        }    

        let clients_to_process = std::mem::take(&mut self.pending_clients);
        let mut still_pending = Vec::new();

        for mut stream in clients_to_process {
            let mut de = serde_json::Deserializer::from_reader(&mut stream);
            
            match IpcCommand::deserialize(&mut de) {
                Ok(ipc_command) => {
                    let resp = self.process_command(&ipc_command);
                    match resp {
                        Ok(ic) => {
                            let resp_json = serde_json::to_string(&ic)
                                .unwrap_or(
                                    "Serialization server-side error occured"
                                        .to_owned()
                                ); 
                            
                            if let Err(e) = stream.write_all(resp_json.as_bytes()) {
                                log(LogLevel::Error, &format!(
                                    "Error while writing response to client: {}", e
                                ));
                            }
                            
                            let _ = stream.shutdown(Shutdown::Write);
                        }
                        Err(e) => {
                            log(LogLevel::Error, &format!(
                                "Error while processing IPC command {:?}: {}", 
                                ipc_command, e
                            ));
                        }
                    };    
                }
                Err(ref e) if e.is_io() => {
                    if let Some(kind) = e.io_error_kind() {
                        if kind == ErrorKind::WouldBlock {
                            still_pending.push(stream);
                        } else {
                            log(LogLevel::Error, &format!(
                                "Failed to read from IPC stream: {}", e
                            ));
                        }
                    } else {
                        log(LogLevel::Error, &format!(
                            "Serde reported I/O error but no kind found: {}", e
                        ));
                    }
                }

                Err(e) => {
                    log(LogLevel::Error, &format!(
                        "Failed to deserialize IPC payload: {}", e
                    ));
                }
            }
        }

        self.pending_clients = still_pending;
    }


    fn process_command(&mut self, command: &IpcCommand) -> 
        Result<IpcCommand, Box<dyn std::error::Error>> {
        match command {
            IpcCommand::Ping => { 
                return Ok(IpcCommand::Response("Pong".to_owned()));
            } 
            IpcCommand::Start(service_name) => {
                todo!()
            }
            IpcCommand::Ps => {
                let mut res = format!(
                    "{:<20} {:<10} {:<10} {:<15}\n", 
                    "SERVICE", "PID", "STATE", "DEPENDS"
                );

                for (name, service) in &self.services {
                    if !service.state.is_active() {
                        continue;
                    }

                    let pid_str = match &service.proc {
                        Some(p) => match p.lock() {
                            Ok(guard) => guard.id().to_string(),
                            Err(_) => "Poisoned".to_owned(), 
                        },
                        None => "-".to_owned(),
                    };

                    let deps_str = service.data.depends.join(", ");

                    let row = format!(
                        "{:<20} {:<10} {:<10} {:<15}\n",
                        name, pid_str, service.state, deps_str
                    );
                    res.push_str(&row);
                }

                return Ok(IpcCommand::Response(res));
            }
            IpcCommand::Status(service_name) => {
                let service = match self.services.get(service_name) {
                    Some(s) => s,
                    None => {
                        return Ok(IpcCommand::Response(format!(
                            "No such service {}", service_name
                        )));
                    }
                };

                let pid: Option<u32> = match &service.proc {
                    Some(p) => {
                        let guard = p.lock().unwrap();
                    
                        Some(guard.id())
                    }
                    None => None,
                };

                let res = format!(
                    "Service {}\n\
                    State: {}\n\
                    PId: {}\n\
                    Depends on: {:#?}",
                    service_name, service.state,
                    pid.map_or("(unavailable)".to_owned(), |p| {p.to_string()}), 
                    service.data.depends
                );

                return Ok(IpcCommand::Response(res));
            }
            IpcCommand::Pid(service_name) => {
                let service = match self.services.get(service_name) {
                    Some(s) => s,
                    None => {
                        return Ok(IpcCommand::Response(format!(
                            "No such service {}", service_name
                        )));
                    }
                };

                if !service.state.is_active() {
                    return Ok(IpcCommand::Response(format!(
                        "Service {} is not up (it is {})",
                        service_name, service.state
                    )));
                }

                if let Some(p) = &service.proc {
                    let guard = p.lock().unwrap();
                    
                    return Ok(IpcCommand::Response(format!(
                        "{}", guard.id()
                    ))); 
                } else {
                    return Ok(IpcCommand::Response(
                        "Internal error: pid of service is unknown.".to_owned()
                    ));
                };
            }
            other => {
                log(LogLevel::Warn, &format!(
                    "Client tried to send unknown command: {:#?}", other
                ));
                return Ok(IpcCommand::Response("Unknown command".to_owned()));
            }
        }
    }
}

impl Drop for ServiceManager {
    fn drop(&mut self) {
        match self.ipc_socket.local_addr() {
            Ok(addr) if addr.as_pathname().is_some() => {
                match fs::remove_file(addr.as_pathname().unwrap()) {
                    Ok(_) => {}
                    Err(e) => {
                        log(LogLevel::Error, &format!(
                            "Failed to delete IPC socket file: {}",
                            e
                        ));
                    }
                }
            }
            Ok(_) => {
                log(LogLevel::Error, 
                    "Failed to retrieve pathname for address of IPC socket."
                );
            }
            Err(e) => {
                log(LogLevel::Error, &format!(
                    "Failed to retrieve local address for IPC socket: {}",
                    e
                ));
            }
        }
    }
}
