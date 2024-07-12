use std::char;

mod token;
mod token_type;

use crate::Lox;
use token::Token;
use token_type::TokenType;

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

    fn identifier(&mut self) {
        while self.peek().is_some_and(|c| c.is_ascii_alphanumeric()) {
            self.advance();
        }
        let token_type = match &self.source[self.start..self.current] {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "fun" => TokenType::Fun,
            "for" => TokenType::For,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        };
        self.add_token(token_type);
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
            _ if c.is_ascii_alphabetic() => self.identifier(),
            _ => self.lox.error(self.line, "Unexpected character."),
        };
        Ok(())
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while self.scan_token().is_ok() {
            self.start = self.current;
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), self.line));
        self.tokens.clone()
    }
}
