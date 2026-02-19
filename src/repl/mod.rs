pub mod helper;
pub mod highlighter;
pub mod validator;

use std::path::PathBuf;

use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Cmd, Config, Editor, EventHandler, KeyCode, KeyEvent, Modifiers};

use crate::cli::VERSION;
use crate::eval::evaluator::Interpretator;
use crate::lang::{evaluate, print, print_ast};
use crate::parser::parse;

use helper::ReplHelper;

pub struct ReplConfig {
    pub ast_mode: bool,
}

pub fn run(config: ReplConfig) {
    print_intro();

    let editor_config = Config::builder().history_ignore_space(true).build();

    let mut editor: Editor<ReplHelper, DefaultHistory> =
        Editor::with_config(editor_config).expect("Failed to initialize line editor");

    editor.set_helper(Some(ReplHelper::new()));

    // Shift+Enter inserts a literal newline (explicit multi-line input).
    editor.bind_sequence(
        KeyEvent(KeyCode::Enter, Modifiers::SHIFT),
        EventHandler::Simple(Cmd::Newline),
    );

    // Load persisted history; ignore errors (e.g. first run).
    let history_path = get_history_path();
    if let Some(ref path) = history_path {
        let _ = editor.load_history(path);
    }

    let interpretator = Interpretator::new();

    loop {
        match editor.readline("> ") {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                match input {
                    "quit" | "exit" => {
                        println!("Goodbye!");
                        break;
                    }
                    _ => {
                        editor.add_history_entry(input).ok();
                        if config.ast_mode {
                            print_ast(parse(input));
                        } else {
                            print(evaluate(input, &interpretator), false);
                        }
                    }
                }
            }

            // Ctrl+C: cancel the current input and continue.
            Err(ReadlineError::Interrupted) => {}

            // Ctrl+D: exit gracefully.
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }

            Err(err) => {
                eprintln!("Read error: {}", err);
                break;
            }
        }
    }

    // Persist history for the next session.
    if let Some(ref path) = history_path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = editor.save_history(path);
    }
}

fn print_intro() {
    println!("shik v{}", VERSION);
    println!(
        "Type 'help' to list built-ins  ·  Ctrl+D or 'quit' to exit  ·  Shift+Enter for multi-line"
    );
    println!();
}

fn get_history_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".shik_history"))
}
