use crate::{
    count_args, define_help, define_native,
    eval::{
        error::{RuntimeError, ShikError},
        evaluator::Interpretator,
        native_functions::native_result,
        value::{EnvRef, NativeClosure, NativeContext, NativeFn, Value, ValueRef},
        EvalResult,
    },
    native_op,
};
use glob::glob;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

// ============================================================================
// File Reading Functions
// ============================================================================

// Read file contents as string
// Usage: file.read "path/to/file.txt"
native_op!(FileRead, "file.read", [path], {
    let path = path.expect_string()?;

    let content = fs::read_to_string(path)
        .map_err(|e| ShikError::default_error(format!("cannot open file - {}", e)))?;

    native_result(Value::String(content))
});

// Try to read file, return null on failure
// Usage: file.read? "path/to/file.txt"
native_op!(FileTryRead, "file.read?", [path], {
    let path = path.expect_string()?;

    match fs::read_to_string(path) {
        Ok(content) => native_result(Value::String(content)),
        Err(_) => native_result(Value::Null),
    }
});

// Read file as binary (returns list of numbers 0-255)
// Usage: file.read-bytes "path/to/file.bin"
native_op!(FileReadBytes, "file.read-bytes", [path], {
    let path = path.expect_string()?;

    let bytes = fs::read(path)
        .map_err(|e| ShikError::default_error(format!("cannot read file - {}", e)))?;

    let result: Vec<ValueRef> = bytes
        .into_iter()
        .map(|b| Rc::new(Value::Number(b as f64)))
        .collect();

    native_result(Value::List(result))
});

// Read file lines as a list
// Usage: file.lines "path/to/file.txt"
native_op!(FileLines, "file.read-lines", [path], {
    let path = path.expect_string()?;

    let content = fs::read_to_string(path)
        .map_err(|e| ShikError::default_error(format!("cannot read file - {}", e)))?;

    let lines: Vec<ValueRef> = content
        .lines()
        .map(|line| Rc::new(Value::String(line.to_string())))
        .collect();

    native_result(Value::List(lines))
});

// ============================================================================
// File Writing Functions
// ============================================================================

// Write string to file (overwrites existing)
// Usage: file.write "path/to/file.txt" "content"
native_op!(FileWrite, "file.write", [path, content], {
    let path = path.expect_string()?;
    let content = content.expect_string()?;

    fs::write(path, content)
        .map_err(|e| ShikError::default_error(format!("cannot write file {}: {}", path, e)))?;

    native_result(Value::Null)
});

// Append string to file
// Usage: file.append "path/to/file.txt" "content"
native_op!(FileAppend, "file.append", [path, content], {
    let path = path.expect_string()?;
    let content = content.expect_string()?;

    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| ShikError::default_error(format!("cannot open file {}: {}", path, e)))?;

    file.write_all(content.as_bytes())
        .map_err(|e| ShikError::default_error(format!("cannot write to file {}: {}", path, e)))?;

    native_result(Value::Null)
});

// Write bytes to file (takes list of numbers 0-255)
// Usage: file.write-bytes "path/to/file.bin" [72 101 108 108 111]
native_op!(FileWriteBytes, "file.write-bytes", [path, bytes], {
    let path = path.expect_string()?;
    let bytes_list = bytes.expect_list()?;

    let mut bytes_vec: Vec<u8> = Vec::with_capacity(bytes_list.len());
    for b in bytes_list.iter() {
        let num = b.expect_number()?;
        if num < 0.0 || num > 255.0 {
            return Err(ShikError::default_error(format!(
                "byte value out of range: {}",
                num
            )));
        }
        bytes_vec.push(num as u8);
    }

    fs::write(path, bytes_vec)
        .map_err(|e| ShikError::default_error(format!("cannot write file {}: {}", path, e)))?;

    native_result(Value::Null)
});

// ============================================================================
// File/Directory Operations
// ============================================================================

