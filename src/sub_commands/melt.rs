use std::println;
use std::str::FromStr;

use anyhow::Result;
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::client::Client;
use cashu_sdk::nuts::Token;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::{Amount, Bolt11Invoice};
use clap::Args;

#[derive(Args)]
pub struct MeltSubCommand {
    /// Cashu Token
    #[arg(short, long)]
    token: String,
    #[arg(short, long)]
    bolt11: String,
}

pub async fn melt(sub_command_args: &MeltSubCommand) -> Result<()> {
    let token = Token::from_str(&sub_command_args.token)?;

    let client = HttpClient {};

    let mint_url = token.token[0].mint.clone();

    let keys = client.get_mint_keys(mint_url.clone().try_into()?).await?;

    let wallet = Wallet::new(client, mint_url, keys);

    let melt = wallet
        .melt(
            Bolt11Invoice::from_str(&sub_command_args.bolt11)?,
            token
                .token
                .into_iter()
                .map(|p| p.proofs)
                .flatten()
                .collect(),
            Amount::ZERO,
        )
        .await?;

    println!("{:?}", melt);

    Ok(())
}
