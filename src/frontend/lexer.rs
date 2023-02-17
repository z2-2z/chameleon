use crate::frontend::keywords;
use crate::frontend::source_view::{SourceView, SourceRange};
use crate::frontend::range::NewRange;

/// All the errors that can occur during lexing
#[derive(Debug)]
pub enum LexerError {
    /// We unexpectedly hit an end of input
    EOF(String),
    
    /// We expected a keyword but found something different
    ExpectedKeyword(usize, String),
    
    /// We expected a character but found something different
    ExpectedChar(usize, String),
    
    /// We expected whitespaces but none were supplied
    MissingWhitespace(usize),
    
    /// We expected a variable identifier but found something different
    ExpectedIdentifier(usize),
    
    /// We encountered an invalid character literal
    InvalidCharLiteral(usize),
    
    /// We expected a literal but found something different
    ExpectedLiteral(usize, String),
    
    /// We encountered an invalid number literal
    InvalidNumber(usize),
}

//TODO: add variable name to tokens
//TODO: make that everything has pos such that pos() always returns something
/// The tokens that get passed to the parser
#[derive(PartialEq, Debug)]
pub enum Token {
    /// A container was created with a given name
    ContainerOpen(usize, SourceRange),
    
    /// The current container is being closed
    ContainerClose,
    
    /// An option was set in a block
    OptionDef(usize, SourceRange, SourceRange),
    
    /// A variable definition follows
    VariableStart(usize),
    
    /// The variable definition finished
    VariableEnd,
    
    /// The variable is optional
    VariableOptional(usize),
    
    /// The variable has a repeats option
    VariableRepeatStart(usize),
    
    /// The variables repeats option has ended
    VariableRepeatEnd,
    
    /// A numberset follows
    NumbersetStart(usize),
    
    /// No more numberset entries follow
    NumbersetEnd,
    
    /// A range of integers was specified
    IntegerRange(SourceRange, SourceRange),
    
    /// A single integer was specified
    Integer(SourceRange),
    
    /// A character constant was specified
    Character(SourceRange),
    
    /// Sets the type of the currently active variable
    VariableType(SourceRange),
    
    /// A string literal was specified
    String(SourceRange),
    
    /// A variable has an assigned value
    VariableValueStart(usize),
    
    /// The end of an assignment to a variable
    VariableValueEnd,
    
    /// A block was opened
    BlockOpen(usize),
    
    /// The currently active block is being closed
    BlockClose,
    
    /// A range of characters was specified
    CharRange(SourceRange, SourceRange),
}

/// Same purpose as Tokens but for faster matching
#[derive(PartialEq)]
pub enum TokenId {
    ContainerOpen,
    ContainerClose,
    OptionDef,
    VariableStart,
    VariableEnd,
    VariableOptional,
    VariableRepeatStart,
    VariableRepeatEnd,
    NumbersetStart,
    NumbersetEnd,
    IntegerRange,
    Integer,
    Character,
    VariableType,
    String,
    VariableValueStart,
    VariableValueEnd,
    BlockOpen,
    BlockClose,
    CharRange,
}
impl TokenId {
    pub fn description(&self) -> &str {
        match self {
            TokenId::ContainerOpen => "a new container",
            TokenId::ContainerClose => "the end of the container",
            TokenId::OptionDef => "an option definition",
            TokenId::VariableStart => "the start of a variable",
            TokenId::VariableEnd => "the end of the variable definition",
            TokenId::VariableOptional => "the optional flag for a variable",
            TokenId::VariableRepeatStart => "the repeats flag for a variable",
            TokenId::VariableRepeatEnd => "the end of the repeats option",
            TokenId::NumbersetStart => "the start of a numberset",
            TokenId::NumbersetEnd => "the end of the numberset",
            TokenId::IntegerRange => "a number range",
            TokenId::Integer => "a number",
            TokenId::Character => "a character",
            TokenId::VariableType => "the type of a variable",
            TokenId::String => "a string",
            TokenId::VariableValueStart => "a value of the variable",
            TokenId::VariableValueEnd => "the end of the value",
            TokenId::BlockOpen => "the opening of a block",
            TokenId::BlockClose => "the closure of the block",
            TokenId::CharRange => "a character range",
        }
    }
}

