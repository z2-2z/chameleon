use std::ops::RangeInclusive;
use std::collections::{HashMap, HashSet};
use petgraph::{graph::DiGraph, visit::Bfs};
use crate::grammar::builder::GrammarBuilder;

#[derive(Debug)]
pub struct NonTerminal(pub(super) String);

impl NonTerminal {
    pub fn id(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Terminal {
    Bytes(Vec<u8>),
    Numberset(Numberset),
}

#[derive(Debug)]
pub enum Symbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
}

#[derive(Debug)]
pub struct ProductionRule {
    pub(super) lhs: NonTerminal,
    pub(super) rhs: Vec<Symbol>,
}

#[derive(Debug)]
pub struct ContextFreeGrammar {
    pub(super) entrypoint: NonTerminal,
    pub(super) rules: Vec<ProductionRule>,
    unused_rules: HashSet<String>,
}

impl ContextFreeGrammar {
    pub fn builder() -> GrammarBuilder {
        GrammarBuilder::new()
    }
    
    pub(super) fn new(entrypoint: NonTerminal, rules: Vec<ProductionRule>) -> Self {
        Self {
            entrypoint,
            rules,
            unused_rules: HashSet::default(),
        }
    }
    
    pub(super) fn remove_unused_rules(&mut self) {
        /* Build graph */
        let mut graph = DiGraph::<&str, ()>::new();
        let mut nodes = HashMap::new();
        
        for rule in &self.rules {
            self.unused_rules.insert(rule.lhs.id().to_owned());
            
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
            self.unused_rules.remove(graph[node]);
        }
        
        /* Remove unused rules */
        let mut i = 0;
        
        while i < self.rules.len() {
            if self.unused_rules.contains(self.rules[i].lhs.id()) {
                self.rules.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
