use std::{collections::VecDeque, sync::Mutex};

use once_cell::sync::Lazy;

use crate::context::CommandContext;

const MAX_HISTORY_RETAINED: usize = 100;

#[derive(Debug)]
pub struct CommandHistory {
    data: VecDeque<String>,
    browse_idx: isize,
    cur_prompt: Option<String>,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self {
            data: VecDeque::with_capacity(MAX_HISTORY_RETAINED),
            browse_idx: 0,
            cur_prompt: None,
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

    pub fn store_cur_prompt(prompt: &str) {
        let mut history = COMMAND_HISTORY.lock().unwrap();
        if history.cur_prompt.is_none() {
            history.cur_prompt = Some(prompt.to_string());
        }
    }

    pub fn reset_browse() {
        let mut history = COMMAND_HISTORY.lock().unwrap();
        history.cur_prompt = None;
        history.browse_idx = 0;
    }

    pub fn browse_next(is_down: bool) -> Option<String> {
        let mut history = COMMAND_HISTORY.lock().unwrap();
        let len = history.data.len();
        if len == 0 {
            return None;
        }
        let mut idx = history.browse_idx + if is_down { -1 } else { 1 };
        idx = idx.clamp(0, len as isize);
        //println!("idx {idx}, len {len}");
        if idx == 0 {
            return Some("".to_string());
        }
        history.browse_idx = idx;
        history.data.get(len - idx as usize).cloned()
    }
}

