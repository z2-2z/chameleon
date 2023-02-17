use crate::{
    grammar::{
        Grammar, HasOptions,
        Container, Endianness,
        Scheduling, Variable,
        VariableType, IntegerValue,
        VariableOptions, NumbersetType,
        BytearrayValue, StringId,
        ContainerId, ContainerType,
        ContainerOptions, Depth,
    },
    frontend::{
        lexer::{Token, TokenId},
        source_view::{SourceRange, SourceView},
        keywords,
        bitpattern::FromBitPattern,
        range::NewRange,
    },
};
use std::ops::Range;
use num_traits::{
    Num,
    cast::NumCast,
};

#[derive(Debug)]
pub enum ParserError {
    UnknownOptionValue(SourceRange),
    UnknownOptionName(SourceRange),
    DuplicateContainerName(SourceRange),
    EOF(String),
    UnexpectedToken(Option<usize>, String),
    InvalidKeyword(SourceRange, String),
    InvalidNumber(usize, SourceRange),
    InvalidRange(SourceRange),
    CharacterNotAllowed(SourceRange),
    InvalidCharacter(SourceRange),
    InvalidNumberset(usize),
    InvalidTypeName(SourceRange),
    InvalidString(SourceRange, String),
    NoRoot,
    UnresolvedRef(SourceRange),
    EmptyBlock(usize),
    IllegalContainerName(SourceRange),
    NonLocalOption(SourceRange),
    IllegalOptionValue(SourceRange),
}

struct TokenScanner<'a> {
    view: &'a SourceView,
    tokens: &'a [Token],
    cursor: usize,
}
impl<'a> TokenScanner<'a> {
    fn new(view: &'a SourceView, tokens: &'a [Token]) -> Self {
        Self {
            view,
            tokens,
            cursor: 0,
        }
    }
    
    fn expect(&mut self, id: TokenId) -> Result<&'a Token, ParserError> {
        if self.cursor < self.tokens.len() {
            if self.tokens[self.cursor].id() != id {
                if let Some(pos) = self.tokens[self.cursor].pos() {
                    Err(ParserError::UnexpectedToken(
                        Some(pos),
                        format!("Expected {}", id.description())
                    ))
                } else {
                    Err(ParserError::UnexpectedToken(
                        None,
                        format!("Expected {} after {}", id.description(), self.tokens[self.cursor - 1].id().description())
                    ))
                }
            } else {
                let idx = self.cursor;
                self.cursor += 1;
                Ok(&self.tokens[idx])
            }
        } else {
            Err(ParserError::EOF(
                format!("Expected {}", id.description())
            ))
        }
    }
    
    fn current(&self) -> Option<&'a Token> {
        if self.cursor < self.tokens.len() {
            Some(&self.tokens[self.cursor])
        } else {
            None
        }
    }
    
    fn forward(&mut self, len: usize) {
        if self.cursor < self.tokens.len() {
            self.cursor += len;
        }
    }
    
    fn done(&self) -> bool {
        self.cursor >= self.tokens.len()
    }
    
    fn get_source(&self, range: &SourceRange) -> &'a str {
        // The lexer ensures that range is in bounds
        self.view.range(&range)
    }
}

#[inline]
fn is_hex_char(c: u8) -> bool {
    (c >= 0x30 && c < 0x3a) || (c >= 0x41 && c <= 0x46) || (c >= 0x61 && c <= 0x66)
}
#[inline]
fn hex_to_dec(c: u8) -> u8 {
    if c < 0x3a {
        c - 0x30
    } else if c <= 0x46 {
        c - 0x41 + 10
    } else {
        c - 0x61 + 10
    }
}

pub struct Parser<'a> {
    scanner: TokenScanner<'a>,
    options_stack: Vec<ContainerOptions>,
}
impl<'a> Parser<'a> {
    pub fn new(view: &'a SourceView, tokens: &'a [Token]) -> Self {
        Self {
            scanner: TokenScanner::new(view, tokens),
            options_stack: Vec::<ContainerOptions>::new(),
        }
    }
    
