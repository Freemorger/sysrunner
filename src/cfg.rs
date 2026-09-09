use std::collections::HashMap;

use serde::Deserialize;

use crate::service::{Service, ServiceState};

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceData {
    pub command: String,
    pub depends: Vec<String>,
}

impl ServiceData {
    pub fn to_service(self) -> Service {
        return Service { 
            data: self, 
            state: ServiceState::Pending
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ServicesList {
    pub services: HashMap<String, ServiceData>
}


