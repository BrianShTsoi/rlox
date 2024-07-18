use crate::scanner::token::Token;
use std::fmt;

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Token, // TODO: supposed to be Object
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", left, operator, right)
            }
            Self::Grouping { expression } => {
                write!(f, "({})", expression)
            }
            Self::Literal { value } => {
                write!(f, "({})", value)
            }
            Self::Unary { operator, right } => {
                write!(f, "({} ({}))", operator, right)
            }
        }
    }
}
