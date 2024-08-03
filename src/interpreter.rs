use std::fmt;

use crate::ast::{Expr, Stmt};
use crate::scanner::token::Token;
use crate::scanner::token_type::TokenType;
use crate::Lox;

pub struct Interpreter<'a> {
    lox: &'a mut Lox,
}

#[derive(Debug, PartialEq)]
enum LoxValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Debug)]
pub enum RuntimeError {
    InvalidBinaryOperand(Token),
    InvalidUnaryOperand(Token),
    UnexpectedLiteralTokenType(Token),
}

impl<'a> Interpreter<'a> {
    pub fn new(lox: &'a mut Lox) -> Self {
        Self { lox }
    }

    pub fn interpret(&mut self, program: Vec<Stmt>) {
        for stmt in program {
            if let Err(err) = self.execute(&stmt) {
                self.lox.runtime_error(err);
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::ExprStmt { expr } => {
                let val = self.evaluate(expr)?;
                println!("{val}");
            }
            Stmt::PrintStmt { expr } => {
                let val = self.evaluate(expr)?;
                println!("PRINT {val}");
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<LoxValue, RuntimeError> {
        let val = match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left, operator, right),
            Expr::Grouping { expression } => self.evaluate(&expression),
            Expr::Literal { value } => self.evaluate_literal(value),
            Expr::Unary { operator, right } => self.evaluate_unary(operator, right),
        };
        val
    }

    fn evaluate_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<LoxValue, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        let result = match operator.token_type() {
            TokenType::Plus => Self::plus(left, right),
            TokenType::Minus => Self::minus(left, right),
            TokenType::Star => Self::multiply(left, right),
            TokenType::Slash => Self::divide(left, right),
            TokenType::BangEqual => Ok(Self::not_equal(left, right)),
            TokenType::EqualEqual => Ok(Self::equal(left, right)),
            TokenType::Greater => Self::greater(left, right),
            TokenType::GreaterEqual => Self::greater_equal(left, right),
            TokenType::Less => Self::less(left, right),
            TokenType::LessEqual => Self::less_equal(left, right),
            _ => return Err(RuntimeError::InvalidBinaryOperand(operator.clone())),
        };
        result.map_err(|_| RuntimeError::InvalidBinaryOperand(operator.clone()))
    }

    fn evaluate_literal(&mut self, token: &Token) -> Result<LoxValue, RuntimeError> {
        match token.token_type() {
            TokenType::Nil => Ok(LoxValue::Nil),
            TokenType::True => Ok(LoxValue::Bool(true)),
            TokenType::False => Ok(LoxValue::Bool(false)),
            TokenType::Number(s) => Ok(LoxValue::Number(s)),
            TokenType::String(s) => Ok(LoxValue::String(s)),
            _ => Err(RuntimeError::UnexpectedLiteralTokenType(token.clone())),
        }
    }

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr) -> Result<LoxValue, RuntimeError> {
        let right = self.evaluate(right)?;
        match operator.token_type() {
            TokenType::Bang => Ok(LoxValue::Bool(!right.truthiness())),
            TokenType::Minus => {
                if let LoxValue::Number(n) = right {
                    Ok(LoxValue::Number(-n))
                } else {
                    Err(RuntimeError::InvalidUnaryOperand(operator.clone()))
                }
            }
            _ => Err(RuntimeError::InvalidBinaryOperand(operator.clone())),
        }
    }

    fn plus(left: LoxValue, right: LoxValue) -> Result<LoxValue, ()> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l - r)),
            (LoxValue::String(l), LoxValue::String(r)) => Ok(LoxValue::String(l + &r)),
            _ => Err(()),
        }
    }
    fn minus(left: LoxValue, right: LoxValue) -> Result<LoxValue, ()> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l - r)),
            _ => Err(()),
        }
    }
    fn multiply(left: LoxValue, right: LoxValue) -> Result<LoxValue, ()> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l * r)),
            _ => Err(()),
        }
    }
    fn divide(left: LoxValue, right: LoxValue) -> Result<LoxValue, ()> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l / r)),
            _ => Err(()),
        }
    }
    fn not_equal(left: LoxValue, right: LoxValue) -> LoxValue {
        LoxValue::Bool(left != right)
    }
    fn equal(left: LoxValue, right: LoxValue) -> LoxValue {
        LoxValue::Bool(left == right)
    }
    fn greater(left: LoxValue, right: LoxValue) -> Result<LoxValue, ()> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Bool(l > r)),
            _ => Err(()),
        }
    }
    fn greater_equal(left: LoxValue, right: LoxValue) -> Result<LoxValue, ()> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Bool(l >= r)),
            _ => Err(()),
        }
    }
    fn less(left: LoxValue, right: LoxValue) -> Result<LoxValue, ()> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Bool(l < r)),
            _ => Err(()),
        }
    }
    fn less_equal(left: LoxValue, right: LoxValue) -> Result<LoxValue, ()> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Bool(l <= r)),
            _ => Err(()),
        }
    }
}

impl LoxValue {
    fn truthiness(&self) -> bool {
        !matches!(self, Self::Nil) && !matches!(self, Self::Bool(false))
    }
}

impl fmt::Display for LoxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => f.write_str("nil"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
        }
    }
}

impl RuntimeError {
    pub fn to_err_msg(&self) -> String {
        let (warning, line) = match self {
            Self::InvalidBinaryOperand(t) => ("Invalid binary operand", t.line()),
            Self::InvalidUnaryOperand(t) => ("Invalid unary operand", t.line()),
            Self::UnexpectedLiteralTokenType(t) => ("Unexpected literal token type", t.line()),
        };
        format!("{}\n[line {}]", warning, line)
    }
}
