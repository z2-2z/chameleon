use clap::Parser;
use anyhow::Result;
use mimalloc::MiMalloc;

mod grammar;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
        grammar: Vec<String>,
    },
}

fn check(grammars: Vec<String>) -> Result<()> {
    let mut builder = grammar::ContextFreeGrammar::builder();
    
    for grammar in grammars {
        builder.load_grammar(&grammar)?;
    }
    
    builder.check()?;
    
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Check { grammar } => check(grammar),
    }
}
