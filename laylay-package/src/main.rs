use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, author)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Create { name: PathBuf },
    Bundle,
    Push,
}

fn create(name: PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(&name).map_err(|e| e.to_string())?;

    Ok(())
}

fn bundle() -> Result<(), String> {
    Ok(())
}

fn push() -> Result<(), String> {
    Ok(())
}

fn main() {
    let args = Args::parse();

    let res = match args.command {
        Commands::Create { name } => create(name),
        Commands::Bundle => bundle(),
        Commands::Push => push(),
    };

    if let Err(e) = res {
        println!("{e}");
    }
}
