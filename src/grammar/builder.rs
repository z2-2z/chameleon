use crate::grammar::{
    cfg::ContextFreeGrammar,
    tokenizer::Tokenizer,
};

pub struct GrammarBuilder {
    
}

impl GrammarBuilder {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn load_grammar(&mut self, path: &str) {
        let mut t = Tokenizer::new();
        let content = std::fs::read_to_string(path).unwrap();
        let v = t.tokenize(&content);
        println!("{:#?}", v);
    }
    
    pub fn build(self) -> ContextFreeGrammar {
        todo!()
    }
}
