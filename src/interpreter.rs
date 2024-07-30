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
    InvalidArithmeticOperand(Token),
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
            if let Err(err) = stmt.execute() {
                self.lox.runtime_error(err);
            }
        }
    }
}

impl Stmt {
    fn execute(&self) -> Result<(), RuntimeError> {
        match self {
            Stmt::ExprStmt { expr } => {
                let val = expr.evaluate()?;
                println!("{val}");
            }
            Stmt::PrintStmt { expr } => {
                let val = expr.evaluate()?;
                println!("PRINT {val}");
            }
        }
        Ok(())
    }
}

impl Expr {
    fn evaluate(&self) -> Result<LoxValue, RuntimeError> {
        let val = match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => Self::evaluate_binary(left, operator, right),
            Expr::Grouping { expression } => expression.evaluate(),
            Expr::Literal { value } => Self::evaluate_literal(value),
            Expr::Unary { operator, right } => Self::evaluate_unary(operator, right),
        };
        val
    }

    fn evaluate_binary(
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<LoxValue, RuntimeError> {
        let left = left.evaluate()?;
        let right = right.evaluate()?;
        match operator.token_type() {
            TokenType::Plus => Self::plus(left, operator, right),
            TokenType::Minus => Self::minus(left, operator, right),
            TokenType::Star => Self::multiply(left, operator, right),
            TokenType::Slash => Self::divide(left, operator, right),
            TokenType::BangEqual => Ok(Self::not_equal(left, right)),
            TokenType::EqualEqual => Ok(Self::equal(left, right)),
            TokenType::Greater => Self::greater(left, operator, right),
            TokenType::GreaterEqual => Self::greater_equal(left, operator, right),
            TokenType::Less => Self::less(left, operator, right),
            TokenType::LessEqual => Self::less_equal(left, operator, right),
            _ => Err(RuntimeError::InvalidBinaryOperand(operator.clone())),
        }
    }

    fn evaluate_literal(token: &Token) -> Result<LoxValue, RuntimeError> {
        match token.token_type() {
            TokenType::Nil => Ok(LoxValue::Nil),
            TokenType::True => Ok(LoxValue::Bool(true)),
            TokenType::False => Ok(LoxValue::Bool(false)),
            TokenType::Number(s) => Ok(LoxValue::Number(s)),
            TokenType::String(s) => Ok(LoxValue::String(s)),
            _ => Err(RuntimeError::UnexpectedLiteralTokenType(token.clone())),
        }
    }

    fn evaluate_unary(operator: &Token, right: &Expr) -> Result<LoxValue, RuntimeError> {
        let right = right.evaluate()?;
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

    fn plus(left: LoxValue, operator: &Token, right: LoxValue) -> Result<LoxValue, RuntimeError> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l - r)),
            (LoxValue::String(l), LoxValue::String(r)) => Ok(LoxValue::String(l + &r)),
            _ => Err(RuntimeError::InvalidArithmeticOperand(operator.clone())),
        }
    }
    fn minus(left: LoxValue, operator: &Token, right: LoxValue) -> Result<LoxValue, RuntimeError> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l - r)),
            _ => Err(RuntimeError::InvalidArithmeticOperand(operator.clone())),
        }
    }
    fn multiply(
        left: LoxValue,
        operator: &Token,
        right: LoxValue,
    ) -> Result<LoxValue, RuntimeError> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l * r)),
            _ => Err(RuntimeError::InvalidArithmeticOperand(operator.clone())),
        }
    }
    fn divide(left: LoxValue, operator: &Token, right: LoxValue) -> Result<LoxValue, RuntimeError> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l / r)),
            _ => Err(RuntimeError::InvalidArithmeticOperand(operator.clone())),
        }
    }
    fn not_equal(left: LoxValue, right: LoxValue) -> LoxValue {
        LoxValue::Bool(left != right)
    }
    fn equal(left: LoxValue, right: LoxValue) -> LoxValue {
        LoxValue::Bool(left == right)
    }
    fn greater(
        left: LoxValue,
        operator: &Token,
        right: LoxValue,
    ) -> Result<LoxValue, RuntimeError> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Bool(l > r)),
            _ => Err(RuntimeError::InvalidBinaryOperand(operator.clone())),
        }
    }
    fn greater_equal(
        left: LoxValue,
        operator: &Token,
        right: LoxValue,
    ) -> Result<LoxValue, RuntimeError> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Bool(l >= r)),
            _ => Err(RuntimeError::InvalidBinaryOperand(operator.clone())),
        }
    }
    fn less(left: LoxValue, operator: &Token, right: LoxValue) -> Result<LoxValue, RuntimeError> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Bool(l < r)),
            _ => Err(RuntimeError::InvalidBinaryOperand(operator.clone())),
        }
    }
    fn less_equal(
        left: LoxValue,
        operator: &Token,
        right: LoxValue,
    ) -> Result<LoxValue, RuntimeError> {
        match (left, right) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Bool(l <= r)),
            _ => Err(RuntimeError::InvalidBinaryOperand(operator.clone())),
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
            Self::InvalidArithmeticOperand(t) => ("Invalid arithmetic operand", t.line()),
            Self::InvalidBinaryOperand(t) => ("Invalid binary operand", t.line()),
            Self::InvalidUnaryOperand(t) => ("Invalid unary operand", t.line()),
            Self::UnexpectedLiteralTokenType(t) => ("Unexpected literal token type", t.line()),
        };
        format!("{}\n[line {}]", warning, line)
    }
}
