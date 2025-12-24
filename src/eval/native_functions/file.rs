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
// Usage: file.copy "source" "destination"
native_op!(FileCopy, ["file.copy", "file.cp"], [src, dst], {
    let src = src.expect_string()?;
    let dst = dst.expect_string()?;

    let src_path = Path::new(src);
    if src_path.is_dir() {
        copy_dir_recursive(src_path, Path::new(dst))?;
    } else {
        fs::copy(src, dst)
            .map_err(|e| ShikError::default_error(format!("cannot copy file: {}", e)))?;
    }

    native_result(Value::Null)
});

// Move/rename file or directory
// Usage: file.move "source" "destination"
native_op!(FileMove, ["file.move", "file.mv"], [src, dst], {
    let src = src.expect_string()?;
    let dst = dst.expect_string()?;

    fs::rename(src, dst)
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
native_op!(FileSymlink, "file.symlink", [target, link_path], {
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
    // Reading
    define_native!(FileRead, env, inter);
    define_native!(FileTryRead, env, inter);
    define_native!(FileReadBytes, env, inter);
    define_native!(FileLines, env, inter);

    // Writing
    define_native!(FileWrite, env, inter);
    define_native!(FileAppend, env, inter);
    define_native!(FileWriteBytes, env, inter);

    // File/Directory operations
    define_native!(FileCopy, env, inter);
    define_native!(FileMove, env, inter);
    define_native!(FileRm, env, inter);
    define_native!(FileRmdir, env, inter);
    define_native!(FileRmdirAll, env, inter);
    define_native!(FileMkdir, env, inter);
    define_native!(FileMkdirAll, env, inter);

    // File information
    define_native!(FileExists, env, inter);
    define_native!(FileIsDir, env, inter);
    define_native!(FileIsFile, env, inter);
    define_native!(FileIsSymlink, env, inter);
    define_native!(FileSize, env, inter);
    define_native!(FileSizeDeep, env, inter);
    define_native!(FileStat, env, inter);

    // Directory listing
    define_native!(FileList, env, inter);
    define_native!(FileListPaths, env, inter);
    define_native!(FileGlob, env, inter);

    // Path manipulation
    define_native!(FileName, env, inter);
    define_native!(FileStem, env, inter);
    define_native!(FileExt, env, inter);
    define_native!(FileParent, env, inter);
    define_native!(FileJoin, env, inter);
    define_native!(FileAbsolute, env, inter);

    // Symlinks
    define_native!(FileSymlink, env, inter);
    define_native!(FileReadLink, env, inter);

    // Temp
    define_native!(FileTempDir, env, inter);
}
