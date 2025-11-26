use std::ops::Range;
use thiserror::Error;
use crate::grammar::syntax;

struct StringView<'a> {
    bytes: &'a [u8],
    indices: Vec<usize>,
}

impl<'a> StringView<'a> {
    fn new(string: &'a str) -> Self {
        let mut indices: Vec<usize> = string.char_indices().map(|(i, _)| i).collect();
        indices.push(string.len());
        
        Self {
            bytes: string.as_bytes(),
            indices,
        }
    }
    
    #[inline]
    fn len(&self) -> usize {
        self.indices.len() - 1
    }
    
    fn offset(&self, index: usize) -> usize {
        let index = std::cmp::min(index, self.len());
        self.indices[index]
    }
    
    fn char_at(&self, index: usize) -> Option<char> {
        if index >= self.len() {
            return None;
        }
        
        let start = self.indices[index];
        let end = self.indices[index + 1];
        
        let s = str::from_utf8(&self.bytes[start..end]).ok()?;
        s.chars().next()
    }
    
    fn str_at(&self, range: Range<usize>) -> Option<&'a str> {
        if range.start >= self.len() || range.end > self.len() {
            return None;
        }
        
        let start = self.indices[range.start];
        let end = self.indices[range.end];
        
        str::from_utf8(&self.bytes[start..end]).ok()
    }
    
    fn bytes_at(&self, index: usize, raw_len: usize) -> Option<&'a [u8]> {
        if index >= self.len() {
            return None;
        }
        
        let start = self.indices[index];
        self.bytes.get(start..start + raw_len)
    }
    
    fn converted_len(s: &str) -> usize {
        s.chars().count()
    }
}

#[cfg(test)]
mod string_view_tests {
    use super::*;
    
    #[test]
    fn test1() {
        let s = "東y̆"; // 3 + 1 + 2
        let view = StringView::new(s);
        
        println!("{} -> {}", s.len(), view.len());
        
        for i in 0..view.len() {
            println!("{}: {:?}", i, view.char_at(i));
        }
        
        println!("{:?}", view.str_at(0..2));
        println!("{:?}", view.str_at(2..3));
        
        assert_eq!(view.str_at(1..1), Some(""));
    }
}

struct Parser<'a> {
    view: StringView<'a>,
    cursor: usize,
}

impl<'a> Parser<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            view: StringView::new(data),
            cursor: 0,
        }
    }
    
    fn cursor(&self) -> usize {
        self.cursor
    }
    
    fn offset(&self, offset: usize) -> usize {
        self.view.offset(offset)
    }
    
    fn eof(&self) -> bool {
        self.cursor >= self.view.len()
    }
    
    fn skip_fn<F>(&mut self, mut f: F)
    where
        F: FnMut(char) -> bool,
    {
        loop {
            if let Some(c) = self.view.char_at(self.cursor) && f(c) {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }
    
    fn skip_str(&mut self, s: &str) {
        debug_assert!(self.has(s));
        self.cursor += StringView::converted_len(s);
    }
    
    fn skip_char(&mut self) {
        self.cursor += 1;
    }
    
    fn has(&self, s: &str) -> bool {
        self.view.bytes_at(self.cursor, s.len()) == Some(s.as_bytes())
    }
    
    fn current_char(&self) -> Option<char> {
        self.view.char_at(self.cursor)
    }
    
    fn collect<F>(&mut self, mut f: F) -> Option<&'a str>
    where
        F: FnMut(char) -> bool,
    {
        let start = self.cursor;
        
        loop {
            if let Some(c) = self.view.char_at(self.cursor) && f(c) {
                self.cursor += 1;
            } else {
                break;
            }
        }
        
        self.view.str_at(start..self.cursor)
    }
    
    fn expect(&mut self, s: &str) -> bool {
        if self.has(s) {
            self.skip_str(s);
            true
        } else {
            false
        }
    }
    
    fn consume(&mut self, n: usize) -> Option<&'a str> {
        let start = self.cursor;
        let ret = self.view.str_at(start..start + n);
        
        if ret.is_some() {
            self.cursor += n;
        }
        
        ret
    }
}

