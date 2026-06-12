pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Repl { ast_mode: bool },
    File { path: String, ast_mode: bool },
    Help,
    Version,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Unknown flag: {flag}")]
pub struct CliError {
    pub flag: String,
}

pub fn parse_args(args: &[String]) -> Result<Command, CliError> {
    let mut ast_mode = false;
    let mut positional: Vec<&str> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => return Ok(Command::Help),
            "--version" | "-V" => return Ok(Command::Version),
            "--ast" => ast_mode = true,
            a if a.starts_with("--") => {
                return Err(CliError {
                    flag: a.to_string(),
                })
            }
            _ => positional.push(arg),
        }
    }

    if let Some(path) = positional.first() {
        return Ok(Command::File {
            path: path.to_string(),
            ast_mode,
        });
    }

    Ok(Command::Repl { ast_mode })
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
    println!("  shik --ast [file]       Print the AST instead of evaluating");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_args_starts_repl() {
        assert_eq!(
            parse_args(&args(&[])),
            Ok(Command::Repl { ast_mode: false })
        );
    }

    #[test]
    fn ast_flag_alone_starts_ast_repl() {
        assert_eq!(
            parse_args(&args(&["--ast"])),
            Ok(Command::Repl { ast_mode: true })
        );
    }

    #[test]
    fn file_path_runs_file() {
        assert_eq!(
            parse_args(&args(&["script.shk"])),
            Ok(Command::File {
                path: "script.shk".to_string(),
                ast_mode: false
            })
        );
    }

    #[test]
    fn ast_flag_with_file_is_honored_in_any_order() {
        let expected = Ok(Command::File {
            path: "script.shk".to_string(),
            ast_mode: true,
        });
        assert_eq!(parse_args(&args(&["--ast", "script.shk"])), expected);
        assert_eq!(parse_args(&args(&["script.shk", "--ast"])), expected);
    }

    #[test]
    fn help_and_version_win_over_everything() {
        assert_eq!(parse_args(&args(&["script.shk", "-h"])), Ok(Command::Help));
        assert_eq!(
            parse_args(&args(&["--version", "script.shk"])),
            Ok(Command::Version)
        );
    }

    #[test]
    fn unknown_flag_is_an_error() {
        assert_eq!(
            parse_args(&args(&["--frobnicate"])),
            Err(CliError {
                flag: "--frobnicate".to_string()
            })
        );
    }
}
