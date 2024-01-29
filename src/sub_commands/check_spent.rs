use std::io::Write;
use std::{io, println};

use anyhow::{bail, Result};
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::url::UncheckedUrl;
use cashu_sdk::wallet::localstore::RedbLocalStore;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::Amount;
use clap::Args;

use crate::DEFAULT_DB_PATH;

#[derive(Args)]
pub struct CheckSpentSubCommand {
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn check_spent(sub_command_args: &CheckSpentSubCommand) -> Result<()> {
    let client = HttpClient {};

    let db_path = sub_command_args
        .db_path
        .clone()
        .unwrap_or(DEFAULT_DB_PATH.to_string());

    let localstore = RedbLocalStore::new(&db_path)?;

    let wallet = Wallet::new(client, localstore, None).await;

    let mints_amounts: Vec<(UncheckedUrl, Amount)> =
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

    let mint_url = mints_amounts[mint_number as usize].0.clone();

    let proofs = wallet.get_proofs(mint_url.clone()).await?.unwrap();

    let send_proofs = wallet
        .check_proofs_spent(mint_url, proofs.iter().map(|p| p.clone().into()).collect())
        .await?;

    for proof in send_proofs {
        println!("{:#?}", proof);
    }

    Ok(())
}
