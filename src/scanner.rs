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

    fn is_at_end(&self) -> bool {
        self.current >= self.source.chars().count()
    }

    // TODO: rewrite advance & peek with options to eliminate use of is_at_end & '\0'
    /// Should only be called after checking is_at_end() is false
    /// Panics if current >= chars().len()
    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn match_next_char(&mut self, expected: char) -> bool {
        if self.peek() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.chars().count() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(token_type, lexeme, self.line))
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            self.lox.error(self.line, "Unterminated string.");
            return;
        }
        self.advance();
        self.add_token(TokenType::String(
            self.source[self.start + 1..self.current - 1].to_string(),
        ));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let literal = self.source[self.start..self.current]
            .parse()
            .expect("Lexeme was checked, should be valid float");
        self.add_token(TokenType::Number(literal));
    }

    fn scan_token(&mut self) {
        let c = self.advance();

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
                while !self.is_at_end() && self.peek() != '\n' {
                    self.advance();
                }
            }
            '/' => self.add_token(TokenType::Slash),

            ' ' | '\r' | 't' => (),
            '\n' => self.line += 1,
            '"' => self.string(),

            _ if c.is_ascii_digit() => self.number(),
            _ => self.lox.error(self.line, "Unexpected character."),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            crate::token_type::TokenType::Eof,
            String::new(),
            self.line,
        ));
        self.tokens.clone()
    }
}
