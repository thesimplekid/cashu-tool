use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use cdk::wallet::Wallet;
use cdk::Mnemonic;
use cdk_redb::RedbWalletDatabase;
use clap::{Parser, Subcommand};

mod sub_commands;
mod types;

/// Simple CLI application to interact with cashu
#[derive(Parser)]
#[command(name = "cashu-tool")]
#[command(author = "thesimplekid <tsk@thesimplekid.com>")]
#[command(version = "0.1")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

const DEFAULT_DB_PATH: &str = "./cashu_tool.redb";
const DEFAULT_SEED_PATH: &str = "./seed.txt";

#[derive(Subcommand)]
enum Commands {
    DecodeToken(sub_commands::decode_token::DecodeTokenSubCommand),
    Melt(sub_commands::melt::MeltSubCommand),
    Receive(sub_commands::receive::ReceiveSubCommand),
    CreateToken(sub_commands::create_token::CreateTokenSubCommand),
    CheckSpendable,
    MintInfo(sub_commands::mint_info::MintInfoSubcommand),
    Mint(sub_commands::mint::MintSubCommand),
    Restore(sub_commands::restore::RestoreSubCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Parse input
    let args: Cli = Cli::parse();

    let db_path = args.db_path.clone().unwrap_or(DEFAULT_DB_PATH.to_string());

    let localstore = RedbWalletDatabase::new(&db_path)?;
    let mnemonic = match fs::metadata(DEFAULT_SEED_PATH) {
        Ok(_) => {
            let contents = fs::read_to_string(DEFAULT_SEED_PATH)?;
            Some(Mnemonic::from_str(&contents)?)
        }
        Err(_e) => None,
    };

    let wallet = Wallet::new(
        Arc::new(localstore),
        &mnemonic.unwrap().to_seed_normalized(""),
        vec![],
    );

    match &args.command {
        Commands::DecodeToken(sub_command_args) => {
            sub_commands::decode_token::decode_token(sub_command_args)
        }
        Commands::Melt(sub_command_args) => {
            sub_commands::melt::melt(wallet, sub_command_args).await
        }
        Commands::Receive(sub_command_args) => {
            sub_commands::receive::receive(wallet, sub_command_args).await
        }
        Commands::CreateToken(sub_command_args) => {
            sub_commands::create_token::create_token(wallet, sub_command_args).await
        }
        Commands::CheckSpendable => sub_commands::check_spent::check_spent(wallet).await,
        Commands::MintInfo(sub_command_args) => {
            sub_commands::mint_info::mint_info(sub_command_args).await
        }
        Commands::Mint(sub_command_args) => {
            sub_commands::mint::mint(wallet, sub_command_args).await
        }
        Commands::Restore(sub_command_args) => {
            sub_commands::restore::restore(wallet, sub_command_args).await
        }
    }
}
