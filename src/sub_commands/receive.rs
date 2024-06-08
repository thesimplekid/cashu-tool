use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use cdk::amount::SplitTarget;
use cdk::nuts::SecretKey;
use cdk::wallet::Wallet;
use cdk::Mnemonic;
use cdk_sqlite::WalletSQLiteDatabase;
use clap::Args;

use crate::{DEFAULT_DB_PATH, DEFAULT_SEED_PATH};

#[derive(Args)]
pub struct ReceiveSubCommand {
    /// Cashu Token
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

    let localstore = WalletSQLiteDatabase::new(&db_path).await?;
    let wallet = Wallet::new(
        Arc::new(localstore),
        &mnemonic.unwrap().to_seed_normalized(""),
        vec![],
    );

    let nostr_key = match sub_command_args.nostr_key.as_ref() {
        Some(nostr_key) => {
            let secret_key = SecretKey::from_str(nostr_key)?;
            wallet.add_p2pk_signing_key(secret_key.clone()).await;
            Some(secret_key)
        }
        None => None,
    };

    if !sub_command_args.signing_key.is_empty() {
        let signing_keys: Vec<SecretKey> = sub_command_args
            .signing_key
            .iter()
            .map(|s| SecretKey::from_str(s).unwrap())
            .collect();

        for signing_key in signing_keys {
            wallet.add_p2pk_signing_key(signing_key).await;
        }
    }

    let preimage = match sub_command_args.preimage.is_empty() {
        true => None,
        false => Some(sub_command_args.preimage.clone()),
    };

    let amount = match nostr_key {
        Some(nostr_key) => {
            assert!(!sub_command_args.relay.is_empty());
            wallet
                .add_nostr_relays(sub_command_args.relay.clone())
                .await?;
            wallet
                .nostr_receive(nostr_key, SplitTarget::default())
                .await?
        }
        None => {
            wallet
                .receive(
                    sub_command_args
                        .token
                        .as_ref()
                        .ok_or(anyhow!("Token Required"))?,
                    &SplitTarget::default(),
                    preimage,
                )
                .await?
        }
    };

    println!("Received: {}", amount);

    Ok(())
}