impl Token {
    pub fn id(&self) -> TokenId {
        match self {
            Token::ContainerOpen(_, _) => TokenId::ContainerOpen,
            Token::ContainerClose => TokenId::ContainerClose,
            Token::OptionDef(_,_,_) => TokenId::OptionDef,
            Token::VariableStart(_) => TokenId::VariableStart,
            Token::VariableEnd => TokenId::VariableEnd,
            Token::VariableOptional(_) => TokenId::VariableOptional,
            Token::VariableRepeatStart(_) => TokenId::VariableRepeatStart,
            Token::VariableRepeatEnd => TokenId::VariableRepeatEnd,
            Token::NumbersetStart(_) => TokenId::NumbersetStart,
            Token::NumbersetEnd => TokenId::NumbersetEnd,
            Token::IntegerRange(_,_) => TokenId::IntegerRange,
            Token::Integer(_) => TokenId::Integer,
            Token::Character(_) => TokenId::Character,
            Token::VariableType(_) => TokenId::VariableType,
            Token::String(_) => TokenId::String,
            Token::VariableValueStart(_) => TokenId::VariableValueStart,
            Token::VariableValueEnd => TokenId::VariableValueEnd,
            Token::BlockOpen(_) => TokenId::BlockOpen,
            Token::BlockClose => TokenId::BlockClose,
            Token::CharRange(_, _) => TokenId::CharRange,
        }
    }
    
    pub fn pos(&self) -> Option<usize> {
        match self {
            Token::ContainerOpen(pos, _) => Some(*pos),
            Token::ContainerClose => None,
            Token::OptionDef(pos,_,_) => Some(*pos),
            Token::VariableStart(pos) => Some(*pos),
            Token::VariableEnd => None,
            Token::VariableOptional(pos) => Some(*pos),
            Token::VariableRepeatStart(pos) => Some(*pos),
            Token::VariableRepeatEnd => None,
            Token::NumbersetStart(pos) => Some(*pos),
            Token::NumbersetEnd => None,
            Token::IntegerRange(range,_) => Some(range.start),
            Token::Integer(range) => Some(range.start),
            Token::Character(range) => Some(range.start),
            Token::VariableType(range) => Some(range.start),
            Token::String(range) => Some(range.start),
            Token::VariableValueStart(pos) => Some(*pos),
            Token::VariableValueEnd => None,
            Token::BlockOpen(pos) => Some(*pos),
            Token::BlockClose => None,
            Token::CharRange(range, _) => Some(range.start),
        }
    }
}

/// Helper struct that provides some higher-level access
/// functions to the SourceView
struct Scanner<'a> {
    view: &'a SourceView,
    cursor: usize,
}
impl<'a> Scanner<'a> {
    fn new(view: &'a SourceView) -> Self {
        Self {
            view,
            cursor: 0,
        }
    }
    
    /// Check if the view at the current position contains the string `buf`
    fn peek(&self, buf: &str) -> bool {
        // we only compare against ASCII strings so buf.len() is fine
        self.view.slice(self.cursor, buf.len()) == buf
    }
    
    /// Increment the cursor
    fn forward(&mut self, len: usize) {
        if self.cursor < self.view.len() {
            self.cursor += len;
        }
    }
    
    /// Given a selector function that gets graphemes as inputs,
    /// advance the cursor until the function returns `false`
    fn skip<F>(&mut self, func: &mut F) -> usize
    where
        F: FnMut(&str) -> bool,
    {
        let mut skipped = 0;
        
        while self.cursor + skipped < self.view.len() {
            if func(self.view.slice(self.cursor + skipped, 1)) {
                skipped += 1;
            } else {
                break;
            }
        }
        
        self.forward(skipped);
        skipped
    }
    
    /// Execute a peek after skipping the content according to `F` like in `skip()` above
    /// but without advancing the cursor
    fn peek_after<F>(&self, func: &mut F, buf: &str) -> bool
    where
        F: FnMut(&str) -> bool,
    {
        let mut skipped = 0;
        
        while self.cursor + skipped < self.view.len() {
            if func(self.view.slice(self.cursor + skipped, 1)) {
                skipped += 1;
            } else {
                break;
            }
        }

        self.view.slice(self.cursor + skipped, buf.len()) == buf
    }
    
