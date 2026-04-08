# Shik

A functional, dynamically-typed scripting language for shell automation — with a minimalist syntax designed to be written left-to-right in the terminal.

```shik
file.glob :./src/**/*.rs $>
  list.map (file.read #> string.lines #> list.len) $>
  list.sum $>
  print
```

Shik draws from **Lisp** (everything is function application) and **Haskell** (whitespace application, automatic currying), adapted for terminal ergonomics. No arithmetic operators, no special syntax — just functions, four application operators, and consistent rules.

## Installation

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/pungy/shik/releases/latest/download/shik-installer.sh | sh

# Windows (PowerShell)
powershell -ExecutionPolicy ByPass -c "irm https://github.com/pungy/shik/releases/latest/download/shik-installer.ps1 | iex"

# From crates.io (requires Rust toolchain)
cargo install shik

# Or build from source
git clone https://github.com/pungy/shik
cd shik && cargo build --release
```

Pre-built binaries for all platforms available on the [Releases](https://github.com/pungy/shik/releases) page.

## Usage

```bash
shik              # Start REPL
shik script.shk   # Run a script file
```

### Built-in documentation

```
> help
-- Type modules
- number.: arithmetic, rounding, comparison, math functions, random
- string.: string manipulation, conversion, iteration
- list.: list operations, higher-order functions
- object.: object operations, iteration
- file.: file system operations
- shell.: shell commands, environment
...

> help list.map
native-lambda: list.map
[function list]: applies function to each element, returns new list
```

## Core Concepts

### Everything is function application

There are no operators in the traditional sense. `+ 1 2` is a call to function `+` with arguments `1` and `2`. `list.map`, `file.glob`, `string.upper` — these are all function names. The dot is part of the name, not a module accessor.

### Space is application

```shik
file.glob :./src/**/*.rs       ; apply :./src/**/*.rs to file.glob
+ 1 2                          ; apply 1 and 2 to +
list.at 0 lst                  ; apply 0 and lst to list.at
```

Like `f(x)`, but without parentheses. Multiple arguments are separated by spaces.

### Automatic currying

Every function supports partial application. Pass fewer arguments than expected — get a new function waiting for the rest:

```shik
let inc (+ 1)                  ; function: add 1
let write (file.write :out.txt) ; function: write to out.txt

[1 2 3] $> list.map inc        ; [2 3 4]
write "hello"                  ; writes "hello" to out.txt
```

### Four operators

From lowest to highest precedence:

| Operator | Name | Description | Example |
|----------|------|-------------|---------|
| `$>` | Pipe | Left-to-right application | `x a $> f a` = `(f a) (x a)` |
| `$` | Chain | Lower-precedence application | `f $ g x` = `f (g x)` |
| `#>` | Flow | Function composition | `f #> g` = `fn [x] g (f x)` |
| ` ` | Space | Standard application | `f x` |

Operators, especially `$` and `$>`, can be used in order to move application chain to the next line.

## Syntax

### Literals

```shik
42                     ; number (f64)
"hello world"          ; string
:hello                 ; inline string (no spaces, no quotes needed)
[1 2 3]                ; list
{:name :Alice :age 30} ; object
true                   ; boolean
```

`:symbol` is shorthand for `"symbol"` — faster to type in the terminal.

### String interpolation

```shik
let name :Alice
print "Hello, {name}!"                ; Hello, Alice!
print "Sum: {+ 10 20}"                ; Sum: 30
```

### Variables

```shik
let x 10                ; bind
set x (+ x 5)           ; mutate
let$ [a b c] [1 2 3]    ; destructure with let$
```

### Functions

```shik
let greet fn [name] "Hello, {name}!"

; Multi-expression body with '()
let fac fn [x] '(
  if (< x 2) $
    1 $
    (* x (fac (- 1 x)))
)
```

### Pattern matching

Matching was tried to be made as simple and straightforward as possible.

The `match` function has two arguments: `value` - what we would try to match, and the object.

