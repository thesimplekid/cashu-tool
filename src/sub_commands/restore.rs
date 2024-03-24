use std::fs;
use std::str::FromStr;

use anyhow::Result;
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::url::UncheckedUrl;
use cashu_sdk::wallet::localstore::RedbLocalStore;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::Mnemonic;
use clap::Args;

use crate::{DEFAULT_DB_PATH, DEFAULT_SEED_PATH};

#[derive(Args)]
pub struct RestoreSubCommand {
    #[arg(short, long)]
    mint_url: UncheckedUrl,
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn restore(sub_command_args: &RestoreSubCommand) -> Result<()> {
    let mint_url = sub_command_args.mint_url.clone();
    let client = HttpClient {};

    let db_path = sub_command_args
        .db_path
        .clone()
        .unwrap_or(DEFAULT_DB_PATH.to_string());

    let mnemonic = match fs::metadata(DEFAULT_SEED_PATH) {
        Ok(_) => {
            let contents = fs::read_to_string(DEFAULT_SEED_PATH)?;
            Some(Mnemonic::from_str(&contents)?)
        }
        Err(_e) => None,
    };

    let localstore = RedbLocalStore::new(&db_path)?;
    let mut wallet = Wallet::new(client, localstore, mnemonic).await;

    let amount = wallet.restore(mint_url).await?;

    println!("Restored {}", amount);

    Ok(())
}
