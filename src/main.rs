use anyhow::Result;
use std::fmt::Display;
use std::fs::metadata;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};
use std::{env, os::unix::fs::PermissionsExt};

fn main() -> ExitCode {
    loop {
        print!("$ ");
        if let Ok((command, args)) = take_input().and_then(|input| parse_input(&input)) {
            match Command::from(&command) {
                Command::Echo => println(args.join(" ")),
                Command::Type => type_command(args.first().unwrap_or(&String::new())),
                Command::Pwd => println(get_current_dir().display()),
                Command::Cd => change_directory(&args).map_or_else(
                    |_| println!("cd: {}: No such file or directory", &args.first().unwrap(),),
                    |_| (),
                ),
                Command::Executable { name, .. } => {
                    run_executable(name, args).map_or_else(println, |x| {
                        if !x.is_empty() {
                            x.lines().for_each(println)
                        }
                    })
                }
                Command::Invalid => println!("{}: command not found", command),
                Command::Exit => {
                    return ExitCode::from(
                        args.first().and_then(|x| x.parse::<u8>().ok()).unwrap_or(0),
                    )
                }
            }
        }
    }
}

fn take_input() -> Result<String> {
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn parse_input(input: &str) -> Result<(String, Vec<String>)> {
    let mut parts = input.split_whitespace();
    Ok((
        parts.next().unwrap_or_default().to_string(),
        parts.map(String::from).collect(),
    ))
}

fn println(val: impl Display) {
    println!("{}", val);
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

impl From<&String> for Command {
    fn from(command: &String) -> Self {
        match command.as_str() {
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
    env::var("PATH")
        .unwrap_or("".to_string())
        .split(':')
        .find_map(|dir| {
            let path = Path::new(dir).join(command);
            metadata(&path)
                .ok()
                .filter(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                .map(|_| path)
        })
}

fn run_executable(executable: String, args: Vec<String>) -> Result<String, std::io::Error> {
    ProcessCommand::new(executable)
        .args(args)
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
}

fn type_command(cmd: &String) {
    match Command::from(cmd) {
        Command::Invalid => println!("{}: not found", cmd),
        Command::Executable {
            name: _,
            full_path: path,
        } => println!("{} is {}", cmd, path.display()),
        _ => println!("{} is a shell builtin", cmd),
    }
}

fn change_directory(args: &[String]) -> Result<(), std::io::Error> {
    env::set_current_dir(match args.first() {
        Some(arg) if arg != "~" => build_path(arg),
        _ => get_home_dir(),
    })
}

fn build_path(arg: &str) -> PathBuf {
    let mut parts = arg.split('/');
    let init = match parts.next() {
        Some("~") => get_home_dir(),
        Some(".") => get_current_dir(),
        Some("") => PathBuf::from("/"),
        Some("..") => get_current_dir().parent().unwrap().to_path_buf(),
        Some(dir) => get_current_dir().join(dir),
        None => return PathBuf::from("/"),
    };
    parts.fold(init, |mut acc, part| {
        match part {
            ".." => {
                acc.pop();
            }
            _ if part.is_empty() || part == "." => (),
            dir => acc.push(dir),
        };
        acc
    })
}

fn get_home_dir() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

fn get_current_dir() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
}
