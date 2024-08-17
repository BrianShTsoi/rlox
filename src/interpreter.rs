use std::collections::HashMap;
use std::fmt;

use crate::ast::{Expr, Stmt};
use crate::scanner::token::Token;
use crate::scanner::token_type::TokenType;

pub struct Interpreter {
    // lox: &'a mut Lox,
    env_list: EnvironmentList,
    errors: Vec<RuntimeError>,
}

struct EnvironmentList {
    env_list: Vec<Environment>,
}

struct Environment {
    map: HashMap<String, LoxValue>,
}

#[derive(Clone, Debug, PartialEq)]
enum LoxValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Clone, Debug)]
pub enum RuntimeError {
    InvalidBinaryOperand(Token),
    InvalidUnaryOperand(Token),
    UnexpectedLiteralTokenType(Token),
    UndefinedVariable(Token),
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            // lox,
            env_list: EnvironmentList::new(),
            errors: Vec::new(),
        }
    }

    pub fn interpret(&mut self, program: Vec<Stmt>) -> Result<(), Vec<RuntimeError>> {
        for stmt in program {
            if let Err(err) = self.execute(&stmt) {
                // self.lox.runtime_error(err);
                self.errors.push(err);
            }
        }

        // TODO: Weird handling of errors
        //       clone is unnecessary if Lox just access the field directly
        self.errors
            .is_empty()
            .then(|| ())
            .ok_or(self.errors.clone())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expr { expr } => {
                let val = self.evaluate(expr)?;
                println!("expr = {val}");
            }
            Stmt::Print { expr } => {
                let val = self.evaluate(expr)?;
                println!("{val}");
            }
            Stmt::VarDecl {
                var_name,
                initializer,
            } => {
                let init_val = initializer
                    .as_ref()
                    .map(|i| self.evaluate(i))
                    .unwrap_or(Ok(LoxValue::Nil))?;
                self.env_list.declare_var(&var_name.lexeme(), init_val);
            }
            Stmt::Block { stmt_list } => {
                self.env_list.push_new_env();

                for stmt in stmt_list {
                    if let Err(err) = self.execute(&stmt) {
                        self.errors.push(err);
                    }
                }

                self.env_list.pop_env();
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                if self.evaluate(condition)?.truthiness() {
                    self.execute(&then_stmt)?
                } else if let Some(else_stmt) = else_stmt {
                    self.execute(&else_stmt)?
                }
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
            Expr::Variable { name } => self.evaluate_var(name),
            Expr::Assignment { var_name, value } => {
                let value = self.evaluate(value)?;
                self.evaluate_assignment(var_name, value)
            }
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

    fn evaluate_literal(&self, token: &Token) -> Result<LoxValue, RuntimeError> {
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

    fn evaluate_var(&self, var: &Token) -> Result<LoxValue, RuntimeError> {
        self.env_list
            .get_var(&var.lexeme())
            .map_err(|_| RuntimeError::UndefinedVariable(var.to_owned()))
    }

    fn evaluate_assignment(
        &mut self,
        var: &Token,
        value: LoxValue,
    ) -> Result<LoxValue, RuntimeError> {
        self.env_list
            .set_var(&var.lexeme(), value)
            .map_err(|_| RuntimeError::UndefinedVariable(var.to_owned()))
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

impl EnvironmentList {
    fn new() -> Self {
        Self {
            env_list: vec![Environment::new()],
        }
    }

    fn declare_var(&mut self, name: &str, val: LoxValue) {
        self.last_env_mut().declare_var(name, val);
    }

    fn get_var(&self, name: &str) -> Result<LoxValue, ()> {
        // TODO: do the for loop with the FP way
        for env in self.env_list.iter().rev() {
            let value = env.get_var(name);
            if value.is_ok() {
                return value;
            }
        }
        return Err(());
    }

    fn set_var(&mut self, name: &str, val: LoxValue) -> Result<LoxValue, ()> {
        // TODO: do the for loop with the FP way
        for env in self.env_list.iter_mut().rev() {
            let value = env.set_var(name, val.clone());
            if value.is_ok() {
                return value;
            }
        }
        return Err(());
        // self.last_env_mut().set_var(name, val)
    }

    fn push_new_env(&mut self) {
        self.env_list.push(Environment::new());
    }

    fn pop_env(&mut self) {
        self.env_list
            .pop()
            .expect("env_list should not be empty when popped");
    }

    fn last_env_mut(&mut self) -> &mut Environment {
        self.env_list
            .last_mut()
            .expect("env_list should never be empty")
    }
}

impl Environment {
    fn new() -> Self {
        Environment {
            map: HashMap::new(),
        }
    }

    fn declare_var(&mut self, name: &str, val: LoxValue) {
        self.map.insert(name.to_string(), val);
    }

    fn get_var(&self, name: &str) -> Result<LoxValue, ()> {
        self.map.get(name).cloned().ok_or(())
    }

    fn set_var(&mut self, name: &str, val: LoxValue) -> Result<LoxValue, ()> {
        if self.map.contains_key(name) {
            self.map.insert(name.to_string(), val.clone());
            Ok(val)
        } else {
            Err(())
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
            Self::UndefinedVariable(t) => ("Undefined variable", t.line()),
        };
        format!("{}\n[line {}]", warning, line)
    }
}
