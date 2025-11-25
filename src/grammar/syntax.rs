
pub fn is_whitespace_nl(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

pub const START_COMMENT: &str = "/*";
pub const END_COMMENT: &str = "*/";
pub const START_NONTERMINAL: &str = "<";

