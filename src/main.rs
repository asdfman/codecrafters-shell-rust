use std::fs::metadata;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::ExitCode;
use std::{env, os::unix::fs::PermissionsExt};

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
            Command::Executable(_) => (),
            Command::Invalid => println!("{}: command not found", input.trim()),
        }
    }
}

enum Command {
    Exit,
    Echo,
    Type,
    Executable(String),
    Invalid,
}

impl From<&str> for Command {
    fn from(command: &str) -> Self {
        match command.to_ascii_lowercase().as_str() {
            "exit" => Command::Exit,
            "echo" => Command::Echo,
            "type" => Command::Type,
            _ => {
                if let Some(path) = try_get_executable_path(command) {
                    Command::Executable(path)
                } else {
                    Command::Invalid
                }
            }
        }
    }
}

fn try_get_executable_path(command: &str) -> Option<String> {
    for path in env::var("PATH").unwrap().split(':') {
        let full_path = format!("{}/{}", path, command);
        if let Ok(metadata) = metadata(&full_path) {
            if metadata.permissions().mode() & 0o111 != 0 {
                return Some(full_path);
            }
        }
    }
    None
}

fn type_command(args: Vec<&str>) {
    let Some(command_text) = args.first() else {
        return;
    };
    let command = Command::from(*command_text);
    match command {
        Command::Invalid => {
            println!("{}: not found", command_text);
        }
        Command::Executable(path) => println!("{} is {}", command_text, path),
        _ => println!("{} is a shell builtin", command_text),
    }
}
