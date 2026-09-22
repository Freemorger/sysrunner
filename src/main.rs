use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time;
use std::{collections::HashMap, fs, process::Command};

use clap::Parser;

use crate::cfg::{ServicesList};
use crate::cli::Commands;
use crate::client::send_command;
use crate::log::{LogLevel, log};
use crate::manager::{IpcCommand, SYSRUNNER_IPC_FILEPATH, ServiceManager};
use crate::service::{Service, ServiceState};

mod cfg;
mod service;
mod manager;
mod log;
mod cli;
mod client;
mod deps;

fn main() -> Result<(), Box<dyn std::error::Error>> { 
    let cli = cli::CliArgs::parse();

    let resp = match cli.command {
        Some(Commands::Serve) => {
            return serve();
        }
        Some(Commands::Start { service_name }) => {
            send_command(IpcCommand::Start(service_name))?
        }
        Some(Commands::Ping) => {
            send_command(IpcCommand::Ping)?
        }
        Some(Commands::Pid { service_name }) => { 
            send_command(IpcCommand::Pid(service_name))?
        }
        Some(Commands::Status { service_name }) => { 
            send_command(IpcCommand::Status(service_name))?
        }
        Some(Commands::Ps) => { 
            send_command(IpcCommand::Ps)?
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

fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let ipc      = ServiceManager::def_sock()?;

    log(LogLevel::Info, &format!(
        "IPC Socket listens at {}.", 
        SYSRUNNER_IPC_FILEPATH
    ));

    let mut mngr = ServiceManager::new(ipc);
    
    // TODO: change paths to /etc/ or whatever in UDS
    mngr.setup_from_cfg("cfg/")?;
    log(LogLevel::Info, "Config processed.");

    mngr.startup()?;
    log(LogLevel::Info, "Startup services initialized.");

    mngr.run()?;

    return Ok(());
}
