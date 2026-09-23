// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 [Freemorger]

use std::{fs::{File, OpenOptions}, sync::{Mutex, OnceLock}, io::{Write}};

use chrono::Local;
use colored::Colorize;

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();
const DBG_LOG_FILE: &str               = "sysrunner.log"; // for debug 
const DEF_LOG_FILE: &str               = "/var/log/sysrunner.log";

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Critical
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now  = Local::now();
        let time = now.format("%m-%d %H:%M:%S");

        match self {
            Self::Info     => write!(f, "[{}, {}]", time.to_string(), "Info"),
            Self::Warn     => write!(f, "[{}, {}]", time.to_string(), "Warning".bright_yellow()),
            Self::Error    => write!(f, "[{}, {}]", time.to_string(), "ERROR".bright_red()),
            Self::Critical => write!(f, "[{}, {}]", time.to_string(), "CRITICAL".red()),
        }
    }
}

impl LogLevel {
    pub fn is_error(self) -> bool {
        match self {
            Self::Error | Self::Critical => {
                true
            }
            _ => false
        }
    }
}

pub fn log(lvl: LogLevel, text: &str) {
    let log_path = if cfg!(debug_assertions) {
        DBG_LOG_FILE
    } else {
        DEF_LOG_FILE
    };


    let mutex = LOG_FILE.get_or_init(|| {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Failed to open log file");
        Mutex::new(file)
    });

    if let Ok(mut file) = mutex.lock() {
        if lvl.is_error() {
            writeln!(file, "{} {}", lvl, text).unwrap();
        } else {
            writeln!(file, "{} {}", lvl, text).unwrap();
        }
    }
}
