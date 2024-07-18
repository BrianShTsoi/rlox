use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String(String),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

// For ToString trait
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LeftParen => f.write_str("LeftParen"),
            Self::RightParen => f.write_str("RightParen"),
            Self::LeftBrace => f.write_str("LeftBrace"),
            Self::RightBrace => f.write_str("RightBrace"),
            Self::Comma => f.write_str("Comma"),
            Self::Dot => f.write_str("Dot"),
            Self::Minus => f.write_str("Minus"),
            Self::Plus => f.write_str("Plus"),
            Self::Semicolon => f.write_str("Semicolon"),
            Self::Slash => f.write_str("Slash"),
            Self::Star => f.write_str("Star"),
            Self::Bang => f.write_str("Bang"),
            Self::BangEqual => f.write_str("BangEqual"),
            Self::Equal => f.write_str("Equal"),
            Self::EqualEqual => f.write_str("EqualEqual"),
            Self::Greater => f.write_str("Greater"),
            Self::GreaterEqual => f.write_str("GreaterEqual"),
            Self::Less => f.write_str("Less"),
            Self::LessEqual => f.write_str("LessEqual"),
            Self::Identifier => f.write_str("Identifier"),
            Self::String(s) => write!(f, "String({s})"),
            Self::Number(n) => write!(f, "Number({n})"),
            Self::And => f.write_str("And"),
            Self::Class => f.write_str("Class"),
            Self::Else => f.write_str("Else"),
            Self::False => f.write_str("False"),
            Self::Fun => f.write_str("Fun"),
            Self::For => f.write_str("For"),
            Self::If => f.write_str("If"),
            Self::Nil => f.write_str("Nil"),
            Self::Or => f.write_str("Or"),
            Self::Print => f.write_str("Print"),
            Self::Return => f.write_str("Return"),
            Self::Super => f.write_str("Super"),
            Self::This => f.write_str("This"),
            Self::True => f.write_str("True"),
            Self::Var => f.write_str("Var"),
            Self::While => f.write_str("While"),
            Self::Eof => f.write_str("Eof"),
        }
    }
}
