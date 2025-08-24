use anyhow::{Context, Result};
use std::process::Command as ProcessCommand;
use std::{
    env,
    fs::metadata,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::context::CommandContext;

pub enum Command {
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

pub fn handle_command(ctx: &CommandContext) -> Result<()> {
    match ctx.command {
        Command::Echo => ctx.writeln(ctx.args.join(" ")),
        Command::Type => type_command(ctx.args.first().unwrap_or(&String::new()), ctx),
        Command::Pwd => ctx.writeln(env::current_dir()?.display()),
        Command::Cd => {
            if change_directory(ctx.args.as_slice()).is_err() {
                ctx.writeln(format_args!(
                    "cd: {}: No such file or directory",
                    ctx.args.first().unwrap(),
                ));
            }
        }
        Command::Executable { ref name, .. } => run_executable(name, ctx)?,
        Command::Invalid => ctx.writeln(format_args!("{}: command not found", ctx.command_str)),
        Command::Exit => {
            let code = ctx
                .args
                .first()
                .and_then(|x| x.parse::<i32>().ok())
                .unwrap_or(0);
            std::process::exit(code)
        }
    }
    Ok(())
}

fn try_get_executable_path(command: &str) -> Option<PathBuf> {
    env::var("PATH").ok()?.split(':').find_map(|dir| {
        let path = Path::new(dir).join(command);
        is_executable(&path).then_some(path)
    })
}

pub fn is_executable(path: &Path) -> bool {
    metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

fn run_executable(executable: &str, ctx: &CommandContext) -> Result<(), std::io::Error> {
    let output = ProcessCommand::new(executable).args(&ctx.args).output()?;
    ctx.write(String::from_utf8_lossy(&output.stdout));
    ctx.ewrite(String::from_utf8_lossy(&output.stderr));
    Ok(())
}

fn type_command(cmd: &str, ctx: &CommandContext) {
    match Command::from(cmd) {
        Command::Invalid => ctx.writeln(format_args!("{}: not found", cmd)),
        Command::Executable {
            name: _,
            full_path: path,
        } => ctx.writeln(format_args!("{} is {}", cmd, path.display())),
        _ => ctx.writeln(format_args!("{} is a shell builtin", cmd)),
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
