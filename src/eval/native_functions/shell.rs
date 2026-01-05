use crate::{
    count_args, define_native, define_help,
    eval::{
        error::{RuntimeError, ShikError},
        evaluator::Interpretator,
        native_functions::native_result,
        value::{
            EnvRef, NativeClosure, NativeContext, NativeFn, SpecialClosure, SpecialFn, Value,
            ValueRef,
        },
        EvalResult,
    },
    native_op,
    parser::Expression,
    special_op,
};
use std::io::{self, Write};
use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};
use std::rc::Rc;

// ============================================================================
// Shell Execution Functions
// ============================================================================

// Execute a shell command and return stdout as a string
// Usage: shell "ls -la"
native_op!(Shell, "shell", [cmd], {
    let cmd = cmd.expect_string()?;

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            native_result(Value::String(stdout))
        }
        Err(e) => Err(ShikError::default_error(format!(
            "shell command failed: {}",
            e
        ))),
    }
});

// Execute a shell command, return exit code and show output in terminal
// Usage: shell.code "ls -la"
native_op!(ShellExec, "shell!", [cmd], {
    let cmd = cmd.expect_string()?;

    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).status()
    } else {
        Command::new("sh").args(["-c", cmd]).status()
    };

    match status {
        Ok(status) => {
            let code = status.code().unwrap_or(-1) as f64;
            native_result(Value::Number(code))
        }
        Err(e) => Err(ShikError::default_error(format!(
            "shell command failed: {}",
            e
        ))),
    }
});

// Execute a shell command and return exit code, discards output
// Usage: shell.code "ls -la"
native_op!(ShellCode, "shell.code", [cmd], {
    let cmd = cmd.expect_string()?;

    let res = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    };

    match res {
        Ok(output) => {
            let code = output.status.code().unwrap_or(-1) as f64;
            native_result(Value::Number(code))
        }
        Err(e) => Err(ShikError::default_error(format!(
            "shell command failed: {}",
            e
        ))),
    }
});

// Execute a shell command and return an object with stdout, stderr, and code
// Usage: shell.full "ls -la"
native_op!(ShellFull, "shell.full", [cmd], {
    let cmd = cmd.expect_string()?;

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let code = output.status.code().unwrap_or(-1) as f64;

            let mut result: HashMap<String, ValueRef> = HashMap::new();
            result.insert("stdout".to_string(), Rc::new(Value::String(stdout)));
            result.insert("stderr".to_string(), Rc::new(Value::String(stderr)));
            result.insert("code".to_string(), Rc::new(Value::Number(code)));
            result.insert(
                "ok".to_string(),
                Rc::new(Value::Bool(output.status.success())),
            );

            native_result(Value::Object(result))
        }
        Err(e) => Err(ShikError::default_error(format!(
            "shell command failed: {}",
            e
        ))),
    }
});

// Try to execute a shell command, return null on failure
// Usage: shell.try "ls -la"
native_op!(ShellTry, "shell?", [cmd], {
    let cmd = cmd.expect_string()?;

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    };

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            native_result(Value::String(stdout))
        }
        _ => native_result(Value::Null),
    }
});

// Execute a shell command silently (discard output), return success boolean
// Usage: shell.ok "mkdir -p /tmp/test"
native_op!(ShellOk, "shell.ok?", [cmd], {
    let cmd = cmd.expect_string()?;

    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    } else {
        Command::new("sh")
            .args(["-c", cmd])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    };

    match status {
        Ok(status) => native_result(Value::Bool(status.success())),
        Err(_) => native_result(Value::Bool(false)),
    }
});

// Execute a shell command and return lines as a list
// Usage: shell.lines "ls"
native_op!(ShellLines, "shell.lines", [cmd], {
    let cmd = cmd.expect_string()?;

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<ValueRef> = stdout
                .lines()
                .map(|line| Rc::new(Value::String(line.to_string())))
                .collect();
            native_result(Value::List(lines))
        }
        Err(e) => Err(ShikError::default_error(format!(
            "shell command failed: {}",
            e
        ))),
    }
});

// ============================================================================
// Input/Output
// ============================================================================

