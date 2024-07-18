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
        while self
            .peek()
            .is_some_and(|t| t.token_type() != TokenType::Eof)
        {
            if self
                .previous()
                .is_some_and(|t| t.token_type() == TokenType::Semicolon)
            {
                return;
            }

            if self.peek().is_some_and(|t| match t.token_type() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => true,
                _ => false,
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
        while self.match_next_token_type(TokenType::EqualEqual)
            || self.match_next_token_type(TokenType::BangEqual)
        {
            let operator = self
                .previous()
                .expect("Just matched next token, expect previous to exist")
                .to_owned();
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
        while self.match_next_token_type(TokenType::Greater)
            || self.match_next_token_type(TokenType::GreaterEqual)
            || self.match_next_token_type(TokenType::Less)
            || self.match_next_token_type(TokenType::LessEqual)
        {
            let operator = self
                .previous()
                .expect("Just matched next token, expect previous to exist")
                .to_owned();
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
        if self.match_next_token_type(TokenType::Plus)
            || self.match_next_token_type(TokenType::Minus)
        {
            let operator = self
                .previous()
                .expect("Just matched next token, expect previous to exist")
                .to_owned();
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
        while self.match_next_token_type(TokenType::Star)
            || self.match_next_token_type(TokenType::Slash)
        {
            let operator = self
                .previous()
                .expect("Just matched next token, expect previous to exist")
                .to_owned();
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
        if self.match_next_token_type(TokenType::Bang)
            || self.match_next_token_type(TokenType::Minus)
        {
            let operator = self
                .previous()
                .expect("Just matched next token, expect previous to exist")
                .to_owned();
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
        if self.match_next_token_type(TokenType::String("".to_string()))
            || self.match_next_token_type(TokenType::Number(0.0))
            || self.match_next_token_type(TokenType::True)
            || self.match_next_token_type(TokenType::False)
            || self.match_next_token_type(TokenType::Nil)
        {
            Ok(Expr::Literal {
                value: self
                    .previous()
                    .expect("Just matched next token, expect previous to exist")
                    .to_owned(),
            })
        } else if self.match_next_token_type(TokenType::LeftParen) {
            let expr = self.expression()?;
            if self.match_next_token_type(TokenType::RightParen) {
                Ok(Expr::Grouping {
                    expression: expr.into(),
                })
            } else {
                let token = self.peek().expect("Should not be the end token").to_owned();
                self.error(ParserError::ExpectParen(token))
            }
        } else {
            let token = self.peek().expect("Should not be the end token").to_owned();
            self.error(ParserError::ExpectExpression(token))
        }
    }

    fn match_next_token_type(&mut self, expected_type: TokenType) -> bool {
        let matching_token_type = |t: &Token| match (t.token_type(), expected_type) {
            (TokenType::String(_), TokenType::String(_))
            | (TokenType::Number(_), TokenType::Number(_)) => true,
            (t1, t2) => t1 == t2,
        };
        if self.peek().is_some_and(matching_token_type) {
            self.current += 1;
            true
        } else {
            false
        }
    }

    fn previous(&self) -> Option<&Token> {
        // TODO: Edge case when current = 0
        // Should be fine unless length of token vector is max usize
        self.tokens.get(self.current - 1)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn error(&mut self, err: ParserError) -> Result<Expr, ParserError> {
        match err.clone() {
            ParserError::ExpectExpression(t) => {
                self.lox.error_with_token(t, "Expect expression");
            }
            ParserError::ExpectParen(t) => {
                self.lox.error_with_token(t, "Expect ')' after expression");
            }
        }
        Err(err)
    }
}

#[derive(Clone)]
pub enum ParserError {
    ExpectExpression(Token),
    ExpectParen(Token),
}
