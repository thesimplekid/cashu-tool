use std::str::FromStr;

use anyhow::Result;
use cashu_sdk::client::minreq_client::HttpClient;
use cashu_sdk::nuts::{Proof, SigningKey, Token};
use cashu_sdk::wallet::localstore::RedbLocalStore;
use cashu_sdk::wallet::Wallet;
use clap::Args;

use crate::DEFAULT_DB_PATH;

#[derive(Args)]
pub struct ReceiveSubCommand {
    /// Cashu Token
    #[arg(short, long)]
    token: String,
    /// Cashu Token
    #[arg(short, long)]
    signing_key: Option<String>,
    /// File Path to save proofs
    #[arg(short, long)]
    db_path: Option<String>,
}

pub async fn receive(sub_command_args: &ReceiveSubCommand) -> Result<()> {
    let client = HttpClient {};

    let db_path = sub_command_args
        .db_path
        .clone()
        .unwrap_or(DEFAULT_DB_PATH.to_string());

    let localstore = RedbLocalStore::new(&db_path)?;
    let mut wallet = Wallet::new(client, localstore, None).await;

    if let Some(signing_key) = &sub_command_args.signing_key {
        let secret_key = SigningKey::from_str(&signing_key)?;
        let token = Token::from_str(&sub_command_args.token)?;
        let proofs: Vec<Proof> = token
            .token
            .clone()
            .into_iter()
            .map(|p| p.proofs)
            .flatten()
            .collect();

        let mut signed_proofs = Vec::with_capacity(proofs.len());

        for proof in proofs {
            let mut proof = proof;
            proof.sign_p2pk_proof(secret_key.clone().try_into()?)?;
            signed_proofs.push(proof);
        }

        wallet
            .claim_p2pk_locked_proof(
                &token.token_info().1.into(),
                &token.unit.unwrap_or_default(),
                secret_key,
                signed_proofs,
            )
            .await?;
    } else {
        wallet.receive(&sub_command_args.token).await?;
    }

    Ok(())
}
