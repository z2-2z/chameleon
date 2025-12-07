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
        /// Sets the non-terminal entrypoint for the grammar
        #[arg(long)]
        entrypoint: Option<String>,
        
        /// Paths to grammar files
        grammars: Vec<String>,
    },
    
    /// Take one or more grammars and emit mutation and generation code
    Translate {
        /// Sets the non-terminal entrypoint for the grammar
        #[arg(long)]
        entrypoint: Option<String>,
        
        /// Paths to grammar files
        grammars: Vec<String>,
    },
}

fn check(entrypoint: Option<String>, grammars: Vec<String>) -> Result<()> {
    let mut builder = grammar::ContextFreeGrammar::builder();
    
    if let Some(entrypoint) = entrypoint {
        builder.set_entrypoint(entrypoint);
    }
    
    for grammar in grammars {
        builder.load_grammar(&grammar)?;
    }
    
    builder.check()?;
    
    Ok(())
}

fn translate(entrypoint: Option<String>, grammars: Vec<String>) -> Result<()> {
    let mut builder = grammar::ContextFreeGrammar::builder();
    
    if let Some(entrypoint) = entrypoint {
        builder.set_entrypoint(entrypoint);
    }
    
    for grammar in grammars {
        builder.load_grammar(&grammar)?;
    }
    
    let cfg = builder.build()?;
    
    if !cfg.unused_nonterms().is_empty() {
        println!("WARNING: The following non-terminals are unreachable when using entrypoint '{}': {:?}", cfg.entrypoint().id(), cfg.unused_nonterms());
    }
    
    //println!("{:#?}", cfg);
    
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Check { entrypoint, grammars } => check(entrypoint, grammars),
        Commands::Translate { entrypoint, grammars } => translate(entrypoint, grammars),
    }
}
