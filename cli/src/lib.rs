use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tx_orderer_node::{self as node, types::*};
use tx_orderer_primitives::error::Error;
use tx_orderer_shared::logger::PanicLog;

#[derive(Debug, Deserialize, Parser, Serialize)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum Commands {
    /// Initializes a node
    Init {
        #[clap(flatten)]
        config_path: ConfigPath,
    },
    /// Starts the node
    Start {
        #[clap(flatten)]
        config_option: ConfigOption,
    },
}

pub async fn run() -> Result<(), Error> {
    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("{:?}", PanicLog::from(panic_info));
    }));

    let cli = Cli::parse();
    match cli.command {
        Commands::Init { config_path } => {
            node::init_node(&config_path).await?;
        }
        Commands::Start { mut config_option } => {
            node::start_node(&mut config_option).await?;
        }
    }

    Ok(())
}