
pub fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t')
}
pub fn is_whitespace_nl(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

pub fn is_nonterminal(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_digit() || matches!(c, '-' | '_')
}

pub const START_COMMENT: &str = "/*";
pub const END_COMMENT: &str = "*/";
pub const START_NONTERMINAL: &str = "<";
pub const END_NONTERMINAL: &str = ">";
pub const RULE_SEPARATOR: &str = "=>";
pub const END_RULE: &str = "\n";