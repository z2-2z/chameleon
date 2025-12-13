use std::ops::RangeInclusive;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, RandomState, BuildHasher};
use petgraph::{graph::DiGraph, visit::Bfs};
use nohash::{IntSet as NoHashSet, IntMap as NoHashMap};
use crate::grammar::builder::GrammarBuilder;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct NonTerminal(pub(super) String);

impl NonTerminal {
    pub fn id(&self) -> &str {
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
        if let Symbol::NonTerminal(nonterm) = &self.rhs[0] && &self.lhs == nonterm {
            true
        } else {
            false
        }
    }
    
    fn is_in_gnf(&self) -> bool {
        matches!(&self.rhs[0], Symbol::Terminal(_))
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
    
    pub(super) fn remove_unused_rules(&mut self) {
        /* Build graph */
        let mut graph = DiGraph::<&str, ()>::new();
        let mut nodes = HashMap::new();
        
        for rule in &self.rules {
            self.unused_nonterms.insert(rule.lhs.id().to_owned());
            
            for symbol in &rule.rhs {
                if let Symbol::NonTerminal(nonterm) = symbol {
                    let src = *nodes.entry(rule.lhs.id()).or_insert_with(|| graph.add_node(rule.lhs.id()));
                    let dst = *nodes.entry(nonterm.id()).or_insert_with(|| graph.add_node(nonterm.id()));
                    graph.update_edge(src, dst, ());
                }
            }
        }
        
        /* Search graph */
        let Some(entry) = nodes.get(self.entrypoint.id()) else {
            return;
        };
        let mut bfs = Bfs::new(&graph, *entry);
        
        while let Some(node) = bfs.next(&graph) {
            self.unused_nonterms.remove(graph[node]);
        }
        
        /* Remove unused rules */
        let mut i = 0;
        
        while i < self.rules.len() {
            if self.unused_nonterms.contains(self.rules[i].lhs.id()) {
                self.rules.remove(i);
            } else {
                i += 1;
            }
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
            //self.nlrg_transform(&nonterm);
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
    
    /*
    fn nlrg_transform(&mut self, nonterm: &NonTerminal) {
        let mut non_left_recursive = Vec::new();
        
        for (i, rule) in self.rules.iter().enumerate().filter(|(_, x)| &x.lhs == nonterm) {
            if !rule.is_left_recursive() {
                non_left_recursive.push(i);
            }
        }
        
        if non_left_recursive.len() > 2 {
            let new_nonterm = NonTerminal(format!("nlrg:{}", nonterm.id()));
            
            for i in non_left_recursive {
                self.rules[i].lhs = new_nonterm.clone();
            }
            
            self.rules.push(ProductionRule {
                lhs: nonterm.clone(),
                rhs: vec![
                    Symbol::NonTerminal(new_nonterm)
                ],
            });
        }
    }
    */
    
    fn remove_direct_left_recursion(&mut self, nonterm: &NonTerminal) {
        let new_nonterm = NonTerminal(format!("lr:{}", nonterm.id()));
        
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
    
    pub(super) fn set_new_entrypoint(&mut self) {
        let mut count = 0;
        
        for rule in &self.rules {
            if rule.lhs == self.entrypoint {
                count += 1;
            }
        }
        
        if count > 1 {
            let new_nonterm = NonTerminal("(new entrypoint)".to_string());
            let new_rule = ProductionRule {
                lhs: new_nonterm.clone(),
                rhs: vec![
                    Symbol::NonTerminal(self.entrypoint.clone()),
                ],
            };
            self.rules.push(new_rule);
            self.entrypoint = new_nonterm;
        }
    }
}
