use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use std::{fs, io, println};

use anyhow::{bail, Result};
use cdk::amount::SplitTarget;
use cdk::nuts::CurrencyUnit;
use cdk::url::UncheckedUrl;
use cdk::wallet::Wallet;
use cdk::{Bolt11Invoice, Mnemonic};
use cdk_sqlite::WalletSQLiteDatabase;
use clap::Args;

use crate::{DEFAULT_DB_PATH, DEFAULT_SEED_PATH};

#[derive(Args)]
pub struct MeltSubCommand {
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn melt(sub_command_args: &MeltSubCommand) -> Result<()> {
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

    let mut wallet = Wallet::new(
        Arc::new(localstore),
        &mnemonic.unwrap().to_seed_normalized(""),
        vec![],
    );

    let mints_amounts: Vec<(UncheckedUrl, HashMap<_, _>)> =
        wallet.mint_balances().await?.into_iter().collect();

    for (i, (mint, amount)) in mints_amounts.iter().enumerate() {
        println!("{}: {}, {:?} sats", i, mint, amount);
    }

    println!("Enter mint number to create token");

    let mut user_input = String::new();
    let stdin = io::stdin();
    io::stdout().flush().unwrap();
    stdin.read_line(&mut user_input)?;

    let mint_number: usize = user_input.trim().parse()?;

    if mint_number.gt(&(mints_amounts.len() - 1)) {
        bail!("Invalid mint number");
    }

    let mint_url = mints_amounts[mint_number].0.clone();

    println!("Enter bolt11 invoice request");

    let mut user_input = String::new();
    let stdin = io::stdin();
    io::stdout().flush().unwrap();
    stdin.read_line(&mut user_input)?;
    let bolt11 = Bolt11Invoice::from_str(user_input.trim())?;

    if bolt11
        .amount_milli_satoshis()
        .unwrap()
        .gt(&(<cdk::Amount as Into<u64>>::into(
            *mints_amounts[mint_number as usize]
                .1
                .get(&CurrencyUnit::Sat)
                .unwrap(),
        ) * 1000_u64))
    {
        bail!("Not enough funds");
    }
    let quote = wallet
        .melt_quote(
            mint_url.clone(),
            cdk::nuts::CurrencyUnit::Sat,
            bolt11.to_string(),
        )
        .await?;

    let melt = wallet
        .melt(&mint_url, &quote.id, SplitTarget::default())
        .await
        .unwrap();

    println!("Paid invoice: {}", melt.paid);
    if let Some(preimage) = melt.preimage {
        println!("Payment preimage: {}", preimage);
    }

    Ok(())
}