    pub fn parse(&mut self) -> Result<Grammar, ParserError> {
        let mut grammar = Grammar::new();
        
        // Before any containers appear a user might define some global options
        *grammar.options_mut() = self.parse_options_list()?;
        
        // Now only containers may follow
        while !self.scanner.done() {
            let container = self.parse_container(&mut grammar)?;
            grammar.add_container(container);
        }
        
        assert_eq!(self.options_stack.len(), 1);
        
        // Find the root container
        if let Some(id) = self.find_container(&grammar, keywords::ROOT_CONTAINER) {
            grammar.set_root(id);
        } else {
            return Err(ParserError::NoRoot);
        }
        
        // Resolve container references
        for (container_id, var, name) in grammar.unresolved_names() {
            let source = self.scanner.get_source(&name);
            
            let target = if let Some(id) = self.find_container(&grammar, source) {
                id
            } else {
                return Err(ParserError::UnresolvedRef(name.clone()));
            };
            
            grammar.container_mut(container_id).unwrap().resolve_reference(var, target);
        }
        
        Ok(grammar)
    }
    
    fn find_container(&self, grammar: &Grammar, dest: &str) -> Option<ContainerId> {
        for container in grammar.containers() {
            if let Some(name) = container.name() {
                if self.scanner.get_source(name) == dest {
                    return Some(container.id());
                }
            }
        }
        
        None
    }
    
    fn parse_options_list(&mut self) -> Result<ContainerOptions, ParserError> {
        let is_global = self.options_stack.is_empty();
        
        let mut ret = if let Some(elem) = self.options_stack.last() {
            elem.clone()
        } else {
            ContainerOptions::default()
        };
        
        while let Some(token) = self.scanner.current() {
            match token {
                Token::OptionDef(_, key, value) => {
                    match self.scanner.get_source(key) {
                        keywords::OPTION_ENDIANNESS => {
                            let value = match self.scanner.get_source(value) {
                                "little" => Endianness::Little,
                                "big" => Endianness::Big,
                                "native" => Endianness::Native,
                                _ => {
                                    return Err(ParserError::UnknownOptionValue(value.clone()));
                                }
                            };
                            
                            ret.set_endianness(value);
                        },
                        keywords::OPTION_SCHEDULING => {
                            let value = match self.scanner.get_source(value) {
                                "round-robin" => Scheduling::RoundRobin,
                                "random" => Scheduling::Random,
                                _ => {
                                    return Err(ParserError::UnknownOptionValue(value.clone()));
                                }
                            };
                            
                            ret.set_scheduling(value);
                        },
                        keywords::OPTION_DEPTH => {
                            if !is_global {
                                return Err(ParserError::NonLocalOption(key.clone()));
                            }
                            
                            let source = self.scanner.get_source(value);
                            let value = if source == keywords::DEPTH_UNLIMITED {
                                Depth::Unlimited
                            } else {
                                if let Ok(result) = source.parse::<usize>() {
                                    if result == 0 {
                                        return Err(ParserError::IllegalOptionValue(value.clone()));
                                    }
                                    
                                    Depth::Limited(result)
                                } else {
                                    return Err(ParserError::IllegalOptionValue(value.clone()));
                                }
                            };
                            
                            ret.set_depth(value);
                        },
                        _ => {
                            return Err(ParserError::UnknownOptionName(key.clone()));
                        },
                    }
                },
                _ => {
                    break;
                }
            }
            
            self.scanner.forward(1);
        }
        
        self.options_stack.push(ret.clone());
        Ok(ret)
    }
    
    fn parse_container(&mut self, grammar: &mut Grammar) -> Result<Container, ParserError> {
        let name = match self.scanner.expect(TokenId::ContainerOpen)? {
            Token::ContainerOpen(_, name) => {
                let source = self.scanner.get_source(&name);
                
                /* check that name isn't a keyword */
                match source {
                    keywords::CONTAINER |
                    keywords::TYPE_U8 |
                    keywords::TYPE_I8 |
                    keywords::TYPE_U16 |
                    keywords::TYPE_I16 |
                    keywords::TYPE_U32 |
                    keywords::TYPE_I32 |
                    keywords::TYPE_U64 |
                    keywords::TYPE_I64 |
                    keywords::TYPE_ONEOF |
                    keywords::TYPE_STRING |
                    keywords::TYPE_BYTES |
                    keywords::TYPE_CHAR => {
                        return Err(ParserError::IllegalContainerName(name.clone()));
                    },
                    _ => {},
                }
                
                /* check if name already exists */
                for container in grammar.containers() {
                    if let Some(other_name) = container.name() {
                        if source == self.scanner.get_source(other_name) {
                            return Err(ParserError::DuplicateContainerName(name.clone()));
                        }
                    }
                }
                
                name.clone()
            },
            _ => unreachable!(),
        };
        
        let id = grammar.reserve_container_id();
        let mut container = Container::new(id, ContainerType::Struct, grammar.options().clone(), Some(name));
        
        // After a container definition a block must be opened
        self.parse_block(grammar, &mut container)?;
        
        // After closing a block the container must end
        self.scanner.expect(TokenId::ContainerClose)?;
        
        Ok(container)
    }
    
