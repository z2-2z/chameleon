use std::collections::{HashMap, HashSet};
use std::borrow::ToOwned;
use std::ops::RangeInclusive;
use crate::grammar::{Terminal as CfgTerminal, Numberset as CfgNumberset, ContextFreeGrammar, Symbol as CfgSymbol};

#[derive(Debug, PartialEq, Eq)]
pub struct NonTerminal(usize);

impl NonTerminal {
    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub enum NumbersetType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
}

impl NumbersetType {
    pub fn c_type(&self) -> &str {
        match self {
            NumbersetType::U8 => "uint8_t",
            NumbersetType::I8 => "int8_t",
            NumbersetType::U16 => "uint16_t",
            NumbersetType::I16 => "int16_t",
            NumbersetType::U32 => "uint32_t",
            NumbersetType::I32 => "int32_t",
            NumbersetType::U64 => "uint64_t",
            NumbersetType::I64 => "int64_t",
        }
    }
}

#[derive(Debug)]
pub struct Numberset {
    typ: NumbersetType,
    set: HashSet<RangeInclusive<u64>>,
}

impl Numberset {
    pub fn typ(&self) -> &NumbersetType {
        &self.typ
    }
    
    pub fn set(&self) -> &HashSet<RangeInclusive<u64>> {
        &self.set
    }
}

impl From<&CfgNumberset> for Numberset {
    fn from(value: &CfgNumberset) -> Self {
        match value {
            CfgNumberset::I8(ranges) => Self {
                typ: NumbersetType::I8,
                set: ranges.iter().map(|r| RangeInclusive::new(*r.start() as u8 as u64, *r.end() as u8 as u64)).collect(),
            },
            CfgNumberset::U8(ranges) => Self {
                typ: NumbersetType::U8,
                set: ranges.iter().map(|r| RangeInclusive::new(*r.start() as u64, *r.end() as u64)).collect(),
            },
            CfgNumberset::I16(ranges) => Self {
                typ: NumbersetType::I16,
                set: ranges.iter().map(|r| RangeInclusive::new(*r.start() as u16 as u64, *r.end() as u16 as u64)).collect(),
            },
            CfgNumberset::U16(ranges) => Self {
                typ: NumbersetType::U16,
                set: ranges.iter().map(|r| RangeInclusive::new(*r.start() as u64, *r.end() as u64)).collect(),
            },
            CfgNumberset::I32(ranges) => Self {
                typ: NumbersetType::I32,
                set: ranges.iter().map(|r| RangeInclusive::new(*r.start() as u32 as u64, *r.end() as u32 as u64)).collect(),
            },
            CfgNumberset::U32(ranges) => Self {
                typ: NumbersetType::U32,
                set: ranges.iter().map(|r| RangeInclusive::new(*r.start() as u64, *r.end() as u64)).collect(),
            },
            CfgNumberset::I64(ranges) => Self {
                typ: NumbersetType::I64,
                set: ranges.iter().map(|r| RangeInclusive::new(*r.start() as u64, *r.end() as u64)).collect(),
            },
            CfgNumberset::U64(ranges) => Self {
                typ: NumbersetType::U64,
                set: ranges.iter().cloned().collect(),
            },
        }
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
    fn insert_sorted(&mut self, rhs: Vec<Symbol>) {
        match self.rules.binary_search_by(|x| x.len().cmp(&rhs.len())) {
            Ok(idx) | Err(idx) => self.rules.insert(idx, rhs)
        }
    }
    
    pub fn nonterm(&self) -> &NonTerminal {
        &self.nonterm
    }
    
    pub fn rules(&self) -> &[Vec<Symbol>] {
        &self.rules
    }
    
    pub fn has_nonterms(&self) -> bool {
        for rule in &self.rules {
            for symbol in rule {
                if matches!(symbol, Symbol::NonTerminal(_)) {
                    return true;
                }
            }
        }
        
        false
    }
    
    pub fn has_terms(&self) -> bool {
        for rule in &self.rules {
            for symbol in rule {
                if matches!(symbol, Symbol::Terminal(_)) {
                    return true;
                }
            }
        }
        
        false
    }
    
    pub fn has_no_symbols(&self) -> bool {
        for rule in &self.rules {
            if !rule.is_empty() {
                return false;
            }
        }
        
        true
    }
    
    pub fn is_triangular(&self) -> bool {
        let mut sim_score = 0;
        let mut prev = usize::MAX;
        
        for rule in &self.rules {
            if prev == rule.len() {
                sim_score += 1;
            }
            
            prev = rule.len();
        }
        
        (100 * sim_score / self.rules.len()) <= 25
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
    numbersets: HashMap<&'a CfgNumberset, usize>,
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
    
    fn numberset_id(&mut self, numberset: &'a CfgNumberset) -> usize {
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
                        if items.is_empty() {
                            continue;
                        }
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
                rule.insert_sorted(rhs);
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
        let numbersets = self.numbersets.into_iter().map(|(k, v)| (v, Numberset::from(k))).collect();
        let mut max_num_rules = 0;
        let mut step_type = "uint8_t";
        
        for ruleset in &self.rules {
            if ruleset.is_triangular() {
                max_num_rules = std::cmp::max(max_num_rules, ruleset.rules().len());
            }
            
            if ruleset.rules().len() >= 256 {
                step_type = "uint16_t";
            }
        }
        
        TranslatorGrammar {
            entrypoint: NonTerminal(entrypoint),
            rules: self.rules,
            numbersets,
            nonterminals: reverse_id_map(self.nonterms),
            terminals: reverse_id_map(self.terminals),
            max_num_rules,
            step_type,
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
    max_num_rules: usize,
    step_type: &'static str,
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
    
    pub fn numbersets(&self) -> &HashMap<usize, Numberset> {
        &self.numbersets
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
    
    pub fn terminals(&self) -> &HashMap<usize, Vec<u8>> {
        &self.terminals
    }
    
    pub fn max_num_of_rules(&self) -> usize {
        self.max_num_rules
    }
    
    pub fn step_type(&self) -> &str {
        self.step_type
    }
}
