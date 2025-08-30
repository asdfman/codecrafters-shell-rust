use anyhow::Result;
use std::{env, fs};

use once_cell::unsync::OnceCell;
use rustyline::{
    completion::{Completer, Pair},
    config::Configurer,
    history::FileHistory,
    Cmd, ConditionalEventHandler, Editor, EventHandler, Helper, Highlighter, Hinter, KeyCode,
    KeyEvent, Modifiers, Validator,
};
use trie_rs::Trie;

use crate::{command::is_executable, history::CommandHistory};

pub type ShellEditor = Editor<ShellCompleter, FileHistory>;
const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

pub fn get_editor() -> ShellEditor {
    let mut editor = Editor::new().unwrap();
    editor.set_completion_type(rustyline::CompletionType::List);
    editor.set_helper(Some(ShellCompleter {
        trie: OnceCell::new(),
    }));
    let handler = Box::new(BrowseHistoryHandler);
    editor.bind_sequence(
        KeyEvent(KeyCode::Up, Modifiers::NONE),
        EventHandler::Conditional(handler.clone()),
    );
    editor.bind_sequence(
        KeyEvent(KeyCode::Down, Modifiers::NONE),
        EventHandler::Conditional(handler.clone()),
    );
    editor
}

#[derive(Helper, Highlighter, Hinter, Validator)]
pub struct ShellCompleter {
    trie: OnceCell<Trie<u8>>,
}
impl Completer for ShellCompleter {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let trie = self.trie.get_or_init(build_completion_trie);
        let start_idx = line[..pos]
            .char_indices()
            .rev()
            .find_map(|(i, c)| c.is_whitespace().then_some(i + c.len_utf8()))
            .unwrap_or(0);
        let results: Vec<String> = trie.predictive_search(&line[start_idx..pos]).collect();
        let candidates: Vec<Pair> = results
            .into_iter()
            .map(|s| Pair {
                display: s.clone(),
                replacement: format!("{s} "),
            })
            .collect();
        Ok((start_idx, candidates))
    }
}

#[derive(Clone)]
struct BrowseHistoryHandler;
impl ConditionalEventHandler for BrowseHistoryHandler {
    fn handle(
        &self,
        evt: &rustyline::Event,
        _: rustyline::RepeatCount,
        _: bool,
        _: &rustyline::EventContext,
    ) -> Option<rustyline::Cmd> {
        let key = evt.get(0)?;
        let is_down = match key {
            KeyEvent(KeyCode::Up, Modifiers::NONE) => Some(false),
            KeyEvent(KeyCode::Down, Modifiers::NONE) => Some(true),
            _ => None,
        }?;
        let entry = CommandHistory::browse_next(is_down);
        Some(Cmd::Replace(rustyline::Movement::WholeLine, entry))
    }
}

fn build_completion_trie() -> Trie<u8> {
    let executables = get_executables();
    let completion_iter = executables
        .into_iter()
        .chain(BUILTIN_COMMANDS.iter().map(|s| s.to_string()));
    Trie::from_iter(completion_iter)
}

fn get_executables() -> Vec<String> {
    env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .filter_map(|path| fs::read_dir(path).ok())
        .flat_map(|dir| dir.filter_map(Result::ok))
        .filter_map(|entry| {
            is_executable(&entry.path()).then(|| entry.file_name().to_string_lossy().into_owned())
        })
        .collect()
}
