use anyhow::{Context, Result};
use codecrafters_shell::command::{handle_command, Command};
use codecrafters_shell::context::{
    create_stderr_file_writer, parse_commands, CommandContext, Writer,
};
use codecrafters_shell::editor::get_editor;
use codecrafters_shell::history::CommandHistory;
use std::cell::RefCell;
use std::process::{Child, Command as ProcessCommand, Stdio};
use std::thread::{self};

fn main() -> Result<()> {
    let mut editor = get_editor();
    loop {
        CommandHistory::reset_browse();
        let input = editor.readline("$ ")?;
        CommandHistory::add(&input);
        let commands = match parse_commands(&input) {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!("Error processing input: {}", e);
                continue;
            }
        };
        execute_commands(commands)?;
    }
}

fn execute_commands(contexts: Vec<CommandContext>) -> Result<()> {
    let mut prev_reader = None;
    let mut handles = vec![];
    let mut children = vec![];
    let last_idx = contexts.len() - 1;
    for (idx, mut ctx) in contexts.into_iter().enumerate() {
        if let Some(prev) = prev_reader.take() {
            ctx.piped_stdin = Some(prev);
        }
        if idx != last_idx {
            let (reader, writer) = os_pipe::pipe()?;
            ctx.writer = RefCell::new(Writer::Pipe(writer));
            prev_reader = Some(reader);
        }
        match &ctx.command {
            Command::Executable { .. } => match run_executable(&mut ctx) {
                Ok(child) => children.push(child),
                Err(e) => ctx.ewriteln(e)?,
            },
            _ if last_idx == 0 => {
                run_builtin(&mut ctx);
                return Ok(());
            }
            _ => {
                let handle = thread::spawn(move || run_builtin(&mut ctx));
                handles.push(handle);
            }
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
    for mut child in children {
        child.wait().unwrap();
    }

    Ok(())
}

fn run_builtin(ctx: &mut CommandContext) {
    if let Err(e) = handle_command(ctx) {
        let _ = ctx.ewriteln(e);
    }
}

fn run_executable(ctx: &mut CommandContext) -> Result<Child> {
    let Command::Executable { name, .. } = &ctx.command else {
        unreachable!("run_executable called with non-executable command");
    };
    let stdin = ctx
        .piped_stdin
        .take()
        .map(Stdio::from)
        .unwrap_or(Stdio::inherit());
    let writer_ref = ctx.writer.borrow();
    let writer = match &*writer_ref {
        Writer::Pipe(p) => Stdio::from(p.try_clone()?),
        Writer::File(f) => Stdio::from(f.try_clone()?),
        Writer::Stdout(_) => Stdio::inherit(),
    };
    let stderr = ctx
        .r_stderr
        .clone()
        .and_then(|(file, append)| create_stderr_file_writer(&file, append))
        .map(Stdio::from)
        .unwrap_or(Stdio::inherit());
    let child = ProcessCommand::new(name)
        .args(&ctx.args)
        .stdin(stdin)
        .stdout(writer)
        .stderr(stderr)
        .spawn();
    child.context("Failed to execute command")
}
