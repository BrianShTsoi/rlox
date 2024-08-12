use crate::scanner::token::Token;
// use std::fmt;

#[derive(Debug)]
pub enum Stmt {
    ExprStmt {
        expr: Box<Expr>,
    },
    PrintStmt {
        expr: Box<Expr>,
    },
    VarStmt {
        var_name: Token,
        initializer: Option<Box<Expr>>,
    },
    BlockStmt {
        stmt_list: Vec<Stmt>,
    },
}

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
    Variable {
        name: Token,
    },
    Assignment {
        var_name: Token,
        value: Box<Expr>,
    },
}

// impl fmt::Display for Expr {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::Binary {
//                 left,
//                 operator,
//                 right,
//             } => {
//                 write!(
//                     f,
//                     "\x1b[31m(\x1b[0m{} {} {}\x1b[31m)\x1b[0m",
//                     left, operator, right
//                 )
//             }
//             Self::Grouping { expression } => {
//                 write!(f, "\x1b[32m(\x1b[0m{}\x1b[32m)\x1b[0m", expression)
//             }
//             Self::Literal { value } => {
//                 write!(f, "\x1b[34m(\x1b[0m{}\x1b[34m)\x1b[0m", value)
//             }
//             Self::Unary { operator, right } => {
//                 write!(f, "\x1b[36m(\x1b[0m{} {}\x1b[36m)\x1b[0m", operator, right)
//             }
//             Self::Variable { name } => {
//                 write!(f, "{}", name)
//             }
//         }
//     }
// }
