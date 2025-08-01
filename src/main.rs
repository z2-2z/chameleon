use std::io::Write;
use std::ops::Range;
use clap::Parser;
use anyhow::Result;

mod parser;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Beautifies a grammar in-place
    Format {
        /// Path to the grammar file to be formatted
        grammar: String,
    },
    
    /// Verifies that a grammar file is syntactically valid
    Check {
        /// Path to grammar file
        grammar: String,
    },
}

fn beautify(grammar: String) -> Result<()> {
    fn get_range<'a>(content: &'a [u8], range: &Range<usize>) -> Result<&'a [u8]> {
        match content.get(range.clone()) {
            Some(buf) => Ok(buf),
            None => anyhow::bail!("Internal error: Syntax node has invalid bytes range"),
        }
    }
    fn whitespace_separated(next_node: Option<&parser::SyntaxNode>) -> bool {
        if let Some(next_node) = next_node {
            return matches!(
                next_node,
                parser::SyntaxNode::Comment(_) |
                parser::SyntaxNode::String(_) |
                parser::SyntaxNode::Char(_) |
                parser::SyntaxNode::NonTerminal(_) |
                parser::SyntaxNode::StartSet(_) |
                parser::SyntaxNode::StartBlock |
                parser::SyntaxNode::BlockSeparator,
            );
        }
        
        false
    }
    
    let mut parser = parser::GrammarParser::new();
    let content = std::fs::read_to_string(&grammar)?;
    let stream = parser.parse(&content)?;
    
    let out_file = std::fs::File::create(&grammar)?;
    let mut out_file = std::io::BufWriter::new(out_file);
    let content = content.as_bytes();
    let mut in_rule = false;
    let mut remaining_elems = 0;
    let mut i = 0;
    
    while let Some(node) = stream.get(i) {
        match node {
            parser::SyntaxNode::Comment(range) => {
                let data = get_range(content, range)?;
                write!(&mut out_file, "# ")?;
                out_file.write_all(data)?;
                if !in_rule {
                    writeln!(&mut out_file)?;
                }
            },
            parser::SyntaxNode::StartRule(range) => {
                let data = get_range(content, range)?;
                out_file.write_all(data)?;
                write!(&mut out_file, " ->")?;
                in_rule = true;
            },
            parser::SyntaxNode::EndRule => {
                writeln!(&mut out_file)?;
                in_rule = false;
            },
            parser::SyntaxNode::String(range) => {
                let data = get_range(content, range)?;
                write!(&mut out_file, "\"")?;
                out_file.write_all(data)?;
                write!(&mut out_file, "\"")?;
            },
            parser::SyntaxNode::Char(range) => {
                let data = get_range(content, range)?;
                write!(&mut out_file, "'")?;
                out_file.write_all(data)?;
                write!(&mut out_file, "'")?;
            },
            parser::SyntaxNode::NonTerminal(range) => {
                let data = get_range(content, range)?;
                out_file.write_all(data)?;
            },
            parser::SyntaxNode::StartSet(range) => {
                let data = get_range(content, range)?;
                write!(&mut out_file, "Set<")?;
                out_file.write_all(data)?;
                write!(&mut out_file, ">(")?;
                
                remaining_elems = 0;
                let mut j = i;
                while let Some(node) = stream.get(j) {
                    if matches!(node, parser::SyntaxNode::EndSet) {
                        break;
                    } else if matches!(node, parser::SyntaxNode::Number(_) | parser::SyntaxNode::Range(_, _)) {
                        remaining_elems += 1;
                    }
                    j += 1;
                }
            },
            parser::SyntaxNode::EndSet => {
                write!(&mut out_file, ")")?;
            },
            parser::SyntaxNode::Number(number_format) => {
                let data = match number_format {
                    parser::NumberFormat::Hex(range) |
                    parser::NumberFormat::Decimal(range) => get_range(content, range)?,
                };
                out_file.write_all(data)?;
                remaining_elems -= 1;
                
                if remaining_elems > 0 {
                    write!(&mut out_file, ", ")?;
                }
            },
            parser::SyntaxNode::Range(left, right) => {
                let data = match left {
                    parser::NumberFormat::Hex(range) => {
                        write!(&mut out_file, "0x")?;
                        get_range(content, range)?
                    },
                    parser::NumberFormat::Decimal(range) => get_range(content, range)?,
                };
                out_file.write_all(data)?;
                write!(&mut out_file, "..")?;
                let data = match right {
                    parser::NumberFormat::Hex(range) => {
                        write!(&mut out_file, "0x")?;
                        get_range(content, range)?
                    },
                    parser::NumberFormat::Decimal(range) => get_range(content, range)?,
                };
                out_file.write_all(data)?;
                remaining_elems -= 1;
                
                if remaining_elems > 0 {
                    write!(&mut out_file, ", ")?;
                }
            },
            parser::SyntaxNode::StartBlock => write!(&mut out_file, "(")?,
            parser::SyntaxNode::BlockSeparator => write!(&mut out_file, "||")?,
            parser::SyntaxNode::EndBlock => write!(&mut out_file, ")")?,
        }
        
        if in_rule && !matches!(node, parser::SyntaxNode::StartBlock) && whitespace_separated(stream.get(i + 1)) {
            write!(&mut out_file, " ")?;
        }
        
        i += 1;
    }
    
    Ok(())
}

fn verify(grammar: String) -> Result<()> {
    let mut parser = parser::GrammarParser::new();
    let content = std::fs::read_to_string(&grammar)?;
    parser.parse(&content)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Format { grammar } => beautify(grammar),
        Commands::Check { grammar } => verify(grammar),
    }
}
