use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    process,
};

use interpreter::RuntimeError;
use scanner::{token::Token, token_type::TokenType};

use crate::{interpreter::Interpreter, parser::Parser};

pub mod ast;
pub mod interpreter;
pub mod parser;
pub mod scanner;

pub struct Lox {
    // TODO: can implement an error emitter?
    had_error: bool,
    had_runtime_error: bool,
}

impl Lox {
    fn new() -> Self {
        Lox {
            had_error: false,
            had_runtime_error: false,
        }
    }

    fn main(&mut self, args: Vec<String>) -> std::io::Result<()> {
        if args.len() > 2 {
            print!("Usage: rlox [script]");
            process::exit(64);
        }

        if let Some(filename) = args.get(1) {
            self.run_file(filename)?;
        } else {
            self.run_prompt()?;
        }

        Ok(())
    }

    fn run_file(&mut self, filename: &String) -> std::io::Result<()> {
        let mut file = File::open(filename)?;
        let mut source = String::new();
        file.read_to_string(&mut source)?;
        self.run(source);

        // TODO: exit with code 64 if had_error
        // TODO: exit with code 70 if had_runtime_error
        Ok(())
    }

    fn run_prompt(&mut self) -> std::io::Result<()> {
        let mut stdout = io::stdout();
        let stdin = io::stdin();

        loop {
            stdout.write_all(b"> ")?;
            stdout.flush()?;

            let mut line = String::new();

            let n = match stdin.read_line(&mut line) {
                Ok(bytes_read) => bytes_read,
                Err(error) => {
                    println!("{error}");
                    return Err(error);
                }
            };

            if n == 0 {
                return Ok(());
            }

            self.run(line);
            self.had_error = false;
        }
    }

    fn run(&mut self, source: String) {
        // println!("{source}");
        let mut scanner = scanner::Scanner::new(self, source);
        let tokens = scanner.scan_tokens();

        println!("SCANNED");
        for t in &tokens {
            println!("{:?}", t);
        }

        let mut parser = Parser::new(self, tokens);
        let expr = parser.parse().ok();
        if let Some(e) = expr {
            println!("PARSED: {}", e);
            let mut interpreter = Interpreter::new(self);
            interpreter.interpret(e);
        } else {
            println!("Parsing went wrong!");
        }
    }

    fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    fn error_with_token(&mut self, token: Token, message: &str) {
        let position = match token.token_type() {
            TokenType::Eof => " at end".to_string(),
            _ => format!(" at '{}'", token.lexeme()),
        };
        self.report(token.line(), &position, message);
    }

    fn runtime_error(&mut self, runtime_err: RuntimeError) {
        eprintln!("{}", runtime_err.to_err_msg());
        self.had_runtime_error = true;
    }

    fn report(&mut self, line: usize, position: &str, message: &str) {
        eprintln!("[line {line}] Error{position}: {message}");
        self.had_error = false;
    }
}

fn main() -> std::io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let mut lox = Lox::new();
    lox.main(args)?;
    Ok(())
}
