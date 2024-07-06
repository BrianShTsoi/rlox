use crate::token_type::TokenType;
use std::fmt;

#[derive(Clone)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    // TODO: Should be an object
    // TODO: Literal can be trait, or an enum on top of TokenType?
    literal: String,
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: String, line: usize) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

// For ToString trait
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.token_type, self.lexeme, self.literal)
    }
}
