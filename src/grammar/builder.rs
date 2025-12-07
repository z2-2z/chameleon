use crate::grammar::{
    cfg::{ContextFreeGrammar, ProductionRule, NonTerminal, Symbol, Terminal, Numberset},
    tokenizer::{Tokenizer, Token, TextMetadata, ParsingError, NumberType},
    syntax,
    post::TokenPostProcessor,
};
use anyhow::Result;
use thiserror::Error;
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;

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
        
        let mut post = TokenPostProcessor::new();
        for tokens in self.tokens.values_mut() {
            post.process(tokens);
        }
        
        /* Convert tokens */
        let mut rules = Vec::new();
        
        for tokens in self.tokens.values() {
            let mut start = 0;
            
            for (i, token) in tokens.iter().enumerate() {
                match token {
                    Token::StartRule(_) => start = i,
                    Token::EndRule => {
                        rules.push(self.convert_rule(&tokens[start..i]));
                    },
                    _ => {},
                }
            }
        }
        
        Ok(ContextFreeGrammar {
            rules,
            entrypoint: NonTerminal(syntax::ENTRYPOINT_RULE.to_owned()),
        })
    }
    
    fn convert_rule(&self, tokens: &[Token]) -> ProductionRule {
        /* Left-hand side */
        let Token::StartRule(nonterm) = &tokens[0] else { unreachable!() };
        let lhs = NonTerminal(nonterm.clone());
        
        /* Right-hand side */
        let mut rhs = Vec::new();
        let mut start = 0;
        
        for (i, token) in tokens[1..].iter().enumerate() {
            match token {
                Token::NonTerminal(_, name) => rhs.push(Symbol::NonTerminal(NonTerminal(name.clone()))),
                Token::String(content) => rhs.push(Symbol::Terminal(Terminal::Bytes(content.clone()))),
                Token::StartNumberset(_) => start = 1 + i,
                Token::NumberRange(_, _) => {},
                Token::EndNumberset => {
                    let numberset = self.convert_numberset(&tokens[start..1 + i]);
                    rhs.push(Symbol::Terminal(Terminal::Numberset(numberset)));
                },
                _ => unreachable!(),
            }
        }
        
        ProductionRule {
            lhs,
            rhs,
        }
    }
    
    fn convert_numberset(&self, tokens: &[Token]) -> Numberset {
        macro_rules! convert_typed {
            ($t:ty, $v:ident) => {{
                let mut set = Vec::new();
                
                for token in &tokens[1..] {
                    let Token::NumberRange(start, end) = token else { unreachable!() };
                    set.push(RangeInclusive::new(*start as $t, *end as $t));
                }
                
                Numberset::$v(set)
            }}
        }
        
        let Token::StartNumberset(typ) = &tokens[0] else { unreachable!() };
        
        match typ {
            NumberType::U8 => convert_typed!(u8, U8),
            NumberType::I8 => convert_typed!(i8, I8),
            NumberType::U16 => convert_typed!(u16, U16),
            NumberType::I16 => convert_typed!(i16, I16),
            NumberType::U32 => convert_typed!(u32, U32),
            NumberType::I32 => convert_typed!(i32, I32),
            NumberType::U64 => convert_typed!(u64, U64),
            NumberType::I64 => convert_typed!(i64, I64),
        }
    }
    
    pub fn check(&self) -> Result<(), BuilderError> {
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