// Copy file or directory
// Usage: file.copy "destination" "source"
native_op!(FileCopy, ["file.copy", "file.cp"], [dst, src], {
    let src = src.expect_string()?;
    let dst = dst.expect_string()?;

    let src_path = Path::new(src);
    let dst_path = Path::new(dst);

    // if destination is an existing directory, move source into it
    let final_dst = if dst_path.is_dir() {
        match src_path.file_name() {
            Some(name) => dst_path.join(name),
            None => return Err(ShikError::default_error("invalid source path".to_string())),
        }
    } else {
        dst_path.to_path_buf()
    };

    if src_path.is_dir() {
        copy_dir_recursive(src_path, &final_dst)?;
    } else {
        fs::copy(src_path, &final_dst)
            .map_err(|e| ShikError::default_error(format!("cannot copy file: {}", e)))?;
    }

    native_result(Value::Null)
});

// Move/rename file or directory
// Usage: file.move "source" "destination"
native_op!(FileMove, ["file.move", "file.mv"], [dst, src], {
    let src = src.expect_string()?;
    let dst = dst.expect_string()?;

    let src_path = Path::new(src);
    let dst_path = Path::new(dst);

    // if destination is an existing directory, move source into it
    let final_dst = if dst_path.is_dir() {
        match src_path.file_name() {
            Some(name) => dst_path.join(name),
            None => return Err(ShikError::default_error("invalid source path".to_string())),
        }
    } else {
        dst_path.to_path_buf()
    };

    fs::rename(src_path, &final_dst)
        .map_err(|e| ShikError::default_error(format!("cannot move file: {}", e)))?;

    native_result(Value::Null)
});

// Delete any file or directory(recursively)
// Usage: file.remove "path/to/file.txt"
native_op!(FileRm, ["file.remove", "file.rm"], [path], {
    let path = path.expect_string()?;

    if Path::new(path).is_dir() {
        fs::remove_dir_all(path)
            .map_err(|e| ShikError::default_error(format!("cannot remove directory: {}", e)))?
    } else {
        fs::remove_file(path)
            .map_err(|e| ShikError::default_error(format!("cannot delete file: {}", e)))?;
    }

    native_result(Value::Null)
});

// Delete directory (must be empty)
// Usage: file.rmdir "path/to/dir"
native_op!(FileRmdir, "file.rmdir", [path], {
    let path = path.expect_string()?;

    fs::remove_dir(path)
        .map_err(|e| ShikError::default_error(format!("cannot remove directory: {}", e)))?;

    native_result(Value::Null)
});

// Delete directory recursively
// Usage: file.rmdir-all "path/to/dir"
native_op!(FileRmdirAll, "file.rmdir!", [path], {
    let path = path.expect_string()?;

    fs::remove_dir_all(path)
        .map_err(|e| ShikError::default_error(format!("cannot remove directory: {}", e)))?;

    native_result(Value::Null)
});

// Create directory
// Usage: file.mkdir "path/to/dir"
native_op!(FileMkdir, "file.mkdir", [path], {
    let path = path.expect_string()?;

    fs::create_dir(path)
        .map_err(|e| ShikError::default_error(format!("cannot create directory: {}", e)))?;

    native_result(Value::Null)
});

// Create directory and all parent directories
// Usage: file.mkdir! "path/to/nested/dir"
native_op!(FileMkdirAll, "file.mkdir!", [path], {
    let path = path.expect_string()?;

    fs::create_dir_all(path)
        .map_err(|e| ShikError::default_error(format!("cannot create directories: {}", e)))?;

    native_result(Value::Null)
});

// ============================================================================
// File Information Functions
// ============================================================================

// Check if path exists
// Usage: file.exists "path"
native_op!(FileExists, "file.exists", [path], {
    let path = path.expect_string()?;
    native_result(Value::Bool(Path::new(path).exists()))
});

// Check if path is a directory
// Usage: file.is-dir "path"
native_op!(FileIsDir, "file.is-dir", [path], {
    let path = path.expect_string()?;
    native_result(Value::Bool(Path::new(path).is_dir()))
});

// Check if path is a file
// Usage: file.is-file "path"
native_op!(FileIsFile, "file.is-file", [path], {
    let path = path.expect_string()?;
    native_result(Value::Bool(Path::new(path).is_file()))
});

// Check if path is a symlink
// Usage: file.is-symlink "path"
native_op!(FileIsSymlink, "file.is-symlink", [path], {
    let path = path.expect_string()?;
    native_result(Value::Bool(Path::new(path).is_symlink()))
});

// Get file size in bytes
// Usage: file.size "path/to/file.txt"
native_op!(FileSize, "file.size", [path], {
    let path = path.expect_string()?;

    let metadata = fs::metadata(path)
        .map_err(|e| ShikError::default_error(format!("cannot get file metadata: {}", e)))?;

    native_result(Value::Number(metadata.len() as f64))
});

