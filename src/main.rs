use anyhow::Result;
use codecrafters_shell::command::handle_command;
use codecrafters_shell::context::CommandContext;
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    loop {
        let context = match process_input() {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!("Error processing input: {}", e);
                continue;
            }
        };
        match handle_command(&context) {
            Ok(Some(code)) => return code,
            Ok(None) => continue,
            Err(e) => {
                context.ewriteln(e);
                continue;
            }
        }
    }
}

fn process_input() -> Result<CommandContext> {
    print!("$ ");
    let input = take_input()?;
    CommandContext::try_from(input.as_str())
}

fn take_input() -> Result<String> {
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}
