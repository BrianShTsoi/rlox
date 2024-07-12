use crate::scanner::token::Token;

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