    fn parse_block(&mut self, grammar: &mut Grammar, container: &mut Container) -> Result<(), ParserError> {
        let mut had_vars = false;
        let block_start = match self.scanner.expect(TokenId::BlockOpen)? {
            Token::BlockOpen(block_start) => *block_start,
            _ => unreachable!(),
        };
        
        // Options may be overwritten in a block
        *container.options_mut() = self.parse_options_list()?;
        
        // After options variables must follow
        while let Some(token) = self.scanner.current() {
            match token {
                Token::BlockClose => {
                    if !had_vars {
                        return Err(ParserError::EmptyBlock(block_start));
                    }
                    
                    assert!(self.options_stack.pop().is_some());
                    
                    self.scanner.forward(1);
                    return Ok(());
                },
                
                Token::VariableStart(_) => {
                    had_vars = true;
                    let variable = self.parse_variable_definition(grammar)?;
                    container.add_variable(variable);
                },
                
                _ => {
                    // In order to get the best error message call expect()
                    self.scanner.expect(TokenId::VariableStart)?;
                    unreachable!();
                },
            }
        }
        
        Err(ParserError::EOF(
            "Block was not closed".to_string()
        ))
    }
    
    fn parse_variable_definition(&mut self, grammar: &mut Grammar) -> Result<Variable, ParserError> {
        let var_start = match self.scanner.expect(TokenId::VariableStart)? {
            Token::VariableStart(var_start) => var_start,
            _ => unreachable!(),
        };
        
        // Parse variable options
        let mut had_optional = false;
        let mut had_repeats = false;
        let mut var_opts = VariableOptions::default();
        
        while let Some(token) = self.scanner.current() {
            match token {
                Token::VariableOptional(pos) => {
                    if had_optional {
                        return Err(ParserError::InvalidKeyword(
                            SourceRange::new(*pos, pos + keywords::VAROPT_OPTIONAL.len()),
                            "Multiple occurences of variable options not allowed".to_string(),
                        ));
                    }
                    
                    var_opts.set_optional();
                    had_optional = true;
                },
                Token::VariableRepeatStart(pos) => {
                    if had_repeats {
                        return Err(ParserError::InvalidKeyword(
                            SourceRange::new(*pos, pos + 1),
                            "Multiple occurences of variable options not allowed".to_string(),
                        ));
                    }
                    
                    self.scanner.forward(1);
                    let ranges = self.parse_numberset::<u32>(false)?;
                    let id = grammar.add_numberset(NumbersetType::U32(ranges));
                    var_opts.set_repeats(id);
                    had_repeats = true;
                },
                _ => {
                    break;
                },
            }
            
            self.scanner.forward(1);
        }
        
        let type_name = match self.scanner.expect(TokenId::VariableType)? {
            Token::VariableType(name) => {
                name.clone()
            },
            _ => unreachable!(),
        };
        
        let var_type = match self.scanner.current() {
            Some(Token::VariableValueStart(_)) => {
                self.scanner.forward(1);
                
                let ret = match self.scanner.current() {
                    Some(Token::String(_)) => {
                        let is_binary = match self.scanner.get_source(&type_name) {
                            keywords::TYPE_STRING => false,
                            keywords::TYPE_BYTES => true,
                            _ => {
                                return Err(ParserError::InvalidTypeName(type_name.clone()));
                            },
                        };
                        
                        // parse string binary or not
                        let id = self.parse_string_literal(grammar, is_binary)?;
                        
                        if is_binary {
                            VariableType::Bytes(BytearrayValue::Literal(id))
                        } else {
                            VariableType::String(BytearrayValue::Literal(id))
                        }
                    },
                    Some(Token::NumbersetStart(_)) => {
                        match self.scanner.get_source(&type_name) {
                            keywords::TYPE_CHAR |
                            keywords::TYPE_U8 => {
                                let ranges = self.parse_numberset::<u8>(true)?;
                                let id = grammar.add_numberset(NumbersetType::U8(ranges));
                                VariableType::U8(IntegerValue::FromSet(id))
                            },
                            keywords::TYPE_I8 => {
                                let ranges = self.parse_numberset::<i8>(true)?;
                                let id = grammar.add_numberset(NumbersetType::I8(ranges));
                                VariableType::I8(IntegerValue::FromSet(id))
                            },
                            keywords::TYPE_U16 => {
                                let ranges = self.parse_numberset::<u16>(false)?;
                                let id = grammar.add_numberset(NumbersetType::U16(ranges));
                                VariableType::U16(IntegerValue::FromSet(id))
                            },
                            keywords::TYPE_I16 => {
                                let ranges = self.parse_numberset::<i16>(false)?;
                                let id = grammar.add_numberset(NumbersetType::I16(ranges));
                                VariableType::I16(IntegerValue::FromSet(id))
                            },
                            keywords::TYPE_U32 => {
                                let ranges = self.parse_numberset::<u32>(false)?;
                                let id = grammar.add_numberset(NumbersetType::U32(ranges));
                                VariableType::U32(IntegerValue::FromSet(id))
                            },
                            keywords::TYPE_I32 => {
                                let ranges = self.parse_numberset::<i32>(false)?;
                                let id = grammar.add_numberset(NumbersetType::I32(ranges));
                                VariableType::I32(IntegerValue::FromSet(id))
                            },
                            keywords::TYPE_U64 => {
                                let ranges = self.parse_numberset::<u64>(false)?;
                                let id = grammar.add_numberset(NumbersetType::U64(ranges));
                                VariableType::U64(IntegerValue::FromSet(id))
                            },
                            keywords::TYPE_I64 => {
                                let ranges = self.parse_numberset::<i64>(false)?;
                                let id = grammar.add_numberset(NumbersetType::I64(ranges));
                                VariableType::I64(IntegerValue::FromSet(id))
                            },
                            keywords::TYPE_STRING => {
                                let ranges = self.parse_numberset::<u32>(false)?;
                                let id = grammar.add_numberset(NumbersetType::U32(ranges));
                                VariableType::String(BytearrayValue::Any(id))
                            },
                            keywords::TYPE_BYTES => {
                                let ranges = self.parse_numberset::<u32>(false)?;
                                let id = grammar.add_numberset(NumbersetType::U32(ranges));
                                VariableType::Bytes(BytearrayValue::Any(id))
                            },
                            _ => {
                                return Err(ParserError::InvalidTypeName(type_name.clone()));
                            },
                        }
                    },
                    _ => unreachable!()
                };
                
                self.scanner.expect(TokenId::VariableValueEnd)?;
                ret
            },
            Some(Token::BlockOpen(_)) => {
                // Check type name
                let typ = match self.scanner.get_source(&type_name) {
                    keywords::TYPE_ONEOF => ContainerType::Oneof,
                    keywords::CONTAINER => ContainerType::Struct,
                    _ => {
                        return Err(ParserError::InvalidKeyword(
                            type_name.clone(),
                            format!("Expected '{}' or '{}'", keywords::TYPE_ONEOF, keywords::CONTAINER),
                        ));
                    },
                };
                
                // Create container
                let id = grammar.reserve_container_id();
                let mut container = Container::new(
                    id, 
                    typ, 
                    grammar.options().clone(), 
                    if typ == ContainerType::Oneof {
                        None
                    } else {
                        Some(SourceRange::new(*var_start, *var_start))
                    }
                );
                self.parse_block(grammar, &mut container)?;
                
                if typ == ContainerType::Oneof && container.variables().len() == 1 {
                    return Err(ParserError::InvalidKeyword(
                        type_name.clone(),
                        format!("'{}' must have more than 1 variable", keywords::TYPE_ONEOF),
                    ));
                }
                
                // Create type
                grammar.add_container(container);
                
                if typ == ContainerType::Oneof {
                    VariableType::Oneof(id)
                } else {
                    VariableType::ContainerRef(id)
                }
            },
            Some(Token::VariableEnd) => self.parse_variable_value_any(type_name)?,
            _ => unreachable!(),
        };
        
        self.scanner.expect(TokenId::VariableEnd)?;
        
        Ok(Variable::new(var_opts, var_type))
    }
    
