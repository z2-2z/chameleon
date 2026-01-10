use crate::translator::TranslatorGrammar;
use askama::Template;
use anyhow::Result;
use std::path::PathBuf;

#[derive(askama::Template)]
#[template(path = "baby/generators.c", escape = "none")]
struct Generators<'a> {
    grammar: &'a TranslatorGrammar,
}

#[derive(askama::Template)]
#[template(path = "numbersets.c", escape = "none")]
struct Numbersets<'a> {
    grammar: &'a TranslatorGrammar,
}

#[derive(askama::Template)]
#[template(path = "baby/root.c", escape = "none")]
struct Root<'a> {
    grammar: &'a TranslatorGrammar,
    numbersets: Numbersets<'a>,
    generators: Generators<'a>,
    prefix: &'a str,
}

#[derive(askama::Template)]
#[template(path = "baby/header.h", escape = "none")]
struct Header<'a> {
    prefix: &'a str,
}

pub fn render<P: Into<PathBuf>>(grammar: TranslatorGrammar, arg_prefix: Option<String>, output: P) -> Result<()> {
    let mut output = output.into();
    let prefix = if let Some(p) = arg_prefix.as_ref() {
        p
    } else {
        chameleon::DEFAULT_PREFIX
    };
    let numbersets = Numbersets {
        grammar: &grammar,
    };
    let generators = Generators {
        grammar: &grammar,
    };
    let root = Root {
        grammar: &grammar,
        numbersets,
        generators,
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