special_op!(ShellRead, "shell.ask", args, ctx, {
    if args.len() > 1 {
        return Err(ShikError::default_error(
            "shell.read expects 0 or 1 arguments".to_string(),
        ));
    }

    // Optional prompt
    if args.len() == 1 {
        let prompt_val = ctx.inter.eval_expr(&args[0], &ctx.env)?;
        let prompt = prompt_val.expect_string()?;

        print!("{prompt}");
        io::stdout()
            .flush()
            .map_err(|e| ShikError::default_error(format!("cannot write prompt: {}", e)))?;
    }

    // Read input
    let mut line = String::new();
    let n = io::stdin()
        .read_line(&mut line)
        .map_err(|e| ShikError::default_error(format!("cannot read from stdin: {}", e)))?;

    // EOF
    if n == 0 {
        return native_result(Value::Null);
    }

    // Strip trailing newline(s)
    let line = line.trim_end_matches(&['\r', '\n'][..]).to_string();
    native_result(Value::String(line))
});

// ============================================================================
// Environment Variable Functions
// ============================================================================

// Get an environment variable, null if not found
// Usage: shell.env "HOME"
native_op!(ShellEnv, "shell.env", [name], {
    let name = name.expect_string()?;

    match env::var(name) {
        Ok(value) => native_result(Value::String(value)),
        Err(_) => native_result(Value::Null),
    }
});

// Set an environment variable (for current process)
// Usage: shell.set-env "MY_VAR" "value"
native_op!(ShellSetEnv, "shell.env.set", [name, value], {
    let name = name.expect_string()?;
    let value = value.expect_string()?;

    // SAFETY: We're setting environment variables in a single-threaded context
    // This is safe as long as no other threads are reading environment variables
    unsafe { env::set_var(name, value) };
    native_result(Value::Null)
});

// Remove an environment variable
// Usage: shell.unset-env "MY_VAR"
native_op!(ShellUnsetEnv, "shell.env.remove", [name], {
    let name = name.expect_string()?;
    // SAFETY: We're removing environment variables in a single-threaded context
    // This is safe as long as no other threads are reading environment variables
    unsafe { env::remove_var(name) };
    native_result(Value::Null)
});

// Get all environment variables as an object
// Usage: shell.env-all
native_op!(ShellEnvAll, "shell.env.all", [], {
    let mut result: HashMap<String, ValueRef> = HashMap::new();

    for (key, value) in env::vars() {
        result.insert(key, Rc::new(Value::String(value)));
    }

    native_result(Value::Object(result))
});

// ============================================================================
// Working Directory Functions
// ============================================================================

// Get current working directory
// Usage: shell.cwd
native_op!(ShellCwd, "shell.cwd", [], {
    match env::current_dir() {
        Ok(path) => native_result(Value::String(path.to_string_lossy().to_string())),
        Err(e) => Err(ShikError::default_error(format!(
            "cannot get current directory: {}",
            e
        ))),
    }
});

// Change current working directory
// Usage: shell.cd "/tmp"
native_op!(ShellCd, "shell.cd", [path], {
    let path = path.expect_string()?;

    match env::set_current_dir(path) {
        Ok(_) => native_result(Value::Null),
        Err(e) => Err(ShikError::default_error(format!(
            "cannot change directory to '{}': {}",
            path, e
        ))),
    }
});

// Get home directory
// Usage: shell.home
native_op!(ShellHome, "shell.home", [], {
    match env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        Ok(home) => native_result(Value::String(home)),
        Err(_) => Err(ShikError::default_error(
            "cannot determine home directory".to_string(),
        )),
    }
});

// ============================================================================
// Path Utilities
// ============================================================================

// Find executable in PATH
// Usage: shell.which "git"
native_op!(ShellWhich, "shell.which", [name], {
    let name = name.expect_string()?;

    let path_var = env::var("PATH").unwrap_or_default();
    let path_sep = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };

    for dir in path_var.split(path_sep) {
        let candidate = std::path::Path::new(dir).join(name);
        if candidate.exists() {
            return native_result(Value::String(candidate.to_string_lossy().to_string()));
        }
        // On Windows, also check with common extensions
        if cfg!(target_os = "windows") {
            for ext in &[".exe", ".cmd", ".bat", ".com"] {
                let with_ext = candidate.with_extension(&ext[1..]);
                if with_ext.exists() {
                    return native_result(Value::String(with_ext.to_string_lossy().to_string()));
                }
            }
        }
    }

    native_result(Value::Null)
});

