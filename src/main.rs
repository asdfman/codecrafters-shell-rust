use anyhow::{Context, Result};
use codecrafters_shell::args::parse_args;
use std::fmt::Display;
use std::fs::metadata;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};
use std::{env, os::unix::fs::PermissionsExt};

fn main() -> ExitCode {
    loop {
        match process_input() {
            Ok(Some(code)) => return code,

            Ok(None) => continue,
            Err(e) => {
                eprintln!("Error: {e}");
                continue;
            }
        }
    }
}

fn process_input() -> Result<Option<ExitCode>> {
    print!("$ ");
    let input = take_input()?;
    let (command, args) = parse_input(&input)?;
    handle_command(command, args)
}

fn take_input() -> Result<String> {
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn parse_input(input: &str) -> Result<(String, Vec<String>)> {
    let (command, remainder) = match input.find(char::is_whitespace) {
        Some(idx) => input.split_at(idx),
        None => ("", ""),
    };
    Ok((command.to_string(), parse_args(remainder)))
}

fn handle_command(command: String, args: Vec<String>) -> Result<Option<ExitCode>> {
    match Command::from(command.as_str()) {
        Command::Echo => println(args.join(" ")),
        Command::Type => type_command(args.first().unwrap_or(&String::new())),
        Command::Pwd => println(env::current_dir()?.display()),
        Command::Cd => change_directory(&args).map_or_else(
            |_| println!("cd: {}: No such file or directory", &args.first().unwrap(),),
            |_| (),
        ),
        Command::Executable { name, .. } => run_executable(name, args)?.lines().for_each(println),
        Command::Invalid => println!("{command}: command not found"),
        Command::Exit => {
            let code = args.first().and_then(|x| x.parse::<u8>().ok()).unwrap_or(0);
            return Ok(Some(ExitCode::from(code)));
        }
    }
    Ok(None)
}

fn println(val: impl Display) {
    println!("{val}");
}

enum Command {
    Exit,
    Echo,
    Type,
    Executable { name: String, full_path: PathBuf },
    Pwd,
    Cd,
    Invalid,
}

impl From<&str> for Command {
    fn from(command: &str) -> Self {
        match command {
            "exit" => Command::Exit,
            "echo" => Command::Echo,
            "type" => Command::Type,
            "pwd" => Command::Pwd,
            "cd" => Command::Cd,
            _ => try_get_executable_path(command)
                .map(|path| Command::Executable {
                    name: command.to_string(),
                    full_path: path,
                })
                .unwrap_or(Command::Invalid),
        }
    }
}

fn try_get_executable_path(command: &str) -> Option<PathBuf> {
    env::var("PATH").ok()?.split(':').find_map(|dir| {
        let path = Path::new(dir).join(command);
        metadata(&path)
            .ok()
            .filter(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .map(|_| path)
    })
}

fn run_executable(executable: String, args: Vec<String>) -> Result<String> {
    ProcessCommand::new(executable)
        .args(args)
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .context("Failed to run executable")
}

fn type_command(cmd: &str) {
    match Command::from(cmd) {
        Command::Invalid => println!("{cmd}: not found"),
        Command::Executable {
            name: _,
            full_path: path,
        } => println!("{} is {}", cmd, path.display()),
        _ => println!("{cmd} is a shell builtin"),
    }
}

fn change_directory(args: &[String]) -> Result<()> {
    env::set_current_dir(match args.first() {
        Some(arg) if arg != "~" => build_path(arg)?,
        _ => get_home_dir()?,
    })
    .context("Failed to change directory")
}

fn build_path(arg: &str) -> Result<PathBuf> {
    let mut parts = arg.split('/');
    let init = match parts.next() {
        Some("~") => get_home_dir()?,
        Some(".") => get_current_dir()?,
        Some("") => PathBuf::from("/"),
        Some("..") => get_current_dir()?.parent().unwrap().to_path_buf(),
        Some(dir) => get_current_dir()?.join(dir),
        None => return Ok(PathBuf::from("/")),
    };
    let path = parts.fold(init, |mut acc, part| {
        match part {
            ".." => {
                acc.pop();
            }
            _ if part.is_empty() || part == "." => (),
            dir => acc.push(dir),
        };
        acc
    });
    Ok(path)
}

fn get_home_dir() -> Result<PathBuf> {
    env::var("HOME")
        .map(PathBuf::from)
        .context("Failed to get home directory")
}

fn get_current_dir() -> Result<PathBuf> {
    env::current_dir().context("Failed to get current directory")
}
