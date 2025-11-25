use clap::Parser;
use anyhow::Result;

mod grammar;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Verifies that a grammar file is syntactically valid
    Check {
        /// Path to grammar file
        grammar: String,
    },
}

fn check(grammar: String) -> Result<()> {
    let mut t = grammar::tokenizer::Tokenizer::new();
    let content = std::fs::read_to_string(&grammar)?;
    let v = t.tokenize(&content);
    println!("{:#?}", v);
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Check { grammar } => check(grammar),
    }
}
