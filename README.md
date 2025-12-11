# Shik Language

## Overview
Shik is a functional, dynamically-typed scripting language designed for shell automation with a minimalist syntax designed to be easily written in the terminal.

## Installation

### Cargo

```bash
# Requires Rust toolchain (https://rustup.rs/)
cargo install shik
```

### From Pre-built Binaries
Download the appropriate binary for your platform from the [Releases](https://github.com/pungy/shik/releases) page.

### Build from Source
```bash
git clone https://github.com/pungy/shik
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

### Make file with content inside

```shik
file.write :sample.txt "some text"
```

### Read file, make content upper case, write back

```shik
file.read :sample.txt $> string.upper $> file.write :sample.txt
print (file.read :sample.txt) ;; SOME TEXT HERE
```

### Make curried writer and reader

```shik
let file.reader (fn [name] (fn [] file.read name))

let write (file.write :sample.txt)
let read (file.reader :sample.txt)

write :hello
call read ;; (zero args function must be called via `call` fn) "hello"

call read $> string.upper $> write $> call read ;; HELLO
```

### Count of lines in all *.rs files in src

```shik
file.glob :./src/**/*.rs $>
  list.map file.read $>
  list.map (fn [c] string.lines c $> list.len) $>
  list.sum $>
  print

```

## Building for Distribution

See [DISTRIBUTION.md](DISTRIBUTION.md) for detailed instructions on building release binaries for multiple platforms.

## License

MIT

