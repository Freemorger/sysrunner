use clap::{Parser, Subcommand, Args, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run sysrunner as init system 
    Serve,

    /// Command to test whether sysrunner is working properly. 
    /// If it is, it will respond with "Pong".
    Ping, 

    Start,
    Stop,
    Restart,
    Enable,
    Disable,

    /// Get some info on current state of service 
    Status {
        #[arg(help = "Service name")]
        service_name: String,
    },
    /// Get PID (Process Identifier) of service (if its running)
    Pid {
        #[arg(help = "Service name")]
        service_name: String, 
    },

    /// Print a table with info about every active service
    Ps, 
}
