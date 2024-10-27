use std::process;
use std::env;

use checklist::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("no command given");
        process::exit(1);
    }

    let command = checklist::parse_command(&args[1]);

    let command = match command {
        Ok(com) => com,
        Err(msg) => panic!("{msg}"),
    };

    let config = Config::build(args).unwrap_or_else(|err| {
        eprintln!("Problem with command arguments: {err}");
        process::exit(1);
    });

    if let Err(msg) = command(config) {
        eprintln!("{msg}");
        process::exit(1);
    }
}

