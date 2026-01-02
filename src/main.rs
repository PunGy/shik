use shik::lang::{eval_file, run_repl};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);
    let mode = args.get(1).map(|arg| {
        if arg.starts_with("--") {
            arg[2..].to_string()
        } else {
            "file".to_string()
        }
    });

    if mode != None && mode.as_deref() == Some("file") {
        eval_file(args[1].clone());
    } else {
        run_repl(mode);
    }
}
