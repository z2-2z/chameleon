use std::path::{Path, PathBuf};
use std::ops::Range;
use anyhow::Result;
use thiserror::Error;

const START_COMMENT: &[u8] = b"#";
const SIDE_SEPARATOR: &[u8] = b"->";
const RULE_SEPARATOR: &[u8] = b";";
const STRING_SEPARATOR: &[u8] = b"\"";
const CHAR_SEPARATOR: &[u8] = b"'";
const SET_OPEN: &[u8] = b"Set<";
const SET_CLOSE_TYPE: &[u8] = b">";

type FilterFunc = fn(u8) -> bool;

fn is_nonterminal(c: u8) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_'
}

fn is_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

fn is_decimal_number(c: u8) -> bool {
    c == b'-' || c.is_ascii_digit()
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
    
    fn skip(&mut self, func: FilterFunc) -> usize {
        let old_cursor = self.cursor;
        
        while let Some(c) = self.line.get(self.cursor) {
            if func(*c) {
                self.cursor += 1;
            } else {
                break;
            }
        }
        
        self.cursor - old_cursor
    }
    
    fn peek(&self, len: usize) -> Option<&'a [u8]> {
        self.line.get(self.cursor..self.cursor + len)
    }
    
    fn advance(&mut self, len: usize) {
        self.cursor = std::cmp::min(self.line.len(), self.cursor + len);
    }
    
    fn rewind(&mut self, len: usize) {
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
    
    fn peek_filter_terminated(&self, func: FilterFunc) -> Option<&'a [u8]> {
        let mut len = 0;
        let start = self.cursor;
        
        while let Some(c) = self.line.get(start + len) {
            if func(*c) {
                len += 1;
            } else {
                break;
            }
        }
        
        if start + len >= self.line.len() {
            None
        } else {
            Some(&self.line[start..start + len])
        }
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
    EndRule,
    String(Range<usize>),
    Char(Range<usize>),
    NonTerminal(Range<usize>),
    StartSet(Range<usize>),
    EndSet,
    Number(Range<usize>),
    Range(Range<usize>, Range<usize>),
}

impl SyntaxNode {
    fn comment(offset: usize, len: usize) -> Self {
        Self::Comment(offset..offset + len)
    }
    
    fn start_rule(offset: usize, len: usize) -> Self {
        Self::StartRule(offset..offset + len)
    }
    
    fn end_rule() -> Self {
        Self::EndRule
    }
    
    fn string(offset: usize, len: usize) -> Self {
        Self::String(offset..offset + len)
    }
    
    fn char(offset: usize, len: usize) -> Self {
        Self::Char(offset..offset + len)
    }
    
    fn non_terminal(offset: usize, len: usize) -> Self {
        Self::NonTerminal(offset..offset + len)
    }
    
    fn start_set(offset: usize, len: usize) -> Self {
        Self::StartSet(offset..offset + len)
    }
    
    fn end_set() -> Self {
        Self::EndSet
    }
}

pub struct GrammarParser {
    stream: Vec<SyntaxNode>,
}

