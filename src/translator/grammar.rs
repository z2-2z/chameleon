use std::collections::HashMap;
use crate::grammar::{Terminal as CfgTerminal, Numberset, ContextFreeGrammar, Symbol as CfgSymbol};

#[derive(Debug, PartialEq, Eq)]
pub struct NonTerminal(usize);

impl NonTerminal {
    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub enum Terminal {
    Numberset(usize),
    Bytes(Vec<u8>),
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
    nonterm_cursor: usize,
    mapping: HashMap<&'a str, usize>,
    rules: Vec<RuleSet>,
    numberset_cursor: usize,
    numbersets: HashMap<Numberset, usize>,
}

impl<'a>  TranslatorGrammarConverter<'a> {
    fn new() -> Self {
        Self {
            nonterm_cursor: 0,
            mapping: HashMap::default(),
            rules: Vec::new(),
            numberset_cursor: 0,
            numbersets: HashMap::default(),
        }
    }
    
    fn nonterm_id(&mut self, nonterm: &'a str) -> usize {
        if let Some(id) = self.mapping.get(nonterm) {
            *id
        } else {
            let id = self.nonterm_cursor;
            self.nonterm_cursor += 1;
            self.mapping.insert(nonterm, id);
            id
        }
    }
    
    fn numberset_id(&mut self, numberset: &'a Numberset) -> usize {
        if let Some(id) = self.numbersets.get(numberset) {
            *id
        } else {
            let id = self.numberset_cursor;
            self.numberset_cursor += 1;
            self.numbersets.insert(numberset.clone(), id);
            id
        }
    }
    
    fn convert_rhs(&mut self, rhs: &'a [CfgSymbol]) -> Vec<Symbol> {
        let mut converted = Vec::new();
        
        for symbol in rhs {
            let symbol = match symbol {
                CfgSymbol::Terminal(terminal) => match terminal {
                    CfgTerminal::Bytes(items) => Symbol::Terminal(Terminal::Bytes(items.clone())),
                    CfgTerminal::Numberset(numberset) => {
                        let id = self.numberset_id(numberset);
                        Symbol::Terminal(Terminal::Numberset(id))
                    },
                },
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
            numbersets: self.numbersets,
        }
    }
}

#[derive(Debug)]
pub struct TranslatorGrammar {
    entrypoint: NonTerminal,
    rules: Vec<RuleSet>,
    numbersets: HashMap<Numberset, usize>,
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
    
    pub fn numbersets(&self) -> &HashMap<Numberset, usize> {
        &self.numbersets
    }
}
