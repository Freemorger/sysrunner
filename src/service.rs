use std::task::Poll::Pending;

use crate::cfg::ServiceData;

#[derive(Debug, Clone)]
pub struct Service {
    pub data:   ServiceData,
    pub state:  ServiceState,
}

#[derive(Debug, Clone, Copy)]
pub enum ServiceState {
    Pending,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed
}

impl ServiceState {
    pub fn is_active(self) -> bool {
        match self {
            Self::Pending | Self::Failed | Self::Stopped => false,
            _ => true,
        }
    }
}