// Check if a command exists in PATH
// Usage: shell.has "git"
native_op!(ShellHas, "shell.has", [name], {
    let name = name.expect_string()?;

    let path_var = env::var("PATH").unwrap_or_default();
    let path_sep = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };

    for dir in path_var.split(path_sep) {
        let candidate = std::path::Path::new(dir).join(name);
        if candidate.exists() {
            return native_result(Value::Bool(true));
        }
        if cfg!(target_os = "windows") {
            for ext in &[".exe", ".cmd", ".bat", ".com"] {
                let with_ext = candidate.with_extension(&ext[1..]);
                if with_ext.exists() {
                    return native_result(Value::Bool(true));
                }
            }
        }
    }

    native_result(Value::Bool(false))
});

// ============================================================================
// Process Information
// ============================================================================

// Get current process ID
// Usage: process.pid
native_op!(ProcessPid, "process.pid", [], {
    native_result(Value::Number(std::process::id() as f64))
});

// Get command line arguments
// Usage: shell.args
native_op!(ShellArgs, "shell.args", [], {
    let args: Vec<ValueRef> = env::args().map(|arg| Rc::new(Value::String(arg))).collect();
    native_result(Value::List(args))
});

// Get command line arguments, without caller and filename.
// Usage: process.args
native_op!(ProcessArgs, "process.args", [], {
    let mut args = env::args();
    if args.len() < 3 {
        return native_result(Value::List([].to_vec()));
    }
    args.next(); // skip shik
    args.next(); // skip filename
    let args: Vec<ValueRef> = args.map(|arg| Rc::new(Value::String(arg))).collect();
    native_result(Value::List(args))
});

// Get name of the file currently executed. In case of repl would return null
// Usage: process.file
native_op!(ProcessFile, "process.file", [], {
    let mut args = env::args();
    if args.len() == 1 {
        return native_result(Value::Null);
    }
    args.next(); // skip shik
    let name = args.next().unwrap();
    native_result(Value::String(name))
});

// Get OS name
// Usage: shell.os
native_op!(ShellOs, "shell.os", [], {
    native_result(Value::String(env::consts::OS.to_string()))
});

// Get architecture
// Usage: shell.arch
native_op!(ShellArch, "shell.arch", [], {
    native_result(Value::String(env::consts::ARCH.to_string()))
});

// ============================================================================
// Process Control Functions
// ============================================================================

// Exit the process with a specific exit code
// Usage: exit 0
native_op!(ProcessExit, "exit", [code], {
    let code = code.expect_number()? as i32;
    std::process::exit(code);
});

// Exit the process with exit code 0 (success)
// Usage: exit!
native_op!(ProcessExitSuccess, "exit!", [], {
    std::process::exit(0);
});

// Abort the process immediately (abnormal termination)
// Usage: process.abort
native_op!(ProcessAbort, "process.abort", [], {
    std::process::abort();
});

// Sleep for specified milliseconds
// Usage: process.sleep 1000
native_op!(ProcessSleep, "process.sleep", [ms], {
    let ms = ms.expect_number()? as u64;
    std::thread::sleep(std::time::Duration::from_millis(ms));
    native_result(Value::Null)
});

// ============================================================================
// Module Binding
// ============================================================================

