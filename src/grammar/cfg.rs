use std::ops::RangeInclusive;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, RandomState, BuildHasher};
use petgraph::{graph::DiGraph, visit::Bfs};
use nohash::{IntSet as NoHashSet, IntMap as NoHashMap};
use crate::grammar::builder::GrammarBuilder;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct NonTerminal(pub(super) String);

impl NonTerminal {
    pub fn name(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Numberset {
    I8(Vec<RangeInclusive<i8>>),
    U8(Vec<RangeInclusive<u8>>),
    I16(Vec<RangeInclusive<i16>>),
    U16(Vec<RangeInclusive<u16>>),
    I32(Vec<RangeInclusive<i32>>),
    U32(Vec<RangeInclusive<u32>>),
    I64(Vec<RangeInclusive<i64>>),
    U64(Vec<RangeInclusive<u64>>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Terminal {
    Bytes(Vec<u8>),
    Numberset(Numberset),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Symbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ProductionRule {
    pub(super) lhs: NonTerminal,
    pub(super) rhs: Vec<Symbol>,
}

impl ProductionRule {
    fn is_left_recursive(&self) -> bool {
        if let Some(Symbol::NonTerminal(nonterm)) = self.rhs.first() && &self.lhs == nonterm {
            true
        } else {
            false
        }
    }
    
    fn is_in_gnf(&self) -> bool {
        matches!(&self.rhs[0], Symbol::Terminal(_))
    }
    
    pub fn lhs(&self) -> &NonTerminal {
        &self.lhs
    }
    
    pub fn rhs(&self) -> &[Symbol] {
        &self.rhs
    }
}

#[derive(Debug)]
pub struct ContextFreeGrammar {
    entrypoint: NonTerminal,
    rules: Vec<ProductionRule>,
    unused_nonterms: HashSet<String>,
}

impl ContextFreeGrammar {
    pub fn builder() -> GrammarBuilder {
        GrammarBuilder::new()
    }
    
    pub fn unused_nonterms(&self) -> &HashSet<String> {
        &self.unused_nonterms
    }
    
    pub fn rules(&self) -> &[ProductionRule] {
        &self.rules
    }
    
    pub fn entrypoint(&self) -> &NonTerminal {
        &self.entrypoint
    }
    
    pub fn grammar_size(&self) -> usize {
        let mut size = 0;
        
        for rule in &self.rules {
            size += rule.rhs.len();
        }
        
        size
    }
    
    pub(super) fn new(entrypoint: NonTerminal, rules: Vec<ProductionRule>) -> Self {
        Self {
            entrypoint,
            rules,
            unused_nonterms: HashSet::default(),
        }
    }
    
    pub(super) fn remove_unused_rules(&mut self, log: bool) {
        /* Build graph */
        let mut graph = DiGraph::<&str, ()>::new();
        let mut nodes = HashMap::new();
        let mut unused_nonterms = HashSet::new();
        
        for rule in &self.rules {
            unused_nonterms.insert(rule.lhs.name().to_owned());
            
            for symbol in &rule.rhs {
                if let Symbol::NonTerminal(nonterm) = symbol {
                    let src = *nodes.entry(rule.lhs.name()).or_insert_with(|| graph.add_node(rule.lhs.name()));
                    let dst = *nodes.entry(nonterm.name()).or_insert_with(|| graph.add_node(nonterm.name()));
                    graph.update_edge(src, dst, ());
                }
            }
        }
        
        /* Search graph */
        let Some(entry) = nodes.get(self.entrypoint.name()) else {
            return;
        };
        let mut bfs = Bfs::new(&graph, *entry);
        
        while let Some(node) = bfs.next(&graph) {
            unused_nonterms.remove(graph[node]);
        }
        
        /* Remove unused rules */
        let mut i = 0;
        
        while i < self.rules.len() {
            if unused_nonterms.contains(self.rules[i].lhs.name()) {
                self.rules.remove(i);
            } else {
                i += 1;
            }
        }
        
        if log {
            self.unused_nonterms.extend(unused_nonterms);
        }
    }
    
    pub(super) fn remove_duplicate_rules(&mut self) {
        let mut rules = NoHashSet::default();
        let hasher = RandomState::new();
        let mut i = 0;
        
        while i < self.rules.len() {
            if !rules.insert(hasher.hash_one(&self.rules[i])) {
                self.rules.remove(i);
            } else {
                i += 1;
            }
        }
    }
    
    pub(super) fn expand_unit_rules(&mut self) {
        'outer:
        loop {
            let old_len = self.rules.len();
            
            for i in 0..old_len {
                if let Symbol::NonTerminal(nonterm) = &self.rules[i].rhs[0] && self.rules[i].rhs.len() == 1 {
                    assert_ne!(&self.rules[i].lhs, nonterm);
                    
                    let nonterm = nonterm.clone();
                    
                    for j in 0..old_len {
                        if self.rules[j].lhs == nonterm {
                            let new_rule = ProductionRule {
                                lhs: self.rules[i].lhs.clone(),
                                rhs: self.rules[j].rhs.clone(),
                            };
                            self.rules.push(new_rule);
                        }
                    }
                    
                    self.rules.remove(i);
                    continue 'outer;
                }
            }
            
            break;
        }
    }
    
    fn find_single_terminal_rules(&self) -> HashSet<NonTerminal> {
        let mut counts: HashMap<&NonTerminal, usize> = HashMap::default();
        let mut set: HashSet<&NonTerminal> = HashSet::default();
        
        for rule in &self.rules {
            *counts.entry(rule.lhs()).or_insert(0) += 1;
            
            if rule.rhs().iter().all(|x| matches!(x, Symbol::Terminal(_))) {
                set.insert(rule.lhs());
            }
        }
        
        set.iter().filter(|x| *counts.get(*x).unwrap() == 1).map(|x| (*x).clone()).collect()
    }
    
    fn remove_single_rule(&mut self, nonterm: &NonTerminal) -> ProductionRule {
        for i in 0..self.rules.len() {
            if self.rules[i].lhs() == nonterm {
                return self.rules.remove(i);
            }
        }
        
        unreachable!()
    }
    
    fn replace_single_rule(&mut self, nonterm: NonTerminal, symbols: &[Symbol]) {
        for rule in &mut self.rules {
            for i in 0..rule.rhs.len() {
                if let Symbol::NonTerminal(n) = &rule.rhs()[i] && n == &nonterm {
                    rule.rhs.splice(i..i+1, symbols.to_owned());
                }
            }
        }
    }
    
    pub(super) fn terminal_substitution(&mut self) {
        loop {
            let old_len = self.rules.len();
            
            for nonterm in self.find_single_terminal_rules() {
                let rule = self.remove_single_rule(&nonterm);
                self.replace_single_rule(nonterm, rule.rhs());
            }
            
            if self.rules.len() == old_len {
                break;
            }
        }
    }
    
    fn prune_empty_strings(&mut self) {
        for rule in &mut self.rules {
            if rule.rhs().len() <= 1 {
                continue;
            }
            
            let mut i = 0;
            
            while i < rule.rhs.len() {
                if let Symbol::Terminal(Terminal::Bytes(content)) = &rule.rhs()[i] && content.is_empty() {
                    rule.rhs.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }
    
    fn concat_terminals(&mut self) {
        fn concat(symbols: &[Symbol]) -> Symbol {
            let mut v = Vec::new();
            
            for sym in symbols {
                let Symbol::Terminal(Terminal::Bytes(content)) = sym else { unreachable!() };
                v.extend_from_slice(content);
            }
            
            Symbol::Terminal(Terminal::Bytes(v))
        }
        
        for rule in &mut self.rules {
            let mut i = 0;
            
            while i < rule.rhs.len() {
                if let Symbol::Terminal(Terminal::Bytes(_)) = &rule.rhs[i] {
                    let mut j = i;
                    
                    while j < rule.rhs.len() {
                        if let Symbol::Terminal(Terminal::Bytes(_)) = &rule.rhs[j] {
                            j += 1;
                        } else {
                            break;
                        }
                    }
                    
                    if j > i + 1 {
                        let new = concat(&rule.rhs()[i..j]);
                        rule.rhs.splice(i..j, [new]);
                    }
                }
                
                i += 1;
            }
        }
    }
    
    pub(super) fn process_terminals(&mut self) {
        self.prune_empty_strings();
        self.concat_terminals();
    }
    
    pub(super) fn prepare_gnf(&mut self) {
        let mut nonterms: NoHashMap<u64, NonTerminal> = NoHashMap::default();
        let hasher = RandomState::new();
        let old_len = self.rules.len();
        let mut cursor = 0;
        
        for i in 0..old_len {
            let rule = &self.rules[i];
            
            if rule.rhs.len() == 1 {
                continue;
            }
            
            for j in 1..rule.rhs.len() {
                if let Symbol::Terminal(term) = &self.rules[i].rhs[j] {
                    let term = term.clone();
                    let key = hasher.hash_one(&term);
                    
                    if let Some(nonterm) = nonterms.get(&key) {
                        self.rules[i].rhs[j] = Symbol::NonTerminal(nonterm.clone());
                    } else {
                        let nonterm = NonTerminal(format!("(terminal:{cursor})"));
                        nonterms.insert(key, nonterm.clone());
                        self.rules[i].rhs[j] = Symbol::NonTerminal(nonterm.clone());
                        self.rules.push(ProductionRule {
                            lhs: nonterm,
                            rhs: vec![
                                Symbol::Terminal(term),
                            ],
                        });
                        cursor += 1;
                    }
                }
            }
        }
    }
    
    fn remove_left_recursions(&mut self) {
        let left_recursions = self.direct_left_recursions();
        
        for nonterm in left_recursions {
            self.remove_direct_left_recursion(&nonterm);
        }
    }
    
    fn direct_left_recursions(&self) -> HashSet<NonTerminal> {
        let mut set = HashSet::default();
        
        for rule in &self.rules {
            if rule.is_left_recursive() {
                assert!(rule.rhs.len() > 1);
                set.insert(rule.lhs.clone());
            }
        }
        
        set
    }
    
    fn remove_direct_left_recursion(&mut self, nonterm: &NonTerminal) {
        let new_nonterm = NonTerminal(format!("lr:{}", nonterm.name()));
        
        for rule in self.rules.iter_mut().filter(|x| &x.lhs == nonterm) {
            if rule.is_left_recursive() {
                rule.lhs = new_nonterm.clone();
                rule.rhs.remove(0);
                rule.rhs.push(Symbol::NonTerminal(new_nonterm.clone()));
            } else {
                rule.rhs.push(Symbol::NonTerminal(new_nonterm.clone()));
            }
        }
        
        self.rules.push(ProductionRule {
            lhs: new_nonterm,
            rhs: vec![
                Symbol::Terminal(Terminal::Bytes(vec![])),
            ],
        });
    }
    
    pub(super) fn convert_to_gnf(&mut self) {
        'outer:
        loop {
            self.remove_left_recursions();
            
            for i in 0..self.rules.len() {
                if !self.rules[i].is_in_gnf() {
                    let rule = self.rules.remove(i);
                    self.expand_rule(rule);
                    continue 'outer;
                }
            }
            
            break;
        }
        
        assert!(self.is_in_gnf());
    }
    
    fn expand_rule(&mut self, rule: ProductionRule) {
        let old_len = self.rules.len();
        let Symbol::NonTerminal(nonterm) = &rule.rhs[0] else { unreachable!() };
        
        for i in 0..old_len {
            if &self.rules[i].lhs == nonterm {
                let mut new_rule = ProductionRule {
                    lhs: rule.lhs.clone(),
                    rhs: self.rules[i].rhs.clone(),
                };
                new_rule.rhs.extend_from_slice(&rule.rhs[1..]);
                self.rules.push(new_rule);
            }
        }
    }
    
    pub(super) fn is_in_gnf(&self) -> bool {
        for rule in &self.rules {
            if !matches!(rule.rhs[0], Symbol::Terminal(_)) {
                return false;
            }
            
            for symbol in &rule.rhs[1..] {
                if !matches!(symbol, Symbol::NonTerminal(_)) {
                    return false;
                }
            }
        }
        
        true
    }
}
