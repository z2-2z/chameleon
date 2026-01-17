use crate::translator::TranslatorGrammar;
use askama::Template;
use anyhow::Result;
use std::path::PathBuf;

#[derive(askama::Template)]
#[template(path = "head.c", escape = "none")]
struct Head<'a> {
    grammar: &'a TranslatorGrammar,
}

#[derive(askama::Template)]
#[template(path = "numbersets.c", escape = "none")]
struct Numbersets<'a> {
    grammar: &'a TranslatorGrammar,
}

pub mod baby {
    use super::*;
    
    #[derive(askama::Template)]
    #[template(path = "baby/header.h", escape = "none")]
    pub(super) struct Header<'a> {
        prefix: &'a str,
    }
    
    #[derive(askama::Template)]
    #[template(path = "baby/generators.c", escape = "none")]
    pub(super) struct Generators<'a> {
        grammar: &'a TranslatorGrammar,
    }

    #[derive(askama::Template)]
    #[template(path = "baby/root.c", escape = "none")]
    pub(super) struct Root<'a> {
        grammar: &'a TranslatorGrammar,
        numbersets: Numbersets<'a>,
        generators: Generators<'a>,
        head: Head<'a>,
        prefix: &'a str,
    }
    
    pub fn render<P: Into<PathBuf>>(grammar: TranslatorGrammar, arg_prefix: Option<String>, output: P) -> Result<()> {
        let mut output = output.into();
        let prefix = if let Some(p) = arg_prefix.as_ref() {
            p
        } else {
            chameleon::DEFAULT_PREFIX
        };
        let root = Root {
            grammar: &grammar,
            numbersets: Numbersets {
                grammar: &grammar,
            },
            generators: Generators {
                grammar: &grammar,
            },
            head: Head {
                grammar: &grammar,
            },
            prefix,
        };
        let source = root.render()?;
        
        std::fs::write(&output, source)?;
        
        if arg_prefix.is_some() {
            let header = Header {
                prefix,
            };
            let source = header.render()?;
            
            output.set_extension("h");
            std::fs::write(&output, source)?;
        }
        
        Ok(())
    }
}

pub mod full {
    use super::*;
    
    #[derive(askama::Template)]
    #[template(path = "full/header.h", escape = "none")]
    pub struct Header<'a> {
        prefix: &'a str,
    }
    
    #[derive(askama::Template)]
    #[template(path = "full/mutations.c", escape = "none")]
    pub struct Mutations<'a> {
        grammar: &'a TranslatorGrammar,
    }

    #[derive(askama::Template)]
    #[template(path = "full/root.c", escape = "none")]
    pub struct Root<'a> {
        grammar: &'a TranslatorGrammar,
        numbersets: Numbersets<'a>,
        mutations: Mutations<'a>,
        head: Head<'a>,
        prefix: &'a str,
    }
    
    pub fn render<P: Into<PathBuf>>(grammar: TranslatorGrammar, arg_prefix: Option<String>, output: P) -> Result<()> {
        let mut output = output.into();
        let prefix = if let Some(p) = arg_prefix.as_ref() {
            p
        } else {
            chameleon::DEFAULT_PREFIX
        };
        let root = Root {
            grammar: &grammar,
            numbersets: Numbersets {
                grammar: &grammar,
            },
            mutations: Mutations {
                grammar: &grammar,
            },
            head: Head {
                grammar: &grammar,
            },
            prefix,
        };
        let source = root.render()?;
        
        std::fs::write(&output, source)?;
        
        if arg_prefix.is_some() {
            let header = Header {
                prefix,
            };
            let source = header.render()?;
            
            output.set_extension("h");
            std::fs::write(&output, source)?;
        }
        
        Ok(())
    }
}
