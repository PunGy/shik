use crate::{
    count_args, define_native,
    eval::{
        error::{RuntimeError, ShikError},
        evaluator::Interpretator,
        native_functions::native_result,
        value::{EnvRef, NativeClosure, NativeContext, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
};
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

// Execute a shell command and return exit code
// Usage: shell.code "ls -la"
native_op!(ShellCode, "shell.code", [cmd], {
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
native_op!(ShellTry, "shell.try", [cmd], {
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
native_op!(ShellOk, "shell.ok", [cmd], {
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
// Environment Variable Functions
// ============================================================================

// Get an environment variable, error if not found
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
// Usage: shell.pid
native_op!(ProcessPid, "proccess.pid", [], {
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
// Module Binding
// ============================================================================

pub fn bind_shell_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Shell execution
    define_native!(Shell, env, inter);
    define_native!(ShellCode, env, inter);
    define_native!(ShellFull, env, inter);
    define_native!(ShellTry, env, inter);
    define_native!(ShellOk, env, inter);
    define_native!(ShellLines, env, inter);

    // Environment variables
    define_native!(ShellEnv, env, inter);
    define_native!(ShellSetEnv, env, inter);
    define_native!(ShellUnsetEnv, env, inter);
    define_native!(ShellEnvAll, env, inter);

    // Working directory
    define_native!(ShellCwd, env, inter);
    define_native!(ShellCd, env, inter);
    define_native!(ShellHome, env, inter);

    // Path utilities
    define_native!(ShellWhich, env, inter);
    define_native!(ShellHas, env, inter);

    // Process information
    define_native!(ProcessPid, env, inter);
    define_native!(ShellArgs, env, inter);
    define_native!(ProcessArgs, env, inter);
    define_native!(ShellOs, env, inter);
    define_native!(ShellArch, env, inter);
}

