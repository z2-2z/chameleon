use crate::grammar::{
    cfg::ContextFreeGrammar,
    tokenizer::Tokenizer,
};
use anyhow::Result;

pub struct GrammarBuilder {
    
}

impl GrammarBuilder {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn load_grammar(&mut self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let tokens = Tokenizer::new().tokenize(&content)?;
        println!("{:#?}", tokens);
        Ok(())
    }
    
    pub fn build(self) -> ContextFreeGrammar {
        todo!()
    }
}
