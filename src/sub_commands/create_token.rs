use std::io::Write;
use std::str::FromStr;
use std::{fs, io, println};

use anyhow::{bail, Result};
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::nuts::nut11::VerifyingKey;
use cashu_sdk::nuts::{CurrencyUnit, P2PKConditions, Token};
use cashu_sdk::url::UncheckedUrl;
use cashu_sdk::wallet::localstore::RedbLocalStore;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::{Amount, Mnemonic};
use clap::Args;

use crate::{DEFAULT_DB_PATH, DEFAULT_SEED_PATH};

#[derive(Args)]
pub struct CreateTokenSubCommand {
    /// Token Memo
    #[arg(short, long)]
    memo: Option<String>,
    /// Required number of signatures
    #[arg(long)]
    required_sigs: Option<u64>,
    /// Locktime before refund keys can be used
    #[arg(short, long)]
    locktime: Option<u64>,
    /// Publey to lock proofs to
    #[arg(short, long, action = clap::ArgAction::Append)]
    pubkey: Vec<String>,
    /// Publey to lock proofs to
    #[arg(long, action = clap::ArgAction::Append)]
    refund_keys: Vec<String>,
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn create_token(sub_command_args: &CreateTokenSubCommand) -> Result<()> {
    let client = HttpClient {};

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
    let mut wallet = Wallet::new(client, localstore, mnemonic).await;

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

    let proofs = if !sub_command_args.pubkey.is_empty() {
        let pubkeys = sub_command_args
            .pubkey
            .iter()
            .map(|p| VerifyingKey::from_str(p).unwrap())
            .collect();

        let refund_keys = sub_command_args
            .refund_keys
            .iter()
            .map(|p| VerifyingKey::from_str(p).unwrap())
            .collect();

        let p2pk_conditions = P2PKConditions::new(
            sub_command_args.locktime,
            pubkeys,
            refund_keys,
            sub_command_args.required_sigs,
            None,
        )
        .unwrap();

        wallet
            .send_p2pk(&mint_url, &CurrencyUnit::Sat, token_amount, p2pk_conditions)
            .await?
    } else {
        wallet
            .send(&mint_url, &CurrencyUnit::Sat, token_amount)
            .await?
    };

    let token = Token::new(
        mint_url.clone(),
        proofs,
        sub_command_args.memo.clone(),
        None,
    )?;

    println!("{}", token.to_string());

    Ok(())
}
