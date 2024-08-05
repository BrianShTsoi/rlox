use crate::ast::{Expr, Stmt};
use crate::scanner::token::Token;
use crate::scanner::token_type::TokenType;
use crate::Lox;

pub struct Parser<'a> {
    lox: &'a mut Lox,
    tokens: Vec<Token>,

    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(lox: &'a mut Lox, tokens: Vec<Token>) -> Self {
        Parser {
            lox,
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while self.peek().is_some() {
            let stmt = self.declaration();
            match stmt {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => {
                    self.error(err);
                    self.synchronize();
                }
            }
        }
        stmts
    }

    fn synchronize(&mut self) {
        // Edge case, might have better handling if we separate `peek` and `at_end`
        if matches!(self.current().token_type(), TokenType::Eof) {
            return;
        }

        self.current += 1;
        while self.peek().is_some() {
            if matches!(self.previous().token_type(), TokenType::Semicolon) {
                return;
            }

            if self.peek().is_some_and(|t| {
                matches!(
                    t.token_type(),
                    TokenType::Class
                        | TokenType::Fun
                        | TokenType::Var
                        | TokenType::For
                        | TokenType::If
                        | TokenType::While
                        | TokenType::Print
                        | TokenType::Return
                )
            }) {
                return;
            }

            self.current += 1;
        }
    }

    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        if self.match_next(TokenType::Var) {
            self.var_decl()
        } else {
            self.statement()
        }
    }

    fn var_decl(&mut self) -> Result<Stmt, ParserError> {
        if self.match_next(TokenType::Identifier) {
            let var_name = self.previous().to_owned();
            let initializer = if self.match_next(TokenType::Equal) {
                Some(Box::new(self.expression()?))
            } else {
                None
            };
            if self.match_next(TokenType::Semicolon) {
                Ok(Stmt::VarStmt {
                    var_name,
                    initializer,
                })
            } else {
                Err(ParserError::ExpectSemicolon(self.current().to_owned()))
            }
        } else {
            Err(ParserError::ExpectIdentifier(self.current().to_owned()))
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_next(TokenType::Print) {
            self.print_stmt()
        } else {
            self.expr_stmt()
        }
    }

    fn print_stmt(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        if self.match_next(TokenType::Semicolon) {
            Ok(Stmt::PrintStmt { expr: expr.into() })
        } else {
            Err(ParserError::ExpectSemicolon(self.current().to_owned()))
        }
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        if self.match_next(TokenType::Semicolon) {
            Ok(Stmt::ExprStmt { expr: expr.into() })
        } else {
            Err(ParserError::ExpectSemicolon(self.current().to_owned()))
        }
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;
        while self.match_next(TokenType::EqualEqual) || self.match_next(TokenType::BangEqual) {
            let operator = self.previous().to_owned();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term()?;
        while self.match_next(TokenType::Greater)
            || self.match_next(TokenType::GreaterEqual)
            || self.match_next(TokenType::Less)
            || self.match_next(TokenType::LessEqual)
        {
            let operator = self.previous().to_owned();
            let right = self.term()?;
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;
        while self.match_next(TokenType::Plus) || self.match_next(TokenType::Minus) {
            let operator = self.previous().to_owned();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;
        while self.match_next(TokenType::Star) || self.match_next(TokenType::Slash) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.match_next(TokenType::Bang) || self.match_next(TokenType::Minus) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            let expr = Expr::Unary {
                operator,
                right: right.into(),
            };
            Ok(expr)
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        if self.match_next(TokenType::String("".to_string()))
            || self.match_next(TokenType::Number(0.0))
            || self.match_next(TokenType::True)
            || self.match_next(TokenType::False)
            || self.match_next(TokenType::Nil)
        {
            Ok(Expr::Literal {
                value: self.previous().to_owned(),
            })
        } else if self.match_next(TokenType::Identifier) {
            Ok(Expr::Variable {
                name: self.previous().to_owned(),
            })
        } else if self.match_next(TokenType::LeftParen) {
            let expr = self.expression()?;
            if self.match_next(TokenType::RightParen) {
                Ok(Expr::Grouping {
                    expression: expr.into(),
                })
            } else {
                Err(ParserError::ExpectRightParen(self.current().to_owned()))
            }
        } else {
            Err(ParserError::ExpectExpression(self.current().to_owned()))
        }
    }

    // TODO: consider an `expect_next` method, equivalent to `consume` in the book
    fn match_next(&mut self, expected_type: TokenType) -> bool {
        let matching = |t: &Token| match (t.token_type(), expected_type) {
            (TokenType::String(_), TokenType::String(_))
            | (TokenType::Number(_), TokenType::Number(_)) => true,
            (t1, t2) => t1 == t2,
        };

        if self.peek().is_some_and(matching) {
            self.current += 1;
            true
        } else {
            false
        }
    }

    // Panics if `self.current` = 0 as it is usize
    fn previous(&self) -> &Token {
        self.tokens
            .get(self.current - 1)
            .expect("previous should exist")
    }

    /// Can return a reference to `TokenType::Eof`
    /// Panics if `self.current` is pointing past the end of `self.tokens`
    fn current(&self) -> &Token {
        self.tokens.get(self.current).expect("current should exist")
    }

    // TODO: refactor `self.peek().is_some()` to `self.is_eof()`
    /// Returns None if `self.current` is pointing at `TokenType::Eof`
    /// Panics if `self.current` is pointing past the end of `self.tokens`
    fn peek(&self) -> Option<&Token> {
        // This is awkward as TokenType::Eof is actually unnecessary
        // but is included to follow the book
        let token = self
            .tokens
            .get(self.current)
            .expect("self.current should never get past last token (TokenType::Eof)");
        if matches!(token.token_type(), TokenType::Eof) {
            None
        } else {
            Some(token)
        }
    }

    fn error(&mut self, err: ParserError) {
        match err.clone() {
            ParserError::ExpectExpression(t) => {
                self.lox.syntax_error(t, "Expect expression");
            }
            ParserError::ExpectRightParen(t) => {
                self.lox.syntax_error(t, "Expect ')' after expression");
            }
            ParserError::ExpectSemicolon(t) => {
                self.lox
                    .syntax_error(t, "Expect ';' at the end of statement");
            }
            ParserError::ExpectIdentifier(t) => {
                self.lox.syntax_error(t, "Expect identifier after `var`");
            }
        }
    }
}

#[derive(Clone)]
pub enum ParserError {
    ExpectExpression(Token),
    ExpectRightParen(Token),
    ExpectSemicolon(Token),
    ExpectIdentifier(Token),
}
