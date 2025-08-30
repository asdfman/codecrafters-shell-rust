use anyhow::Result;
use once_cell::sync::Lazy;
use std::io::Write;
use std::{collections::VecDeque, sync::Mutex};

use crate::context::{create_file_writer, CommandContext};

const MAX_HISTORY_RETAINED: usize = 100;
const HISTFILE: &str = "HISTFILE";

#[derive(Debug)]
pub struct CommandHistory {
    data: VecDeque<String>,
    browse_idx: isize,
    last_append_idx: Option<usize>,
}

impl Default for CommandHistory {
    fn default() -> Self {
        let data = init_from_file().unwrap_or(VecDeque::with_capacity(MAX_HISTORY_RETAINED));
        let last_append_idx = if !data.is_empty() {
            Some(data.len() - 1)
        } else {
            None
        };
        Self {
            data,
            browse_idx: 0,
            last_append_idx,
        }
    }
}

fn init_from_file() -> Result<VecDeque<String>> {
    let read_path = std::env::var(HISTFILE)?;
    let content = std::fs::read_to_string(read_path)?;
    Ok(content.lines().map(String::from).collect())
}

enum HistoryArgs {
    None,
    Limit(usize),
    WriteFile(String),
    AppendFile(String),
    ReadFile(String),
}
impl From<&Vec<String>> for HistoryArgs {
    fn from(value: &Vec<String>) -> Self {
        match value.as_slice() {
            [flag, path] if flag == "-r" => Self::ReadFile(path.to_string()),
            [flag, path] if flag == "-w" => Self::WriteFile(path.to_string()),
            [flag, path] if flag == "-a" => Self::AppendFile(path.to_string()),
            [limit] => limit
                .parse::<usize>()
                .map(Self::Limit)
                .unwrap_or(Self::None),
            _ => Self::None,
        }
    }
}

static COMMAND_HISTORY: Lazy<Mutex<CommandHistory>> =
    Lazy::new(|| Mutex::new(CommandHistory::default()));

impl CommandHistory {
    pub fn add(command: &str) {
        let mut history = COMMAND_HISTORY.lock().unwrap();
        if history.data.len() == MAX_HISTORY_RETAINED {
            history.data.pop_front();
        }
        history.data.push_back(command.to_string());
    }

    pub fn handle_command(ctx: &CommandContext) {
        match HistoryArgs::from(&ctx.args) {
            HistoryArgs::None => print_history(ctx, None),
            HistoryArgs::Limit(n) => print_history(ctx, Some(n)),
            HistoryArgs::ReadFile(path) => read_history_file(path),
            HistoryArgs::WriteFile(path) => write_history_file(path, false),
            HistoryArgs::AppendFile(path) => write_history_file(path, true),
        }
    }

    pub fn reset_browse() {
        let mut history = COMMAND_HISTORY.lock().unwrap();
        history.browse_idx = 0;
    }

    pub fn browse_next(is_down: bool) -> Option<String> {
        let mut history = COMMAND_HISTORY.lock().unwrap();
        let len = history.data.len();
        if len == 0 || (history.browse_idx == 0 && is_down) {
            return None;
        }
        let mut idx = history.browse_idx + if is_down { -1 } else { 1 };
        idx = idx.clamp(0, len as isize);
        if idx == 0 {
            return Some("".to_string());
        }
        history.browse_idx = idx;
        history.data.get(len - idx as usize).cloned()
    }
}

fn print_history(ctx: &CommandContext, limit: Option<usize>) {
    let history = COMMAND_HISTORY.lock().unwrap();
    let skip_count = limit.map(|n| history.data.len() - n).unwrap_or(0);
    for (index, command) in history.data.iter().enumerate().skip(skip_count) {
        let _ = ctx.writeln(format_args!("  {}  {}", index + 1, command));
    }
}

fn read_history_file(path: String) {
    if let Ok(content) = std::fs::read_to_string(path) {
        let mut history = COMMAND_HISTORY.lock().unwrap();
        for line in content.lines().map(String::from) {
            history.data.push_back(line);
        }
    }
}

fn write_history_file(path: String, append: bool) {
    let Some(mut file) = create_file_writer(&path, append) else {
        return;
    };
    let mut history = COMMAND_HISTORY.lock().unwrap();
    let mut skip_count = 0;
    if append {
        skip_count = history.last_append_idx.map(|n| n + 1).unwrap_or(0);
        history.last_append_idx = Some(history.data.len() - 1);
    }
    for entry in history.data.iter().skip(skip_count) {
        let _ = writeln!(file, "{}", entry);
    }
}

pub fn write_history_on_exit() {
    if let Ok(path) = std::env::var(HISTFILE) {
        write_history_file(path, true);
    }
}

