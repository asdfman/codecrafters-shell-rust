use crate::{args::parse_args, command::Command};
use anyhow::{bail, Result};
use std::{
    cell::RefCell,
    fmt::Display,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub struct CommandContext {
    pub command: Command,
    pub command_str: String,
    pub args: Vec<String>,
    pub r_stderr: Option<(String, bool)>,
    pub r_stdout: bool,
    pub writer: RefCell<Box<dyn std::io::Write>>,
}

impl TryFrom<&str> for CommandContext {
    type Error = anyhow::Error;
    fn try_from(input: &str) -> Result<Self> {
        let mut args = parse_args(input);
        let command_str = args.remove(0);
        let command = Command::from(command_str.as_str());
        let (r_stdout, r_stderr, append) = args
            .iter()
            .find_map(|arg| match arg.as_str() {
                ">" | "1>" => Some((true, false, false)),
                ">>" | "1>>" => Some((true, false, true)),
                "2>" => Some((false, true, false)),
                "2>>" => Some((false, true, true)),
                "&>" => Some((true, true, false)),
                "&>>" => Some((true, true, true)),
                _ => None,
            })
            .unwrap_or((false, false, false));

        let file = if r_stdout || r_stderr {
            let Some(file) = args.pop() else {
                bail!("No file specified for redirection");
            };
            args.truncate(args.len() - 1);
            let path = Path::new(&file);
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                    fs::File::create(&file)?;
                }
            }
            Some(file)
        } else {
            None
        };

        Ok(Self {
            command,
            command_str,
            args,
            r_stderr: if r_stderr {
                file.clone().map(|f| (f, append))
            } else {
                None
            },
            r_stdout,
            writer: if r_stdout {
                RefCell::new(Box::new(
                    OpenOptions::new()
                        .write(true)
                        .append(append)
                        .truncate(!append)
                        .open(file.unwrap())?,
                ))
            } else {
                RefCell::new(Box::new(std::io::stdout()))
            },
        })
    }
}

impl CommandContext {
    pub fn writeln(&self, msg: impl Display) {
        let _ = writeln!(self.writer.borrow_mut(), "{msg}");
    }

    pub fn write(&self, msg: impl Display) {
        let _ = write!(self.writer.borrow_mut(), "{msg}");
    }

    pub fn ewriteln(&self, err: impl Display) {
        self.ewrite(format!("{}\n", err));
    }

    pub fn ewrite(&self, err: impl Display) {
        if let Some((file, append)) = &self.r_stderr {
            if self.r_stdout {
                let _ = write!(self.writer.borrow_mut(), "{}", err);
            } else if let Some(parent) = std::path::Path::new(&file).parent() {
                if fs::create_dir_all(parent).is_ok() {
                    if let Ok(mut writer) = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(*append)
                        .truncate(!append)
                        .open(file)
                    {
                        let _ = write!(writer, "{}", err);
                    }
                }
            }
        } else {
            eprint!("{}", err);
        }
    }
}
