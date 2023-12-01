use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;

use anyhow::Result;
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::client::Client;
use cashu_sdk::nuts::{Proofs, Token};
use cashu_sdk::url::UncheckedUrl;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::Amount;
use clap::Args;

#[derive(Args)]
pub struct ReceiveSubCommand {
    /// Cashu Token
    #[arg(short, long)]
    token: String,
    /// File Path to save proofs
    #[arg(short, long)]
    file_path: Option<String>,
}

pub async fn receive(sub_command_args: &ReceiveSubCommand) -> Result<()> {
    let token = Token::from_str(&sub_command_args.token)?;

    let client = HttpClient {};

    let mint_url = token.token[0].mint.clone();

    let keys = client.get_mint_keys(mint_url.clone().try_into()?).await?;

    let wallet = Wallet::new(client, mint_url.clone(), keys);

    let mut proofs = wallet.receive(&sub_command_args.token).await?;
    let receive_amount = proofs.iter().map(|p| p.amount).sum::<Amount>().to_sat();

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

    if let Some(existing_proofs) = saved_proofs.get_mut(&mint_url) {
        existing_proofs.append(proofs.as_mut());
    } else {
        saved_proofs.insert(mint_url.clone(), proofs.clone());
    }

    let mut file = File::create(file_path)?;

    file.write_all(serde_json::to_string(&saved_proofs)?.as_bytes())?;
    file.flush()?;

    println!("Received {receive_amount} from mint {mint_url}");

    Ok(())
}
