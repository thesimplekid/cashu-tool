use std::println;
use std::str::FromStr;

use anyhow::Result;
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::nuts::Token;
use cashu_sdk::wallet::localstore::RedbLocalStore;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::Bolt11Invoice;
use clap::Args;

use crate::DEFAULT_DB_PATH;

#[derive(Args)]
pub struct MeltSubCommand {
    /// Cashu Token
    #[arg(short, long)]
    token: String,
    #[arg(short, long)]
    bolt11: String,
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn melt(sub_command_args: &MeltSubCommand) -> Result<()> {
    let token = Token::from_str(&sub_command_args.token)?;
    let bolt11 = Bolt11Invoice::from_str(&sub_command_args.bolt11)?;

    let client = HttpClient {};

    let mint_url = token.token[0].mint.clone();
    let db_path = sub_command_args
        .db_path
        .clone()
        .unwrap_or(DEFAULT_DB_PATH.to_string());

    let localstore = RedbLocalStore::new(&db_path)?;

    let mut wallet = Wallet::new(client, localstore, vec![], vec![], None, vec![]).await;

    let quote = wallet
        .melt_quote(
            mint_url.clone(),
            cashu_sdk::nuts::CurrencyUnit::Sat,
            bolt11.to_string(),
        )
        .await?;

    let melt = wallet.melt(&mint_url, &quote.id).await.unwrap();

    println!("{:?}", melt);

    Ok(())
}
