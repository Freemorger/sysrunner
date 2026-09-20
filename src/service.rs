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

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending  => write!(f, "Pending"),
            Self::Starting => write!(f, "Starting"),
            Self::Running  => write!(f, "Running"),
            Self::Stopping => write!(f, "Stopping"),
            Self::Stopped  => write!(f, "Stopped"),
            Self::Exited   => write!(f, "Exited"),
            Self::Failed(c) if c.is_some() => 
                write!(f, "Failed (status code {})", c.unwrap()),
            Self::Failed(_) => write!(f, "Failed"),
        }
    }
}
