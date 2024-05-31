use std::fs;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use cdk::amount::SplitTarget;
use cdk::nuts::CurrencyUnit;
use cdk::url::UncheckedUrl;
use cdk::wallet::Wallet;
use cdk::{Amount, Mnemonic};
use cdk_redb::RedbWalletDatabase;
use clap::Args;
use tokio::time::sleep;

use crate::{DEFAULT_DB_PATH, DEFAULT_SEED_PATH};

#[derive(Args)]
pub struct MintSubCommand {
    #[arg(short, long)]
    amount: u64,
    #[arg(short, long)]
    unit: String,
    #[arg(short, long)]
    mint_url: UncheckedUrl,
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn mint(sub_command_args: &MintSubCommand) -> Result<()> {
    let mint_url = sub_command_args.mint_url.clone();

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

    let quote = wallet
        .mint_quote(
            mint_url.clone(),
            Amount::from(sub_command_args.amount),
            CurrencyUnit::from(&sub_command_args.unit),
        )
        .await?;

    println!("Quote: {:#?}", quote);

    println!("Please pay: {}", quote.request.to_string());

    loop {
        let status = wallet
            .mint_quote_status(mint_url.clone(), &quote.id)
            .await?;

        if status.paid {
            break;
        }

        sleep(Duration::from_secs(2)).await;
    }

    let receive_amount = wallet
        .mint(mint_url.clone(), &quote.id, SplitTarget::default(), None)
        .await?;

    println!("Received {receive_amount} from mint {mint_url}");

    Ok(())
}
