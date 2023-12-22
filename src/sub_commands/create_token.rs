use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::println;

use anyhow::{bail, Result};
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::client::Client;
use cashu_sdk::nuts::{Proofs, Token};
use cashu_sdk::url::UncheckedUrl;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::Amount;
use clap::Args;

#[derive(Args)]
pub struct CreateTokenSubCommand {
    /// Token Memo
    #[arg(short, long)]
    memo: Option<String>,
    /// File Path to save proofs
    #[arg(short, long)]
    file_path: Option<String>,
}

pub async fn create_token(sub_command_args: &CreateTokenSubCommand) -> Result<()> {
    let client = HttpClient {};

    let file_path = sub_command_args
        .file_path
        .clone()
        .unwrap_or("./proofs".to_string());

    let mut saved_proofs: HashMap<UncheckedUrl, Proofs> = match File::open(&file_path) {
        Ok(mut file) => {
            // Read the contents of the file into a String
            let mut saved_proofs = String::new();
            file.read_to_string(&mut saved_proofs)?;
            serde_json::from_str(&saved_proofs).unwrap()
        }
        Err(_) => HashMap::new(),
    };

    let mints_amounts: Vec<(&UncheckedUrl, Amount)> = saved_proofs
        .iter()
        .map(|(k, v)| (k, v.iter().map(|p| p.amount).sum()))
        .collect();

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

    println!("Enter value of token in sats");

    let mut user_input = String::new();
    let stdin = io::stdin();
    io::stdout().flush().unwrap();
    stdin.read_line(&mut user_input)?;
    let token_amount = Amount::from(user_input.trim().parse::<u64>()?);

    if token_amount.gt(&mints_amounts[mint_number as usize].1) {
        bail!("Not enough funds");
    }

    let mint_url = mints_amounts[mint_number as usize].0;

    let keys = client.get_mint_keys(mint_url.clone().try_into()?).await?;

    let wallet = Wallet::new(client, mint_url.clone(), keys);

    let proofs = saved_proofs.get(mint_url).unwrap().clone();

    let send_proofs = wallet.send(token_amount, proofs).await.unwrap();

    let token = Token::new(
        mint_url.clone(),
        send_proofs.send_proofs,
        sub_command_args.memo.clone(),
        None,
    )?;

    println!("{}", token.to_string());

    let mut file = File::create(file_path)?;

    saved_proofs.insert(mint_url.clone(), send_proofs.change_proofs);

    file.write_all(serde_json::to_string(&saved_proofs)?.as_bytes())?;
    file.flush()?;

    Ok(())
}
