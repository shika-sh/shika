mod commands;
mod config;
mod language;

use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
struct ShikaArguments {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Pull,
    Push,
    Init,
    Generate {
        #[arg(long, short)]
        pull: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let arguments = ShikaArguments::parse();

    match arguments.command {
        Command::Pull => commands::pull(),
        Command::Push => todo!(),
        Command::Init => commands::init(),
        Command::Generate { pull } => commands::generate(pull),
    }
    .unwrap_or_else(|error| println!("{}", format!("{} {}", "✕".purple(), error).bold()));

    println!();
    Ok(())
}