impl GrammarParser {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            stream: Vec::with_capacity(4096),
        }
    }
    
    pub fn parse(&mut self, data: &str) -> Result<&[SyntaxNode]> {
        self.stream.clear();
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
        
        Ok(&self.stream)
    }
    
    fn parse_line(&mut self, parser: &mut LineParser) -> Result<()> {
        while parser.has_more_data() {
            parser.skip(is_whitespace);
            
            if parser.has(START_COMMENT) {
                parser.skip(is_whitespace);
                if parser.has_more_data() {
                    self.stream.push(SyntaxNode::comment(
                        parser.offset(),
                        parser.remaining_data().len(),
                    ));
                }
                break;
            } else if !parser.has_more_data() {
                break;
            }
            
            self.parse_non_terminal(parser, SyntaxNode::start_rule)?;
            
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
    
    fn parse_non_terminal(&mut self, parser: &mut LineParser, node_fn: fn(usize, usize) -> SyntaxNode) -> Result<()> {
        let nonterm = parser.peek_filter(is_nonterminal);
        
        if nonterm.is_empty() {
            return parser.error(
                "Expected a non-terminal. Got this instead.",
                1,
            );
        }
        
        self.stream.push(node_fn(
            parser.offset(),
            nonterm.len(),
        ));
        parser.advance(nonterm.len());
        
        Ok(())
    }
    
    fn parse_rhs(&mut self, parser: &mut LineParser) -> Result<()> {
        let mut rhs_count = 0;
        
        loop {
            let ws_count = parser.skip(is_whitespace);
            
            match parser.peek(1) {
                None => if rhs_count == 0 {
                    return parser.error(
                        "Expected the right-hand side of a rule",
                        1,
                    );
                } else {
                    self.stream.push(SyntaxNode::end_rule());
                    break;
                },
                Some(START_COMMENT) => if rhs_count == 0 {
                    return parser.error(
                        "No elements on the right-hand side of this rule",
                        1,
                    );
                } else {
                    self.stream.push(SyntaxNode::end_rule());
                    parser.advance(1);
                    parser.skip(is_whitespace);
                    if parser.has_more_data() {
                        self.stream.push(SyntaxNode::comment(
                            parser.offset(),
                            parser.remaining_data().len(),
                        ));
                        parser.go_to_end();
                    }
                    break;
                },
                Some(RULE_SEPARATOR) => if rhs_count == 0 {
                    return parser.error(
                        "No elements on the right-hand side of this rule",
                        1,
                    );
                } else {
                    self.stream.push(SyntaxNode::end_rule());
                    parser.advance(1);
                    break;
                },
                _ => {
                    if rhs_count > 0 && ws_count == 0 {
                        return parser.error("Elements on the right-hand side of a rule must be separated by whitespaces", 1);
                    }
                    
                    self.parse_rhs_element(parser)?;
                },
            }
            
            rhs_count += 1;
        }
        
        Ok(())
    }
    
    fn parse_rhs_element(&mut self, parser: &mut LineParser) -> Result<()> {
        // set
        // block
        
        match parser.peek(1).unwrap() {
            STRING_SEPARATOR => self.parse_string(parser)?,
            CHAR_SEPARATOR => self.parse_char(parser)?,
            _ => if parser.peek(4) == Some(SET_OPEN) {
                self.parse_set(parser)?;
            } else {
                self.parse_non_terminal(parser, SyntaxNode::non_terminal)?;
            },
        }
        
        Ok(())
    }
    
    fn parse_string(&mut self, parser: &mut LineParser) -> Result<()> {
        parser.advance(1);
        
        if let Some(contents) = parser.peek_filter_escaped(|c| c != STRING_SEPARATOR[0]) {
            if contents.is_empty() {
                parser.rewind(1);
                return parser.error("Empty string", 2);
            }
            
            if let Err(region) = Self::check_valid_escape_sequences(contents, true) {
                parser.advance(region.start);
                return parser.error("Invalid escape sequence", region.len());
            }
            
            self.stream.push(SyntaxNode::string(
                parser.offset(),
                contents.len(),
            ));
            parser.advance(contents.len() + 1);
        } else {
            parser.rewind(1);
            return parser.error("Unterminated string", parser.remaining_data().len());
        }
        
        Ok(())
    }
    
    fn parse_char(&mut self, parser: &mut LineParser) -> Result<()> {
        parser.advance(1);
        
        if let Some(contents) = parser.peek_filter_escaped(|c| c != CHAR_SEPARATOR[0]) {
            if contents.is_empty() {
                parser.rewind(1);
                return parser.error("Empty char", 2);
            }
            
            match Self::check_valid_escape_sequences(contents, false) {
                Ok(count) => if count != 1 {
                    parser.rewind(1);
                    return parser.error("Too many characters in char", contents.len() + 2);
                },
                Err(region) => {
                    parser.advance(region.start);
                    return parser.error("Invalid escape sequence", region.len());
                },
            }
            
            self.stream.push(SyntaxNode::char(
                parser.offset(),
                contents.len(),
            ));
            parser.advance(contents.len() + 1);
        } else {
            parser.rewind(1);
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
    
    fn parse_set(&mut self, parser: &mut LineParser) -> Result<()> {
        parser.advance(SET_OPEN.len());
        
        let datatype: &[u8];
        
        if let Some(content) = parser.peek_filter_terminated(|c| c != SET_CLOSE_TYPE[0]) {
            if !Self::check_set_datatype(content) {
                return parser.error("Invalid data type", content.len());
            }
            
            self.stream.push(SyntaxNode::start_set(
                parser.offset(),
                content.len(),
            ));
            
            datatype = content;
            parser.advance(content.len() + 1);
        } else {
            return parser.error("Unterminated set type", 1);
        }
        
        if !parser.has(b"(") {
            return parser.error("Expected set definition in brackets", 1);
        }
        
        let mut num_elements = 0;
        
        loop {
            parser.skip(is_whitespace);
            
            self.parse_set_element(parser, datatype)?;
            num_elements += 1;
            
            match parser.peek(1) {
                None | Some(b")") => break,
                Some(b",") => parser.advance(1),
                _ => return parser.error("Unexpected character in set", 1),
            }
        }
        
        if !parser.has(b")") {
            return parser.error("Missing closing bracket", 1);
        }
        
        if num_elements == 0 {
            parser.rewind(1);
            return parser.error("Empty set", 1);
        }
        
        self.stream.push(SyntaxNode::end_set());
        Ok(())
    }
    
    fn check_set_datatype(data: &[u8]) -> bool {
        matches!(data, b"u64" | b"i64" | b"u32" | b"i32" | b"u16" | b"i16" | b"u8" | b"i8")
    }
    
    fn parse_set_element(&mut self, parser: &mut LineParser, datatype: &[u8]) -> Result<()> {
        let first_number = self.parse_number(parser, datatype)?;
        
        if parser.peek(2) == Some(b"..") {
            parser.advance(2);
            let second_number = self.parse_number(parser, datatype)?;
            self.stream.push(SyntaxNode::Range(first_number, second_number));
        } else {
            self.stream.push(SyntaxNode::Number(first_number));
        }
        
        Ok(())
    }
    
    fn parse_number(&mut self, parser: &mut LineParser, datatype: &[u8]) -> Result<Range<usize>> {
        if parser.peek(2) == Some(b"0x") {
            let max_digits = match datatype {
                b"u64" | b"i64" => 16,
                b"u32" | b"i32" => 8,
                b"u16" | b"i16" => 4,
                b"u8" | b"i8" => 2,
                _ => unreachable!(),
            };
            
            parser.advance(2);
            let number = parser.peek_filter(|c| c.is_ascii_hexdigit());
            parser.rewind(2);
            
            if number.is_empty() {
                parser.error("Expected a hex string", 3)?;
            } else if number.len() > max_digits {
                parser.error("Too many hex characters for given datatype", 2 + number.len())?;
            }
            
            let ret = parser.offset()..parser.offset() + number.len();
            parser.advance(2 + number.len());
            Ok(ret)
        } else {
            let number = parser.peek_filter(is_decimal_number);
            
            if number.is_empty() {
                parser.error("Expected a number", 1)?;
            } else if number.iter().skip(1).any(|c| *c == b'-') {
                parser.error("Invalid dashes in number", number.len())?;
            } else if datatype[0] == b'u' && number[0] == b'-' {
                parser.error("Supplied a negative number for an unsigned datatype", number.len())?;
            }
            
            let ret = parser.offset()..parser.offset() + number.len();
            parser.advance(number.len());
            Ok(ret)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser() {
        let mut parser = GrammarParser::new();
        let stream = parser.parse("   ASDF_asdf -> \"asdf\\xFF\\\"\" '\\x00' nonterm#\n  x -> Set<i8>(1, -1..-2, 0xFF..-1)").unwrap();
        println!("{stream:#?}");
    }
    
    #[test]
    fn test_min() {
        let mut parser = GrammarParser::new();
        let stream = parser.parse("0->\"string\" '\\x00'").unwrap();
        println!("{stream:#?}");
    }
}