    /// Run a given function that gets a grapheme as input and outputs
    /// a boolean on the grapheme at the current cursor position
    fn check<F>(&mut self, func: &mut F) -> bool
    where
        F: FnMut(&str) -> bool,
    {
        if self.cursor < self.view.len() {
            func(self.view.slice(self.cursor, 1))
        } else {
            false
        }
    }
    
    /// Assert that the source contains `buf` at the current cursor position
    fn expect(&mut self, buf: &str) -> Result<(), LexerError> {
        if self.peek(buf) {
            self.forward(buf.len());
            Ok(())
        } else {
            if buf.len() > 1 {
                Err(LexerError::ExpectedKeyword(
                    self.cursor,
                    buf.to_string(),
                ))
            } else {
                Err(LexerError::ExpectedChar(
                    self.cursor,
                    buf.to_string(),
                ))
            }
        }
    }
    
    /// Indicate whether there are any graphemes left to parse
    fn done(&self) -> bool {
        self.cursor >= self.view.len()
    }
}

#[inline]
fn is_whitespace_nonl(s: &str) -> bool {
    s == " " || s == "\t"
}

#[inline]
fn is_whitespace(s: &str) -> bool {
    is_whitespace_nonl(s) || s == "\r" || s == "\n" || s == "\r\n"
}

/// An identifier is [0-9a-zA-Z_]+
#[inline]
fn is_identifier(s: &str) -> bool {
    if s.len() == 1 {
        let c = s.as_bytes()[0];
        (c >= 0x30 && c < 0x3a) || (c >= 0x41 && c < 0x5B) || (c >= 0x61 && c < 0x7B) || c == b'_'
    } else {
        true
    }
}

/// Charset for an option value: all chars except whitespaces and ';'
#[inline]
fn is_option_value(s: &str) -> bool {
    if s.len() == 1 {
        let c = s.as_bytes()[0];
        (c > 0x20 && c < 0x3B) || (c > 0x3B && c < 0x7F)
    } else {
        false
    }
}

/// charset for signed integers + prefix characters 0x 0o 0b + hex characters
#[inline]
fn is_integer(s: &str) -> bool {
    if s.len() == 1 {
        let c = s.as_bytes()[0];
        (c >= 0x30 && c < 0x3A) || c == b'-' || c == b'x' || c == b'o' || c == b'b' || (c >= b'A' && c <= b'F') || (c >= b'a' && c <= b'f')
    } else {
        false
    }
}

/// charset for chars: everything printable, escape sequences are allowed!
#[inline]
fn is_char(s: &str) -> bool {
    if s.len() == 1 {
        let c = s.as_bytes()[0];
        c >= 0x20 && c < 0x7F
    } else {
        false
    }
}

/// Struct that implements the lexing logic.
/// Converts a stream of graphemes into a stream
/// of higher-level tokens.
pub struct Lexer<'a> {
    scanner: Scanner<'a>,
}
impl<'a> Lexer<'a> {
    pub fn new(view: &'a SourceView) -> Self {
        Self {
            scanner: Scanner::new(view),
        }
    }
    
    /// Main function of this struct: Does the lexing.
    pub fn lex(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::<Token>::new();
        
        while !self.scanner.done() {
            // Is it a container ?
            if self.scanner.peek(keywords::CONTAINER) {
                self.parse_container(&mut tokens)?;
            }
            // Is it an option ?
            else if self.scanner.peek(keywords::OPTION) {
                self.parse_option(&mut tokens)?;
            }
            // Is it a comment ?
            else if self.scanner.peek(keywords::COMMENT_OPEN) {
                self.parse_comment()?;
            }
            // then it must be whitespace
            else if self.scanner.skip(&mut is_whitespace) == 0 {
                return Err(LexerError::ExpectedKeyword(
                    self.scanner.cursor,
                    format!("{} OR {}", keywords::OPTION, keywords::CONTAINER),
                ));
            }
        }
        
        if tokens.is_empty() {
            Err(LexerError::EOF(
                format!("Expected '{}' or '{}'", keywords::CONTAINER, keywords::OPTION)
            ))
        } else {
            Ok(tokens)
        }
    }
    
