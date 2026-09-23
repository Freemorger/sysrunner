// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 [Freemorger]

use std::collections::HashMap;

use serde::Deserialize;

use crate::service::{Service, ServiceState, StartReason};

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceData {
    pub name:    String,
    pub enabled: bool,
    pub command: String,
    pub depends: Vec<String>,
}

impl ServiceData {
    pub fn to_service(self) -> Service {
        return Service { 
            data:  self,
            state: ServiceState::None,
            proc:  None,
            reasn: StartReason::None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ServicesList {
    pub services: HashMap<String, ServiceData>
}


