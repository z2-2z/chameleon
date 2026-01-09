use clap::Parser;
use anyhow::Result;
use mimalloc::MiMalloc;
use libafl::prelude::{Input, HasTargetBytes};
use std::io::Write;

mod grammar;
mod translator;
mod beautifier;

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
    /// Verifies that a grammar file is valid
    Check {
        /// Sets the entrypoint for the grammar
        #[arg(short, long)]
        entrypoint: Option<String>,
        
        /// Paths to .chm grammar files
        #[arg(required = true)]
        grammars: Vec<String>,
    },
    
    /// Take one or more grammars and emit code
    Translate {
        /// Sets the entrypoint for the grammar
        #[arg(short, long)]
        entrypoint: Option<String>,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
        
        /// Enable "baby" mode to output just a simple generator and not a full
        /// mutation procedure
        #[arg(short, long)]
        baby: bool,
        
        /// Sets a prefix for the generated function names
        #[arg(short, long)]
        prefix: Option<String>,
        
        /// Name of resulting .c file
        #[arg(short, long)]
        output: String,
        
        /// Paths to .chm grammar files
        #[arg(required = true)]
        grammars: Vec<String>,
    },
    
    /// Merge multiple grammar files into one
    Join {
        /// Sets the entrypoint for the grammar
        #[arg(short, long)]
        entrypoint: Option<String>,
        
        /// Name of resulting .chm file
        #[arg(short, long)]
        output: String,
        
        /// Paths to .chm grammar files
        #[arg(required = true)]
        grammars: Vec<String>,
    },
    
    /// Given a .bin file, print the corresponding generator output to stdout
    Print {
        /// Path to a .bin file
        input: String,
    }
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

fn translate(entrypoint: Option<String>, verbose: bool, baby: bool, prefix: Option<String>, output: String, grammars: Vec<String>) -> Result<()> {
    let mut builder = grammar::ContextFreeGrammar::builder();
    if let Some(entrypoint) = entrypoint {
        builder.set_entrypoint(entrypoint);
    }
    for grammar in grammars {
        builder.load_grammar(&grammar)?;
    }
    let cfg = builder.build(verbose)?;
    
    if verbose && !cfg.unused_nonterms().is_empty() {
        println!("WARNING: The following non-terminals are unreachable when using entrypoint '{}': {:?}", cfg.entrypoint().name(), cfg.unused_nonterms());
    }
    let cfg = translator::TranslatorGrammar::converter().convert(&cfg);
    
    if baby {
        translator::baby::render(cfg, prefix, output)?;
    } else {
        translator::full::render(cfg, prefix, output)?;
    }
    
    Ok(())
}

fn join(entrypoint: Option<String>, output: String, grammars: Vec<String>) -> Result<()> {
    let mut builder = grammar::ContextFreeGrammar::builder();
    if let Some(entrypoint) = entrypoint {
        builder.set_entrypoint(entrypoint);
    }
    for grammar in grammars {
        builder.load_grammar(&grammar)?;
    }
    let cfg = builder.build(true)?;
    
    beautifier::beautify(cfg, output)?;
    
    Ok(())
}

fn print(input: String) -> Result<()> {
    let input = chameleon::ChameleonInput::from_file(input)?;
    let bytes = input.target_bytes();
    std::io::stdout().write_all(&bytes)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Check { entrypoint, grammars } => check(entrypoint, grammars),
        Commands::Translate { entrypoint, verbose, baby, prefix, output, grammars } => translate(entrypoint, verbose, baby, prefix, output, grammars),
        Commands::Join { entrypoint, output, grammars } => join(entrypoint, output, grammars),
        Commands::Print { input } => print(input),
    }
}