    fn parse_comment(&mut self) -> Result<(), LexerError> {
        self.scanner.expect(keywords::COMMENT_OPEN)?;
        
        while !self.scanner.done() {
            // Is it a nested comment ?
            if self.scanner.peek(keywords::COMMENT_OPEN) {
                self.parse_comment()?;
            }
            // Is it a comment close ?
            else if self.scanner.peek(keywords::COMMENT_CLOSE) {
                self.scanner.forward(keywords::COMMENT_CLOSE.len());
                return Ok(());
            }
            // It is a normal char
            else {
                self.scanner.forward(1);
            }
        }
        
        Err(LexerError::EOF(
            "Got an unclosed comment".to_string()
        ))
    }
    
    fn parse_option(&mut self, tokens: &mut Vec<Token>) -> Result<(), LexerError> {
        let identifier_start;
        let identifier_end;
        let value_start;
        let value_end;
        let option_start = self.scanner.cursor;
        let option_end;
        
        self.scanner.expect(keywords::OPTION)?;
        
        // At least one whitespace required after keyword
        if self.scanner.skip(&mut is_whitespace_nonl) == 0 {
            return Err(LexerError::MissingWhitespace(
                self.scanner.cursor
            ));
        }
        
        // After whitespaces an identifier follows
        identifier_start = self.scanner.cursor;
        identifier_end = match self.scanner.skip(&mut is_identifier) {
            0 => {
                return Err(LexerError::ExpectedIdentifier(
                    self.scanner.cursor
                ));
            },
            len @ _ => identifier_start + len,
        };
        
        // After the identifier whitespaces may follow
        self.scanner.skip(&mut is_whitespace_nonl);
        
        // ... until we hit an equals sign
        self.scanner.expect(keywords::ASSIGNMENT)?;
        
        // after an equals sign whitespaces may follow again
        self.scanner.skip(&mut is_whitespace_nonl);
        
        // ... until we hit the value of an option
        value_start = self.scanner.cursor;
        value_end = match self.scanner.skip(&mut is_option_value) {
            0 => {
                return Err(LexerError::ExpectedIdentifier(
                    self.scanner.cursor
                ));
            },
            len @ _ => value_start + len,
        };
        
        // after the value whitespaces may follow
        self.scanner.skip(&mut is_whitespace_nonl);
        
        // and an option definition ends with ';'
        self.scanner.expect(keywords::TERMINATE_STATEMENT)?;
        
        option_end = self.scanner.cursor;
        
        if option_end > option_start && value_end > value_start && identifier_end > identifier_start {
            tokens.push(
                Token::OptionDef(
                    option_start,
                    SourceRange::new(identifier_start, identifier_end),
                    SourceRange::new(value_start, value_end),
                )
            );
        }
        
        Ok(())
    }
    
    fn parse_container(&mut self, tokens: &mut Vec<Token>) -> Result<(), LexerError> {
        let name_start;
        let name_end;
        let container_start = self.scanner.cursor;
        
        self.scanner.expect(keywords::CONTAINER)?;
        
        // after the keyword whitespaces may follow
        if self.scanner.skip(&mut is_whitespace_nonl) < 1 {
            return Err(LexerError::MissingWhitespace(
                self.scanner.cursor
            ));
        }
        
        // after the whitespaces a container name follows
        name_start = self.scanner.cursor;
        name_end = match self.scanner.skip(&mut is_identifier) {
            0 => {
                return Err(LexerError::ExpectedIdentifier(
                    self.scanner.cursor
                ));
            },
            len @ _ => name_start + len,
        };
        
        tokens.push(
            Token::ContainerOpen(container_start, SourceRange::new(name_start, name_end))
        );
        
        // after the name whitespaces may follow
        self.scanner.skip(&mut is_whitespace);
        
        // after the container name a block must be opened
        self.parse_block(tokens)?;
        
        tokens.push(Token::ContainerClose);
        
        Ok(())
    }
    
    fn parse_block(&mut self, tokens: &mut Vec<Token>) -> Result<(), LexerError> {
        let block_start = self.scanner.cursor;
        
        self.scanner.expect(keywords::BLOCK_OPEN)?;
        
        tokens.push(Token::BlockOpen(block_start));
        
        self.parse_variable_listing(tokens)?;
        
        // Before a block delimiter whitespaces may precede
        self.scanner.skip(&mut is_whitespace);
        
        // A block delimiter must close the container
        self.scanner.expect(keywords::BLOCK_CLOSE)?;
        
        tokens.push(Token::BlockClose);
        
        Ok(())
    }
    
