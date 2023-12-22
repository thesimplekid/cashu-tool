use anyhow::Result;
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::client::Client;
use cashu_sdk::url::UncheckedUrl;
use clap::Args;

#[derive(Args)]
pub struct MintInfoSubcommand {
    /// Cashu Token
    #[arg(short, long)]
    mint_url: UncheckedUrl,
}

pub async fn mint_info(sub_command_args: &MintInfoSubcommand) -> Result<()> {
    let client = HttpClient {};

    let info = client
        .get_mint_info(sub_command_args.mint_url.clone().try_into()?)
        .await?;

    println!("{:#?}", info);

    Ok(())
}
