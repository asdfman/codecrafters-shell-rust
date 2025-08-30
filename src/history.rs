use std::{collections::VecDeque, sync::Mutex};

use once_cell::sync::Lazy;

use crate::context::CommandContext;

const MAX_HISTORY_RETAINED: usize = 100;

#[derive(Debug)]
pub struct CommandHistory {
    data: VecDeque<String>,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self {
            data: VecDeque::with_capacity(MAX_HISTORY_RETAINED),
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
        let history = COMMAND_HISTORY.lock().unwrap();
        let skip_count = ctx
            .args
            .first()
            .and_then(|n| n.parse::<usize>().ok())
            .map(|n| history.data.len() - n)
            .unwrap_or(0);
        for (index, command) in history.data.iter().enumerate().skip(skip_count) {
            let _ = ctx.writeln(format_args!("  {}  {}", index + 1, command));
        }
    }
}
