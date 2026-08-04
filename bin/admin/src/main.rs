use anyhow::Result;
use bin_shared::get_finance_writer_service;
use rustic_admin::finance::load_tickers;
use rustic_boot::schema::update_rustic_platform;
use rustic_core::set_logger;
use rustic_economic::schema::update_economic_db;
use rustic_finance::schema::update_finance_db;
use std::{env, path::PathBuf};
use tracing::{error, info};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "admin")]
struct Cli {
    #[command(subcommand)]
    command: AdminCommands,
}

#[derive(Subcommand)]
enum AdminCommands {
    CheckEnv,
    LoadTickers {
        #[arg(short, long)]
        file: PathBuf,
    },
    UpdateEconomicSchema,
    UpdateFinanceSchema,
    UpdatePlatformSchema,
    UpdateTickersEOD {
        #[arg(short, long)]
        symbols: Option<String>,
        #[arg(short, long)]
        update: Option<bool>
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        "rustic_admin=info,rustic_finance=info,rustic-economic=info".to_string()
    });
    set_logger(filter);
    let cli = Cli::parse();

    // uri is the same for all
    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI envrionment variable not set");
    info!("Mongo uri: {}", mongo_uri);

    let rustic_platform_mongo_db = env::var("RUSTIC_PLATFORM_DB_NAME")
        .expect("RUSTIC_PLATFORM_DB_NAME envrionment variable not set");
    let rustic_economic_mongo_db = env::var("RUSTIC_ECONOMIC_DB_NAME")
        .expect("RUSTIC_ECONOMIC_DB_NAME envrionment variable not set");
    let rustic_finance_mongo_db = env::var("RUSTIC_FINANCE_DB_NAME")
        .expect("RUSTIC_FINANCE_DB_NAME envrionment variable not set");

    // cli commands
    match cli.command {
        AdminCommands::CheckEnv => {}
        AdminCommands::LoadTickers { file } => {
            info!("Load Tickers PipeLine started...");
            load_tickers(&mongo_uri, file).await?;
            info!("Load Tickers PipeLine done.");
        }
        AdminCommands::UpdateEconomicSchema => {
            update_economic_db(&mongo_uri, &rustic_economic_mongo_db).await?;
        }
        AdminCommands::UpdateFinanceSchema => {
            update_finance_db(&mongo_uri, &rustic_finance_mongo_db).await?;
        }
        AdminCommands::UpdatePlatformSchema => {
            update_rustic_platform(&mongo_uri, &rustic_platform_mongo_db).await?;
        }
        AdminCommands::UpdateTickersEOD { symbols, update } => {
            let symbols_str = symbols.as_deref().unwrap_or("");
            let update = update.is_none();
            info!("Tickers EOD PipeLine started...");

            let service = get_finance_writer_service(&mongo_uri).await?;
            match service.update_eod_tickers(symbols_str, update).await {
                Ok(_) => info!("Tickers EOD update completed successfully."),
                Err(e) => error!("Tickers EOD update failed: {:?}", e),
            }
        }
    }

    Ok(())
}
