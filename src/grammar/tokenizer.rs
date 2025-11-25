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
}

#[derive(Debug, Error)]
pub enum ParsingErrorKind {
    #[error("This comment was never closed")]
    UnclosedComment,
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
}

#[derive(Debug)]
pub enum Token<'a> {
    StartRule(&'a str),
}

pub struct Tokenizer {
    
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {}
    }
    
    fn reset(&mut self) {
        
    }
    
    pub fn tokenize<'a>(&mut self, content: &'a str) -> Result<Vec<Token<'a>>, ParsingError> {
        self.reset();
        
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
                    if parser.has(syntax::END_COMMENT) {
                        parser.skip_str(syntax::END_COMMENT);
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
}
