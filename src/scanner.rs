use std::char;

use crate::token::Token;
use crate::token_type::TokenType;
use crate::Lox;

pub struct Scanner<'a> {
    lox: &'a mut Lox,
    source: String,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(lox: &'a mut Lox, source: String) -> Self {
        Scanner {
            lox,
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.current += 1;
        }
        c
    }

    fn peek(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }

    fn match_next_char(&mut self, expected: char) -> bool {
        if self.peek().is_some_and(|c| c == expected) {
            self.current += 1;
            true
        } else {
            false
        }
    }

    fn peek_next(&self) -> Option<char> {
        self.source[self.current + 1..].chars().next()
    }

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(token_type, lexeme, self.line))
    }

    fn string(&mut self) {
        loop {
            match self.advance() {
                Some('"') => {
                    self.add_token(TokenType::String(
                        self.source[self.start + 1..self.current - 1].to_string(),
                    ));
                    return;
                }
                Some('\n') => self.line += 1,
                Some(_) => (),
                None => {
                    self.lox.error(self.line, "Unterminated string.");
                    return;
                }
            }
        }
    }

    fn number(&mut self) {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }
        if self.peek().is_some_and(|c| c == '.')
            && self.peek_next().is_some_and(|c| c.is_ascii_digit())
        {
            self.advance();
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        let literal = self.source[self.start..self.current]
            .parse()
            .expect("Lexeme was checked, should be valid float");
        self.add_token(TokenType::Number(literal));
    }

    fn scan_token(&mut self) -> Result<(), &str> {
        let Some(c) = self.advance() else {
            return Err("End of source");
        };

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),

            '!' if self.match_next_char('=') => self.add_token(TokenType::BangEqual),
            '!' => self.add_token(TokenType::Bang),
            '=' if self.match_next_char('=') => self.add_token(TokenType::EqualEqual),
            '=' => self.add_token(TokenType::Equal),
            '>' if self.match_next_char('=') => self.add_token(TokenType::GreaterEqual),
            '>' => self.add_token(TokenType::Greater),
            '<' if self.match_next_char('=') => self.add_token(TokenType::LessEqual),
            '<' => self.add_token(TokenType::Less),
            '/' if self.match_next_char('/') => {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.advance();
                }
            }
            '/' => self.add_token(TokenType::Slash),

            ' ' | '\r' | 't' => (),
            '\n' => self.line += 1,
            '"' => self.string(),

            _ if c.is_ascii_digit() => self.number(),
            _ => self.lox.error(self.line, "Unexpected character."),
        };
        Ok(())
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while self.scan_token().is_ok() {
            self.start = self.current;
        }

        self.tokens.push(Token::new(
            crate::token_type::TokenType::Eof,
            String::new(),
            self.line,
        ));
        self.tokens.clone()
    }
}
