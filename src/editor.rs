use anyhow::Result;
use std::{env, fs};

use once_cell::unsync::OnceCell;
use rustyline::{
    completion::Completer, history::FileHistory, Editor, Helper, Highlighter, Hinter, Validator,
};
use trie_rs::Trie;

use crate::command::is_executable;

pub type ShellEditor = Editor<ShellCompleter, FileHistory>;
const BUILTIN_COMMANDS: &[&str] = &["exit ", "echo ", "type ", "pwd ", "cd "];

pub fn get_editor() -> ShellEditor {
    let mut editor = Editor::new().unwrap();
    editor.set_helper(Some(ShellCompleter {
        trie: OnceCell::new(),
    }));
    editor
}

#[derive(Helper, Highlighter, Hinter, Validator)]
pub struct ShellCompleter {
    trie: OnceCell<Trie<u8>>,
}
impl Completer for ShellCompleter {
    type Candidate = String;
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
        let prefix = &line[start_idx..pos];
        let result: Vec<String> = trie.predictive_search(prefix).collect();
        if result.iter().any(|s| s == prefix) {
            Ok((start_idx, vec![]))
        } else {
            Ok((start_idx, result))
        }
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
            is_executable(&entry.path()).then(|| {
                let mut file = entry.file_name().to_string_lossy().into_owned();
                file.push(' ');
                file
            })
        })
        .collect()
}
