use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time;
use std::{collections::HashMap, fs, process::Command};

use crate::cfg::{ServicesList};
use crate::log::{LogLevel, log};
use crate::manager::ServiceManager;
use crate::service::{Service, ServiceState};

mod cfg;
mod service;
mod manager;
mod log;

fn main() -> Result<(), Box<dyn std::error::Error>> { 
    let mut mngr = ServiceManager::default();
  
    mngr.setup_from_cfg("services.toml")?;
    log(LogLevel::Info, "Config processed.");

    mngr.startup()?;
    log(LogLevel::Info, "Startup services initialized.");

    mngr.run()?;

    return Ok(());
}