    fn parse_variable_listing(&mut self, tokens: &mut Vec<Token>) -> Result<(), LexerError> {
        while !self.scanner.done() {
            self.scanner.skip(&mut is_whitespace);
            
            // Is it the end of a block, then return
            if self.scanner.peek(keywords::BLOCK_CLOSE) {
                return Ok(());
            }
            // Is it a local option ?
            else if self.scanner.peek(keywords::OPTION) && !self.scanner.peek(keywords::VAROPT_OPTIONAL) {
                self.parse_option(tokens)?;
            }
            // Is it a comment ?
            else if self.scanner.peek(keywords::COMMENT_OPEN) {
                self.parse_comment()?;
            }
            // Otherwise it must be a variable definition
            else {
                self.parse_variable_definition(tokens)?;
            }
        }
        
        Err(LexerError::EOF(
            "Block was not closed".to_string()
        ))
    }
    
    fn parse_variable_definition(&mut self, tokens: &mut Vec<Token>) -> Result<(), LexerError> {        
        tokens.push(Token::VariableStart(self.scanner.cursor));
        
        // Variables may start with options
        while !self.scanner.done() {
            // variable optional ?
            if self.scanner.peek(keywords::VAROPT_OPTIONAL) {
                tokens.push(Token::VariableOptional(self.scanner.cursor));
                self.scanner.forward(keywords::VAROPT_OPTIONAL.len());
                
                // white space must follow the keyword
                if self.scanner.skip(&mut is_whitespace_nonl) == 0 {
                    return Err(LexerError::MissingWhitespace(
                        self.scanner.cursor
                    ));
                }
            } 
            // variable repeatable ?
            else if self.scanner.peek(keywords::VAROPT_REPEATS) {
                let repeats_start = self.scanner.cursor;
                self.scanner.forward(keywords::VAROPT_REPEATS.len());
                
                // white space must follow the keyword
                if self.scanner.skip(&mut is_whitespace_nonl) == 0 {
                    return Err(LexerError::MissingWhitespace(
                        self.scanner.cursor
                    ));
                }
                
                tokens.push(Token::VariableRepeatStart(repeats_start));
                
                self.parse_numberset(tokens)?;
                
                tokens.push(Token::VariableRepeatEnd);
                
                // white space must follow after numberset
                if self.scanner.skip(&mut is_whitespace_nonl) == 0 {
                    return Err(LexerError::MissingWhitespace(
                        self.scanner.cursor
                    ));
                }
            }
            // No known option, assume its a variable name
            else {
                break;
            }
        }
        
        // after variable options comes the variable name, but we ignore
        // the concrete value of the name. It is just a help for the developer
        if self.scanner.skip(&mut is_identifier) == 0 {
            return Err(LexerError::ExpectedIdentifier(
                self.scanner.cursor
            ));
        }
        
        // white space may follow after name
        self.scanner.skip(&mut is_whitespace_nonl);
        
        // A type separator is expected
        self.scanner.expect(keywords::VAR_TYPE_SEP)?;
        
        // Optionally whitespaces may follow the type separator
        self.scanner.skip(&mut is_whitespace_nonl);
        
        // Then a typename must follow
        let type_start = self.scanner.cursor;
        let type_end = match self.scanner.skip(&mut is_identifier) {
            0 => {
                return Err(LexerError::ExpectedIdentifier(
                    self.scanner.cursor
                ));
            },
            len @ _ => type_start + len,
        };
        
        tokens.push(Token::VariableType(SourceRange::new(type_start, type_end)));
        
        // Optionally whitespaces may follow the type
        self.scanner.skip(&mut is_whitespace_nonl);
        
        // After the type we either have an assignment with '='
        if self.scanner.peek(keywords::ASSIGNMENT) {
            self.scanner.forward(keywords::ASSIGNMENT.len());
            
            self.scanner.skip(&mut is_whitespace_nonl);
            
            tokens.push(Token::VariableValueStart(self.scanner.cursor));
            
            // After an equals sign we either expect a string literal or a numberset
            if self.scanner.peek(keywords::STRING_DELIM) {
                self.parse_string_literal(tokens)?;
            } else if self.scanner.check(&mut |s| s == keywords::CHAR_DELIM || is_integer(s)) {
                self.parse_numberset(tokens)?;
            } else {
                return Err(LexerError::ExpectedLiteral(
                    self.scanner.cursor,
                    "string OR numberset".to_string(),
                ));
            }
            
            tokens.push(Token::VariableValueEnd);
            
            self.scanner.skip(&mut is_whitespace_nonl);
        }
        // or we may have a new block
        else if self.scanner.peek(keywords::BLOCK_OPEN) {
            self.parse_block(tokens)?;
            
            // after a block whitespaces may follow
            //self.scanner.skip(&mut is_whitespace);
        }
        // or we don't have any value
        else if !self.scanner.peek(keywords::TERMINATE_STATEMENT) {
            return Err(LexerError::ExpectedChar(
                self.scanner.cursor,
                format!("{} OR {} OR {}", keywords::ASSIGNMENT, keywords::BLOCK_OPEN, keywords::TERMINATE_STATEMENT)
            ));
        }
        
        self.scanner.expect(keywords::TERMINATE_STATEMENT)?;
        
        tokens.push(Token::VariableEnd);
        
        Ok(())
    }
    
