use std::collections::HashMap;
use std::borrow::ToOwned;
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
    Bytes(usize),
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

impl RuleSet {
    pub fn nonterm(&self) -> &NonTerminal {
        &self.nonterm
    }
    
    pub fn rules(&self) -> &[Vec<Symbol>] {
        &self.rules
    }
}

#[inline]
fn reverse_id_map<T: ToOwned + ?Sized>(map: HashMap<&T, usize>) -> HashMap<usize, T::Owned> {
    map.into_iter().map(|(k, v)| (v, k.to_owned())).collect()
}

pub struct TranslatorGrammarConverter<'a> {
    nonterm_cursor: usize,
    nonterms: HashMap<&'a str, usize>,
    rules: Vec<RuleSet>,
    numberset_cursor: usize,
    numbersets: HashMap<&'a Numberset, usize>,
    terminal_cursor: usize,
    terminals: HashMap<&'a Vec<u8>, usize>,
}

impl<'a>  TranslatorGrammarConverter<'a> {
    fn new() -> Self {
        Self {
            nonterm_cursor: 0,
            nonterms: HashMap::default(),
            rules: Vec::new(),
            numberset_cursor: 0,
            numbersets: HashMap::default(),
            terminal_cursor: 0,
            terminals: HashMap::default(),
        }
    }
    
    fn nonterm_id(&mut self, nonterm: &'a str) -> usize {
        if let Some(id) = self.nonterms.get(nonterm) {
            *id
        } else {
            let id = self.nonterm_cursor;
            self.nonterm_cursor += 1;
            self.nonterms.insert(nonterm, id);
            id
        }
    }
    
    fn numberset_id(&mut self, numberset: &'a Numberset) -> usize {
        if let Some(id) = self.numbersets.get(numberset) {
            *id
        } else {
            let id = self.numberset_cursor;
            self.numberset_cursor += 1;
            self.numbersets.insert(numberset, id);
            id
        }
    }
    
    fn terminal_id(&mut self, terminal: &'a Vec<u8>) -> usize {
        if let Some(id) = self.terminals.get(terminal) {
            *id
        } else {
            let id = self.terminal_cursor;
            self.terminal_cursor += 1;
            self.terminals.insert(terminal, id);
            id
        }
    }
    
    fn convert_rhs(&mut self, rhs: &'a [CfgSymbol]) -> Vec<Symbol> {
        let mut converted = Vec::new();
        
        for symbol in rhs {
            let symbol = match symbol {
                CfgSymbol::Terminal(terminal) => match terminal {
                    CfgTerminal::Bytes(items) => {
                        let id = self.terminal_id(items);
                        Symbol::Terminal(Terminal::Bytes(id))
                    },
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
            numbersets: reverse_id_map(self.numbersets),
            nonterminals: reverse_id_map(self.nonterms),
            terminals: reverse_id_map(self.terminals),
        }
    }
}

#[derive(Debug)]
pub struct TranslatorGrammar {
    entrypoint: NonTerminal,
    rules: Vec<RuleSet>,
    numbersets: HashMap<usize, Numberset>,
    nonterminals: HashMap<usize, String>,
    terminals: HashMap<usize, Vec<u8>>,
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
    
    pub fn numberset(&self, id: usize) -> &Numberset {
        self.numbersets.get(&id).unwrap()
    }
    
    pub fn nonterminal(&self, id: usize) -> &str {
        self.nonterminals.get(&id).unwrap()
    }
    
    pub fn nonterminals(&self) -> &HashMap<usize, String> {
        &self.nonterminals
    }
    
    pub fn terminal(&self, id: usize) -> &[u8] {
        self.terminals.get(&id).unwrap()
    }
    
    pub fn terminals(&self) -> &HashMap<usize, Vec<u8>> {
        &self.terminals
    }
}
