# Shik Language

## Overview
Shik is a functional, dynamically-typed scripting language designed for shell automation with a minimalist syntax designed to be easily written in the terminal.

## Installation

### From Source (Recommended)
```bash
# Requires Rust toolchain (https://rustup.rs/)
cargo install --git https://github.com/yourusername/shik
```

### From Pre-built Binaries
Download the appropriate binary for your platform from the [Releases](https://github.com/yourusername/shik/releases) page.

### Build from Source
```bash
git clone https://github.com/yourusername/shik
cd shik
cargo build --release
# Binary will be at target/release/shik
```

## Usage

```bash
# Run a script file
shik script.shk

# Start REPL (interactive mode)
shik
```

## Language Features
- Pipeline operator (`$>`) for function composition
- First-class functions and lambdas
- Pattern matching capabilities
- Rich standard library for working with system
- Lazy evaluation where appropriate

## Example
```shik
;; get all files in lang-dev with ext name .dk
;; Any string without whitespaces could be written with `:` symbol in the start
glob :../lang-dev/**/*.dk $>
  ;; Redirect it to function list mapping function.
  ;; Replace all :src/file.dk with ["src/file.dk", "src/file.ts"].
  map (fn [f] [f, ext.replace f :ts]) $>
  ;; Iterate each entry, and for each file copy if the file exists to "./"
  iterate (iterate (fn [f] copy.if-exists f :./))
```

## Building for Distribution

See [DISTRIBUTION.md](DISTRIBUTION.md) for detailed instructions on building release binaries for multiple platforms.

## License

MIT

