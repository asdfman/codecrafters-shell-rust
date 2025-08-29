use crate::{args::parse_args, command::Command};
use anyhow::{bail, Result};
use os_pipe::{PipeReader, PipeWriter};
use std::{
    cell::RefCell,
    fmt::Display,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

#[derive(Debug)]
pub struct CommandContext {
    pub command: Command,
    pub command_str: String,
    pub args: Vec<String>,
    pub r_stderr: Option<(String, bool)>,
    pub r_stdout: bool,
    pub writer: RefCell<Writer>,
    pub piped_stdin: Option<PipeReader>,
}

#[derive(Debug)]
pub enum Writer {
    Pipe(PipeWriter),
    File(fs::File),
    Stdout(std::io::Stdout),
}
impl Writer {
    fn ref_mut(&mut self) -> &mut dyn Write {
        match self {
            Writer::Pipe(p) => p,
            Writer::File(f) => f,
            Writer::Stdout(s) => s,
        }
    }
}

pub fn parse_commands(input: &str) -> Result<Vec<CommandContext>> {
    if input.is_empty() {
        return Ok(vec![]);
    }
    let mut contexts = Vec::new();
    for command in input.split('|') {
        contexts.push(CommandContext::try_from(command.trim())?);
    }
    Ok(contexts)
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
            piped_stdin: None,
            writer: if r_stdout {
                RefCell::new(Writer::File(
                    OpenOptions::new()
                        .write(true)
                        .append(append)
                        .truncate(!append)
                        .open(file.unwrap())?,
                ))
            } else {
                RefCell::new(Writer::Stdout(std::io::stdout()))
            },
        })
    }
}

impl CommandContext {
    pub fn writeln(&self, msg: impl Display) -> Result<()> {
        let mut writer = self.writer.borrow_mut();
        writeln!(writer.ref_mut(), "{}", msg)?;
        writer.ref_mut().flush()?;
        Ok(())
    }

    pub fn write(&self, msg: impl Display) -> Result<()> {
        let mut writer = self.writer.borrow_mut();
        write!(writer.ref_mut(), "{}", msg)?;
        writer.ref_mut().flush()?;
        Ok(())
    }

    pub fn ewriteln(&self, err: impl Display) -> Result<()> {
        self.ewrite(format!("{}\n", err))
    }

    pub fn ewrite(&self, err: impl Display) -> Result<()> {
        if let Some((file, append)) = &self.r_stderr {
            if self.r_stdout {
                let mut writer = self.writer.borrow_mut();
                write!(writer.ref_mut(), "{}", err)?;
                writer.ref_mut().flush()?;
            } else if let Some(mut writer) = create_stderr_file_writer(file, *append) {
                write!(writer, "{}", err)?;
                writer.flush()?
            }
        } else {
            eprint!("{}", err);
        }
        Ok(())
    }
}

pub fn create_stderr_file_writer(file: &str, append: bool) -> Option<fs::File> {
    let parent = std::path::Path::new(&file).parent()?;
    fs::create_dir_all(parent).ok()?;
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(file)
        .ok()
}
