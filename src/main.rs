use anyhow::Result;
use clap::{Parser, Subcommand};

mod sub_commands;

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

#[derive(Subcommand)]
enum Commands {
    DecodeToken(sub_commands::decode_token::DecodeTokenSubCommand),
    Melt(sub_commands::melt::MeltSubCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse input
    let args: Cli = Cli::parse();

    match &args.command {
        Commands::DecodeToken(sub_command_args) => {
            sub_commands::decode_token::decode_token(sub_command_args)
        }
        Commands::Melt(sub_command_args) => sub_commands::melt::melt(sub_command_args).await,
    }
}
