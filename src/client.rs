// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 [Freemorger]

use std::{io::{Read, Write}, os::unix::net::UnixStream};

use crate::manager::{IpcCommand, SYSRUNNER_IPC_FILEPATH};

pub fn send_command(command: IpcCommand) 
    -> Result<IpcCommand, Box<dyn std::error::Error>> {
    let mut sock = UnixStream::connect(SYSRUNNER_IPC_FILEPATH)?;
    
    let json = serde_json::to_string(&command)?;
    sock.write(json.as_bytes())?;

    let mut resp_json = String::new();
    let _             = sock.read_to_string(&mut resp_json)?;

    let resp: IpcCommand = serde_json::from_str(&resp_json)?;
    Ok(resp)
}
