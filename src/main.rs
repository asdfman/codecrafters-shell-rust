#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let mut input_parts = input.split_ascii_whitespace();
        let Some(command) = input_parts.next() else {
            continue;
        };
        let command = Command::from(command);
        let args = input_parts.collect::<Vec<_>>();
        match command {
            Command::Exit => {
                let val = args.first().unwrap_or(&"0").parse::<u8>().unwrap_or(0);
                return ExitCode::from(val);
            }
            Command::Echo => {
                println!("{}", args.join(" "))
            }
            Command::Type => type_command(args),
            Command::Invalid => println!("{}: command not found", input.trim()),
        }
    }
}

enum Command {
    Exit,
    Echo,
    Type,
    Invalid,
}

impl From<&str> for Command {
    fn from(command: &str) -> Self {
        match command.to_ascii_lowercase().as_str() {
            "exit" => Command::Exit,
            "echo" => Command::Echo,
            "type" => Command::Type,
            _ => Command::Invalid,
        }
    }
}

fn type_command(args: Vec<&str>) {
    let Some(command_text) = args.first() else {
        return;
    };
    let command = Command::from(*command_text);
    match command {
        Command::Invalid => println!("{}: not found", command_text),
        _ => println!("{} is a shell builtin", command_text),
    }
}
