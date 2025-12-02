use shik::lang::{eval_file, run_repl};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);
    if args.len() > 1 {
        eval_file(args[1].clone());
    } else {
        run_repl();
    }
}
