use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::context::CommandContext;

#[derive(Debug, Default)]
pub struct CommandHistory {
    data: Vec<String>,
}

static COMMAND_HISTORY: Lazy<Mutex<CommandHistory>> =
    Lazy::new(|| Mutex::new(CommandHistory::default()));

impl CommandHistory {
    pub fn add(command: &str) {
        let mut history = COMMAND_HISTORY.lock().unwrap();
        history.data.push(command.to_string());
    }

    pub fn handle_command(ctx: &CommandContext) {
        let history = COMMAND_HISTORY.lock().unwrap();
        for (index, command) in history.data.iter().enumerate() {
            let _ = ctx.writeln(format_args!("  {}  {}", index + 1, command));
        }
    }
}
