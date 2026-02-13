use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "soteric", version, about = "Very basic Soteric skeleton")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    AddProfile,
    ListProfiles,
    EncryptNow,
    DecryptNow,
    Run,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::AddProfile => println!("[TODO] add-profile not implemented yet"),
        Command::ListProfiles => println!("[TODO] list-profiles not implemented yet"),
        Command::EncryptNow => println!("[TODO] encrypt-now not implemented yet"),
        Command::DecryptNow => println!("[TODO] decrypt-now not implemented yet"),
        Command::Run => println!("[TODO] run service not implemented yet"),
    }

    Ok(())
}
