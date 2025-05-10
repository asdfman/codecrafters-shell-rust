use std::fs::metadata;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::{Command as ProcessCommand, ExitCode};
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
        let args: Vec<_> = input_parts.collect();
        match command {
            Command::Exit => {
                let val = args.first().unwrap_or(&"0").parse::<u8>().unwrap_or(0);
                return ExitCode::from(val);
            }
            Command::Echo => {
                println!("{}", args.join(" "))
            }
            Command::Type => type_command(args),
            Command::Executable { file, path: _ } => run_executable(file, args),
            Command::Invalid => println!("{}: command not found", input.trim()),
        }
    }
}

enum Command {
    Exit,
    Echo,
    Type,
    Executable { file: String, path: String },
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
                    Command::Executable {
                        file: command.to_string(),
                        path,
                    }
                } else {
                    Command::Invalid
                }
            }
        }
    }
}

fn run_executable(file: String, args: Vec<&str>) {
    let output = ProcessCommand::new(file)
        .args(args)
        .output()
        .expect("Failed to execute command");
    if let Ok(stdout) = String::from_utf8(output.stdout) {
        print!("{}", stdout);
    }
}

fn try_get_executable_path(command: &str) -> Option<String> {
    for path in env::var("PATH").unwrap().split(':') {
        let full_path = format!("{}/{}", path, command);
        if let Ok(metadata) = metadata(&full_path) {
            if metadata.permissions().mode() & 0o111 != 0 {
                return Some(full_path.to_string());
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
        Command::Invalid => println!("{}: not found", command_text),
        Command::Executable { file: _, path } => println!("{} is {}", command_text, path),
        _ => println!("{} is a shell builtin", command_text),
    }
}
