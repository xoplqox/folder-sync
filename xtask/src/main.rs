use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate test fixture drive folders for local development.
    GenFixtures,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::GenFixtures => {
            anyhow::bail!("gen-fixtures is not implemented yet");
        }
    }
}
