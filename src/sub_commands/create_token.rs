use std::io::Write;
use std::{io, println};

use anyhow::{bail, Result};
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::nuts::{CurrencyUnit, Token};
use cashu_sdk::url::UncheckedUrl;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::{Amount, RedbLocalStore};
use clap::Args;

#[derive(Args)]
pub struct CreateTokenSubCommand {
    /// Token Memo
    #[arg(short, long)]
    memo: Option<String>,
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn create_token(sub_command_args: &CreateTokenSubCommand) -> Result<()> {
    let client = HttpClient {};

    let db_path = sub_command_args
        .db_path
        .clone()
        .unwrap_or("./cashu_tool.redb".to_string());

    let localstore = RedbLocalStore::new(&db_path)?;
    let mut wallet = Wallet::new(client, localstore, vec![], vec![], None, vec![]).await;

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

    let proofs = wallet
        .send(&mint_url, &CurrencyUnit::Sat, token_amount)
        .await
        .unwrap();

    let token = Token::new(
        mint_url.clone(),
        proofs,
        sub_command_args.memo.clone(),
        None,
    )?;

    println!("{}", token.to_string());

    Ok(())
}