    fn parse_variable_value_any(&mut self, type_name: SourceRange) -> Result<VariableType, ParserError> {
        match self.scanner.get_source(&type_name) {
            keywords::TYPE_CHAR |
            keywords::TYPE_U8 => Ok(VariableType::U8(IntegerValue::Any)),
            keywords::TYPE_I8 => Ok(VariableType::I8(IntegerValue::Any)),
            keywords::TYPE_U16 => Ok(VariableType::U16(IntegerValue::Any)),
            keywords::TYPE_I16 => Ok(VariableType::I16(IntegerValue::Any)),
            keywords::TYPE_U32 => Ok(VariableType::U32(IntegerValue::Any)),
            keywords::TYPE_I32 => Ok(VariableType::I32(IntegerValue::Any)),
            keywords::TYPE_U64 => Ok(VariableType::U64(IntegerValue::Any)),
            keywords::TYPE_I64 => Ok(VariableType::I64(IntegerValue::Any)),
            keywords::CONTAINER |
            keywords::TYPE_STRING |
            keywords::TYPE_BYTES |
            keywords::TYPE_ONEOF => Err(ParserError::InvalidKeyword(
                type_name.clone(),
                "Only number types can be specified without an assignment".to_string()
            )),
            _ => Ok(VariableType::ResolveContainerRef(type_name)),
        }
    }
    
