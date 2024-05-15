use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use cdk::nuts::SigningKey;
use cdk::wallet::Wallet;
use cdk::Mnemonic;
use cdk_redb::RedbWalletDatabase;
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
    /// Preimage
    #[arg(short, long,  action = clap::ArgAction::Append)]
    preimage: Vec<String>,
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn receive(sub_command_args: &ReceiveSubCommand) -> Result<()> {
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

    let localstore = RedbWalletDatabase::new(&db_path)?;
    let mut wallet = Wallet::new(
        Arc::new(localstore),
        &mnemonic.unwrap().to_seed_normalized(""),
    );

    let preimage = match sub_command_args.preimage.is_empty() {
        true => None,
        false => Some(sub_command_args.preimage.clone()),
    };

    if !sub_command_args.signing_key.is_empty() {
        let secret_keys = sub_command_args
            .signing_key
            .iter()
            .map(|s| SigningKey::from_str(s).unwrap())
            .collect();

        wallet
            .receive(&sub_command_args.token, Some(secret_keys), preimage)
            .await?;
    } else {
        wallet
            .receive(&sub_command_args.token, None, preimage)
            .await?;
    }

    Ok(())
}
