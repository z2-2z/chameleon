use std::path::{Path, PathBuf};
use std::ops::Range;
use anyhow::Result;
use thiserror::Error;

type FilterFunc = fn(u8) -> bool;

fn is_nonterminal(c: u8) -> bool {
    c.is_ascii_uppercase() || c == b'-'
}

fn is_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

struct ParserHelper<'a> {
    buffer: &'a [u8],
    cursor: usize,
}

impl<'a> ParserHelper<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            cursor: 0,
        }
    }
    
    fn go_to_end(&mut self) {
        self.cursor = self.buffer.len();
    }
    
    fn has_more_data(&self) -> bool {
        self.cursor < self.buffer.len()
    }
    
    fn position(&self) -> usize {
        self.cursor
    }
    
    fn column(&self) -> usize {
        self.cursor + 1
    }
    
    fn skip(&mut self, func: FilterFunc) {
        while let Some(c) = self.buffer.get(self.cursor) {
            if func(*c) {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }
    
    fn peek(&self, len: usize) -> Option<&'a [u8]> {
        self.buffer.get(self.cursor..self.cursor + len)
    }
    
    fn advance(&mut self, len: usize) {
        self.cursor += len;
    }
    
    fn has(&mut self, data: &[u8]) -> bool {
        if let Some(buf_data) = self.peek(data.len()) && buf_data == data {
            self.advance(data.len());
            true
        } else {
            false
        }
    }
    
    fn data(&self) -> &'a [u8] {
        &self.buffer[self.cursor..]
    }
    
    fn collect(&self, func: FilterFunc) -> &'a [u8] {
        let mut len = 0;
        let start = self.cursor;
        
        while let Some(c) = self.buffer.get(start + len) {
            if func(*c) {
                len += 1;
            } else {
                break;
            }
        }
        
        &self.buffer[start..start + len]
    }
}

#[derive(Error, Debug)]
pub struct SyntaxError {
    description: String,
    lineno: usize,
    column: usize,
    line: Vec<u8>,
    region: Range<usize>,
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.region.len() > 1 {
            writeln!(f, "SyntaxError in line {} columns {}-{}:", self.lineno, self.column, self.column + self.region.len() - 1)?;
        } else {
            writeln!(f, "SyntaxError in line {} column {}:", self.lineno, self.column)?;
        }
        writeln!(f, "> ")?;
        writeln!(f, "> {}", String::from_utf8_lossy(&self.line))?;
        writeln!(f, "> {0:1$}{2:^<3$}", "", self.region.start, "^", self.region.len())?;
        writeln!(f, "{}", self.description)?;
        Ok(())
    }
}

pub enum NodeContent<'a> {
    Comment(&'a [u8]),
    LhsNonTerminal(&'a [u8]),
    EndOfRule,
}

pub struct SyntaxNode<'a> {
    lineno: usize,
    column: usize,
    content: NodeContent<'a>,
}

impl<'a> SyntaxNode<'a> {
    fn comment(lineno: usize, column: usize, data: &'a [u8]) -> Self {
        Self {
            lineno,
            column,
            content: NodeContent::Comment(data),
        }
    }
    
    fn lhs_non_terminal(lineno: usize, column: usize, data: &'a [u8]) -> Self {
        Self {
            lineno,
            column,
            content: NodeContent::LhsNonTerminal(data),
        }
    }
    
    fn end_of_rule(lineno: usize, column: usize) -> Self {
        Self {
            lineno,
            column,
            content: NodeContent::EndOfRule,
        }
    }
}

const START_COMMENT: &[u8] = b"#";
const SIDE_SEPARATOR: &[u8] = b"->";
const RULE_SEPARATOR: &[u8] = b";";

pub struct GrammarParser<'a> {
    nodes: Vec<SyntaxNode<'a>>,
}