    fn parse_numberset<T>(&mut self, allow_chars: bool) -> Result<Vec<Range<T>>, ParserError>
    where
        T: Num + Copy + core::cmp::Ord + NumCast + std::fmt::Debug + FromBitPattern,
    {
        let numberset_start = if let Token::NumbersetStart(start) = self.scanner.expect(TokenId::NumbersetStart)? {
            *start
        } else {
            unreachable!();
        };
        
        let mut ranges = Vec::<Range<T>>::new();
        
        while let Some(token) = self.scanner.current() {
            match token {
                Token::NumbersetEnd => {
                    self.scanner.forward(1);
                    break;
                },
                Token::Integer(literal) => {
                    let number = self.parse_single_integer(literal)?;
                    ranges.push(Range::new(number, number));
                },
                Token::IntegerRange(lower, upper) => {
                    let lower_number: T = self.parse_single_integer(lower)?;
                    let upper_number: T = self.parse_single_integer(upper)?;
                    
                    // upper bound must be greater than lower bound
                    if upper_number.cmp(&lower_number) != core::cmp::Ordering::Greater {
                        return Err(ParserError::InvalidRange(
                            SourceRange::new(lower.start, upper.end)
                        ));
                    }
                    
                    ranges.push(Range::new(lower_number, upper_number));
                },
                Token::CharRange(lower, upper) => {
                    if !allow_chars {
                        return Err(ParserError::CharacterNotAllowed(
                            SourceRange::new(lower.start - 1, upper.end + 1)
                        ));
                    }
                    
                    let lower_char = self.parse_char_literal(lower)?;
                    let upper_char = self.parse_char_literal(upper)?;
                    
                    if upper_char < lower_char {
                        return Err(ParserError::InvalidRange(
                            SourceRange::new(lower.start - 1, upper.end + 1)
                        ));
                    }
                    
                    let lower_t = if let Some(t) = T::from(lower_char) {
                        t
                    } else {
                        return Err(ParserError::InvalidCharacter(lower.clone()));
                    };
                    let upper_t = if let Some(t) = T::from(upper_char) {
                        t
                    } else {
                        return Err(ParserError::InvalidCharacter(upper.clone()));
                    };
                    
                    ranges.push(Range::new(lower_t, upper_t));
                },
                Token::Character(literal) => {
                    if !allow_chars {
                        return Err(ParserError::CharacterNotAllowed(
                            SourceRange::new(literal.start - 1, literal.end + 1)
                        ));
                    }
                    
                    let c = self.parse_char_literal(literal)?;
                    
                    if let Some(number) = T::from(c) {
                        ranges.push(Range::new(number, number));
                    } else {
                        return Err(ParserError::InvalidCharacter(literal.clone()));
                    }
                },
                _ => unreachable!(),
            }
            
            self.scanner.forward(1);
        }
        
        if ranges.is_empty() {
            return Err(ParserError::InvalidNumberset(
                numberset_start
            ));
        }
        
        // Minimize ranges
        ranges.sort_by(|a, b| (a.start, a.end).cmp(&(b.start, b.end)));
        
        let mut i = 0;
        while i < ranges.len() - 1 {
            if ranges[i] == ranges[i + 1] {
                ranges.remove(i + 1);
                i = i.wrapping_sub(1)
            } else if ranges[i].end >= ranges[i + 1].start || ranges[i].end + T::from(1).unwrap() == ranges[i + 1].start {
                // combine adjacent ranges
                let a = ranges.remove(i);
                let b = ranges.remove(i);
                ranges.insert(i, Range::new(a.start, b.end));
                i = i.wrapping_sub(1)
            }
            
            i = i.wrapping_add(1);
        }
        
        Ok(ranges)
    }
    
