use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::str::FromStr;

use anyhow::Result;
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::client::Client;
use cashu_sdk::nuts::{CurrencyUnit, Proofs};
use cashu_sdk::url::UncheckedUrl;
use cashu_sdk::wallet::Wallet;
use cashu_sdk::Amount;
use clap::Args;

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
    file_path: Option<String>,
}

pub async fn mint(sub_command_args: &MintSubCommand) -> Result<()> {
    let mint_url = sub_command_args.mint_url.clone();
    let client = HttpClient {};

    let keys = client.get_mint_keys(mint_url.clone().try_into()?).await?;
    let mut wallet = Wallet::new(client, mint_url.clone(), vec![], vec![], keys);

    let quote = wallet
        .mint_quote(
            Amount::from(sub_command_args.amount),
            CurrencyUnit::from_str(&sub_command_args.unit)?,
        )
        .await?;

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

    let mut proofs = wallet.mint(&quote.id).await?;
    let receive_amount = proofs.iter().map(|p| p.amount).sum::<Amount>();

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
