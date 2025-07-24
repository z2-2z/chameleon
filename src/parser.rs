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

#[derive(Error, Debug)]
pub struct ParserError {
    description: String,
    lineno: usize,
    column: usize,
    line: Vec<u8>,
    region: Range<usize>,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.region.len() > 1 {
            writeln!(f, "In line {} columns {}-{}:", self.lineno, self.column, self.column + self.region.len() - 1)?;
        } else {
            writeln!(f, "In line {} column {}:", self.lineno, self.column)?;
        }
        writeln!(f, "> ")?;
        writeln!(f, "> {}", String::from_utf8_lossy(&self.line))?;
        writeln!(f, "> {0:1$}{2:^<3$}", "", self.region.start, "^", self.region.len())?;
        writeln!(f, "{}", self.description)?;
        Ok(())
    }
}

struct LineParser<'a> {
    line: &'a [u8],
    cursor: usize,
    offset: usize,
    lineno: usize,
}

impl<'a> LineParser<'a> {
    fn new(line: &'a [u8], lineno: usize, offset: usize) -> Self {
        Self {
            line,
            cursor: 0,
            offset,
            lineno,
        }
    }
    
    fn line(&self) -> &[u8] {
        &self.line
    }
    
    fn lineno(&self) -> usize {
        self.lineno
    }
    
    fn go_to_end(&mut self) {
        self.cursor = self.line.len();
    }
    
    fn has_more_data(&self) -> bool {
        self.cursor < self.line.len()
    }
    
    fn offset(&self) -> usize {
        self.offset + self.cursor
    }
    
    fn skip(&mut self, func: FilterFunc) {
        while let Some(c) = self.line.get(self.cursor) {
            if func(*c) {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }
    
    fn peek(&self, len: usize) -> Option<&'a [u8]> {
        self.line.get(self.cursor..self.cursor + len)
    }
    
    fn advance(&mut self, len: usize) {
        self.cursor = std::cmp::min(self.line.len(), self.cursor + len);
    }
    
    fn recede(&mut self, len: usize) {
        self.cursor = self.cursor.saturating_sub(len);
    }
    
    fn has(&mut self, data: &[u8]) -> bool {
        if let Some(buf_data) = self.peek(data.len()) && buf_data == data {
            self.advance(data.len());
            true
        } else {
            false
        }
    }
    
    fn remaining_data(&self) -> &'a [u8] {
        &self.line[self.cursor..]
    }
    
    fn peek_filter(&self, func: FilterFunc) -> &'a [u8] {
        let mut len = 0;
        let start = self.cursor;
        
        while let Some(c) = self.line.get(start + len) {
            if func(*c) {
                len += 1;
            } else {
                break;
            }
        }
        
        &self.line[start..start + len]
    }
    
    fn peek_filter_escaped(&self, func: FilterFunc) -> Option<&'a [u8]> {
        let mut len = 0;
        let start = self.cursor;
        
        while let Some(c) = self.line.get(start + len) {
            if *c == b'\\' {
                len += 2;
            } else if func(*c) {
                len += 1;
            } else {
                break;
            }
        }
        
        let end_idx = std::cmp::min(self.line.len(), start + len);
        
        if end_idx >= self.line.len() {
            None
        } else {
            Some(&self.line[start..end_idx])
        }
    }
    
    fn error<S: Into<String>>(&self, description: S, region_len: usize) -> Result<()> {
        Err(ParserError {
            description: description.into(),
            lineno: self.lineno,
            column: self.cursor + 1,
            line: self.line.to_vec(),
            region: self.cursor..self.cursor + region_len,
        }.into())
    }
}

#[derive(Debug)]
pub enum SyntaxNode {
    Comment(Range<usize>),
    StartRule(Range<usize>),
    EndRule(usize),
    String(Range<usize>),
    Char(Range<usize>),
}

impl SyntaxNode {
    fn comment(offset: usize, len: usize) -> Self {
        Self::Comment(offset..offset + len)
    }
    
