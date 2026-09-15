use std::{process::Child, sync::{Arc, Mutex}, task::Poll::Pending};

use crate::cfg::ServiceData;

#[derive(Debug, Clone)]
pub struct Service {
    pub data:   ServiceData,
    pub state:  ServiceState,
    pub proc:   Option<Arc<Mutex<Child>>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ServiceState {
    Pending,
    Starting,
    Running,
    Stopping,
    Stopped,
    Exited,
    Failed(Option<i32>)
}

impl ServiceState {
    pub fn is_active(self) -> bool {
        match self {
            Self::Pending | Self::Failed(_) | Self::Stopped | Self::Exited 
                => false,
            _ => true,
        }
    }
}
