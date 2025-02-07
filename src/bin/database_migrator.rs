use std::{env, path::PathBuf};

use clap::{Parser, Subcommand};
use radius_sdk::kvstore::KvStoreBuilder;
use sequencer::{
    error::Error, logger::PanicLog, migration::version_0_0_2::migrate_rollup_data, types::*,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Parser, Serialize)]
#[command(author, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn init() -> Self {
        Cli::parse()
    }
}

#[derive(Debug, Deserialize, Parser, Serialize)]

struct ConfigOption {
    #[doc = "Set the data path to load from"]
    #[clap(long = "data_path", short = 'd')]
    data_path: Option<PathBuf>,

    #[doc = "Set the migration version "]
    #[clap(long = "migration_version", short = 'm')]
    #[clap(default_value = REQURIED_DATABASE_VERSION)]
    migration_version: String,
}

#[derive(Subcommand, Debug, Deserialize, Serialize)]

pub enum Commands {
    /// Starts the node
    Migrate {
        #[clap(flatten)]
        config_option: ConfigOption,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("{:?}", PanicLog::from(panic_info));
    }));
    tracing_subscriber::fmt().init();

    let cli = Cli::init();
    match cli.command {
        Commands::Migrate { mut config_option } => {
            let data_path = config_option.data_path.take().unwrap_or_else(|| {
                PathBuf::from(env::var("HOME").unwrap()).join(DEFAULT_DATA_PATH)
            });
            let database_path = data_path.join(DATABASE_DIR_NAME);
            let migration_version = config_option.migration_version;

            tracing::info!("Database path: {:?}", database_path);
            tracing::info!("Migration version: {:?}", migration_version);

            let kv_store = KvStoreBuilder::default()
                .build(database_path)
                .map_err(Error::Database)?;

            match migration_version.as_str() {
                "v0.0.2" => {
                    migrate_rollup_data(kv_store).await?;
                }
                _ => {
                    tracing::error!("Invalid migration version: {}", migration_version);
                }
            }
        }
    }

    Ok(())
}