    fn parse_numberset(&mut self, tokens: &mut Vec<Token>) -> Result<(), LexerError> {
        tokens.push(Token::NumbersetStart(self.scanner.cursor));
        
        while !self.scanner.done() {
            // Do we have a simple char ?
            if self.scanner.peek(keywords::CHAR_DELIM) {
                let (char_start, char_end) = self.parse_char_literal()?;
                
                // Is this a char range ?
                if self.scanner.peek_after(&mut is_whitespace_nonl, keywords::RANGE_OP) {
                    self.scanner.skip(&mut is_whitespace_nonl);
                    self.scanner.forward(keywords::RANGE_OP.len());
                    self.scanner.skip(&mut is_whitespace_nonl);
                    
                    let (limit_start, limit_end) = self.parse_char_literal()?;
                    
                    tokens.push(Token::CharRange(
                        SourceRange::new(char_start, char_end),
                        SourceRange::new(limit_start, limit_end),
                    ));
                } else {
                    tokens.push(Token::Character(SourceRange::new(char_start, char_end)));
                }
            }
            // Otherwise we must have a number
            else {
                let number_start = self.scanner.cursor;
                let number_end = match self.scanner.skip(&mut is_integer) {
                    0 => {
                        return Err(LexerError::InvalidNumber(
                            number_start
                        ));
                    },
                    len @ _ => number_start + len,
                };
                
                // Is this a number range ?
                if self.scanner.peek_after(&mut is_whitespace_nonl, keywords::RANGE_OP) {
                    self.scanner.skip(&mut is_whitespace_nonl);
                    self.scanner.forward(keywords::RANGE_OP.len());
                    self.scanner.skip(&mut is_whitespace_nonl);
                    
                    let limit_start = self.scanner.cursor;
                    let limit_end = match self.scanner.skip(&mut is_integer) {
                        0 => {
                            return Err(LexerError::InvalidNumber(
                                limit_start
                            ));
                        },
                        len @ _ => limit_start + len,
                    };
                    
                    tokens.push(Token::IntegerRange(
                        SourceRange::new(number_start, number_end),
                        SourceRange::new(limit_start, limit_end),
                    ));
                }
                // This is a single number
                else {
                    tokens.push(Token::Integer(SourceRange::new(number_start, number_end)));
                }
            }
            
            // If we have a ',', parse again
            if self.scanner.peek_after(&mut is_whitespace_nonl, keywords::NUMBERSET_DELIM) {
                self.scanner.skip(&mut is_whitespace_nonl);
                self.scanner.forward(keywords::NUMBERSET_DELIM.len());
                self.scanner.skip(&mut is_whitespace_nonl);
            } else {
                break;
            }
        }
        
        tokens.push(Token::NumbersetEnd);
        
        Ok(())
    }
    
