
pub fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t')
}
pub fn is_whitespace_nl(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

pub fn is_nonterminal(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | ':')
}

pub fn is_forbidden_in_string(c: char) -> bool {
    c == '\n'
}

pub const START_COMMENT: &str = "/*";
pub const END_COMMENT: &str = "*/";
pub const START_NONTERMINAL: &str = "<";
pub const END_NONTERMINAL: &str = ">";
pub const RULE_SEPARATOR: &str = "=>";
pub const END_RULE: &str = "\n";
pub const START_STRING: &str = "\"";
pub const END_STRING: &str = "\"";
pub const START_GROUP: &str = "(";
pub const END_GROUP: &str = ")";
pub const OPERATOR_OR: &str = "||";
pub const TYPE_U8: &str = "u8";
pub const TYPE_I8: &str = "i8";
pub const TYPE_U16: &str = "u16";
pub const TYPE_I16: &str = "i16";
pub const TYPE_U32: &str = "u32";
pub const TYPE_I32: &str = "i32";
pub const TYPE_U64: &str = "u64";
pub const TYPE_I64: &str = "i64";
pub const START_NUMBERSET: &str = "{";
pub const END_NUMBERSET: &str = "}";
pub const PREFIX_HEXADECIMAL: &str = "0x";
pub const OPERATOR_RANGE: &str = "..";
pub const OPERATOR_SET_SEPARATOR: &str = ",";
pub const DIRECTIVE_NAMESPACE: &str = "namespace";
pub const OPERATOR_NAMESPACE_SEPARATOR: &str = "::";
pub const DIRECTIVE_CLEAR: &str = "clear";