    fn start_rule(offset: usize, len: usize) -> Self {
        Self::StartRule(offset..offset + len)
    }
    
    fn end_rule(offset: usize) -> Self {
        Self::EndRule(offset)
    }
    
    fn string(offset: usize, len: usize) -> Self {
        Self::String(offset..offset + len)
    }
    
    fn char(offset: usize, len: usize) -> Self {
        Self::Char(offset..offset + len)
    }
}

const START_COMMENT: &[u8] = b"#";
const SIDE_SEPARATOR: &[u8] = b"->";
const RULE_SEPARATOR: &[u8] = b";";
const STRING_SEPARATOR: &[u8] = b"\"";
const CHAR_SEPARATOR: &[u8] = b"'";

pub struct GrammarParser {
    nodes: Vec<SyntaxNode>,
}

impl GrammarParser {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }
    
    pub fn nodes(&self) -> &[SyntaxNode] {
        &self.nodes
    }
    
    pub fn parse(&mut self, data: &str) -> Result<()> {
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
            
            let mut parser = LineParser::new(
                &data[start..end],
                lineno,
                start
            );
            self.parse_line(&mut parser)?;
            
            start = end + 1;
        }
        
        Ok(())
    }
    
    fn parse_line(&mut self, parser: &mut LineParser) -> Result<()> {
        while parser.has_more_data() {
            parser.skip(is_whitespace);
            
            if parser.has(START_COMMENT) {
                parser.skip(is_whitespace);
                if parser.has_more_data() {
                    self.nodes.push(SyntaxNode::comment(
                        parser.offset(),
                        parser.remaining_data().len(),
                    ));
                }
                break;
            } else if !parser.has_more_data() {
                break;
            }
            
            self.parse_lhs(parser)?;
            
            parser.skip(is_whitespace);
            
            if !parser.has(SIDE_SEPARATOR) {
                return parser.error(
                    format!("Expected '{}'. Got this instead.", std::str::from_utf8(SIDE_SEPARATOR).unwrap()),
                    1,
                );
            }
            
            self.parse_rhs(parser)?;
        }
        
        Ok(())
    }
    
    fn parse_lhs(&mut self, parser: &mut LineParser) -> Result<()> {
        let lhs_nonterm = parser.peek_filter(is_nonterminal);
        
        if lhs_nonterm.is_empty() {
            return parser.error(
                "Expected a non-terminal. Got this instead.",
                1,
            );
        } else if lhs_nonterm.starts_with(b"-") {
            return parser.error(
                "Non-terminals are not allowed to start with a hyphen",
                lhs_nonterm.len(),
            );
        } else if lhs_nonterm.ends_with(b"-") {
            return parser.error(
                "Non-terminals are not allowed to end with a hyphen",
                lhs_nonterm.len(),
            );
        }
        
        self.nodes.push(SyntaxNode::start_rule(
            parser.offset(),
            lhs_nonterm.len(),
        ));
        parser.advance(lhs_nonterm.len());
        
        Ok(())
    }
    
    fn parse_rhs(&mut self, parser: &mut LineParser) -> Result<()> {
        let mut rhs_count = 0;
        
        loop {
            parser.skip(is_whitespace);
            
            match parser.peek(1) {
                None => if rhs_count == 0 {
                    return parser.error(
                        "Expected the right-hand side of a rule",
                        1,
                    );
                } else {
                    self.nodes.push(SyntaxNode::end_rule(
                        parser.offset(),
                    ));
                    break;
                },
                Some(START_COMMENT) => if rhs_count == 0 {
                    return parser.error(
                        "No elements on the right-hand side of this rule",
                        1,
                    );
                } else {
                    parser.advance(1);
                    parser.skip(is_whitespace);
                    if parser.has_more_data() {
                        self.nodes.push(SyntaxNode::comment(
                            parser.offset(),
                            parser.remaining_data().len(),
                        ));
                    }
                    parser.go_to_end();
                    self.nodes.push(SyntaxNode::end_rule(
                        parser.offset(),
                    ));
                    break;
                },
                Some(RULE_SEPARATOR) => if rhs_count == 0 {
                    return parser.error(
                        "No elements on the right-hand side of this rule",
                        1,
                    );
                } else {
                    self.nodes.push(SyntaxNode::end_rule(
                        parser.offset(),
                    ));
                    parser.advance(1);
                    break;
                },
                _ => self.parse_rhs_element(parser)?,
            }
            
            rhs_count += 1;
        }
        
        Ok(())
    }
    
    fn parse_rhs_element(&mut self, parser: &mut LineParser) -> Result<()> {
        // set
        // block
        // non-terminal
        
        match parser.peek(1).unwrap() {
            STRING_SEPARATOR => self.parse_string(parser)?,
            CHAR_SEPARATOR => self.parse_char(parser)?,
            _ => todo!(),
        }
                
        Ok(())
    }
    
    fn parse_string(&mut self, parser: &mut LineParser) -> Result<()> {
        parser.advance(1);
        
        if let Some(contents) = parser.peek_filter_escaped(|c| c != STRING_SEPARATOR[0]) {
            if contents.is_empty() {
                parser.recede(1);
                return parser.error("Empty string", 2);
            }
            
            if let Err(region) = Self::check_valid_escape_sequences(contents, true) {
                parser.advance(region.start);
                return parser.error("Invalid escape sequence", region.len());
            }
            
            self.nodes.push(SyntaxNode::string(
                parser.offset(),
                contents.len(),
            ));
            parser.advance(contents.len() + 1);
        } else {
            parser.recede(1);
            return parser.error("Unterminated string", parser.remaining_data().len());
        }
        
        Ok(())
    }
    
    fn parse_char(&mut self, parser: &mut LineParser) -> Result<()> {
        parser.advance(1);
        
        if let Some(contents) = parser.peek_filter_escaped(|c| c != CHAR_SEPARATOR[0]) {
            if contents.is_empty() {
                parser.recede(1);
                return parser.error("Empty char", 2);
            }
            
            match Self::check_valid_escape_sequences(contents, false) {
                Ok(count) => if count != 1 {
                    parser.recede(1);
                    return parser.error("Too many characters in char", contents.len() + 2);
                },
                Err(region) => {
                    parser.advance(region.start);
                    return parser.error("Invalid escape sequence", region.len());
                },
            }
            
            self.nodes.push(SyntaxNode::char(
                parser.offset(),
                contents.len(),
            ));
            parser.advance(contents.len() + 1);
        } else {
            parser.recede(1);
            return parser.error("Unterminated char", parser.remaining_data().len());
        }
        
        Ok(())
    }
    
    fn check_valid_escape_sequences(data: &[u8], in_string: bool) -> Result<usize, Range<usize>> {
        let mut char_count = 0;
        let mut cursor = 0;
        
        while let Some(c) = data.get(cursor) {
            if *c == b'\\' {
                match data[cursor + 1] {
                    b'0' | b'a' | b'b' | b't' | b'n' | b'v' | b'f' | b'r' => cursor += 1,
                    b'"' => if in_string {
                        cursor += 1;
                    } else {
                        return Err(cursor..cursor + 2);
                    },
                    b'\'' => if !in_string {
                        cursor += 1;
                    } else {
                        return Err(cursor..cursor + 2);
                    },
                    b'x' => if let Some(hexdigits) = data.get(cursor + 2..cursor + 4) {
                        if hexdigits.iter().all(|c| c.is_ascii_hexdigit()) {
                            cursor += 3;
                        } else {
                            return Err(cursor..cursor + 4);
                        }
                    } else {
                        return Err(cursor..data.len());
                    },
                    _ => return Err(cursor..cursor + 2),
                }
            }
            
            cursor += 1;
            char_count += 1;
        }
        
        Ok(char_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser() {
        let mut parser = GrammarParser::new();
        parser.parse("   ASDF-ASDF -> \"asdf\\xFF\\\"\" '\\x00' #\n  #").unwrap();
        println!("{:#?}", parser.nodes());
    }
}