#[derive(Debug, Error)]
pub enum ParsingErrorKind {
    #[error("This comment was never closed")]
    UnclosedComment,
    
    #[error("Invalid characters in non-terminal")]
    InvalidNonterminal,
    
    #[error("Expected a separator")]
    MissingSeparator,
    
    #[error("No symbols were supplied on the right-hand side of the rule")]
    MissingRhs,
    
    #[error("Invalid string: {0}")]
    InvalidString(&'static str),
    
    #[error("Invalid group: {0}")]
    InvalidGroup(&'static str),
    
    #[error("Error with OR: {0}")]
    OrError(&'static str),
    
    #[error("Encountered an unexpected element")]
    UnexpectedElement,
}

#[derive(Debug)]
pub struct ParsingError {
    range: Range<usize>,
    kind: ParsingErrorKind,
}

impl ParsingError {
    fn unclosed_comment(parser: &Parser, start: usize) -> Self {
        let start = parser.offset(start);
        
        Self {
            range: start..start + syntax::START_COMMENT.len(),
            kind: ParsingErrorKind::UnclosedComment,
        }
    }
    
    fn invalid_nonterminal(parser: &Parser) -> Self {
        let start = parser.offset(parser.cursor());
        
        Self {
            range: start..start + 1,
            kind: ParsingErrorKind::InvalidNonterminal,
        }
    }
    
    fn missing_separator(parser: &Parser) -> Self {
        let start = parser.offset(parser.cursor());
        
        Self {
            range: start..start + 1,
            kind: ParsingErrorKind::MissingSeparator,
        }
    }
    
    fn missing_rhs(parser: &Parser) -> Self {
        let start = parser.offset(parser.cursor());
        
        Self {
            range: start..start + 1,
            kind: ParsingErrorKind::MissingRhs,
        }
    }
    
    fn invalid_string(parser: &Parser, description: &'static str) -> Self {
        let start = parser.offset(parser.cursor());
        
        Self {
            range: start..start + 1,
            kind: ParsingErrorKind::InvalidString(description),
        }
    }
    
    fn invalid_group(parser: &Parser, start: usize, description: &'static str) -> Self {
        let start = parser.offset(start);
        
        Self {
            range: start..start + syntax::START_GROUP.len(),
            kind: ParsingErrorKind::InvalidGroup(description),
        }
    }
    
    fn or_error(parser: &Parser, description: &'static str) -> Self {
        let start = parser.offset(parser.cursor());
        
        Self {
            range: start..start + syntax::OPERATOR_OR.len(),
            kind: ParsingErrorKind::OrError(description),
        }
    }
    
    fn unexpected_element(parser: &Parser) -> Self {
        let start = parser.offset(parser.cursor());
        
        Self {
            range: start..start + 1,
            kind: ParsingErrorKind::UnexpectedElement,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NumberType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
}

impl NumberType {
    fn parse_decimal(&self, s: &str) -> Option<u64> {
        if s.is_empty() {
            return None;
        }
        
        match self {
            NumberType::U8 => s.parse::<u8>().map(|v| v as u64),
            NumberType::I8 => s.parse::<i8>().map(|v| v as u8 as u64),
            NumberType::U16 => s.parse::<u16>().map(|v| v as u64),
            NumberType::I16 => s.parse::<i16>().map(|v| v as u16 as u64),
            NumberType::U32 => s.parse::<u32>().map(|v| v as u64),
            NumberType::I32 => s.parse::<i32>().map(|v| v as u32 as u64),
            NumberType::U64 => s.parse::<u64>(),
            NumberType::I64 => s.parse::<i64>().map(|v| v as u64),
        }.ok()
    }
    
    fn parse_hexadecimal(&self, s: &str) -> Option<u64> {
        if s.is_empty() {
            return None;
        }
        
        match self {
            NumberType::I8 |
            NumberType::U8 => if s.len() > 2 {
                return None;
            },
            NumberType::I16 |
            NumberType::U16 => if s.len() > 4 {
                return None;
            },
            NumberType::I32 |
            NumberType::U32 => if s.len() > 8 {
                return None;
            },
            NumberType::I64 |
            NumberType::U64 => if s.len() > 16 {
                return None;
            },
        }
        
        u64::from_str_radix(s, 16).ok()
    }
}

#[derive(Debug)]
pub enum Token<'a> {
    StartRule(&'a str),
    NonTerminal(&'a str),
    String(Vec<u8>),
    StartGroup,
    EndGroup,
    Or,
    StartNumberset(NumberType),
    EndNumberset,
    NumberRange(u64, u64),
}

impl<'a> Token<'a> {
    fn has_content(&self) -> bool {
        match self {
            Token::StartRule(_) => false,
            Token::NonTerminal(_) => true,
            Token::String(_) => true,
            Token::StartGroup => false,
            Token::EndGroup => true,
            Token::Or => false,
            Token::StartNumberset(_) => false,
            Token::EndNumberset => false,
            Token::NumberRange(_, _) => true,
        }
    }
    
    fn needs_following_content(&self) -> bool {
        match self {
            Token::StartRule(_) => true,
            Token::NonTerminal(_) => false,
            Token::String(_) => false,
            Token::StartGroup => true,
            Token::EndGroup => false,
            Token::Or => true,
            Token::StartNumberset(_) => true,
            Token::EndNumberset => false,
            Token::NumberRange(_, _) => false,
        }
    }
}

pub struct Tokenizer {
    group_level: usize,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            group_level: 0,
        }
    }
    
    pub fn tokenize<'a>(&mut self, content: &'a str) -> Result<Vec<Token<'a>>, ParsingError> {
        let mut parser = Parser::new(content);
        let mut tokens = Vec::new();
        
        self.parse_top_level(&mut parser, &mut tokens)?;
        
        Ok(tokens)
    }
    
    fn parse_top_level<'a>(&mut self, parser: &mut Parser<'a>, tokens: &mut Vec<Token<'a>>) -> Result<(), ParsingError> {
        loop {
            parser.skip_fn(syntax::is_whitespace_nl);
            
            if parser.eof() {
                break;
            } else if parser.has(syntax::START_COMMENT) {
                self.skip_comment(parser)?;
            } else if parser.has(syntax::START_NONTERMINAL) {
                self.parse_rule_definition(parser, tokens)?;
            } else {
                todo!("Encountered invalid thing error");
            }
        }
        
        Ok(())
    }

    fn skip_comment(&mut self, parser: &mut Parser) -> Result<(), ParsingError> {
        let start_comment = parser.cursor();
        
        parser.skip_str(syntax::START_COMMENT);
        
        let start_first = syntax::START_COMMENT.chars().next().unwrap();
        let end_first = syntax::END_COMMENT.chars().next().unwrap();
        assert_ne!(start_first, end_first);
        
        loop {
            parser.skip_fn(|c| c != start_first && c != end_first);
            
            match parser.current_char() {
                None => return Err(ParsingError::unclosed_comment(parser, start_comment)),
                Some(c) => if c == start_first {
                    if parser.has(syntax::START_COMMENT) {
                        self.skip_comment(parser)?;
                    } else {
                        parser.skip_char();
                    }
                } else if c == end_first {
                    if parser.expect(syntax::END_COMMENT) {
                        break;
                    } else {
                        parser.skip_char();
                    }
                } else {
                    unreachable!()
                },
            }
        }
        
        Ok(())
    }
    
    fn parse_rule_definition<'a>(&mut self, parser: &mut Parser<'a>, tokens: &mut Vec<Token<'a>>) -> Result<(), ParsingError> {
        /* Left-hand side: a non-terminal */
        let nonterm = self.parse_nonterminal(parser)?;
        tokens.push(Token::StartRule(nonterm));
        
        /* Then, a separator */
        parser.skip_fn(syntax::is_whitespace);
        
        if !parser.expect(syntax::RULE_SEPARATOR) {
            return Err(ParsingError::missing_separator(parser));
        }
        
        /* Then, the right-hand side */
        let mut num_elems = 0;
        
        loop {
            parser.skip_fn(syntax::is_whitespace);
            
            if parser.has(syntax::END_RULE) {
                if num_elems == 0 {
                    return Err(ParsingError::missing_rhs(parser));
                }
                
                parser.skip_str(syntax::END_RULE);
                break;
            } else if parser.eof() {
                if num_elems == 0 {
                    return Err(ParsingError::missing_rhs(parser));
                }
                
                break;
            } else {
                self.parse_one_element(parser, tokens)?;
                num_elems += 1;
            }
        }
        
        Ok(())
    }
    
