// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 [Freemorger]

use clap::{Parser, Subcommand, Args, ValueEnum};

use crate::{client::send_command, manager::IpcCommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run sysrunner as init system 
    Serve,

    /// Command to test whether sysrunner is working properly. 
    /// If it is, it will respond with "Pong".
    Ping, 

    /// Start unstarted service(s)
    Start {
        #[arg(help = "Service name (or many, if you want to start multiple at \
            once, space separated")]
        service_names: Vec<String>,
    },

    /// Stop active service(s)
    Stop {
        #[arg(help = "Service name (or many, if you want to stop multiple at \
            once, space separated)")]
        service_names: Vec<String>
    },

    Restart,
    
    Enable,
    
    Disable,

    /// Get some info on current state of service 
    Status {
        #[arg(help = "Service name")]
        service_name: String,
    },
    /// Get PID (Process Identifier) of service (if its running)
    Pid {
        #[arg(help = "Service name")]
        service_name: String, 
    },

    /// Print a table with info about every active service
    Ps, 

    /// Shutdown sysrunner 
    Shutdown {},
}

impl CliArgs {
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let resp = match &self.command {
            Some(Commands::Serve) => {
                return crate::serve();
            }
            Some(Commands::Start { service_names }) => {
                for s in service_names.iter().cloned() {
                    send_command(IpcCommand::Start(s))?
                        .print_if_resp();
                }
                return Ok(());
            }
            Some(Commands::Stop { service_names }) => {
                for s in service_names.iter().cloned() {
                    send_command(IpcCommand::Stop(s))?
                        .print_if_resp();
                }
                return Ok(());
            }
            Some(Commands::Ping) => {
                send_command(IpcCommand::Ping)?
            }
            Some(Commands::Pid { service_name }) => { 
                send_command(IpcCommand::Pid(service_name.clone()))?
            }
            Some(Commands::Status { service_name }) => { 
                send_command(IpcCommand::Status(service_name.clone()))?
            }
            Some(Commands::Ps) => { 
                send_command(IpcCommand::Ps)?
            }
            Some(Commands::Shutdown {}) => { 
                send_command(IpcCommand::Shutdown())?
            }
            _ => {
                eprintln!("Please, specify command or try fencyc --help.");
                return Ok(());
            }
        };
        

        match resp {
            IpcCommand::Response(s) => println!("{}", s),
            _ => {}
        }
        return Ok(());
    }
}
