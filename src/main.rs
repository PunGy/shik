use shik::cli::{parse_args, print_help, print_version, Command};
use shik::lang::eval_file;
use shik::repl;
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match parse_args(&args) {
        Command::Help => print_help(),
        Command::Version => print_version(),
        Command::File { path } => eval_file(path),
        Command::Repl { ast_mode } => repl::run(repl::ReplConfig { ast_mode }),
    }
}