```shik
let value [1 2 3 4]
let variable 500

match value {
  :literal    :exact-match
  variable    "exactly 500"
  [x y #rest] "destructure list: head({x}), tail({rest})"
  _           :wildcard
} $> print
```

You can match against:

- `literal`: some literal value;
- `literal under variable`: if you provide variable, it would try to match against the literal value the variable is hold;
- `list pattern`: same rules as in destructuring;
- `object pattern`: same rules as in destructuring;
- `wildcard`: fallthrough case, if any other doesn't matched
- `#otherwise`: same as wildcard, but put the value into the `otherwise` variable;

### Control flow

`if` is a special function with dynamic number of arguments.

The first argument is a condition, and all following is a branches.

Here is a rules for `if` arguments:

- **one argument**: error;
- **two arguments**: `first` - condition, `second` - executed if condition is true;
- **three arguments**: `first` - condition, `second` - executed if condition is true, `third` - executed if condition is false;

Following rules are similar for any number of arguments (for arguments count > than 1):

- **if even count of arguments:**:
    - **every odd argument**: is a condition;
    - **every even argument**: executed if previous condition is true;
- **if odd count of arguments:**:
    - **every odd argument**: is a condition;
    - **every even argument**: executed if previous condition is true;
    - **last argument**: is an `else` block, executed if all previous conditions false

```shik
let x 11
if (< x 10) (print :small) (print :big)

; of course, it can be done with matching, but just for example
let dice (number.rand 1 7)
if (< dice 3)  $ ; if you want to move application to the next line - add $ at the end
     :loose    $
   (<= dice 5) $
     :win      $
   :Jackpot!   $> print

;; While loop. Until internal function returns true - continue execution
while (fn [] '(
  set x (- 1 x)
  print "x is {x}"
  > x 0
))
```

### Shell commands

```shik
shell "git status"                     ; run command, return stdout
shell.lines "git branch"               ; return as list of lines
shell.has :docker                      ; check if command exists
shell.env :HOME                        ; environment variable
shell.os                               ; "Darwin", "Linux", etc.
```

### Polymorphic functions

Some library functions has a polymorpic nature - it can work differently with different types of values provided.

- **+**:
    - `Number + Number`: summation;
    - `String + String`: string concatenation (`string.+`);
    - `String + other`: Convert to string counterpart and make sring concatenation (`string.+ str (string other)`);
- **at**:
    - `String`: get character under the index (`string.at`)
    - `List`: get elemenet under the index (`list.at`)
- **iterate**:
    - `String`: iterate over string (`string.iterate`);
    - `List`: iterate over list (`list.iterate`);
- **iterate-backward, <iterate**:
    - `String`: iterate right-to-left over string (`string.iterate-backward`);
    - `List`: iterate right-to-left over list (`list.iterate-backward`);
- **print**: print anything to stdout;

### Non-curried functions

Some functions has dynamic number of arguments, so cannot be curried:

- `if`
- `help`
- `number.rand`
- `list.range`
- `shell.ask`

## Application Operators

### Pipe `$>`

Left-to-right application — the core of Shik's ergonomics. Result flows left to right:

```shik
file.read :data.txt $> string.lines $> list.len

; Equivalent to:
list.len (string.lines (file.read :data.txt))

; Multi-line (place $> at end of line):
file.glob :./**/*.txt $>
  list.map file.size $>
  list.sum $>
  print
```

### Chain `$`

Right-to-left application with lower precedence — eliminates parentheses:

```shik
print $ list.map string.upper $ file.list :./

; Equivalent to:
print (list.map string.upper (file.list :./))
```

Also extends expressions to the next line:

```shik
if (= shell.os :Darwin) $
  print "macOS" $
  print "other"
```

### Flow `#>`

Function composition — creates a new function from a chain:

```shik
let read-lines (file.read #> string.lines)

read-lines :.gitignore    ; ["target" "docs" "releases"]

; Useful inside list.map:
file.glob :./src/**/*.rs $>
  list.map (file.read #> string.lines #> list.len)
```

### Non-piped functions

