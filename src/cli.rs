pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum Command {
    Repl { ast_mode: bool },
    File { path: String },
    Help,
    Version,
}

pub fn parse_args(args: &[String]) -> Command {
    let mut ast_mode = false;
    let mut positional: Vec<&str> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => return Command::Help,
            "--version" | "-V" => return Command::Version,
            "--ast" => ast_mode = true,
            a if a.starts_with("--") => {
                eprintln!("Unknown flag: {}", a);
                eprintln!("Run 'shik --help' for usage.");
                std::process::exit(1);
            }
            _ => positional.push(arg),
        }
    }

    if let Some(path) = positional.first() {
        return Command::File {
            path: path.to_string(),
        };
    }

    Command::Repl { ast_mode }
}

pub fn print_help() {
    println!(
        "shik {} — a functional scripting language for shell automation",
        VERSION
    );
    println!();
    println!("USAGE:");
    println!("  shik                    Start interactive REPL");
    println!("  shik <file>             Run a Shik script file");
    println!("  shik --help             Show this help message");
    println!("  shik --version          Print version");
    println!();
    println!("REPL SHORTCUTS:");
    println!("  ↑ / ↓                   Navigate history");
    println!("  ← / →                   Move cursor");
    println!("  Ctrl+← / Ctrl+→         Jump by word");
    println!("  Alt+← / Alt+→           Jump by word (alternative)");
    println!("  Shift+Enter             Insert newline (multi-line input)");
    println!("  Ctrl+C                  Cancel current input");
    println!("  Ctrl+D                  Exit");
    println!("  Ctrl+A / Ctrl+E         Jump to start / end of line");
}

pub fn print_version() {
    println!("shik {}", VERSION);
}
