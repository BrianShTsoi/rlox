use crate::ast::Expr;
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

    pub fn parse(&mut self) -> Result<Expr, ParserError> {
        self.expression()
    }

    fn synchronize(&mut self) {
        // TODO: modify later once this is actually used
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
        } else if self.match_next(TokenType::LeftParen) {
            let expr = self.expression()?;
            if self.match_next(TokenType::RightParen) {
                Ok(Expr::Grouping {
                    expression: expr.into(),
                })
            } else {
                self.error(ParserError::ExpectRightParen(self.previous().to_owned()))
            }
        } else {
            self.error(ParserError::ExpectExpression(self.previous().to_owned()))
        }
    }

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

    // Panics if `self.current` = 0
    fn previous(&self) -> &Token {
        self.tokens
            .get(self.current - 1)
            .expect("Previous should exist")
    }

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

    fn error(&mut self, err: ParserError) -> Result<Expr, ParserError> {
        match err.clone() {
            ParserError::ExpectExpression(t) => {
                self.lox.error_with_token(t, "Expect expression");
            }
            ParserError::ExpectRightParen(t) => {
                self.lox.error_with_token(t, "Expect ')' after expression");
            }
        }
        Err(err)
    }
}

#[derive(Clone)]
pub enum ParserError {
    ExpectExpression(Token),
    ExpectRightParen(Token),
}