    fn parse_char_literal(&mut self) -> Result<(usize, usize), LexerError> {
        self.scanner.expect(keywords::CHAR_DELIM)?;
        
        let start = self.scanner.cursor;
        let mut seen_backslash = false;
        
        self.scanner.skip(&mut |s| {
            if s == keywords::CHAR_DELIM {
                let ret = seen_backslash;
                seen_backslash = false;
                ret
            } else if seen_backslash {
                seen_backslash = false;
                true
            } else if s == "\\" {
                seen_backslash = true;
                true
            } else {
                is_char(s)
            }
        });
        
        let end = self.scanner.cursor;
        
        if start == end || end - start > 2 {
            return Err(LexerError::InvalidCharLiteral(
                start
            ));
        }
        
        self.scanner.forward(1);
        
        Ok((start, end))
    }
    
    fn parse_string_literal(&mut self, tokens: &mut Vec<Token>) -> Result<(), LexerError> {
        self.scanner.expect(keywords::STRING_DELIM)?;
        
        let mut seen_backslash = false;
        let string_start = self.scanner.cursor;
        let string_end = string_start + self.scanner.skip(&mut |s| {
            if s == keywords::STRING_DELIM {
                let ret = seen_backslash;
                seen_backslash = false;
                ret
            } else if seen_backslash {
                seen_backslash = false;
                true
            } else if s == "\\" {
                seen_backslash = true;
                true
            } else {
                is_char(s)
            }
        });
        
        tokens.push(Token::String(SourceRange::new(string_start, string_end)));
        
        self.scanner.expect(keywords::STRING_DELIM)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, SourceView};
    
    #[test]
    #[should_panic]
    fn unclosed_comment() {
        let input = "/*";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn empty_comment() {
        let input = "/**/struct x{}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn valid_nested_comment() {
        let input = "/*/**/*/struct x{}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    #[should_panic]
    fn unclosed_nested_comment() {
        let input = "/*/**/struct x{}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    #[should_panic]
    fn double_closed_comment() {
        let input = "/**/*/struct x{}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn comment_at_block_start() {
        let input = "struct x{/**/}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn comment_at_block_end() {
        let input = "struct x{x:y;/**/}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn comment_inbetween_vars() {
        let input = "struct x{x:y;/**/x:y;}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    #[should_panic]
    fn option_no_assignment() {
        let input = "option key;";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    #[should_panic]
    fn option_no_value() {
        let input = "option key=;";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn option_valid() {
        let input = "option key=value;";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    #[should_panic]
    fn option_reject_key() {
        let input = "option x!y=value;";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn valid_number_variable() {
        let input = "struct x{x:u16=3;}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn valid_numberset() {
        let input = "struct x{x:u16=-8,-3,0..1,101;}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn valid_nonsense_numberset() {
        let input = "struct x{x:u16=-,-,0..0,'\\X';}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn valid_numberset_chars() {
        let input = "struct x{x:u16='A','SS','D','FF';}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    #[should_panic]
    fn invalid_numberset_chars() {
        let input = "struct x{x:u16='AAAA','SSSS','DDDD','FFFF';}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn empty_string_literal() {
        let input = "struct x{x:string=\"\";}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn valid_string_literal() {
        let input = "struct x{x:string=\"\\\" hello world! \\xCC\\x0d\";}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn duplicate_variable_flags() {
        let input = "struct x{optional optional optional repeats 3 repeats 4 x:x;}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn multiple_structs() {
        let input = "struct x{}struct x{}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn test_nested_blocks() {
        let input = "struct x{x:y{x:y{x:y;};};}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn test_number_formats() {
        let input = "struct x{x:x=0x01..0o77,0b10101;}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn test_numberset_whitespaces() {
        let input = "struct x{x:x=0x01 .. 0o77 , 0b10101 .. 1 , 2;}";
        let view = SourceView::new(input);
        Lexer::new(&view).lex().unwrap();
    }
    
    #[test]
    fn valid_grammar() {
        let input = "
option endianness = little;

struct ELFIdent {
    magic: string = \"\\x7FELF\";
    class: u8 = 1,2;
    data: u8 = 1,2;
    version: u8 = 1;
    osabi: u8 = 0x00..0x04,0x06..0x12;
    abiversion: u8 = 0;
    repeats 7 pad: u8;
}

struct Root {
    ident: ELFIdent;
}        
";
        let view = SourceView::new(input);

        Lexer::new(&view).lex().unwrap();
    }
}
