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

        let mut input_parts = input.split_whitespace();
        match input_parts.next().unwrap_or("").to_lowercase().as_str() {
            "exit" => {
                if let Some(val) = input_parts.next() {
                    let num = val.parse::<u8>().unwrap_or(0);
                    return ExitCode::from(num);
                }
            }
            _ => println!("{}: command not found", input.trim()),
        }
    }
}
