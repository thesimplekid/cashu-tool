use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use cdk::nuts::SigningKey;
use cdk::wallet::localstore::RedbLocalStore;
use cdk::wallet::Wallet;
use cdk::{HttpClient, Mnemonic};
use clap::Args;

use crate::{DEFAULT_DB_PATH, DEFAULT_SEED_PATH};

#[derive(Args)]
pub struct ReceiveSubCommand {
    /// Cashu Token
    #[arg(short, long)]
    token: String,
    /// Cashu Token
    #[arg(short, long, action = clap::ArgAction::Append)]
    signing_key: Vec<String>,
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn receive(sub_command_args: &ReceiveSubCommand) -> Result<()> {
    let client = HttpClient::default();

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
    let mut wallet = Wallet::new(client, Arc::new(localstore), mnemonic).await;

    if !sub_command_args.signing_key.is_empty() {
        let secret_keys = sub_command_args
            .signing_key
            .iter()
            .map(|s| SigningKey::from_str(s).unwrap())
            .collect();

        wallet
            .receive_p2pk(&sub_command_args.token, secret_keys)
            .await?;
    } else {
        wallet.receive(&sub_command_args.token).await?;
    }

    Ok(())
}
