use std::collections::HashMap;
use crate::grammar::{Terminal, ContextFreeGrammar, Symbol as CfgSymbol};

#[derive(Debug, PartialEq, Eq)]
pub struct NonTerminal(usize);

impl NonTerminal {
    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub enum Symbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
}

#[derive(Debug)]
pub struct RuleSet {
    nonterm: NonTerminal,
    rules: Vec<Vec<Symbol>>,
}

pub struct TranslatorGrammarConverter<'a> {
    cursor: usize,
    mapping: HashMap<&'a str, usize>,
    rules: Vec<RuleSet>,
}

impl<'a>  TranslatorGrammarConverter<'a> {
    fn new() -> Self {
        Self {
            cursor: 0,
            mapping: HashMap::default(),
            rules: Vec::new(),
        }
    }
    
    fn nonterm_id(&mut self, nonterm: &'a str) -> usize {
        if let Some(id) = self.mapping.get(nonterm) {
            *id
        } else {
            let id = self.cursor;
            self.cursor += 1;
            self.mapping.insert(nonterm, id);
            id
        }
    }
    
    fn convert_rhs(&mut self, rhs: &'a [CfgSymbol]) -> Vec<Symbol> {
        let mut converted = Vec::new();
        
        for symbol in rhs {
            let symbol = match symbol {
                CfgSymbol::Terminal(terminal) => Symbol::Terminal(terminal.clone()),
                CfgSymbol::NonTerminal(nonterm) => {
                    let id = self.nonterm_id(nonterm.name());
                    Symbol::NonTerminal(NonTerminal(id))
                },
            };
            converted.push(symbol);
        }
        
        converted
    }
    
    fn insert_rule(&mut self, nonterm: usize, rhs: &'a [CfgSymbol]) {
        let nonterm = NonTerminal(nonterm);
        let rhs = self.convert_rhs(rhs);
        
        for rule in &mut self.rules {
            if rule.nonterm == nonterm {
                rule.rules.push(rhs);
                return;
            }
        }
        
        self.rules.push(RuleSet {
            nonterm,
            rules: vec![rhs],
        });
    }
    
    pub fn convert(mut self, cfg: &'a ContextFreeGrammar) -> TranslatorGrammar {
        for rule in cfg.rules() {
            let id = self.nonterm_id(rule.lhs().name());
            self.insert_rule(id, rule.rhs());
        }
        
        let entrypoint = self.nonterm_id(cfg.entrypoint().name());
        
        TranslatorGrammar {
            entrypoint: NonTerminal(entrypoint),
            rules: self.rules,
        }
    }
}

#[derive(Debug)]
pub struct TranslatorGrammar {
    entrypoint: NonTerminal,
    rules: Vec<RuleSet>,
}

impl TranslatorGrammar {
    pub fn converter<'a>() -> TranslatorGrammarConverter<'a> {
        TranslatorGrammarConverter::new()
    }
    
    pub fn entrypoint(&self) -> &NonTerminal {
        &self.entrypoint
    }
    
    pub fn rules(&self) -> &[RuleSet] {
        &self.rules
    }
}