    fn parse_nonterminal<'a>(&mut self, parser: &mut Parser<'a>) -> Result<&'a str, ParsingError> {
        parser.skip_str(syntax::START_NONTERMINAL);
        
        let nonterm = parser.collect(syntax::is_nonterminal).unwrap_or("");
        
        if nonterm.is_empty() || !parser.expect(syntax::END_NONTERMINAL) {
            return Err(ParsingError::invalid_nonterminal(parser));
        }
        
        Ok(nonterm)
    }
    
    fn parse_one_element<'a>(&mut self, parser: &mut Parser<'a>, tokens: &mut Vec<Token<'a>>) -> Result<(), ParsingError> {
        if parser.has(syntax::START_NONTERMINAL) {
            let nonterm = self.parse_nonterminal(parser)?;
            tokens.push(Token::NonTerminal(nonterm));
        } else if parser.has(syntax::START_STRING) {
            let string = self.parse_string(parser)?;
            tokens.push(Token::String(string));
        } else if parser.has(syntax::START_GROUP) {
            self.parse_group(parser, tokens)?;
        } else if parser.has(syntax::OPERATOR_OR) {
            self.parse_or(parser, tokens)?;
        } else if self.has_numberset(parser) {
            self.parse_numberset(parser, tokens)?;
        } else {
            return Err(ParsingError::unexpected_element(parser));
        }
        
        Ok(())
    }
    
    fn has_numberset(&mut self, parser: &mut Parser) -> bool {
        parser.has(syntax::TYPE_U8) ||
        parser.has(syntax::TYPE_I8) ||
        parser.has(syntax::TYPE_U16) ||
        parser.has(syntax::TYPE_I16) ||
        parser.has(syntax::TYPE_U32) ||
        parser.has(syntax::TYPE_I32) ||
        parser.has(syntax::TYPE_U64) ||
        parser.has(syntax::TYPE_I64)
    }
    
    fn parse_string(&mut self, parser: &mut Parser) -> Result<Vec<u8>, ParsingError> {
        let mut buf = [0; 4];
        let mut result = Vec::new();
        
        parser.skip_str(syntax::START_STRING);
        
        while let Some(c) = parser.current_char() {
            if parser.expect(syntax::END_STRING) {
                return Ok(result);
            } else if syntax::is_forbidden_in_string(c) {
                return Err(ParsingError::invalid_string(parser, "Newlines are forbidden in a string"));
            }
            
            if c == '\\' {
                let c = self.parse_escape_character(parser)?;
                result.push(c);
            } else {
                result.extend(c.encode_utf8(&mut buf).as_bytes());
            }
            
            parser.skip_char();
        }
        
        Err(ParsingError::invalid_string(parser, "String was not closed"))
    }
    
    fn parse_escape_character(&mut self, parser: &mut Parser) -> Result<u8, ParsingError> {
        parser.skip_str("\\");
        
        match parser.current_char() {
            Some('0') => Ok(b'\0'),
            Some('a') => Ok(7),
            Some('b') => Ok(8),
            Some('t') => Ok(b'\t'),
            Some('n') => Ok(b'\n'),
            Some('v') => Ok(11),
            Some('f') => Ok(12),
            Some('r') => Ok(b'\r'),
            Some('\\') => Ok(b'\\'),
            Some('"') => Ok(b'"'),
            Some('x') => {
                parser.skip_char();
                self.parse_hexdigits(parser).ok_or_else(|| ParsingError::invalid_string(parser, "Expected two hexademical digits"))
            },
            _ => Err(ParsingError::invalid_string(parser, "Invalid escape character")),
        }
    }
    
    fn parse_hexdigits(&mut self, parser: &mut Parser) -> Option<u8> {
        let first = parser.current_char()?.to_digit(16)?;
        parser.skip_char();
        let second = parser.current_char()?.to_digit(16)?;
        Some((first * 16 + second) as u8)
    }
    
    fn parse_group<'a>(&mut self, parser: &mut Parser<'a>, tokens: &mut Vec<Token<'a>>) -> Result<(), ParsingError> {
        let mut num_elements = 0;
        let start_group = parser.cursor();
        
        parser.skip_str(syntax::START_GROUP);
        self.group_level += 1;
        tokens.push(Token::StartGroup);
        
        loop {
            parser.skip_fn(syntax::is_whitespace_nl);
            
            if parser.expect(syntax::END_GROUP) {
                break;
            } else {
                self.parse_one_element(parser, tokens)?;
                num_elements += 1;
            }
        }
        
        if num_elements == 0 {
            return Err(ParsingError::invalid_group(parser, start_group, "Group is empty"));
        } else if tokens.last().unwrap().needs_following_content() {
            return Err(ParsingError::invalid_group(parser, start_group, "Group closed prematurely"));
        }
        
        tokens.push(Token::EndGroup);
        self.group_level -= 1;
        
        Ok(())
    }
    
    fn parse_or<'a>(&mut self, parser: &mut Parser<'a>, tokens: &mut Vec<Token<'a>>) -> Result<(), ParsingError> {
        if self.group_level == 0 {
            return Err(ParsingError::or_error(parser, "For clarity, the OR operator is only allowed inside a group"));
        } else if !tokens.last().unwrap().has_content() {
            return Err(ParsingError::or_error(parser, "OR is not separating elements"));
        }
        
        parser.skip_str(syntax::OPERATOR_OR);
        tokens.push(Token::Or);
        
        Ok(())
    }
    
    fn parse_numberset<'a>(&mut self, parser: &mut Parser<'a>, tokens: &mut Vec<Token<'a>>) -> Result<(), ParsingError> {
        let mut num_elements = 0;
        let typ = if parser.expect(syntax::TYPE_U8) {
            NumberType::U8
        } else if parser.expect(syntax::TYPE_I8) {
            NumberType::I8
        } else if parser.expect(syntax::TYPE_U16) {
            NumberType::U16
        } else if parser.expect(syntax::TYPE_I16) {
            NumberType::I16
        } else if parser.expect(syntax::TYPE_U32) {
            NumberType::U32
        } else if parser.expect(syntax::TYPE_I32) {
            NumberType::I32
        } else if parser.expect(syntax::TYPE_U64) {
            NumberType::U64
        } else if parser.expect(syntax::TYPE_I64) {
            NumberType::I64
        } else {
            unreachable!()
        };
        
        if !parser.expect(syntax::START_NUMBERSET) {
            todo!()
        }
        
        tokens.push(Token::StartNumberset(typ.clone()));
        
        loop {
            parser.skip_fn(syntax::is_whitespace_nl);
            
            let value = self.parse_number(parser, &typ)?;
            
            if parser.expect(syntax::OPERATOR_RANGE) {
                let other_value = self.parse_number(parser, &typ)?;
                tokens.push(Token::NumberRange(value, other_value));
            } else {
                tokens.push(Token::NumberRange(value, value));
            }
            
            num_elements += 1;
            
            if !parser.expect(syntax::OPERATOR_SET_SEPARATOR) {
                break;
            }
        }
        
        parser.skip_fn(syntax::is_whitespace_nl);
        
        if !parser.expect(syntax::END_NUMBERSET) {
            todo!()
        } else if num_elements == 0 {
            todo!()
        }
        
        tokens.push(Token::EndNumberset);
        
        Ok(())
    }
    
    fn parse_number(&mut self, parser: &mut Parser, typ: &NumberType) -> Result<u64, ParsingError> {
        if parser.expect(syntax::PREFIX_HEXADECIMAL) {
            let Some(hexstring) = parser.collect(|c| c.is_ascii_hexdigit()) else {
                todo!()
            };
            let Some(value) = typ.parse_hexadecimal(hexstring) else {
                todo!()
            };
            Ok(value)
        } else {
            let Some(string) = parser.collect(|c| c.is_ascii_digit() || c == '-') else {
                todo!()
            };
            let Some(value) = typ.parse_decimal(string) else {
                todo!()
            };
            Ok(value)
        }
    }
}