pub fn bind_shell_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Module help
    env.define_help("shell.".to_string(), "shell module:

Execution:
- shell: executes command, returns stdout
- shell!: executes with output shown, returns exit code
- shell.code: executes silently, returns exit code
- shell.full: returns object with stdout, stderr, code, ok
- shell?: tries to execute, returns null on failure
- shell.ok?: returns true if command succeeds
- shell.lines: returns stdout as list of lines

Environment:
- shell.env: gets environment variable
- shell.env.set: sets environment variable
- shell.env.remove: removes environment variable
- shell.env.all: returns all env vars as object

I/O:
- shell.ask: reads line from stdin

Directory:
- shell.cwd: returns current directory
- shell.cd: changes directory
- shell.home: returns home directory

Path utilities:
- shell.which: finds executable in PATH
- shell.has: checks if command exists

System info:
- shell.args: all command line arguments
- shell.os: operating system name
- shell.arch: CPU architecture".to_string());

    env.define_help("process.".to_string(), "process module:

- process.pid: current process ID
- process.file: currently executed file
- process.args: command line arguments (without shik and filename)
- process.sleep: sleeps for milliseconds
- process.abort: aborts process".to_string());

    env.define_help("exit".to_string(), "exit [number]: exits process with given exit code

exit 0  ; success
exit 1  ; failure".to_string());

    // Shell execution
    define_native!(Shell, env, inter);
    define_help!(Shell, env, "[cmd:string]: executes shell command, returns stdout as string\n\nshell \"ls -la\"");

    define_native!(ShellExec, env, inter);
    define_help!(ShellExec, env, "[cmd:string]: executes shell command with output shown in terminal, returns exit code\n\nshell! \"npm install\"");

    define_native!(ShellCode, env, inter);
    define_help!(ShellCode, env, "[cmd:string]: executes shell command silently, returns exit code\n\nshell.code \"test -f file.txt\"");

    define_native!(ShellFull, env, inter);
    define_help!(ShellFull, env, "[cmd:string]: executes shell command, returns object with stdout, stderr, code, ok\n\nshell.full \"ls\"");

    define_native!(ShellTry, env, inter);
    define_help!(ShellTry, env, "[cmd:string]: executes shell command, returns stdout or null on failure\n\nshell? \"cat maybe.txt\"");

    define_native!(ShellOk, env, inter);
    define_help!(ShellOk, env, "[cmd:string]: executes shell command silently, returns true if successful\n\nshell.ok? \"which git\"");

    define_native!(ShellLines, env, inter);
    define_help!(ShellLines, env, "[cmd:string]: executes shell command, returns stdout lines as list\n\nshell.lines \"ls\"");

    // Environment variables
    define_native!(ShellEnv, env, inter);
    define_help!(ShellEnv, env, "[name:string]: gets environment variable, returns null if not found\n\nshell.env \"HOME\"");

    define_native!(ShellSetEnv, env, inter);
    define_help!(ShellSetEnv, env, "[name:string value:string]: sets environment variable for current process\n\nshell.env.set \"MY_VAR\" \"value\"");

    define_native!(ShellUnsetEnv, env, inter);
    define_help!(ShellUnsetEnv, env, "[name:string]: removes environment variable\n\nshell.env.remove \"MY_VAR\"");

    define_native!(ShellEnvAll, env, inter);
    define_help!(ShellEnvAll, env, "[]: returns all environment variables as object\n\nshell.env.all");

    // IO
    ShellRead::define(&env, Rc::clone(&inter));
    define_help!(ShellRead, env, "[prompt:string?]: reads line from stdin, optional prompt\n\nshell.ask \"Enter name: \"");

    // Working directory
    define_native!(ShellCwd, env, inter);
    define_help!(ShellCwd, env, "[]: returns current working directory\n\nshell.cwd");

    define_native!(ShellCd, env, inter);
    define_help!(ShellCd, env, "[path:string]: changes current working directory\n\nshell.cd \"/tmp\"");

    define_native!(ShellHome, env, inter);
    define_help!(ShellHome, env, "[]: returns home directory path\n\nshell.home");

    // Path utilities
    define_native!(ShellWhich, env, inter);
    define_help!(ShellWhich, env, "[name:string]: finds executable in PATH, returns path or null\n\nshell.which \"git\"");

    define_native!(ShellHas, env, inter);
    define_help!(ShellHas, env, "[name:string]: checks if command exists in PATH\n\nshell.has \"git\"");

    // Process information
    define_native!(ProcessPid, env, inter);
    define_help!(ProcessPid, env, "[]: returns current process ID\n\nprocess.pid");

    define_native!(ProcessFile, env, inter);
    define_help!(ProcessFile, env, "[]: returns name of currently executed file, null in REPL\n\nprocess.file");

    define_native!(ShellArgs, env, inter);
    define_help!(ShellArgs, env, "[]: returns all command line arguments as list\n\nshell.args");

    define_native!(ProcessArgs, env, inter);
    define_help!(ProcessArgs, env, "[]: returns command line arguments (without shik and filename)\n\nprocess.args");

    define_native!(ShellOs, env, inter);
    define_help!(ShellOs, env, "[]: returns operating system name (linux, macos, windows)\n\nshell.os");

    define_native!(ShellArch, env, inter);
    define_help!(ShellArch, env, "[]: returns CPU architecture (x86_64, aarch64, etc.)\n\nshell.arch");

    // Process control
    define_native!(ProcessExit, env, inter);
    define_help!(ProcessExit, env, "[code:number]: exits process with given exit code\n\nexit 0");

    define_native!(ProcessExitSuccess, env, inter);
    define_help!(ProcessExitSuccess, env, "[]: exits process with code 0 (success)\n\nexit!");

    define_native!(ProcessAbort, env, inter);
    define_help!(ProcessAbort, env, "[]: aborts process immediately (abnormal termination)\n\nprocess.abort");

    define_native!(ProcessSleep, env, inter);
    define_help!(ProcessSleep, env, "[ms:number]: sleeps for specified milliseconds\n\nprocess.sleep 1000");
}
