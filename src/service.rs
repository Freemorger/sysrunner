// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 [Freemorger]

use std::{process::Child, sync::{Arc, Mutex}, task::Poll::Pending};

use crate::cfg::ServiceData;

#[derive(Debug, Clone)]
pub struct Service {
    pub data:   ServiceData,
    pub state:  ServiceState,
    pub proc:   Option<Arc<Mutex<Child>>>,

    pub reasn:  StartReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    None, // not processed for startup yet 
    Pending, // to be executed
    Starting,
    Running,
    Stopping,
    Stopped,
    Exited,
    Failed(Option<i32>),
    SpawnFailure,
}

impl ServiceState {
    pub fn is_active(self) -> bool {
        match self {
            Self::Failed(_) | Self::Stopped | Self::Exited | Self::None |
                Self::SpawnFailure | Self::Pending
                => false,
            _ => true,
        }
    }

    pub fn is_pending(self) -> bool {
        match self {
            Self::Pending => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None     => write!(f, "Not processed"),
            Self::Pending  => write!(f, "Pending"),
            Self::Starting => write!(f, "Starting"),
            Self::Running  => write!(f, "Running"),
            Self::Stopping => write!(f, "Stopping"),
            Self::Stopped  => write!(f, "Stopped"),
            Self::Exited   => write!(f, "Exited"),
            Self::Failed(c) if c.is_some() => 
                write!(f, "Failed (status code {})", c.unwrap()),
            Self::Failed(_) => write!(f, "Failed"),
            Self::SpawnFailure => write!(f, "Process spawn failed"),
        }
    }
}

/// Result of starting a service.
#[derive(Debug)]
pub enum StartResult {
    Success,
    AlreadyActive,
    DepFault,
    SpawnFault,
}

#[derive(Debug, Clone, Copy)]
pub enum StartReason {
    None, // not started 
    Enabled,
    AsDep,
    OnDemand,
}