Some functions, actually, are not really functions, but grammatical constructions in a shape of function application. For that reason, it cannot be used with pipe `$>` (and essential any other) operator. Hopefully, there is only three of them:

- `match` - pattern matching;
- `fn` - function literal;
- `let$` - that is the reason why there is two `let` functions. `let` is an ordinary function, while `let$` is a grammatical construction allowing pattern matching;

## Function Arguments Position Rule

Argument order is designed to maximize currying effectiveness. The rule: **the first argument is the one you fix** when partially applying.

### Mutation: PLACE first, then CONTENT

```shik
file.write :out.txt "content"         ; place, then what
file.copy :dest :source               ; where to, then from
list.set 0 lst 10                     ; index, list, value

; Why: enables curried patterns like
files $> list.iterate (file.copy :backup)
```

### Arithmetic: MODIFIER first, then BASE

```shik
- 1 5    ; 4   (subtract 1 from 5)
/ 2 10   ; 5   (divide 10 by 2)
^ 3 5    ; 125 (raise 5 to power 3)
```

This is unconventional, but it makes currying uniform:

```shik
lst $> list.map (+ 1)    ; add 1 to each
lst $> list.map (- 1)    ; subtract 1 from each
lst $> list.map (* 2)    ; multiply each by 2
lst $> list.map (^ 2)    ; square each
```

All four lines follow the same pattern. If `-` worked as "first minus second", `(- 1)` would mean "1 minus something", breaking the symmetry.

### Read: HOW first, then WHERE

```shik
list.at 0 lst                         ; what index, from what list
list.map (+ 1) lst                    ; what function, on what list
string.has :a :banana                 ; what to find, in what string
```

## Standard Library

All modules are available without imports.

| Module | Description | Key functions |
|--------|-------------|---------------|
| `number.` | Arithmetic & math | `abs`, `ceil`, `floor`, `round`, `sqrt`, `sin`, `cos`, `rand` |
| `string.` | String manipulation | `upper`, `lower`, `trim`, `split`, `join`, `has`, `replace`, `lines`, `len`, `iterate` |
| `list.` | List operations | `map`, `filter`, `fold`, `sort`, `head`, `tail`, `push`, `at`, `len`, `range`, `take`, `drop`, `find` |
| `object.` | Key-value maps | `get`, `set`, `has`, `keys`, `values`, `merge`, `map`, `filter`, `fold`, `pick`, `omit` |
| `file.` | Filesystem | `read`, `write`, `glob`, `copy`, `move`, `remove`, `size`, `exists`, `list`, `mkdir`, `stat` |
| `shell.` | System & processes | `shell`, `lines`, `os`, `arch`, `cwd`, `home`, `env`, `has`, `which`, `ask`, `cd` |
| `path.` | Path manipulation | `name`, `stem`, `ext`, `parent`, `join`, `absolute` |
| `fn.` | Function utilities | `id`, `invoke` |
| `var.` | Variable access | `get` (access variables by string at runtime) |

Use `help <module>.` in the REPL to explore.

## REPL

The REPL includes:

- **Syntax highlighting** (via tree-sitter)
- **History** (persisted to `~/.shik_history`)
- **Multi-line input** (Shift+Enter or place `$>` / `$` at end of line)
- **Built-in help** (`help`, `help module.`, `help function`)

## Examples

See the [`demo/`](demo/) directory for runnable examples:

```bash
shik demo/line-count.shk      # Count lines in source files
shik demo/match.shk           # Pattern matching showcase
shik demo/files-new-demo.shk  # File operations pipeline
shik demo/dice-game.shk       # Codewars kata solution
```

## Status

Shik is in active development (v0.7). Usable for real tasks, but expect rough edges.

**Planned, from highest to lowest priority:**
- Shebang support (`#!/usr/bin/env shik`);
- Regular expressions;
- Multiple statements per line with `,`;
- Networking;
- Lambda shorthand: `#(- #1 #2)` instead of `fn [a b] - a b`;
- JSON parsing;
- User-facing error handling (`try`/`catch` or similar);
- Threading;

## License

MIT
