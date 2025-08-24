use anyhow::Result;
use codecrafters_shell::command::handle_command;
use codecrafters_shell::context::CommandContext;
use codecrafters_shell::editor::get_editor;

fn main() -> Result<()> {
    let mut editor = get_editor();
    loop {
        let input = editor.readline("$ ")?;
        let context = match CommandContext::try_from(input.as_str()) {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!("Error processing input: {}", e);
                continue;
            }
        };
        match handle_command(&context) {
            Ok(_) => continue,
            Err(e) => {
                context.ewriteln(e);
                continue;
            }
        }
    }
}
