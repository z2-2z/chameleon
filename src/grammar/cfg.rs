use std::ops::RangeInclusive;
use crate::grammar::builder::GrammarBuilder;

#[derive(Debug)]
pub struct NonTerminal(pub(super) String);

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
}

impl ContextFreeGrammar {
    pub fn builder() -> GrammarBuilder {
        GrammarBuilder::new()
    }
}
