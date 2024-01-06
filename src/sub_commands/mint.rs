use std::io::{self, Read, Write};
use std::str::FromStr;

use anyhow::Result;
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::nuts::CurrencyUnit;
use cashu_sdk::url::UncheckedUrl;
use cashu_sdk::wallet::localstore::RedbLocalStore;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::Amount;
use clap::Args;

use crate::DEFAULT_DB_PATH;

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
    let client = HttpClient {};

    let db_path = sub_command_args
        .db_path
        .clone()
        .unwrap_or(DEFAULT_DB_PATH.to_string());

    let localstore = RedbLocalStore::new(&db_path)?;
    let mut wallet = Wallet::new(client, localstore, vec![], vec![], None, vec![]).await;

    let quote = wallet
        .mint_quote(
            mint_url.clone(),
            Amount::from(sub_command_args.amount),
            CurrencyUnit::from_str(&sub_command_args.unit)?,
        )
        .await?;

    println!("Quote: {:#?}", quote);

    println!("Please pay: {}", quote.request.to_string());

    loop {
        println!("Press any key to continue once request is paid");

        let _ = io::stdout().flush();

        let mut buffer = [0; 1];
        if io::stdin().read_exact(&mut buffer).is_ok() {
            break; // Exit the loop when any key is pressed
        }

        println!("Failed to read input.");
        break;
    }

    let receive_amount = wallet.mint(mint_url.clone(), &quote.id).await?;

    println!("Received {receive_amount} from mint {mint_url}");

    Ok(())
}
