use anyhow::Result;
use clap::{Parser, Subcommand};

mod sub_commands;
mod types;

/// Simple CLI application to interact with cashu
#[derive(Parser)]
#[command(name = "cashu-tool")]
#[command(author = "thesimplekid <tsk@thesimplekid.com>")]
#[command(version = "0.1")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

const DEFAULT_DB_PATH: &str = "./cashu_tool.redb";
const DEFAULT_SEED_PATH: &str = "./seed.txt";

#[derive(Subcommand)]
enum Commands {
    DecodeToken(sub_commands::decode_token::DecodeTokenSubCommand),
    Melt(sub_commands::melt::MeltSubCommand),
    Receive(sub_commands::receive::ReceiveSubCommand),
    CreateToken(sub_commands::create_token::CreateTokenSubCommand),
    CheckSpendable(sub_commands::check_spent::CheckSpentSubCommand),
    MintInfo(sub_commands::mint_info::MintInfoSubcommand),
    Mint(sub_commands::mint::MintSubCommand),
    Restore(sub_commands::restore::RestoreSubCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    // Parse input
    let args: Cli = Cli::parse();

    match &args.command {
        Commands::DecodeToken(sub_command_args) => {
            sub_commands::decode_token::decode_token(sub_command_args)
        }
        Commands::Melt(sub_command_args) => sub_commands::melt::melt(sub_command_args).await,
        Commands::Receive(sub_command_args) => {
            sub_commands::receive::receive(sub_command_args).await
        }
        Commands::CreateToken(sub_command_args) => {
            sub_commands::create_token::create_token(sub_command_args).await
        }
        Commands::CheckSpendable(sub_command_args) => {
            sub_commands::check_spent::check_spent(sub_command_args).await
        }
        Commands::MintInfo(sub_command_args) => {
            sub_commands::mint_info::mint_info(sub_command_args).await
        }
        Commands::Mint(sub_command_args) => sub_commands::mint::mint(sub_command_args).await,
        Commands::Restore(sub_command_args) => {
            sub_commands::restore::restore(sub_command_args).await
        }
    }
}
