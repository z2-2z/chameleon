use crate::grammar::{
    cfg::ContextFreeGrammar,
    tokenizer::{Tokenizer, Token, TextMetadata, ParsingError},
    syntax,
    post::TokenPostProcessor,
};
use anyhow::Result;
use thiserror::Error;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("In '{}' line {}:{}: Non-terminal '{}' does not refer to any defined rule", file, meta.line, meta.column, nonterminal)]
    InvalidNonterminalReference {
        file: String,
        meta: TextMetadata,
        nonterminal: String,
    },
    
    #[error("No entrypoint rule has been defined ('{}{}{}')", syntax::START_NONTERMINAL, syntax::ENTRYPOINT_RULE, syntax::END_NONTERMINAL)]
    MissingEntrypoint,
    
    #[error("Invalid syntax in '{}': {}", file, error)]
    SyntaxError {
        file: String,
        error: ParsingError,
    },
}

pub struct GrammarBuilder {
    tokens: HashMap<String, Vec<Token>>,
}

impl GrammarBuilder {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::default(),
        }
    }
    
    pub fn load_grammar(&mut self, path: &str) -> Result<()> {
        if !self.tokens.contains_key(path) {
            let content = std::fs::read_to_string(path)?;
            
            match Tokenizer::new().tokenize(&content) {
                Ok(tokens) => self.tokens.insert(path.to_owned(), tokens),
                Err(error) => return Err(BuilderError::SyntaxError {
                    file: path.to_owned(),
                    error,
                }.into()),
            };
        }
        
        Ok(())
    }
    
    pub fn build(mut self) -> Result<ContextFreeGrammar> {
        //self.check()?;
        
        for tokens in self.tokens.values_mut() {
            TokenPostProcessor::new().process(tokens);
        }
        
        println!("{:#?}", self.tokens);
        
        todo!()
    }
    
    pub fn check(&self) -> Result<(), BuilderError> {
        println!("{:#?}", self.tokens);
        
        let mut rules: HashSet<&str> = HashSet::new();
        
        for tokens in self.tokens.values() {
            for token in tokens {
                if let Token::StartRule(name) = token {
                    rules.insert(name.as_ref());
                }
            }
        }
        
        /* Check if every reference is valid */
        for (file, tokens) in &self.tokens {
            for token in tokens {
                if let Token::NonTerminal(meta, nonterm) = token && !rules.contains(nonterm.as_str()) {
                    return Err(BuilderError::InvalidNonterminalReference {
                        file: file.clone(),
                        meta: meta.clone(),
                        nonterminal: nonterm.clone(),
                    });
                }
            }
        }
        
        /* Check if there is an entrypoint */
        if !rules.contains(syntax::ENTRYPOINT_RULE) {
            return Err(BuilderError::MissingEntrypoint);
        }
        
        Ok(())
    }
}
