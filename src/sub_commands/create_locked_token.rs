use std::io::Write;
use std::str::FromStr;
use std::{io, println};

use anyhow::{bail, Result};
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::nuts::{CurrencyUnit, P2PKConditions, Token, VerifyingKey};
use cashu_sdk::url::UncheckedUrl;
use cashu_sdk::wallet::localstore::RedbLocalStore;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::Amount;
use clap::Args;

use crate::DEFAULT_DB_PATH;

#[derive(Args)]
pub struct CreateLockedTokenSubCommand {
    /// Token Memo
    #[arg(short, long)]
    memo: Option<String>,
    #[arg(short, long)]
    pubkey: String,
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn create_locked_token(sub_command_args: &CreateLockedTokenSubCommand) -> Result<()> {
    let client = HttpClient {};

    let db_path = sub_command_args
        .db_path
        .clone()
        .unwrap_or(DEFAULT_DB_PATH.to_string());

    let localstore = RedbLocalStore::new(&db_path)?;
    let mut wallet = Wallet::new(client, localstore, None).await;

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

    let mint_url = mints_amounts[mint_number].0.clone();

    println!("Enter value of token in sats");

    let mut user_input = String::new();
    let stdin = io::stdin();
    io::stdout().flush().unwrap();
    stdin.read_line(&mut user_input)?;
    let token_amount = Amount::from(user_input.trim().parse::<u64>()?);

    if token_amount.gt(&mints_amounts[mint_number as usize].1) {
        bail!("Not enough funds");
    }

    let input_proofs = wallet
        .select_proofs(mint_url.clone(), &CurrencyUnit::Sat, token_amount)
        .await?;

    let pubkey = VerifyingKey::from_str(&sub_command_args.pubkey).unwrap();

    let p2pk_conditions = P2PKConditions::new(None, vec![pubkey], vec![], None, None).unwrap();

    let proofs = wallet
        .create_p2pk_proofs(&mint_url, &CurrencyUnit::Sat, input_proofs, p2pk_conditions)
        .await?;

    println!("{:?}", proofs);

    let token = Token::new(
        mint_url.clone(),
        proofs,
        sub_command_args.memo.clone(),
        None,
    )?;

    println!("{}", token.to_string());

    Ok(())
}
