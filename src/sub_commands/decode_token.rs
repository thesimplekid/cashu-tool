use std::str::FromStr;

use anyhow::Result;
use cashu_sdk::nuts::Token;
use clap::Args;

#[derive(Args)]
pub struct DecodeTokenSubCommand {
    /// Cashu Token
    #[arg(short, long)]
    token: String,
}

pub fn decode_token(sub_command_args: &DecodeTokenSubCommand) -> Result<()> {
    let token = Token::from_str(&sub_command_args.token)?;

    println!("{:}", serde_json::to_string_pretty(&token)?);
    Ok(())
}
