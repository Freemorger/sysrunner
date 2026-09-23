// sysrunner - A lightweight init system for Unix-like systems
// Copyright (C) 2026 [Freemorger]
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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

    cli.execute()?;

    return Ok(());
}

pub fn serve() -> Result<(), Box<dyn std::error::Error>> {
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
