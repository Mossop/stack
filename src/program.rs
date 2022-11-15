use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

use crate::{
    commands::{normal_order_run, reverse_order_run, single_stack},
    config::Config,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, subcommand_precedence_over_arg(true))]
pub struct Program {
    /// The location of the stacks config file. By default looks for stacks.yml
    /// in the current and parent directories.
    #[arg(short, long, env = "STACKS_FILE")]
    pub file: Option<String>,

    #[clap(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    #[command(flatten)]
    pub globals: Globals,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args, Debug)]
pub struct Globals {
    /// A comma separated list of stacks to apply the command to. Applies to all
    /// stacks if not present.
    pub stacks: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build or rebuild services
    Build {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Copy files/folders between a service container and the local filesystem
    Cp {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Creates containers for a service.
    Create {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Stop and remove containers, networks
    Down {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Receive real time events from containers.
    Events {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Execute a command in a running container.
    Exec {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// List images used by the created containers
    Images {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Force stop service containers.
    Kill {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// View output from containers
    Logs {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// List running compose projects
    Ls {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Pause services
    Pause {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Print the public port for a port binding.
    Port {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// List containers
    Ps {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Pull service images
    Pull {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Push service images
    Push {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Restart service containers
    Restart {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Removes stopped service containers
    Rm {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Run a one-off command on a service.
    Run {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Start services
    Start {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Stop services
    Stop {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Display the running processes
    Top {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Unpause services
    Unpause {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
    /// Create and start containers
    Up {
        /// Arguments to pass through to docker compose
        args: Vec<String>,
    },
}

impl Commands {
    pub fn run(&self, globals: &Globals, config: &Config) -> Result<(), String> {
        match self {
            Commands::Build { args } => normal_order_run("build", globals, config, args),
            Commands::Cp { args } => single_stack("cp", globals, config, args),
            Commands::Create { args } => normal_order_run("create", globals, config, args),
            Commands::Down { args } => reverse_order_run("down", globals, config, args),
            Commands::Events { args } => single_stack("events", globals, config, args),
            Commands::Exec { args } => single_stack("exec", globals, config, args),
            Commands::Images { args } => normal_order_run("images", globals, config, args),
            Commands::Kill { args } => reverse_order_run("kill", globals, config, args),
            Commands::Logs { args } => single_stack("logs", globals, config, args),
            Commands::Ls { args } => normal_order_run("ls", globals, config, args),
            Commands::Pause { args } => reverse_order_run("pause", globals, config, args),
            Commands::Port { args } => single_stack("port", globals, config, args),
            Commands::Ps { args } => normal_order_run("ps", globals, config, args),
            Commands::Pull { args } => normal_order_run("pull", globals, config, args),
            Commands::Push { args } => normal_order_run("push", globals, config, args),
            Commands::Restart { args } => {
                reverse_order_run("down", globals, config, args)?;
                normal_order_run("up", globals, config, &vec!["--wait".to_string()])
            }
            Commands::Rm { args } => reverse_order_run("rm", globals, config, args),
            Commands::Run { args } => single_stack("run", globals, config, args),
            Commands::Start { args } => single_stack("start", globals, config, args),
            Commands::Stop { args } => single_stack("stop", globals, config, args),
            Commands::Top { args } => single_stack("top", globals, config, args),
            Commands::Unpause { args } => normal_order_run("unpause", globals, config, args),
            Commands::Up { args } => {
                let mut args = args.clone();
                args.insert(0, "--wait".to_string());
                normal_order_run("up", globals, config, &args)
            }
        }
    }
}
