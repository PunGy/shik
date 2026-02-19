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

# Read help
shik --help
```

## Language Features

- Clear application rules with whitespace
- First-class functions
- Operators for arguments piping, function compositions and more
- Pattern matching capabilities (in progress)
- Rich standard library for working with system
- Builtin documentation with `help` command
- Nice REPL

## Example

### Read documentation

```
> help
-- Type modules
- number.: arithmetic, rounding, comparison, math functions, random
- string.: string manipulation, conversion, iteration
- list.: list operations, higher-order functions
...

> help list.
list module:

- list.set: sets element at index (mutates list)
- list.push, list.push>, list.push-right: appends value to end (mutates list)
- list.<push, list.push-left: prepends value to start (mutates list)
- list.at: gets element at index
- list.len: returns length
...

> help list.push
native-lambda: list.push, list.push>, list.push-right
[list value]: appends value to end of list (mutates list)

list.push mylist 42
```

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
let file.reader fn [name] (fn [] file.read name)

let write (file.write :sample.txt)
let read (file.reader :sample.txt)

write :hello
read ;; "hello"

read $> string.upper $> write $> read ;; HELLO
```

### Count of lines in all *.rs files in src

```shik
file.glob :./src/**/*.rs $>
  list.map file.read $>
  list.map (fn [c] string.lines c $> list.len) $>
  list.sum $>
  print

```

### String interpolation

```shik
let greet (fn [name] "Hello, {string.upper name}!")

print $ greet :max ;; Hello, MAX!
```

## Application operators

### Pipe with `$>`

Piping - left-to-right application:

```
(f a b) == (b $> f a)
```

Example:

```shik
let files (file.list "./") ;; [ "a.txt"  "b.txt" ]
list.map (fn [path] file.size path) (files) ;; [ 5012 3024 ]

;; Same with piping

file.list "./" $> let files
files $> list.map (fn [path] file.size path)

;; Same but one line and minimalistic strings and without new function

file.list :./ $> list.map file.size
```

`$>` operator can also continue application on the next line (must be at the end of the line):

```shik
file.glob :./**/*.txt $> list.map file.size $> list.sum

;; Same as

file.glob :./**/*.txt $>
  list.map file.size $>
  list.sum
```

### Less priority apply with `$`

`$` is the same right-to-left application as usual, but with lesser priority, which allows to avoid grouping functions with parantesis in some cases.

```
(f (a b)) == (f $ a b)
```

```shik
let files (file.list :./)
print (list.map string.upper files)

;; Same with $

let files $ file.list :./
print $ list.map string.upper files
```

```shik
let lst [10 20 30 40]

list.map (+ "number: ") lst ;; ["number: 10" "number: 20" ...]

;; Same with $

list.map $ + "number: " $ lst
```

It is also allow you to extend the function application to the next line:

```shik
if (= shell.cwd :/) $
    print "You are on the root!" $
    print "nah"
```

## Composition operator

TBD

```shik
let inc $ + 1

let inc3 (inc #> inc #> inc)

print $ inc  0 ;; 1
print $ inc3 0 ;; 3


let read-lines (file.read #> string.lines)
;; same for
; let read-lines (fn [path] file.read path $> string.lines)

read-lines :.gitignore ;; [ :target :docs :releases ]
```

## Prescedence

From lowest to highest:

- `$>` operator: lowest prescedence
- `$` operator
- ` `: function application via whitespace
- `#>`: function composition, highest prescedence

### Special notes

Declaration of the labmda with `fn` is not a usual function, and it is always tighten together and treated as a single value, so, there is no need to wrap `fn [] ...` with parantesis:

```shik
let say-hello fn [name] "hello my neighbour {name}!"

[1 2 3] $>
    list.iterate fn [n] print "number: {n}"
```

## Function arguments position rule

Argument position is always a controversary topic. In `shik`, argument position plays crucial role, since everything is a function, and everything automatically curried.

The ultimate goal of `shik` is to write minimal amount of code. So, the agrument position designed to utilize currying at a maximum. In order to achieve it, the following rules applied:

### Mutation: into the PLACE put SOMETHING

When `mutation` is applied, first comes the destination of the mutation, and next is the payload. In case if `place` have a parts (`index` in `list`). The argument sequence is:

```
PLACE: from MOST specific, to LEAST specific

;; SET: INDEX , LIST , VALUE
list.set 0 lst 10
```

Examples:

```shik
;; LIST

let lst [ 0 1 2 3 ]

list.push lst 4
list.set 0 lst -1

;; FILES

let dir :./copy-dest

; PLACE , CONTENT
file.copy dir :local-file.txt
file.write :local-file.txt "new content"

;;;; why?

let files (file.glob ./src/**.ts)

files $> list.iterate (file.copy dir) ;; copy each file from files to `dir`
```

### Numeric operations: apply MUTATOR to the BASE

The most unintuitive and controversal decicion, but tho I made it: for all non-associative operations (`-`, `/`, `%`, etc), the first goes the `mutation` part, and then the `base`:

```shik
print $ - 1 5  ; 4

print $ / 2 10 ; 5

print $ ^ 3 5  ; 125
```

The reason is again the ease of use with currying: **associative** and **non-associative** must be written in the **same way** with currying.

```shik
let lst [ 1 2 3 4 ]

lst $> list.map $ + 1 ; [ 2 3 4  5 ]
lst $> list.map $ - 1 ; [ 0 1 2  3 ]
lst $> list.map $ ^ 2 ; [ 1 4 9 16 ]
lst $> list.map $ * 2 ; [ 2 4 6  8 ]
```

### Read value: read HOW from WHERE

When we want to read something, we use an opposite logic from the mutation: first come is `HOW` we want to read, then from `WHERE` we want to read it:

```shik
let lst [ 1 2 3 4 ]

list.at 0 lst

;; HOW to iterate LST
list.iterate print lst

string.has :a :bbaa

;; Although it might be correct to suppose the `map` should be in a `mutate` field of rules, since it generates something from something, the primary here is PEEKING the content, and only then the application
list.map (+ 1) lst
```


## Building for Distribution

See [DISTRIBUTION.md](DISTRIBUTION.md) for detailed instructions on building release binaries for multiple platforms.

## License

MIT

