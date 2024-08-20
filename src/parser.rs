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
            match self.declaration() {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => {
                    if err.should_panic() {
                        self.synchronize();
                    }
                    self.error(err);
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
        self.expect_next(TokenType::Identifier)?;
        let var_name = self.previous().to_owned();
        let initializer = if self.match_next(TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        self.expect_next(TokenType::Semicolon)?;
        Ok(Stmt::VarDecl {
            var_name,
            initializer,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_next(TokenType::Print) {
            self.print_stmt()
        } else if self.match_next(TokenType::LeftBrace) {
            self.block_stmt()
        } else if self.match_next(TokenType::If) {
            self.if_stmt()
        } else {
            self.expr_stmt()
        }
    }

    fn print_stmt(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        self.expect_next(TokenType::Semicolon)?;
        Ok(Stmt::Print { expr: expr.into() })
    }

    fn block_stmt(&mut self) -> Result<Stmt, ParserError> {
        let mut stmt_list = Vec::new();
        while self
            .peek()
            .is_some_and(|t| !matches!(t.token_type(), TokenType::RightBrace))
        {
            match self.declaration() {
                Ok(stmt) => stmt_list.push(stmt),
                Err(err) => {
                    if err.should_panic() {
                        self.synchronize();
                    }
                    self.error(err);
                }
            }
        }

        self.expect_next(TokenType::RightBrace)?;
        Ok(Stmt::Block { stmt_list })
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        self.expect_next(TokenType::Semicolon)?;
        Ok(Stmt::Expr { expr: expr.into() })
    }

    fn if_stmt(&mut self) -> Result<Stmt, ParserError> {
        let condition = self.expression()?;
        let then_stmt = Box::new(self.statement()?);
        let else_stmt = if self.match_next(TokenType::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_stmt,
            else_stmt,
        })
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.or()?;
        if self.match_next(TokenType::Equal) {
            match expr {
                Expr::Variable { name } => {
                    let value = self.assignment()?;
                    expr = Expr::Assignment {
                        var_name: name,
                        value: value.into(),
                    };
                }
                _ => {
                    let equals = self.previous().to_owned();
                    return Err(ParserError::InvalidAssignmentTarget(equals));
                }
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.and()?;
        while self.match_next(TokenType::Or) {
            let operator = self.previous().to_owned();
            let right = self.and()?;
            expr = Expr::Logical {
                left: expr.into(),
                operator,
                right: right.into(),
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.equality()?;
        while self.match_next(TokenType::And) {
            let operator = self.previous().to_owned();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: expr.into(),
                operator,
                right: right.into(),
            };
        }
        Ok(expr)
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
            self.expect_next(TokenType::RightParen)?;
            Ok(Expr::Grouping {
                expression: expr.into(),
            })
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

    /// Consumes and return Ok(()) if the next token matches the expected type,
    /// Otherwise, return ParserError corresponding to the expected type
    /// Panics if `expected_type` does not correspond to any `ParserError`
    fn expect_next(&mut self, expected_type: TokenType) -> Result<(), ParserError> {
        let err = match expected_type {
            TokenType::RightParen => ParserError::ExpectRightParen(self.current().to_owned()),
            TokenType::RightBrace => ParserError::ExpectRightBrace(self.current().to_owned()),
            TokenType::Semicolon => ParserError::ExpectSemicolon(self.current().to_owned()),
            TokenType::Identifier => ParserError::ExpectIdentifier(self.current().to_owned()),
            _ => panic!("expected_type of expect_next does not correspond to any parser error"),
        };
        if self.match_next(expected_type) {
            Ok(())
        } else {
            Err(err)
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
        match err {
            ParserError::ExpectExpression(t) => {
                self.lox.syntax_error(t, "Expect expression");
            }
            ParserError::ExpectRightParen(t) => {
                self.lox.syntax_error(t, "Expect ')' after expression");
            }
            ParserError::ExpectRightBrace(t) => {
                self.lox.syntax_error(t, "Expect '}' after block");
            }
            ParserError::ExpectSemicolon(t) => {
                self.lox
                    .syntax_error(t, "Expect ';' at the end of statement");
            }
            ParserError::ExpectIdentifier(t) => {
                self.lox.syntax_error(t, "Expect identifier after `var`");
            }
            ParserError::InvalidAssignmentTarget(t) => {
                self.lox.syntax_error(t, "Invalid assignment target");
            }
        }
    }
}

#[derive(Clone)]
pub enum ParserError {
    ExpectExpression(Token),
    ExpectRightParen(Token),
    ExpectRightBrace(Token),
    ExpectSemicolon(Token),
    ExpectIdentifier(Token),
    InvalidAssignmentTarget(Token),
}

impl ParserError {
    fn should_panic(&self) -> bool {
        matches!(
            self,
            Self::ExpectExpression(_)
                | Self::ExpectRightParen(_)
                | Self::ExpectSemicolon(_)
                | Self::ExpectIdentifier(_)
        )
    }
}
