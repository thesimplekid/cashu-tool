use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use cdk::wallet::Wallet;
use cdk::Mnemonic;
use cdk_sqlite::WalletSQLiteDatabase;
use clap::Args;

use crate::{DEFAULT_DB_PATH, DEFAULT_SEED_PATH};

#[derive(Args)]
pub struct CheckSpentSubCommand {
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn check_spent(sub_command_args: &CheckSpentSubCommand) -> Result<()> {
    let db_path = sub_command_args
        .db_path
        .clone()
        .unwrap_or(DEFAULT_DB_PATH.to_string());

    let localstore = WalletSQLiteDatabase::new(&db_path).await?;
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
    let p = wallet.check_all_pending_proofs(None).await?;

    println!("Amount {} already spent and removed", p);

    Ok(())
}