/// Compute the size of a directory recursively (in bytes).
/// - Follows only real directories (symlinks are skipped).
/// - Counts only regular files.
fn dir_size(root: &Path) -> io::Result<u64> {
    let mut total: u64 = 0;
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry_res in fs::read_dir(&dir)? {
            let entry = entry_res?;

            // Use symlink_metadata so we can see if it's a symlink and skip it.
            let metadata = fs::symlink_metadata(entry.path())?;
            let file_type = metadata.file_type();

            if file_type.is_dir() {
                // Recurse into real directories
                stack.push(entry.path());
            } else if file_type.is_file() {
                // Add file size; saturating_add avoids overflow panics
                total = total.saturating_add(metadata.len());
            } else {
                // Symlinks, sockets, devices, etc. are ignored
            }
        }
    }

    Ok(total)
}

native_op!(FileSizeDeep, "file.size.deep", [path], {
    use std::path::Path;

    let path = path.expect_string()?;
    let path = Path::new(&path);

    // Use symlink_metadata here so we can distinguish symlinks if needed.
    let metadata = fs::symlink_metadata(path)
        .map_err(|e| ShikError::default_error(format!("cannot get file metadata: {}", e)))?;

    let size = if metadata.is_file() {
        metadata.len()
    } else if metadata.is_dir() {
        dir_size(path)
            .map_err(|e| ShikError::default_error(format!("cannot traverse directory: {}", e)))?
    } else {
        // For symlinks, devices, etc. we return 0
        0
    };

    native_result(Value::Number(size as f64))
});

// Get file metadata as object
// Usage: file.stat "path/to/file.txt"
native_op!(FileStat, "file.stat", [path], {
    let path = path.expect_string()?;

    let metadata = fs::metadata(path)
        .map_err(|e| ShikError::default_error(format!("cannot get file metadata: {}", e)))?;

    let mut result: HashMap<String, ValueRef> = HashMap::new();
    result.insert(
        "size".to_string(),
        Rc::new(Value::Number(metadata.len() as f64)),
    );
    result.insert(
        "is_file".to_string(),
        Rc::new(Value::Bool(metadata.is_file())),
    );
    result.insert(
        "is_dir".to_string(),
        Rc::new(Value::Bool(metadata.is_dir())),
    );
    result.insert(
        "is_symlink".to_string(),
        Rc::new(Value::Bool(metadata.is_symlink())),
    );
    result.insert(
        "readonly".to_string(),
        Rc::new(Value::Bool(metadata.permissions().readonly())),
    );

    native_result(Value::Object(result))
});

// ============================================================================
// Directory Listing Functions
// ============================================================================

// List directory contents
// Usage: file.list "path/to/dir"
native_op!(FileList, "file.list", [path], {
    let path = path.expect_string()?;

    let entries = fs::read_dir(path)
        .map_err(|e| ShikError::default_error(format!("cannot read directory: {}", e)))?;

    let mut result: Vec<ValueRef> = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|e| ShikError::default_error(format!("cannot read entry: {}", e)))?;
        let name = entry.file_name().to_string_lossy().to_string();
        result.push(Rc::new(Value::String(name)));
    }

    native_result(Value::List(result))
});

// List directory contents with full paths
// Usage: file.list! "path/to/dir"
native_op!(FileListPaths, "file.list!", [path], {
    let path = path.expect_string()?;

    let entries = fs::read_dir(path)
        .map_err(|e| ShikError::default_error(format!("cannot read directory: {}", e)))?;

    let mut result: Vec<ValueRef> = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|e| ShikError::default_error(format!("cannot read entry: {}", e)))?;
        let path_str = entry.path().to_string_lossy().to_string();
        result.push(Rc::new(Value::String(path_str)));
    }

    native_result(Value::List(result))
});

// Glob pattern matching
// Usage: file.glob "*.txt"
native_op!(FileGlob, "file.glob", [pattern], {
    let pattern = pattern.expect_string()?;

    let paths = glob(pattern)
        .map_err(|e| ShikError::default_error(format!("invalid glob pattern: {}", e)))?;

    let mut result: Vec<ValueRef> = Vec::new();
    for entry in paths {
        match entry {
            Ok(path) => {
                let path_str = path.to_string_lossy().to_string();
                result.push(Rc::new(Value::String(path_str)));
            }
            Err(e) => {
                return Err(ShikError::default_error(format!("glob error: {}", e)));
            }
        }
    }

    native_result(Value::List(result))
});

