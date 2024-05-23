use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use cdk::nuts::SecretKey;
use cdk::wallet::Wallet;
use cdk::Mnemonic;
use cdk_redb::RedbWalletDatabase;
use clap::Args;

use crate::{DEFAULT_DB_PATH, DEFAULT_SEED_PATH};

#[derive(Args)]
pub struct ReceiveSubCommand {
    /// Cashu Token
    #[arg(short, long)]
    token: Option<String>,
    /// Nostr key
    #[arg(short, long)]
    nostr_key: Option<String>,
    /// Signing Key
    #[arg(short, long, action = clap::ArgAction::Append)]
    signing_key: Vec<String>,
    /// Nostr relay
    #[arg(short, long, action = clap::ArgAction::Append)]
    relay: Vec<String>,
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

    let nostr_key = match sub_command_args.nostr_key.as_ref() {
        Some(nostr_key) => Some(SecretKey::from_str(nostr_key)?),
        None => None,
    };

    let localstore = RedbWalletDatabase::new(&db_path)?;
    let wallet = Wallet::new(
        Arc::new(localstore),
        &mnemonic.unwrap().to_seed_normalized(""),
    );

    let preimage = match sub_command_args.preimage.is_empty() {
        true => None,
        false => Some(sub_command_args.preimage.clone()),
    };

    let signing_key = match sub_command_args.signing_key.is_empty() {
        false => Some(
            sub_command_args
                .signing_key
                .iter()
                .map(|s| SecretKey::from_str(s).unwrap())
                .collect(),
        ),
        true => None,
    };

    let amount = match nostr_key {
        Some(nostr_key) => {
            assert!(!sub_command_args.relay.is_empty());
            wallet
                .add_nostr_relays(sub_command_args.relay.clone())
                .await?;
            wallet.nostr_receive(nostr_key).await?
        }
        None => {
            wallet
                .receive(
                    sub_command_args
                        .token
                        .as_ref()
                        .ok_or(anyhow!("Token Required"))?,
                    signing_key,
                    preimage,
                )
                .await?
        }
    };

    println!("Received: {}", amount);

    Ok(())
}
