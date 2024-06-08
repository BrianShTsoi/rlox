use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    process,
};

struct Lox {}

impl Lox {
    fn new() -> Self {
        Lox {}
    }

    fn main(&self, args: Vec<String>) -> std::io::Result<()> {
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

    fn run(&self, source: String) {
        print!("{source}");
        // TODO: Scan to tokens
    }

    fn run_file(&self, filename: &String) -> std::io::Result<()> {
        let mut file = File::open(filename)?;
        let mut source = String::new();
        file.read_to_string(&mut source)?;
        self.run(source);

        Ok(())
    }

    fn run_prompt(&self) -> std::io::Result<()> {
        let mut stdout = io::stdout();
        let stdin = io::stdin();

        loop {
            stdout.write(b"> ")?;
            stdout.flush()?;

            let mut line = String::new();

            let n: usize;
            match stdin.read_line(&mut line) {
                Ok(bytes_read) => n = bytes_read,
                Err(error) => {
                    println!("{error}");
                    return Err(error);
                }
            }

            if n == 0 {
                return Ok(());
            }

            self.run(line)
        }
    }
}

fn main() -> std::io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let lox = Lox::new();
    lox.main(args)?;
    Ok(())
}