// ============================================================================
// Path Manipulation Functions
// ============================================================================

// Get file name from path
// Usage: file.name "/path/to/file.txt" -> "file.txt"
native_op!(FileName, "path.name", [path], {
    let path = path.expect_string()?;
    let p = Path::new(path);

    match p.file_name() {
        Some(name) => native_result(Value::String(name.to_string_lossy().to_string())),
        None => native_result(Value::Null),
    }
});

// Get file stem (name without extension)
// Usage: file.stem "/path/to/file.txt" -> "file"
native_op!(FileStem, "path.stem", [path], {
    let path = path.expect_string()?;
    let p = Path::new(path);

    match p.file_stem() {
        Some(stem) => native_result(Value::String(stem.to_string_lossy().to_string())),
        None => native_result(Value::Null),
    }
});

// Get file extension
// Usage: file.ext "/path/to/file.txt" -> "txt"
native_op!(FileExt, "path.ext", [path], {
    let path = path.expect_string()?;
    let p = Path::new(path);

    match p.extension() {
        Some(ext) => native_result(Value::String(ext.to_string_lossy().to_string())),
        None => native_result(Value::Null),
    }
});

// Get parent directory
// Usage: file.parent "/path/to/file.txt" -> "/path/to"
native_op!(FileParent, "path.parent", [path], {
    let path = path.expect_string()?;
    let p = Path::new(path);

    match p.parent() {
        Some(parent) => native_result(Value::String(parent.to_string_lossy().to_string())),
        None => native_result(Value::Null),
    }
});

// Join path components
// Usage: file.join "/path/to" "file.txt" -> "/path/to/file.txt"
native_op!(FileJoin, "path.join", [base, component], {
    let base = base.expect_string()?;
    let component = component.expect_string()?;

    let result = Path::new(base).join(component);
    native_result(Value::String(result.to_string_lossy().to_string()))
});

// Get absolute path
// Usage: file.absolute "./relative/path"
native_op!(FileAbsolute, "path.absolute", [path], {
    let path = path.expect_string()?;

    let abs_path = fs::canonicalize(path)
        .map_err(|e| ShikError::default_error(format!("cannot resolve path: {}", e)))?;

    native_result(Value::String(abs_path.to_string_lossy().to_string()))
});

// ============================================================================
// Symlink Functions
// ============================================================================

// Create symbolic link
// Usage: file.symlink "target" "link_path"
native_op!(FileSymlink, "file.symlink", [link_path, target], {
    let target = target.expect_string()?;
    let link_path = link_path.expect_string()?;

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link_path)
            .map_err(|e| ShikError::default_error(format!("cannot create symlink: {}", e)))?;
    }

    #[cfg(windows)]
    {
        let target_path = Path::new(target);
        if target_path.is_dir() {
            std::os::windows::fs::symlink_dir(target, link_path)
                .map_err(|e| ShikError::default_error(format!("cannot create symlink: {}", e)))?;
        } else {
            std::os::windows::fs::symlink_file(target, link_path)
                .map_err(|e| ShikError::default_error(format!("cannot create symlink: {}", e)))?;
        }
    }

    native_result(Value::Null)
});

// Read symlink target
// Usage: file.read-link "path/to/symlink"
native_op!(FileReadLink, "file.read-link", [path], {
    let path = path.expect_string()?;

    let target = fs::read_link(path)
        .map_err(|e| ShikError::default_error(format!("cannot read symlink: {}", e)))?;

    native_result(Value::String(target.to_string_lossy().to_string()))
});

// ============================================================================
// Temporary Files
// ============================================================================

// Get system temp directory
// Usage: file.temp-dir
native_op!(FileTempDir, "file.temp-dir", [], {
    let temp_dir = std::env::temp_dir();
    native_result(Value::String(temp_dir.to_string_lossy().to_string()))
});

