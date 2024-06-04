use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    process,
};

fn main() -> std::io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() > 2 {
        print!("Usage: rlox [script]");
        process::exit(64);
    }

    if let Some(filename) = args.get(1) {
        run_file(filename)?;
    } else {
        run_prompt()?;
    }

    Ok(())
}

fn run(source: String) {
    print!("{source}");
}

fn run_file(filename: &String) -> std::io::Result<()> {
    let mut file = File::open(filename)?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    run(source);

    Ok(())
}

fn run_prompt() -> std::io::Result<()> {
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

        run(line)
    }
}
