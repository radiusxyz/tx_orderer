mod config_option;
mod config_path;
mod error;

pub use config_option::ConfigOption;
pub use config_path::ConfigPath;
use serde::{Deserialize, Serialize};

pub(crate) const DEFAULT_HOME_PATH: &str = ".radius";

#[derive(Debug, Deserialize, Parser, Serialize)]
#[serde(crate = "primitives::serde")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn init() -> Self {
        Cli::parse()
    }
}

#[derive(Subcommand, Debug, Deserialize, Serialize)]
#[serde(crate = "primitives::serde")]
pub enum Commands {
    /// Initializes a node
    Init {
        #[clap(flatten)]
        config_path: Box<ConfigPath>,
    },

    /// Starts the node
    Start {
        #[clap(flatten)]
        config_option: Box<ConfigOption>,
    },
}