// ============================================================================
// Helper Functions
// ============================================================================

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), RuntimeError> {
    fs::create_dir_all(dst)
        .map_err(|e| ShikError::default_error(format!("cannot create directory: {}", e)))?;

    for entry in fs::read_dir(src)
        .map_err(|e| ShikError::default_error(format!("cannot read directory: {}", e)))?
    {
        let entry =
            entry.map_err(|e| ShikError::default_error(format!("cannot read entry: {}", e)))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| ShikError::default_error(format!("cannot copy file: {}", e)))?;
        }
    }

    Ok(())
}

// ============================================================================
// Module Binding
// ============================================================================

pub fn bind_file_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Module help
    env.define_help(
        "file.".to_string(),
        "file module:

Reading:
- file.read: reads file as string
- file.read?: tries to read, returns null on failure
- file.read-bytes: reads as binary (list of bytes)
- file.read-lines: reads lines as list

Writing:
- file.write: writes string to file
- file.append: appends to file
- file.write-bytes: writes bytes to file

Operations:
- file.copy, file.cp: copies file/directory
- file.move, file.mv: moves/renames
- file.remove, file.rm: deletes file/directory
- file.rmdir: removes empty directory
- file.rmdir!: removes directory recursively
- file.mkdir: creates directory
- file.mkdir!: creates nested directories

Information:
- file.exists: checks if path exists
- file.is-dir: checks if directory
- file.is-file: checks if file
- file.is-symlink: checks if symlink
- file.size: returns file size
- file.size.deep: returns total size (recursive)
- file.stat: returns metadata object

Listing:
- file.list: lists directory (names)
- file.list!: lists with full paths
- file.glob: finds files by pattern

Symlinks:
- file.symlink: creates symlink
- file.read-link: reads symlink target

Other:
- file.temp-dir: returns temp directory"
            .to_string(),
    );

    env.define_help(
        "path.".to_string(),
        "path module:

- path.name: extracts file name
- path.stem: extracts name without extension
- path.ext: extracts extension
- path.parent: extracts parent directory
- path.join: joins path components
- path.absolute: converts to absolute path"
            .to_string(),
    );

    // Reading
    define_native!(FileRead, env, inter);
    define_help!(
        FileRead,
        env,
        "[string]: reads file contents as string\n\nfile.read \"config.txt\""
    );

    define_native!(FileTryRead, env, inter);
    define_help!(
        FileTryRead,
        env,
        "[string]: tries to read file, returns null on failure\n\nfile.read? \"maybe.txt\""
    );

    define_native!(FileReadBytes, env, inter);
    define_help!(FileReadBytes, env, "[string]: reads file as binary, returns list of numbers 0-255\n\nfile.read-bytes \"image.png\"");

    define_native!(FileLines, env, inter);
    define_help!(
        FileLines,
        env,
        "[string]: reads file lines as a list of strings\n\nfile.read-lines \"data.txt\""
    );

    // Writing
    define_native!(FileWrite, env, inter);
    define_help!(FileWrite, env, "[string string]: writes string to file (overwrites existing)\n\nfile.write \"out.txt\" \"hello\"");

    define_native!(FileAppend, env, inter);
    define_help!(
        FileAppend,
        env,
        "[string string]: appends string to file\n\nfile.append \"log.txt\" \"new line\""
    );

    define_native!(FileWriteBytes, env, inter);
    define_help!(FileWriteBytes, env, "[string list]: writes bytes (list of numbers 0-255) to file\n\nfile.write-bytes \"out.bin\" [72 101 108 108 111]");

    // File/Directory operations
    define_native!(FileCopy, env, inter);
    define_help!(FileCopy, env, "[string string]: copies file or directory (bash-like: if dest is a directory, copies into it)\n\nfile.copy \"dest.txt\" \"src.txt\"\nfile.copy \"./dir\" \"file.txt\"  ; copies file.txt into ./dir/");

    define_native!(FileMove, env, inter);
    define_help!(FileMove, env, "[string string]: moves/renames file or directory (bash-like: if dest is a directory, moves into it)\n\nfile.move \"new.txt\" \"old.txt\"\nfile.move \"./dir\" \"file.txt\"  ; moves file.txt into ./dir/");

    define_native!(FileRm, env, inter);
    define_help!(
        FileRm,
        env,
        "[string]: deletes file or directory (recursively)\n\nfile.remove \"unwanted.txt\""
    );

    define_native!(FileRmdir, env, inter);
    define_help!(
        FileRmdir,
        env,
        "[string]: removes empty directory\n\nfile.rmdir \"empty-dir\""
    );

    define_native!(FileRmdirAll, env, inter);
    define_help!(
        FileRmdirAll,
        env,
        "[string]: removes directory recursively\n\nfile.rmdir! \"dir-with-contents\""
    );

    define_native!(FileMkdir, env, inter);
    define_help!(
        FileMkdir,
        env,
        "[string]: creates directory\n\nfile.mkdir \"new-dir\""
    );

    define_native!(FileMkdirAll, env, inter);
    define_help!(FileMkdirAll, env, "[string]: creates directory and all parent directories\n\nfile.mkdir! \"path/to/nested/dir\"");

    // File information
    define_native!(FileExists, env, inter);
    define_help!(
        FileExists,
        env,
        "[string]: checks if path exists\n\nfile.exists \"config.txt\""
    );

    define_native!(FileIsDir, env, inter);
    define_help!(
        FileIsDir,
        env,
        "[string]: checks if path is a directory\n\nfile.is-dir \"src\""
    );

    define_native!(FileIsFile, env, inter);
    define_help!(
        FileIsFile,
        env,
        "[string]: checks if path is a file\n\nfile.is-file \"main.rs\""
    );

    define_native!(FileIsSymlink, env, inter);
    define_help!(
        FileIsSymlink,
        env,
        "[string]: checks if path is a symlink\n\nfile.is-symlink \"link\""
    );

    define_native!(FileSize, env, inter);
    define_help!(
        FileSize,
        env,
        "[string]: returns file size in bytes\n\nfile.size \"data.bin\""
    );

    define_native!(FileSizeDeep, env, inter);
    define_help!(FileSizeDeep, env, "[string]: returns total size of file or directory (recursive) in bytes\n\nfile.size.deep \"project\"");

    define_native!(FileStat, env, inter);
    define_help!(FileStat, env, "[string]: returns file metadata as object with size, is_file, is_dir, is_symlink, readonly\n\nfile.stat \"file.txt\"");

    // Directory listing
    define_native!(FileList, env, inter);
    define_help!(
        FileList,
        env,
        "[string]: lists directory contents (names only)\n\nfile.list \".\""
    );

    define_native!(FileListPaths, env, inter);
    define_help!(
        FileListPaths,
        env,
        "[string]: lists directory contents with full paths\n\nfile.list! \"src\""
    );

    define_native!(FileGlob, env, inter);
    define_help!(FileGlob, env, "[string]: finds files matching glob pattern\n\nfile.glob \"*.txt\"\nfile.glob \"src/**/*.rs\"");

    // Path manipulation
    define_native!(FileName, env, inter);
    define_help!(
        FileName,
        env,
        "[string]: extracts file name from path\n\npath.name \"/path/to/file.txt\"  ; \"file.txt\""
    );

    define_native!(FileStem, env, inter);
    define_help!(FileStem, env, "[string]: extracts file name without extension\n\npath.stem \"/path/to/file.txt\"  ; \"file\"");

    define_native!(FileExt, env, inter);
    define_help!(
        FileExt,
        env,
        "[string]: extracts file extension\n\npath.ext \"file.txt\"  ; \"txt\""
    );

    define_native!(FileParent, env, inter);
    define_help!(
        FileParent,
        env,
        "[string]: extracts parent directory\n\npath.parent \"/path/to/file.txt\"  ; \"/path/to\""
    );

    define_native!(FileJoin, env, inter);
    define_help!(FileJoin, env, "[string string]: joins path components\n\npath.join \"/path/to\" \"file.txt\"  ; \"/path/to/file.txt\"");

    define_native!(FileAbsolute, env, inter);
    define_help!(
        FileAbsolute,
        env,
        "[string]: converts to absolute path\n\npath.absolute \"./relative\""
    );

    // Symlinks
    define_native!(FileSymlink, env, inter);
    define_help!(
        FileSymlink,
        env,
        "[string string]: creates symbolic link\n\nfile.symlink \"link\" \"target\""
    );

    define_native!(FileReadLink, env, inter);
    define_help!(
        FileReadLink,
        env,
        "[string]: reads symlink target\n\nfile.read-link \"link\""
    );

    // Temp
    define_native!(FileTempDir, env, inter);
    define_help!(
        FileTempDir,
        env,
        "[]: returns system temp directory path\n\nfile.temp-dir"
    );
}
