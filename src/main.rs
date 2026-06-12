use shik::cli::{parse_args, print_help, print_version, Command};
use shik::lang::eval_file;
use shik::repl;
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let command = match parse_args(&args) {
        Ok(command) => command,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("Run 'shik --help' for usage.");
            return ExitCode::from(2);
        }
    };

    match command {
        Command::Help => print_help(),
        Command::Version => print_version(),
        Command::File { path, ast_mode } => {
            if let Err(e) = eval_file(&path, ast_mode) {
                eprintln!("{}", e);
                return ExitCode::FAILURE;
            }
        }
        Command::Repl { ast_mode } => repl::run(repl::ReplConfig { ast_mode }),
    }

    ExitCode::SUCCESS
}