impl<'a> GrammarParser<'a> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }
    
    pub fn parse(&mut self, data: &'a str) -> Result<()> {
        let data = data.as_bytes();
        let mut lineno = 0;
        let mut start = 0;
        
        while start < data.len() {
            lineno += 1;
            let mut end = start;
            
            while let Some(c) = data.get(end) {
                if *c == b'\n' {
                    break;
                } else {
                    end += 1;
                }
            }
            
            self.parse_line(&data[start..end], lineno)?;
            start = end + 1;
        }
        
        Ok(())
    }
    
    fn parse_line(&mut self, line: &'a [u8], lineno: usize) -> Result<()> {
        let mut helper = ParserHelper::new(line);
        
        // Each iteration parses one rule definition
        while helper.has_more_data() {
            helper.skip(is_whitespace);
            
            if helper.has(START_COMMENT) {
                helper.skip(is_whitespace);
                self.nodes.push(SyntaxNode::comment(
                    lineno,
                    helper.column(),
                    helper.data(),
                ));
                break;
            }
            
            self.parse_lhs(&mut helper, line, lineno)?;
            
            helper.skip(is_whitespace);
            
            if !helper.has(SIDE_SEPARATOR) {
                return self.syntax_error(
                    format!("Expected '{}'. Got this instead.", std::str::from_utf8(SIDE_SEPARATOR).unwrap()),
                    lineno,
                    &helper,
                    line,
                    1,
                );
            }
            
            self.parse_rhs(&mut helper, line, lineno)?;
        }
        
        Ok(())
    }
    
    fn parse_lhs(&mut self, helper: &mut ParserHelper<'a>, line: &'a [u8], lineno: usize) -> Result<()> {
        let lhs_nonterm = helper.collect(is_nonterminal);
        
        if lhs_nonterm.is_empty() {
            return self.syntax_error(
                "Expected a non-terminal. Got this instead.",
                lineno,
                helper,
                line,
                1,
            );
        } else if lhs_nonterm.starts_with(b"-") {
            return self.syntax_error(
                "Non-terminals are not allowed to start with a hyphen",
                lineno,
                helper,
                line,
                lhs_nonterm.len(),
            );
        } else if lhs_nonterm.ends_with(b"-") {
            return self.syntax_error(
                "Non-terminals are not allowed to end with a hyphen",
                lineno,
                helper,
                line,
                lhs_nonterm.len(),
            );
        }
        
        self.nodes.push(SyntaxNode::lhs_non_terminal(
            lineno,
            helper.column(),
            lhs_nonterm
        ));
        helper.advance(lhs_nonterm.len());
        
        Ok(())
    }
    
    fn parse_rhs(&mut self, helper: &mut ParserHelper<'a>, line: &'a [u8], lineno: usize) -> Result<()> {
        let mut rhs_count = 0;
        
        loop {
            helper.skip(is_whitespace);
            
            match helper.peek(1) {
                None => return self.syntax_error(
                    "Expected the right-hand side of a rule",
                    lineno,
                    helper,
                    line,
                    1,
                ),
                Some(START_COMMENT) => if rhs_count == 0 {
                    return self.syntax_error(
                        "No elements on the right-hand side of this rule",
                        lineno,
                        helper,
                        line,
                        1,
                    );
                } else {
                    helper.advance(1);
                    helper.skip(is_whitespace);
                    self.nodes.push(SyntaxNode::comment(
                        lineno,
                        helper.column(),
                        helper.data(),
                    ));
                    helper.go_to_end();
                    self.nodes.push(SyntaxNode::end_of_rule(
                        lineno,
                        helper.column(),
                    ));
                    break;
                },
                Some(RULE_SEPARATOR) => if rhs_count == 0 {
                    return self.syntax_error(
                        "No elements on the right-hand side of this rule",
                        lineno,
                        helper,
                        line,
                        1,
                    );
                } else {
                    self.nodes.push(SyntaxNode::end_of_rule(
                        lineno,
                        helper.column(),
                    ));
                    helper.advance(1);
                    break;
                },
                _ => {
                    todo!("non-terminal");
                },
            }
            
            rhs_count += 1;
        }
        
        Ok(())
    }
    
    fn syntax_error<S: Into<String>>(&self, description: S, lineno: usize, helper: &ParserHelper, line: &[u8], region_len: usize) -> Result<()> {
        Err(SyntaxError {
            description: description.into(),
            lineno,
            column: helper.column(),
            line: line.to_vec(),
            region: helper.position()..helper.position() + region_len,
        }.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser() {
        let mut parser = GrammarParser::new();
        parser.parse("   ASDF-ASDF -> ; asdf\nasdf asd fasd fasd fsadf ").unwrap();
    }
}
