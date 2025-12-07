use std::collections::HashSet;
use crate::grammar::tokenizer::{Token, NumberType, TextMetadata};

pub struct TokenPostProcessor {
    remove: HashSet<usize>,
    cursor: usize,
}

impl TokenPostProcessor {
    pub fn new() -> Self {
        Self {
            remove: HashSet::default(),
            cursor: 0,
        }
    }
    
    pub fn process(mut self, tokens: &mut Vec<Token>) {
        /* First, clean the token stream */
        self.clean_groups(tokens);
        self.reorder_number_ranges(tokens);
        self.clean_numbersets(tokens);
        self.purge(tokens);
        
        /* Then, desugar grammar */
        self.remove_groups(tokens);
        self.split_ors(tokens);
    }
    
    fn clean_groups(&mut self, tokens: &[Token]) {
        let mut stack = Vec::new();
        
        for (i, token) in tokens.iter().enumerate() {
            match token {
                Token::StartGroup => stack.push((i, false)),
                Token::Or => stack.last_mut().unwrap().1 = true,
                Token::EndGroup => {
                    let last = stack.pop().unwrap();
                    
                    if !last.1 {
                        self.remove.insert(last.0);
                        self.remove.insert(i);
                    }
                },
                _ => {},
            }
        }
    }
    
    fn reorder_number_ranges(&mut self, tokens: &mut Vec<Token>) {
        let mut latest_typ = NumberType::U8;
        
        for token in tokens {
            match token {
                Token::StartNumberset(typ) => latest_typ = typ.clone(),
                Token::NumberRange(start, end) => {
                    macro_rules! typed_swap {
                        ($s:ty, $u:ty) => {{
                            let a = *start as $s;
                            let b = *end as $s;
                            *start = std::cmp::min(a, b) as $u as u64;
                            *end = std::cmp::max(a, b) as $u as u64;
                        }}
                    }
                    
                    match &latest_typ {
                        NumberType::I8 => typed_swap!(i8, u8),
                        NumberType::I16 => typed_swap!(i16, u16),
                        NumberType::I32 => typed_swap!(i32, u32),
                        NumberType::I64 => typed_swap!(i64, u64),
                        _ => if start > end {
                            std::mem::swap(&mut *start, &mut *end);
                        },
                    }
                },
                _ => {},
            }
        }
    }
    
    fn clean_numbersets(&mut self, tokens: &[Token]) {
        let mut start = 0;
        
        while let Some(token) = tokens.get(start) {
            if let Token::StartNumberset(_) = token {
                let mut end = start + 1;
                
                while !matches!(&tokens[end], Token::EndNumberset) {
                    end += 1;
                }
                
                self.deduplicate_numberset(start + 1, &tokens[start + 1..end]);
                start = end;
            }
            
            start += 1;
        }
    }
    
    fn deduplicate_numberset(&mut self, base: usize, set: &[Token]) {
        assert!(!set.is_empty());
        let mut seen: HashSet<(u64, u64)> = HashSet::default();
        
        for (i, token) in set.iter().enumerate() {
            let Token::NumberRange(start, end) = token else { unreachable!() };
            let elem = (*start, *end);
            
            if !seen.insert(elem) {
                self.remove.insert(base + i);
            }
        }
    }
    
    fn purge(&mut self, tokens: &mut Vec<Token>) {
        let mut remove: Vec<&usize> = self.remove.iter().collect();
        
        remove.sort_by(|a, b| (*b).cmp(*a));
        
        for idx in remove {
            tokens.remove(*idx);
        }
    }
    
    fn new_nonterm(&mut self) -> String {
        let nonterm = format!("(group {})", self.cursor);
        self.cursor += 1;
        nonterm
    }
    
    fn remove_groups(&mut self, tokens: &mut Vec<Token>) {
        let mut extra_tokens = Vec::new();
        let mut stack = Vec::new();
        let mut i = 0;
        
        while i < tokens.len() {
            match &tokens[i] {
                Token::StartGroup => stack.push(i),
                Token::EndGroup => {
                    let start = stack.pop().unwrap();
                    let nonterm = self.new_nonterm();
                    let mut group: Vec<Token> = tokens.splice(start..i + 1, [
                        Token::NonTerminal(TextMetadata {
                            line: 0,
                            column: 0,
                        },
                        nonterm.clone())
                    ]).collect();
                    
                    *group.first_mut().unwrap() = Token::StartRule(nonterm);
                    *group.last_mut().unwrap() = Token::EndRule;
                    extra_tokens.extend(group);
                    
                    i = start;
                },
                _ => {},
            }
            
            i += 1;
        }
        
        tokens.extend(extra_tokens);
    }
    
    fn split_ors(&mut self, tokens: &mut Vec<Token>) {
        let mut extra_tokens = Vec::new();
        let mut i = 0;
        let mut start_rule = 0;
        let mut first_or = None;
        
        while i < tokens.len() {
            match &tokens[i] {
                Token::StartRule(_) => {
                    start_rule = i;
                    first_or = None;
                },
                Token::Or => if first_or.is_none() {
                    first_or = Some(i);
                },
                Token::EndRule => if let Some(j) = first_or {
                    let rest: Vec<Token> = tokens.splice(j..i, []).collect();
                    
                    for subgroup in rest.split(|x| matches!(x, Token::Or)) {
                        if !subgroup.is_empty() {
                            extra_tokens.push(tokens[start_rule].clone());
                            extra_tokens.extend_from_slice(subgroup);
                            extra_tokens.push(Token::EndRule);
                        }
                    }
                    
                    i = j;
                },
                _ => {},
            }
            
            i += 1;
        }
        
        tokens.extend(extra_tokens);
    }
}
