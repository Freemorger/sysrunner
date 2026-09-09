use std::{collections::HashMap, fs, process::Command};

use crate::cfg::{ServicesList};
use crate::service::{Service, ServiceState};

mod cfg;
mod service;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg_str = fs::read_to_string("services.toml")?;

    let sd_list: ServicesList = toml::from_str(&cfg_str)?;

    let mut services: HashMap<String, Service> = sd_list.services
        .iter().map(|sd| {
            let c = sd.clone();
            (c.0.to_owned(), c.1.clone().to_service())
    }).collect(); 

    let mut unchanged_iters: usize = 0;
    while !services.iter().all(|s| {
        s.1.state.is_active()
    }) {
        if unchanged_iters > 3 {
            eprintln!(
                "ERROR: Seems like there's a deadlock in services \
                dependencies which causes startup to fail.\n\
                Unstarted services: {:#?}",
                services.values()
                    .filter(|s| !s.state.is_active())
                    .cloned()
                    .collect::<Vec<Service>>()
            );
            break;
        }

        let mut to_activate: Vec<String> = Vec::new();

        for (snm, srvc) in &services {
            if srvc.state.is_active() {
                continue;
            }

            let mut can_proceed = true;
            for depnm in &srvc.data.depends {
                let dep = match services.get(depnm) {
                    Some(v) => v,
                    None => {
                        eprintln!("Unknown service in {}'s dependency list: {}",
                            snm, depnm
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

            if let Some(program) = parts.next() {
                let output = Command::new(program)
                    .args(parts) 
                    .output()
                    .expect("Failed to execute command");

                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            to_activate.push(snm.clone());
        }

        if to_activate.len() == 0 {
            unchanged_iters += 1;
        } else {
            unchanged_iters = 0;
        }

        for snm in to_activate {
            let s = services.get_mut(&snm).unwrap();
            s.state  = ServiceState::Running;
        }
        
    }

    return Ok(());
}
