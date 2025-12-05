use std::ops::Range;
use thiserror::Error;
use crate::grammar::syntax;

#[derive(Debug)]
pub struct TextMetadata {
    pub line: usize,
    pub column: usize,
}

struct StringView<'a> {
    bytes: &'a [u8],
    indices: Vec<usize>,
    last_line: usize,
    last_column: usize,
    last_index: usize,
}

impl<'a> StringView<'a> {
    fn new(string: &'a str) -> Self {
        let mut indices: Vec<usize> = string.char_indices().map(|(i, _)| i).collect();
        indices.push(string.len());
        
        Self {
            bytes: string.as_bytes(),
            indices,
            last_line: 1,
            last_column: 0,
            last_index: 0,
        }
    }
    
    #[inline]
    fn len(&self) -> usize {
        self.indices.len() - 1
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
    
    fn get_metadata(&mut self, index: usize) -> TextMetadata {
        assert!(index >= self.last_index);
        
        while self.last_index <= index {
            if self.bytes.get(self.indices[self.last_index]) == Some(&b'\n') {
                self.last_line += 1;
                self.last_column = 0;
            } else {
                self.last_column += 1;
            }
            
            self.last_index += 1;
        }
        
        TextMetadata {
            line: self.last_line,
            column: self.last_column,
        }
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
    
    fn eof(&self) -> bool {
        self.cursor >= self.view.len()
    }
    
    fn skip_fn<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(char) -> bool,
    {
        let mut skipped = false;
        
        loop {
            if let Some(c) = self.view.char_at(self.cursor) && f(c) {
                self.cursor += 1;
                skipped = true;
            } else {
                break;
            }
        }
        
        skipped
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
    
    fn metadata(&mut self, offset: usize) -> TextMetadata {
        self.view.get_metadata(offset)
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
    
    #[error("Invalid numberset: {0}")]
    InvalidNumberset(&'static str),
    
    #[error("Invalid number: {0}")]
    InvalidNumber(&'static str),
    
    #[error("Expected a rule definition")]
    MissingRule,
    
    #[error("Invalid namespace: {0}")]
    InvalidNamespace(&'static str),
    
    #[error("Invalid clear statement: {0}")]
    InvalidClear(&'static str),
}

#[derive(Debug, Error)]
#[error("Error while parsing grammar in line {}:{}: {}", meta.line, meta.column, kind)]
pub struct ParsingError {
    meta: TextMetadata,
    kind: ParsingErrorKind,
}

impl ParsingError {
    fn unclosed_comment(parser: &mut Parser, start: usize) -> Self {
        Self {
            meta: parser.metadata(start),
            kind: ParsingErrorKind::UnclosedComment,
        }
    }
    
    fn invalid_nonterminal(parser: &mut Parser) -> Self {
        Self {
            meta: parser.metadata(parser.cursor()),
            kind: ParsingErrorKind::InvalidNonterminal,
        }
    }
    
    fn missing_separator(parser: &mut Parser) -> Self {
        Self {
            meta: parser.metadata(parser.cursor()),
            kind: ParsingErrorKind::MissingSeparator,
        }
    }
    
    fn missing_rhs(parser: &mut Parser) -> Self {
        Self {
            meta: parser.metadata(parser.cursor()),
            kind: ParsingErrorKind::MissingRhs,
        }
    }
    
    fn invalid_string(parser: &mut Parser, description: &'static str) -> Self {
        Self {
            meta: parser.metadata(parser.cursor()),
            kind: ParsingErrorKind::InvalidString(description),
        }
    }
    
    fn invalid_group(parser: &mut Parser, start: usize, description: &'static str) -> Self {
        Self {
            meta: parser.metadata(start),
            kind: ParsingErrorKind::InvalidGroup(description),
        }
    }
    
    fn or_error(parser: &mut Parser, description: &'static str) -> Self {
        Self {
            meta: parser.metadata(parser.cursor()),
            kind: ParsingErrorKind::OrError(description),
        }
    }
    
    fn unexpected_element(parser: &mut Parser) -> Self {
        Self {
            meta: parser.metadata(parser.cursor()),
            kind: ParsingErrorKind::UnexpectedElement,
        }
    }
    
    fn invalid_numberset(parser: &mut Parser, start: usize, description: &'static str) -> Self {
        Self {
            meta: parser.metadata(start),
            kind: ParsingErrorKind::InvalidNumberset(description),
        }
    }
    
    fn invalid_number(parser: &mut Parser, start: usize, description: &'static str) -> Self {
        Self {
            meta: parser.metadata(start),
            kind: ParsingErrorKind::InvalidNumber(description),
        }
    }
    
    fn missing_rule(parser: &mut Parser) -> Self {
        Self {
            meta: parser.metadata(parser.cursor()),
            kind: ParsingErrorKind::MissingRule,
        }
    }
    
    fn invalid_namespace(parser: &mut Parser, start: usize, description: &'static str) -> Self {
        Self {
            meta: parser.metadata(start),
            kind: ParsingErrorKind::InvalidNamespace(description),
        }
    }
    
    fn invalid_clear(parser: &mut Parser, start: usize, description: &'static str) -> Self {
        Self {
            meta: parser.metadata(start),
            kind: ParsingErrorKind::InvalidClear(description),
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
pub enum Token {
    StartRule(String),
    EndRule,
    NonTerminal(TextMetadata, String),
    String(Vec<u8>),
    StartGroup,
    EndGroup,
    Or,
    StartNumberset(NumberType),
    EndNumberset,
    NumberRange(u64, u64),
}

impl Token {
    fn has_content(&self) -> bool {
        match self {
            Token::StartRule(_) => false,
            Token::EndRule => false,
            Token::NonTerminal(_, _) => true,
            Token::String(_) => true,
            Token::StartGroup => false,
            Token::EndGroup => true,
            Token::Or => false,
            Token::StartNumberset(_) => false,
            Token::EndNumberset => true,
            Token::NumberRange(_, _) => true,
        }
    }
    
    fn needs_following_content(&self) -> bool {
        match self {
            Token::StartRule(_) => true,
            Token::EndRule => false,
            Token::NonTerminal(_, _) => false,
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
    namespace: Option<String>,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            group_level: 0,
            namespace: None,
        }
    }
    
    pub fn tokenize(&mut self, content: &str) -> Result<Vec<Token>, ParsingError> {
        let mut parser = Parser::new(content);
        let mut tokens = Vec::new();
        
        self.parse_top_level(&mut parser, &mut tokens)?;
        
        Ok(tokens)
    }
    
    fn parse_top_level(&mut self, parser: &mut Parser, tokens: &mut Vec<Token>) -> Result<(), ParsingError> {
        loop {
            parser.skip_fn(syntax::is_whitespace_nl);
            
            if parser.eof() {
                break;
            } else if parser.has(syntax::START_COMMENT) {
                self.skip_comment(parser)?;
            } else if parser.has(syntax::START_NONTERMINAL) {
                self.parse_rule_definition(parser, tokens)?;
            } else if parser.has(syntax::DIRECTIVE_NAMESPACE) {
                self.parse_namespace(parser)?;
            } else if parser.has(syntax::DIRECTIVE_CLEAR) {
                self.parse_clear(parser)?;
            } else {
                return Err(ParsingError::missing_rule(parser));
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
    
    fn parse_rule_definition(&mut self, parser: &mut Parser, tokens: &mut Vec<Token>) -> Result<(), ParsingError> {
        /* Left-hand side: a non-terminal */
        let nonterm = self.parse_nonterminal(parser)?;
        
        let nonterm = if let Some(namespace) = &self.namespace {
            format!("{namespace}{0}{nonterm}", syntax::OPERATOR_NAMESPACE_SEPARATOR)
        } else {
            nonterm.to_owned()
        };
        
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
        
        tokens.push(Token::EndRule);
        Ok(())
    }
    
    fn parse_nonterminal<'a>(&mut self, parser: &mut Parser<'a>) -> Result<&'a str, ParsingError> {
        parser.skip_str(syntax::START_NONTERMINAL);
        
        let Some(nonterm) = parser.collect(syntax::is_nonterminal) else {
            return Err(ParsingError::invalid_nonterminal(parser));
        };
        
        // Check that nonterm has no consecutive namespace separators
        
        
        if !parser.expect(syntax::END_NONTERMINAL) {
            return Err(ParsingError::invalid_nonterminal(parser));
        }
        
        Ok(nonterm)
    }
    
    fn parse_one_element(&mut self, parser: &mut Parser, tokens: &mut Vec<Token>) -> Result<(), ParsingError> {
        if parser.has(syntax::START_NONTERMINAL) {
            let metadata = parser.metadata(parser.cursor());
            let nonterm = self.parse_nonterminal(parser)?;
            let nonterm = if !nonterm.contains(syntax::OPERATOR_NAMESPACE_SEPARATOR) && let Some(namespace) = &self.namespace {
                format!("{namespace}{0}{nonterm}", syntax::OPERATOR_NAMESPACE_SEPARATOR)
            } else if let Some(result) = nonterm.strip_prefix(syntax::OPERATOR_NAMESPACE_SEPARATOR) {
                result.to_owned()
            } else {
                nonterm.to_owned()
            };
            tokens.push(Token::NonTerminal(metadata, nonterm));
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
    
    fn parse_group(&mut self, parser: &mut Parser, tokens: &mut Vec<Token>) -> Result<(), ParsingError> {
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
    
    fn parse_or(&mut self, parser: &mut Parser, tokens: &mut Vec<Token>) -> Result<(), ParsingError> {
        if self.group_level == 0 {
            return Err(ParsingError::or_error(parser, "For clarity, the OR operator is only allowed inside a group"));
        } else if !tokens.last().unwrap().has_content() {
            return Err(ParsingError::or_error(parser, "OR is not separating elements with content"));
        }
        
        parser.skip_str(syntax::OPERATOR_OR);
        tokens.push(Token::Or);
        
        Ok(())
    }
    
    fn parse_numberset(&mut self, parser: &mut Parser, tokens: &mut Vec<Token>) -> Result<(), ParsingError> {
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
        
        parser.skip_fn(syntax::is_whitespace);
        
        let start_numberset = parser.cursor();
        
        if !parser.expect(syntax::START_NUMBERSET) {
            return Err(ParsingError::invalid_numberset(parser, start_numberset, "Numberset is missing"));
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
            return Err(ParsingError::invalid_numberset(parser, start_numberset, "Missing end of numberset"));
        } else if num_elements == 0 {
            return Err(ParsingError::invalid_numberset(parser, start_numberset, "Numberset is empty"));
        }
        
        tokens.push(Token::EndNumberset);
        
        Ok(())
    }
    
    fn parse_number(&mut self, parser: &mut Parser, typ: &NumberType) -> Result<u64, ParsingError> {
        let start_number = parser.cursor();
        
        if parser.expect(syntax::PREFIX_HEXADECIMAL) {
            let Some(hexstring) = parser.collect(|c| c.is_ascii_hexdigit()) else {
                return Err(ParsingError::invalid_number(parser, start_number, "Missing hexadecimal digits"));
            };
            let Some(value) = typ.parse_hexadecimal(hexstring) else {
                return Err(ParsingError::invalid_number(parser, start_number, "Invalid hexadecimal number"));
            };
            Ok(value)
        } else {
            let Some(string) = parser.collect(|c| c.is_ascii_digit() || c == '-') else {
                return Err(ParsingError::invalid_number(parser, start_number, "Missing decimal digits"));
            };
            let Some(value) = typ.parse_decimal(string) else {
                return Err(ParsingError::invalid_number(parser, start_number, "Invalid decimal number"));
            };
            Ok(value)
        }
    }
    
    fn parse_namespace(&mut self, parser: &mut Parser) -> Result<(), ParsingError> {
        let start_namespace = parser.cursor();
        
        parser.skip_str(syntax::DIRECTIVE_NAMESPACE);
        
        if !parser.skip_fn(syntax::is_whitespace) {
            return Err(ParsingError::invalid_namespace(parser, start_namespace, "Missing whitespace"));
        }
        
        let Some(name) = parser.collect(syntax::is_nonterminal) else {
            return Err(ParsingError::invalid_namespace(parser, start_namespace, "Invalid namespace definition"));
        };
        
        self.namespace = Some(name.to_owned());
        
        parser.skip_fn(syntax::is_whitespace);
        
        if !parser.expect(syntax::END_RULE) {
            return Err(ParsingError::invalid_namespace(parser, start_namespace, "Invalid name for namespace"));
        }
        
        Ok(())
    }
    
    fn parse_clear(&mut self, parser: &mut Parser) -> Result<(), ParsingError> {
        let start_clear = parser.cursor();
        
        parser.skip_str(syntax::DIRECTIVE_CLEAR);
        
        if !parser.skip_fn(syntax::is_whitespace) {
            return Err(ParsingError::invalid_clear(parser, start_clear, "Missing whitespace"));
        }
        
        if parser.expect(syntax::DIRECTIVE_NAMESPACE) {
            self.namespace = None;
        } else {
            return Err(ParsingError::invalid_clear(parser, start_clear, "Invalid argument"));
        }
        
        parser.skip_fn(syntax::is_whitespace);
        
        if !parser.expect(syntax::END_RULE) {
            return Err(ParsingError::invalid_clear(parser, start_clear, "Invalid arguments"));
        }
        
        Ok(())
    }
}