    fn parse_single_integer<T>(&mut self, literal: &SourceRange) -> Result<T, ParserError>
    where
        T: Num + Copy + core::cmp::Ord + NumCast + FromBitPattern,
    {
        let source = self.scanner.get_source(literal);
        
        // Is it a hexadecimal number ?
        if source.len() > 2 && source.starts_with("0x") {
            if let Some(number) = T::from_hex_pattern(&source[2..]) {
                Ok(number)
            } else {
                Err(ParserError::InvalidNumber(
                    16,
                    literal.clone(),
                ))
            }
        }
        // Is it a octal number ?
        else if source.len() > 2 && source.starts_with("0o") {
            if let Some(number) = T::from_oct_pattern(&source[2..]) {
                Ok(number)
            } else {
                Err(ParserError::InvalidNumber(
                    8,
                    literal.clone(),
                ))
            }
        }
        // Is it a binary number ?
        else if source.len() > 2 && source.starts_with("0b") {
            if let Some(number) = T::from_bin_pattern(&source[2..]) {
                Ok(number)
            } else {
                Err(ParserError::InvalidNumber(
                    2,
                    literal.clone(),
                ))
            }
        }
        // Then it must be a decimal number
        else {
            if let Ok(number) = T::from_str_radix(source, 10) {
                Ok(number)
            } else {
                Err(ParserError::InvalidNumber(
                    10,
                    literal.clone(),
                ))
            }
        }
    }
    
    fn parse_char_literal(&mut self, literal: &SourceRange) -> Result<u8, ParserError> {
        let source = self.scanner.get_source(literal);
        
        if source.len() == 2 {
            if source.as_bytes()[0] == b'\\' {
                match &source[1..] {
                    "\\" => Ok(b'\\'),
                    "r" => Ok(b'\r'),
                    "'" => Ok(b'\''),
                    "n" => Ok(b'\n'),
                    "t" => Ok(b'\t'),
                    "0" => Ok(0),
                    "a" => Ok(7),
                    "b" => Ok(8),
                    "v" => Ok(11),
                    "f" => Ok(12),
                    _ => Err(ParserError::InvalidCharacter(literal.clone()))
                }
            } else {
                Err(ParserError::InvalidCharacter(literal.clone()))
            }
        } else {
            Ok(source.as_bytes()[0])
        }
    }
    
    fn parse_string_literal(&mut self, grammar: &mut Grammar, is_binary: bool) -> Result<StringId, ParserError> {
        let literal = match self.scanner.expect(TokenId::String)? {
            Token::String(literal) => literal,
            _ => unreachable!(),
        };
        let source = self.scanner.get_source(&literal).as_bytes();
        
        if source.is_empty() {
            return Err(ParserError::InvalidString(
                SourceRange::new(literal.start - 1, literal.end + 1),
                "strings cannot be empty".to_string(),
            ));
        }
        
        let mut buf = Vec::<u8>::new();
        let mut i = 0;
        
        while i < source.len() {
            let c = if source[i] == b'\\' {
                i += 1;
                match source[i] {
                    b'\\' => b'\\', 
                    b'r' => b'\r',
                    b'"' => b'"',
                    b'n' => b'\n',
                    b't' => b'\t',
                    b'0' => 0,
                    b'a' => 7,
                    b'b' => 8,
                    b'v' => 11,
                    b'f' => 12,
                    b'x' => {
                        i += 2;
                        
                        if i >= source.len() || !is_hex_char(source[i - 1]) || !is_hex_char(source[i]) {
                            return Err(ParserError::InvalidString(
                                SourceRange::new(literal.start + i - 3, std::cmp::min(literal.start + i + 1, literal.end)),
                                "Invalid escape character".to_string(),
                            ));
                        }
                        
                        if !is_binary {
                            return Err(ParserError::InvalidString(
                                SourceRange::new(literal.start + i - 3, literal.start + i + 1),
                                format!("This escape sequence is only allowed in variables of type '{}'", keywords::TYPE_BYTES)
                            ));
                        }
                        
                        hex_to_dec(source[i - 1]) * 16 + hex_to_dec(source[i])
                    },
                    _ => {
                        return Err(ParserError::InvalidString(
                            SourceRange::new(literal.start + i, literal.start + i + 2),
                            "Invalid escape character".to_string(),
                        ))
                    },
                }
            } else {
                source[i]
            };
            
            buf.push(c);
            
            i += 1;
        }
        
        Ok(grammar.add_string(buf))
    }
}
