# Shik

**A functional, dynamically-typed scripting language for shell automation — with a minimalist syntax designed to be written left-to-right in the terminal.**

[![version](https://img.shields.io/badge/version-0.7.1-blue)](https://github.com/pungy/shik/releases) [![license](https://img.shields.io/badge/license-MIT-green)](LICENSE) [![platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)](https://github.com/pungy/shik/releases)

Shik is a scripting language built around one idea: the thought in your head and the code you type should be the same shape. Data flows left to right through function pipelines. Everything is function application — no operators, no special syntax, no imports. A full standard library for files, strings, lists, objects, and shell commands is available from the first line.

Shik is for people who write small automation scripts every few days — moving files, counting things, pulling shell output into structured data — and who are tired of fighting the tools instead of solving the problem. Read the [origin story](https://blog.pungy.me/articles/shik).

![Demo](https://raw.githubusercontent.com/pungy/shik/main/shik-demo.gif)

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Language Reference](#language-reference)
  - [Literals](#41-literals)
  - [Core Concepts](#42-core-concepts)
  - [Functions](#43-functions)
  - [Variables & Scope](#44-variables--scope)
  - [Control Flow](#45-control-flow)
  - [Special Functions](#46-special-functions)
  - [Built-in Modules](#47-built-in-modules)
  - [Error Handling](#48-error-handling)
  - [Patterns & Idioms](#49-patterns--idioms)
- [Examples](#5-examples)
- [Performance](#6-performance)
- [Roadmap](#7-roadmap)
- [Contributing](#8-contributing)

---

## Installation

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/pungy/shik/releases/latest/download/shik-installer.sh | sh

# Windows (PowerShell)
powershell -ExecutionPolicy ByPass -c "irm https://github.com/pungy/shik/releases/latest/download/shik-installer.ps1 | iex"

# From crates.io (requires Rust toolchain)
cargo install shik

# From source
git clone https://github.com/pungy/shik
cd shik && cargo build --release
```

Pre-built binaries for all platforms are on the [Releases](https://github.com/pungy/shik/releases) page.

Verify the installation:

```bash
shik --version
```

---

## Quick Start

### REPL

```bash
shik
```

```
> + 1 2
3

> let greet fn [name] "Hello, {name}!"
> greet :world
Hello, world!

> file.glob :./src/**/*.rs $> list.map (file.read #> string.lines #> list.len) $> list.sum $> print
9823
```

### Running a script

```bash
shik script.shk
```

### Built-in help

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

> help list.
-- list. module
- list.map
- list.filter
- list.fold
...
```

---

## Language Reference

### 4.1 Literals

#### Numbers

Numbers are 64-bit floats (`f64`). Integer syntax is supported but all values are floats internally.

```shik
42
3.14
-17
0
```

#### Strings

Strings are UTF-8, delimited by double quotes.

```shik
"hello world"
"line one\nline two"
"tab\there"
"quote: \""
```

**String interpolation:** any expression inside `{...}` is evaluated and embedded into the string.

```shik
let name :Alice
"Hello, {name}!"         ; Hello, Alice!
"Sum: {+ 10 20}"         ; Sum: 30
"Path: {path.join :src file}"
```

**String shorthand**:  A `:word` is shorthand for the string `"word"`. It is a regular string, just another syntax for string declaration, usefull when type strings without spaces (which is, in practice, most of strings).

```shik
:hello            ; same as "hello"
:./src/**/*.rs    ; same as "./src/**/*.rs"
:Darwin           ; same as "Darwin"
```

Rule of thumb: if the string has no spaces and you don't need interpolation, use `:`.

#### Booleans

```shik
true
false
```

#### Lists

Ordered sequences, delimited by `[` and `]`, whitespace-separated. Elements can be any type. Lists may span multiple lines.

```shik
[]
[1 2 3]
[:a :b :c]
[[1 2] [3 4]]         ; nested
[1 "two" true null]   ; mixed types

; Multi-line
[
  1
  2
  3
]
```

#### Objects

Key-value maps, delimited by `{` and `}`. It might be strange looking at first - **no separators**? The idea is (not mine, that's a lisp thing, just briliant) - keys are strings, and key is an each **odd** item in the object. Values can be any type, the value is an each **even** item in the object. Items count must be **even**, so each key is associated with the next **even** item (value). Objects may span multiple lines.

```shik
{}
{:name :Alice :age 30}
{:nested {:x 1 :y 2}}

; Multi-line
{
  :name :Alice
  :age 30
}
```

#### Null

```shik
null
```

`null` is the absence of a value. Some functions return `null` to signal "not found" or "failed."

#### Functions

`fn [args] body` is a literal that produces a function value. It is a complete syntax construction — not an application of `[args]` to `fn`, so the application operators are no use here. The argument list is part of the syntax, not a list literal being passed as a value.

```shik
fn [x] + x 1
fn [x y] * x y
fn [] :constant       ; zero arguments
```

---

### 4.2 Core Concepts

#### Everything is function application

There are no operators in the traditional sense. `+ 1 2` is calling the function `+` with arguments `1` and `2`. `list.map`, `file.glob`, `string.upper`, `if`, `let`, `while` — all functions. One rule covers everything.

The dot in `list.map` is part of the name, not a module accessor. `list.map` is one identifier.

#### Space as application

```shik
file.glob :./src/**/*.rs    ; apply :./src/**/*.rs to file.glob
+ 1 2                        ; apply 1 and 2 to +
list.at 0 lst                ; apply 0 and lst to list.at
```

Like `f (x)`, without the parentheses.

#### Prefix notation

All functions use prefix notation: function name first, then arguments.

```shik
+ 1 2          ; 3
> 5 3          ; true  (is 3 greater than 5? No — see argument order section)
not true       ; false
string.len "hello"    ; 5
```

#### The four operators

All four operators are about function application. They differ in direction and precedence.

| Operator | Name | Direction | Precedence |
|----------|------|-----------|------------|
| ` ` | Space / Apply | left→right application | highest |
| `#>` | Flow / Compose | left→right composition | — |
| `$` | Chain | right-to-left | lower |
| `$>` | Pipe | left-to-right | lowest |

**Space — function application:**

```shik
f x           ; f(x)
f x y         ; f(x, y)
list.map (+ 1) [1 2 3]
```

**`#>` — function composition:**

Creates a new function that applies the left function, then the right:

```shik
file.read #> string.lines          ; fn [x] string.lines (file.read x)
file.read #> string.lines #> list.len
let read-lines (file.read #> string.lines)
read-lines :.gitignore             ; ["target" "docs" "releases"]
```

**`$` — chain / right-to-left application:**

Like Haskell's `$`. Eliminates parentheses for right-to-left composition. Right-associative.

```shik
print $ + 1 2                       ; print (+ 1 2)
print $ list.map string.upper $ file.list :./
; same as: print (list.map string.upper (file.list :./))
```

Also extends expressions to the next line:

```shik
if (= shell.os :Darwin) $
  print "macOS" $
  print "other"
```

**`$>` — pipe (lowest precedence):**

Left-to-right application. The result of the left side is passed as the **last argument** to the right side.

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

#### Operator precedence

From tightest binding to loosest: space > `#>` > `$` > `$>`

```shik
; Example: how does f g #> h parse?
; space binds tighter than #>, so: (f g) #> h
f g #> h        ; = (f g) #> h  — compose the result of (f g) with h

; Example: how does f #> g h parse?
; #> binds tighter than space here at its own level, so right side is g applied to h
f #> g h        ; = f #> (g h)  — compose f with (g applied to h)

; Example: a $> f b
a $> f b        ; b applies to f first (space), then a pipes: (f b) a

; Example: combining all four
file.glob :./src/**/*.rs $>
  list.map (file.read #> string.lines #> list.len) $>
  list.sum $>
  print
; space applies args, #> composes, $> pipes data through the chain
```

#### Move application to the next line

You can use operators to move application to the next line, as you seen before. The rule is: `$`, `$>` or `#>` **must be** on the **end of the line**.

```shik
let x 10

;; Wrong: 5 unrelated applications, if requires at least 2 arguments
if (> x 10)
    (print "x is big")
   (= x 10)
    (print "x is ten")
    (print "x is small")

;; Error: syntax error, $ must have left counterpart
if (> x 10)
   $ (print "x is big)
   $ (= x 10)
   $  (print "x is ten")
   $  (print "x is small")



```

```shik
;; Correct
if (> x 10) $
    (print "x is big") $
    (= x 10) $
     (print "x is ten") $
     (print "x is small")

;; Correct: and beautiful (but not diff friendly)
if (> x 10)               $
    (print "x is big")    $
    (= x 10)              $
     (print "x is ten")   $
     (print "x is small")
```

#### Parentheses — grouping only

Parentheses group expressions. They are not function-call syntax.

```shik
(+ 1 2)            ; the value 3
(file.read #> string.lines)    ; a composed function
list.map (+ 1) [1 2 3]         ; (+ 1) is a partial application
```

#### Magic symbols — one place to understand them all

There is a actually a two types of magic symbols: high-born, low-born.

**High-born**:

High-born ones cannot be used in names of variables. A true magic beasts. Right now, there is the only one of such: `#`.

**Low-born**:

They are tricker. They might appear in the names and non-magic occasions. It is a: `$`, `_` and `'`. Poor `>` and `<` just hang around and have nothing to do with magic, they just helping out sometimes.

| Prefix | Where | Meaning |
|--------|-------|---------|
| `'(` | Anywhere | Multi-expression block |
| `#>` | Between functions | Composition operator |
| `#rest` | Inside `[...]` pattern | Captures remaining list elements |
| `#name` | Inside `match` block | Captures matched value into `name` |
| `_` | Inside `match` block or pattern | The wildcard, a fall-all option |
| `$>` | Between expressions | Pipe operator |
| `$` | Between expressions | Chain operator |

---

### 4.3 Functions

Functions are first-class values. The `fn` literal (see [Literals → Functions](#41-literals)) produces a function; `let` binds it to a name.

```shik
let greet fn [name] "Hello, {name}!"
greet :Alice      ; Hello, Alice!

let add fn [x y] + x y
add 3 4           ; 7
```

Arguments can be destructured directly in the argument list:

```shik
fn [[x y]] + x y        ; expects a two-element list, binds x and y
fn [[head #rest]] head  ; expects a list, binds head and rest

; Common in higher-order functions:
list.iterate (fn [[key val]] print "{key}: {val}") (object.entries obj)
list.sort (fn [[_ a] [_ b]] - a b) pairs
```

The same `#rest` pattern used in `match` works here: `[x #rest]` binds the first element to `x` and the remainder to `rest`.

#### Multi-expression body with `'(...)`

When a function body needs multiple statements, wrap them in `'(...)`. The `'` prefix distinguishes a block from a parenthesized expression. The last expression is the return value.

```shik
let fac fn [x] '(
  if (< x 2) $
    1 $
    (* x (fac (- 1 x)))
)

let counter fn [base] '(
  let x base
  let bump fn [much] set x (+ much x)
  bump             ; returned: a function that mutates x
)
```

`'(...)` can also be used standalone, outside a function, to sequence expressions.

#### Currying

Every function supports partial application. Provide fewer arguments than expected — get a new function waiting for the rest.

```shik
let add-one (+ 1)            ; function: add 1 to its argument
add-one 5                    ; 6

let write-to (file.write :out.txt)   ; function: write to out.txt
write-to "hello"             ; writes "hello" to out.txt

let inc (+ 1)
[1 2 3] $> list.map inc      ; [2 3 4]

let has-todo (string.has :TODO)
file.glob :./src/**/*.rs $> list.filter (file.read #> has-todo)

let subtract-one (- 1)       ; function: subtract 1 from argument
[10 20 30] $> list.map subtract-one   ; [9 19 29]
```

Currying makes `#>` and `$>` pipelines possible without lambdas in most cases.

Functions that cannot be curried (variadic / special forms): `if`, `help`, `number.rand`, `list.range`, `shell.ask`.

#### Argument order philosophy

Argument order is designed to maximize the usefulness of partial application. The convention: **fix what you know, receive what varies last.**

**Arithmetic**: _MODIFIER_ first, _BASE_ last.

`- 1 5` = `4` (subtract 1 from 5). `/ 2 10` = `5` (divide 10 by 2). `^ 3 5` = `125` (raise 5 to power 3).

This is unconventional but makes currying uniform:

```shik
lst $> list.map (+ 1)    ; add 1 to each
lst $> list.map (- 1)    ; subtract 1 from each
lst $> list.map (* 2)    ; multiply each by 2
lst $> list.map (^ 2)    ; square each
```

All four lines follow the same pattern. If `-` worked as "first minus second," `(- 1)` would mean "1 minus something," breaking the symmetry. You would need `fn [x] - x 1` instead.

**Read**: _SPECIFIER_ first, then _TARGET_.

```shik
list.at 0 lst            ; which index, from which list
list.map (+ 1) lst       ; which function, on which list
string.has :a :banana    ; what to find, in what string
object.get :name obj     ; which key, from which object
```

**Write/mutation**: _DESTINATION_ first, then _CONTENT_.

```shik
file.write :out.txt "content"    ; where, then what
file.copy :dest :source          ; where to, then from
list.set 0 lst 10                ; index, list, value
object.set :key obj val          ; key, object, value
```

This allows:

```shik
files $> list.iterate (file.copy :backup/)   ; copy each file to backup/
```

#### Composition with `#>`

`f #> g` creates `fn [x] g (f x)`.

```shik
let read-lines (file.read #> string.lines)
let count-lines (file.read #> string.lines #> list.len)

; Point-free predicate
let has-todo (file.read #> string.has :TODO)
file.glob :./src/**/*.rs $> list.filter has-todo

; In a pipeline
file.glob :./src/**/*.rs $>
  list.map (file.read #> string.lines #> list.len) $>
  list.sum $>
  print
```

---

### 4.4 Variables & Scope

#### Binding: `let`

```shik
let x 10
let name :Alice
let double fn [n] * n 2
```

`let` binds a name in the current scope. If the name already exists in the current scope, it is rebound (shadowed within that scope). Inner scopes can shadow outer ones.


#### Naming

Variable names can include any characters: letters, digits, `-`, `.`, `!`, `?`, `+`, `*`, etc. `.` is just part of the name; `!` and `?` are conventional suffixes (not syntax).

```shik
let my-function fn [x] x
let file.reader fn [name] fn [] file.read name
let empty? fn [lst] list.empty? lst
```

#### Mutation: `set`

```shik
let x 10
set x 20       ; x is now 20
set x (+ x 5) ; x is now 25
```

`set` modifies an existing binding in the nearest enclosing scope that has it. If the variable does not exist anywhere in scope, it is an error.

Compound assignment shorthands:

```shik
set+ x 5    ; x = x + 5
set- x 3    ; x = x - 3
```

#### Destructuring: `let$`

`let$` binds multiple names from a list in one step:

```shik
let$ [a b c] [1 2 3]     ; a=1, b=2, c=3
let$ [head #rest] lst    ; head=first element, rest=remaining list
let$ [x _] [10 20]       ; x=10, _ discarded

; Nested
let$ [KITTY-PATH FISH-PATH] [(make-path :kitty) (make-path :fish)]
```

The pattern syntax inside `let$` is the same as in `match` (see [Control Flow](#45-control-flow)).

#### Scope rules

Shik uses lexical scoping. Free variables in a function are resolved in the scope where the function is written. Closures hold a reference to the variable binding, so `set` mutations are visible:

```shik
let x 100
let f fn [n] + x n

f 5      ; 105
set x 10
f 5      ; 15
```

Each function call gets its own scope for locals; mutations to `let`-bound variables inside a function do not escape:

```shik
let x 0
let counter fn [] '(
  let x 99    ; new binding in this call's scope
  x
)
counter    ; 99
x          ; still 0
```

To share mutable state across calls, capture the variable from an outer scope:

```shik
let x 0
let bump fn [] set x (+ x 1)    ; mutates the outer x
bump   ; x becomes 1
bump   ; x becomes 2
x      ; 2
```

**if and '( blocks**:

Variables are not block-scoped:

```shik
if true
    '(let x 10
      print "I have x: {x}") ;; called
    '(let y 20
      print "I have y: {y}")

print x ;; 10
print y ;; Error: UndefinedVariable
```

#### Dynamic variable lookup: `var.get`

```shik
var.get "x"          ; returns the value of variable x, or null
var.get "list.map"   ; works for any name in scope
```

This allows treating variable names as data — an `eval` like behavior, but sometimes might be usefull:

---

### 4.5 Control Flow

#### `if`

`if` is a function with a dynamic number of arguments.

```shik
; Two args: condition + then-branch (no else, returns null on false)
if (< x 10) (print :small)

; Three args: condition + then + else
if (< x 10) (print :small) (print :big)

; Even number of args: pairs of (condition, branch)
if
  (< x 0)  (print :negative) $
  (< x 10) (print :small)    $
  (< x 100) (print :medium)

; Odd number of args (> 3): same but last arg is the else-branch
if (< x 0)  (print :negative) $
   (< x 10) (print :small)    $
   (print :big)
```

Combining with `$` to span multiple lines:

```shik
let dice (number.rand 1 7)
if (< dice 3)  $
     :loose    $
   (<= dice 5) $
     :win      $
   :Jackpot!   $> print
```

#### `match`

Pattern matching against a value. `match` takes a value and an object-like block of patterns and results.

```shik
match value {
  pattern1   result1
  pattern2   result2
  _          default-result
}
```

**Pattern types:**

| Pattern | Matches | Notes |
|---------|---------|-------|
| `42` | exactly the number 42 | Any literal |
| `:hello` | exactly the string "hello" | Symbol literal |
| `variable` | the current value of `variable` | Variable must be in scope |
| `_` | anything | Wildcard, no binding |
| `#name` | anything | Wildcard, binds matched value to `name` |
| `[x y]` | list with exactly 2 elements | Binds x and y |
| `[x y #rest]` | list with 2+ elements | x=head, y=second, rest=tail |
| `[]` | empty list | |

```shik
let value [1 2 3 4]
let variable 500

match value {
  :literal    :exact-match              ; matches string "literal"
  variable    "exactly 500"             ; matches value of variable (500)
  [x y #rest] "head={x}, rest={rest}"  ; matches list, binds x=1, y=2, rest=[3 4]
  _           :wildcard
} $> print
; Output: head=1, rest=[3 4]

; Wildcard with binding
match value {
  []    :empty
  #v    "got: {v}"    ; captures value in v
}

; Nested list patterns
match pairs {
  [[k v] #rest]   "first key is {k}"
  _               :empty
}
```

`match` returns the result of the matched branch. The last expression in a `'(...)` branch is the return value.

```shik
let score match grade {
  :A  100
  :B  80
  :C  60
  _   0
}
```

#### `while`

Loops while a zero-argument function returns `true`:

```shik
let x 10
while (fn [] '(
  set x (- 1 x)
  print "x is {x}"
  > x 0
))
```

The function is called on each iteration. The loop stops when it returns a falsy value.

#### Early exit

There is no `break` or `return`. Use recursion, `match`, or mutable state to control flow. For current process, use `process.abort` to terminate execution.

---

### 4.6 Special Functions

In Lisp known as special forms. Most functions in Shik are uniform: arguments are evaluated left-to-right before the call, the function receives values, currying works automatically. A small set of built-ins breaks some rules. This section documents them and the constraints that follow.

#### Non-functions

`fn`, `match`, and `let$` look like function application but are complete syntax constructions parsed by the language itself — not callable values.

```shik
fn [x] + x 1          ; lambda literal
match value { ... }   ; pattern match
let$ [a b] [1 2]      ; destructuring bind
```

**Constraints:**

- Cannot appear on the right-hand side of `$>`, `$`, or `#>`. The operators require an expression that evaluates to a callable value; these constructions are not values.
- Cannot be stored, passed, or returned — they have no runtime representation.
- Cannot be partially applied.

```shik
; All of these are errors:
[1 2] $> match { ... }
[1 2] $> let$ [a b]

let bind (let$ [a b])
```

#### Conditionally evaluated arguments

These functions receive some arguments as unevaluated expressions and decide whether to evaluate them at all. The evaluation happens inside the function, not before the call.

| Function | Lazy arguments | Condition |
|----------|---------------|-----------|
| `if` | all branches and predicates | only the matching branch and tested predicate is evaluated |
| `and` | second argument | skipped if first is falsy |
| `or` | second argument | skipped if first is truthy |
| `or?` | default (first argument) | skipped if value (second) is not null |
| `while` | body | re-evaluated on each iteration |
| `let` | name (first argument) | always treated as a symbol, never evaluated as an expression |
| `set`, `set+`, `set-` | name (first argument) | same as `let` |
| `'` and `fn.quote` | its argument | identifiers inside are not resolved |

The practical consequence: side-effecting expressions in skipped branches do not run.

```shik
; The print never executes when x > 0:
if (> x 0) :ok (print "only when x <= 0")

; The second branch is never evaluated when the first is truthy:
or true (print "never runs")
```

`let` and `set` require their first argument to be a plain identifier — it is never evaluated as an expression:

```shik
let x 10     ; x is a name, not looked up
set x 20     ; x is a name, not looked up
```

**Constraints:** No such. You can use any kind of operators and currying on them (but only to which are not in the variadic category).

#### Variadic functions

These functions accept a variable number of arguments. Because the arity is not fixed, they cannot be curried — partial application would be ambiguous.

| Function | Accepted argument counts |
|----------|--------------------------|
| `if` | 2 (condition + then), 3 (+ else), or any even/odd count for multi-branch form |
| `number.rand` | 0 (float 0–1), 1 (int 0–max), 2 (int min–max) |
| `list.range` | 1 (end), 2 (start end), 3 (start end step) |
| `shell.ask` | 0 (no prompt), 1 (prompt string) |
| `help` | 0 (overview), 1 (topic) |

**Constraints:** Cannot be curried. Passing fewer arguments than any valid form does not produce a partial function — it is an error, or undesired evaluation.

```shik
; This does not create a partially-applied if — it is an error:
let check (if (> x 0))

; Rand would be a number!
let rand number.rand
```

#### Polymorphic functions

Shik is a strong-typed language, and type missmatching would cause RuntimeError. But, there are some functions that work across multiple types:

| Function | Types | Description |
|----------|-------|-------------|
| `+` | Number+Number, String+String, String+other | Addition or concatenation, with String+other - other would be converted to string |
| `at` | (index, String) or (index, List) | Get element at index |
| `iterate` | (fn, String) or (fn, List) | Iterate over characters or elements |
| `iterate-backward` / `<iterate` | (fn, String) or (fn, List) | Iterate in reverse |
| `print` | any | Print value to stdout |

**Constraints:** No such.

---

### 4.7 Error Handling

#### Error types

**Parse errors** — caught before execution:

```
Parsing failed: Unexpected token 'internal-name' at line 1, column 14. Expected: LeftBracket
```

**Runtime errors — undefined variable:**

```shik
let x (blabla 10)
```
```
UndefinedVariable: Variable 'blabla' is not defined
  --> at line 1, column 8
```

**Runtime errors — type mismatch:**

Shik is dynamically but strictly typed. Functions reject wrong types with a runtime error.

**Runtime errors — IO:**

```shik
file.list :nonexistent/
```
```
RuntimeError: cannot read directory: No such file or directory (os error 2)
  --> at line 1, column 1
```

All errors show the line and column of the offending expression.

#### Shell command variants

`file.*` functions terminate the script on failure. Shell commands have variants for different error-handling strategies:

| Variant | On failure | Returns |
|---------|------------|---------|
| `shell` | Throws error | stdout string |
| `shell!` | Throws error | exit code (shows output) |
| `shell.code` | Silent | exit code |
| `shell.full` | Silent | `{stdout stderr code ok}` |
| `shell?` | Silent | `null` |
| `shell.ok?` | Silent | `false` |

```shik
; Check if command succeeded
if (shell.ok? "git pull") $
  print "updated" $
  print "pull failed"

; Get output or default
let result $ or? (shell? "some-command") "default output"

; Full control
let res $ shell.full "risky-command"
if (object.get :ok res) $
  print (object.get :stdout res) $
  print "Error: {object.get :stderr res}"
```

#### Current limitations

There is no user-facing `try`/`catch`. Error recovery requires using the `?`/`.code`/`.full` shell variants or `file.read?`. Planned for a future version.

---

### 4.8 Patterns & Idioms

#### Pipeline pattern

```shik
data $>
  transform-step-1 $>
  transform-step-2 $>
  output
```

Each `$>` passes the result as the last argument to the next function. Build up complex operations step by step.

#### Composition pattern

```shik
let my-fn (f #> g #> h)
; equivalent to: fn [x] h (g (f x))

let count-lines (file.read #> string.lines #> list.len)
let has-todo    (file.read #> string.has :TODO)
```

Use `#>` to build named predicates and transformers for use in `list.filter`, `list.map`, etc.

#### Curried predicate pattern

```shik
; Filter list elements greater than 10
list.filter (> 10) numbers      ; elements where x > 10

; Filter strings containing a keyword
list.filter (string.has :error) log-lines

; Map: add prefix to all strings
list.map (+ "prefix-") names
```

#### REPL exploration

```
> help list.
-- list. functions: map, filter, fold, sort, ...

> help list.fold
native-lambda: list.fold
[initial reducer list]: reduces list to single value with accumulator

> let f (file.read #> string.lines)
> f :README.md
["# Shik" "" "A functional..." ...]
```

---

### 4.9 Built-in Modules

First of all, I will mention it another time - there is **NO module system** in Shik. I use the word "modules" to tie different functions together under one specialization, it is just a convention. There is no module `file`, there is just a bunch of functions with the `file.` prefix.

All modules are available without imports. Use `help module.` in the REPL to explore.

---

#### `number.` — Arithmetic & math

| Function | Args | Description |
|----------|------|-------------|
| `+` | `a b` | Addition (also: string concatenation when both strings) |
| `-` | `mod base` | Subtraction: `base - mod`. `(- 1)` = "subtract 1" |
| `*` | `mod base` | Multiplication |
| `/` | `mod base` | Division: `base / mod`. `(/ 2)` = "divide by 2" |
| `%` | `mod base` | Modulo: `base % mod` |
| `^` | `exp base` | Exponentiation: `base ^ exp`. `(^ 2)` = "square" |
| `number.abs` | `n` | Absolute value |
| `number.floor` | `n` | Round down |
| `number.ceil` | `n` | Round up |
| `number.round` | `n` | Round to nearest integer |
| `number.sqrt` | `n` | Square root |
| `number.sin` | `n` | Sine (radians) |
| `number.cos` | `n` | Cosine (radians) |
| `number.tan` | `n` | Tangent (radians) |
| `number.log` | `n` | Natural logarithm |
| `number.log10` | `n` | Base-10 logarithm |
| `number.min` | `a b` | Smaller of two numbers |
| `number.max` | `a b` | Larger of two numbers |
| `number.rand` | `[max]` or `[min max]` | Random number (0 args: float 0–1; 1 arg: int 0–max; 2 args: int min–max) |

```shik
- 1 5        ; 4  (5 - 1)
/ 2 10       ; 5  (10 / 2)
^ 3 5        ; 125 (5^3)
[1 2 3 4] $> list.map (* 2)     ; [2 4 6 8]
[1 2 3 4] $> list.map (^ 2)     ; [1 4 9 16]
number.rand 1 7                  ; random integer 1–6
```

---

#### `string.` — String manipulation

| Function | Args | Description |
|----------|------|-------------|
| `string` | `val` | Convert any value to string |
| `string.len` | `s` | Character count |
| `string.upper` | `s` | Convert to uppercase |
| `string.lower` | `s` | Convert to lowercase |
| `string.trim` | `s` | Remove leading and trailing whitespace |
| `string.trim-start` | `s` | Remove leading whitespace |
| `string.trim-end` | `s` | Remove trailing whitespace |
| `string.split` | `sep s` | Split `s` by `sep`, return list |
| `string.lines` | `s` | Split into lines (equivalent to `string.split "\n"`) |
| `string.join` | `sep lst` | Join list of strings with `sep` |
| `string.+` | `a b` | Concatenate two strings |
| `string.has` | `needle haystack` | True if `haystack` contains `needle` |
| `string.starts-with` | `prefix s` | True if `s` starts with `prefix` |
| `string.ends-with` | `suffix s` | True if `s` ends with `suffix` |
| `string.index-of` | `needle haystack` | Index of first occurrence, or `-1` |
| `string.replace` | `from to s` | Replace all occurrences of `from` with `to` in `s` |
| `string.at` | `i s` | Character at index `i`, or `null` |
| `string.slice` | `start end s` | Substring from `start` to `end` |
| `string.bytes` | `n` | Format number of bytes as human-readable (e.g. `"1.5 KiB"`) |
| `string.iterate` | `fn s` | Call `fn` for each character (left to right) |
| `string.iterate-backward` / `string.<iterate` | `fn s` | Call `fn` for each character (right to left) |
| `string.push` / `string.push>` / `string.push-right` | `s char` | Append `char` to `s` (mutates `s`) |
| `string.push-left` / `string.<push` | `s char` | Prepend `char` to `s` (mutates `s`) |
| `string.set` | `i s char` | Replace character at index `i` in `s` with `char` (mutates) |
| `string.=` | `a b` | String equality |

```shik
string.split "," "a,b,c"         ; ["a" "b" "c"]
string.join ", " ["a" "b" "c"]   ; "a, b, c"
string.has :TODO (file.read :main.rs)
"hello" $> string.upper           ; "HELLO"
string.replace "o" "0" "hello"   ; "hell0"
```

---

#### `list.` — List operations

| Function | Args | Description |
|----------|------|-------------|
| `list.len` | `lst` | Number of elements |
| `list.head` | `lst` | First element, or `null` |
| `list.tail` | `lst` | List without first element (O(1)) |
| `list.last` | `lst` | Last element, or `null` |
| `list.init` | `lst` | List without last element (O(1)) |
| `list.at` | `i lst` | Element at index `i`, or `null` |
| `list.empty?` | `lst` | True if list is empty |
| `list.sum` | `lst` | Sum all numbers |
| `list.reverse` | `lst` | Reversed list |
| `list.concat` | `a b` | Concatenate two lists |
| `list.take` | `n lst` | First `n` elements |
| `list.drop` | `n lst` | All elements after the first `n` |
| `list.slice` | `start end lst` | Sublist from `start` to `end` |
| `list.choice` | `lst` | Random element, or `null` |
| `list.range` | `[start] end [step]` | Create numeric range (see below) |
| `list.map` | `fn lst` | Apply `fn` to each element, return new list |
| `list.filter` | `pred lst` | Keep elements where `pred` returns true |
| `list.fold` | `init fn lst` | Reduce list to single value |
| `list.iterate` | `fn lst` | Call `fn` for each element (for side effects) |
| `list.iterate-backward` / `list.<iterate` | `fn lst` | Iterate right to left |
| `list.any` | `pred lst` | True if any element matches `pred` |
| `list.all` | `pred lst` | True if all elements match `pred` |
| `list.find` | `pred lst` | First matching element, or `null` |
| `list.find-index` | `pred lst` | Index of first match, or `-1` |
| `list.sort` | `cmp lst` | Sort using comparator `fn [a b] number` (negative = a before b) |
| `list.set` | `i lst val` | Set element at index `i` to `val` (mutates) |
| `list.push` / `list.push>` / `list.push-right` | `lst val` | Append `val` (mutates) |
| `list.push-left` / `list.<push` | `lst val` | Prepend `val` (mutates) |

`list.range`:

```shik
list.range 5        ; [0 1 2 3 4]
list.range 2 5      ; [2 3 4]
list.range 0 10 2   ; [0 2 4 6 8]
```

`list.fold` example:

```shik
; Sum of squares
list.fold 0 (fn [acc x] + acc (* x x)) [1 2 3 4]   ; 30

; Build a frequency map
list.fold {} (fn [acc item] '(
  let key (string item)
  if (object.has key acc) $
    object.set key acc (+ (object.get key acc) 1) $
    object.set key acc 1
  acc
)) [:a :b :a :c :a :b]
; {:a 3 :b 2 :c 1}
```

`list.sort` example:

```shik
; Sort numbers ascending
list.sort (fn [a b] - a b) [3 1 4 1 5 9]    ; [1 1 3 4 5 9]

; Sort descending
list.sort (fn [a b] - b a) [3 1 4 1 5 9]    ; [9 5 4 3 1 1]

; Sort pairs by second element
list.sort (fn [[_ a] [_ b]] - a b) [[:x 3] [:y 1] [:z 2]]
; [[:y 1] [:z 2] [:x 3]]
```

---

#### `object.` — Key-value maps

| Function | Args | Description |
|----------|------|-------------|
| `object.get` | `key obj` | Value for `key`, or `null` |
| `object.set` | `key obj val` | Set `key` to `val` (mutates) |
| `object.has` | `key obj` | True if `key` exists |
| `object.remove` | `key obj` | Remove `key` (mutates), returns removed value or `null` |
| `object.keys` | `obj` | List of all keys |
| `object.values` | `obj` | List of all values |
| `object.entries` | `obj` | List of `[key value]` pairs |
| `object.len` | `obj` | Number of keys |
| `object.empty?` | `obj` | True if no keys |
| `object.merge` | `a b` | Merge `a` and `b` (keys in `b` override `a`) |
| `object.clone` | `obj` | Shallow copy |
| `object.pick` | `keys obj` | New object with only the listed keys |
| `object.omit` | `keys obj` | New object without the listed keys |
| `object.from-entries` | `lst` | Create object from list of `[key value]` pairs |
| `object.map` | `fn obj` | Transform values: `fn [key value]` → new value |
| `object.map-entries` | `fn obj` | Transform entries: `fn [key value]` → `[new-key new-value]` |
| `object.filter` | `pred obj` | Keep entries where `pred [key value]` is true |
| `object.fold` | `init fn obj` | Reduce: `fn [acc [key value]]` |
| `object.iterate` | `fn obj` | Call `fn [key value]` for each entry |
| `object.any` | `pred obj` | True if any entry matches |
| `object.all` | `pred obj` | True if all entries match |
| `object.find` | `pred obj` | First matching `[key value]` pair, or `null` |
| `object.find-key` | `pred obj` | Key of first matching entry, or `null` |

```shik
let config {:host :localhost :port 8080}
object.get :host config          ; "localhost"
object.has :port config          ; true
object.keys config               ; ["host" "port"]

object.fold 0 (fn [acc [_ v]] + acc v) {:a 1 :b 2 :c 3}   ; 6
```

---

#### `file.` — Filesystem

**Reading:**

| Function | Args | Description |
|----------|------|-------------|
| `file.read` | `path` | Read file as string (error on failure) |
| `file.read?` | `path` | Read file as string, or `null` on failure |
| `file.read-lines` | `path` | Read file as list of lines |
| `file.read-bytes` | `path` | Read file as list of byte values (0–255) |

**Writing:**

| Function | Args | Description |
|----------|------|-------------|
| `file.write` | `path content` | Write string to file (overwrites) |
| `file.append` | `path content` | Append string to file |
| `file.write-bytes` | `path bytes` | Write byte list to file |

**Operations:**

| Function | Args | Description |
|----------|------|-------------|
| `file.copy` / `file.cp` | `dst src` | Copy file or directory |
| `file.move` / `file.mv` | `dst src` | Move or rename file or directory |
| `file.remove` / `file.rm` | `path` | Delete file or directory recursively |
| `file.rmdir` | `path` | Remove empty directory |
| `file.rmdir!` | `path` | Remove directory recursively |
| `file.mkdir` | `path` | Create directory |
| `file.mkdir!` | `path` | Create directory and all parent directories |
| `file.symlink` | `link target` | Create symlink |
| `file.read-link` | `path` | Read symlink target |
| `file.temp-dir` | — | Return system temp directory |

**Information:**

| Function | Args | Description |
|----------|------|-------------|
| `file.exists` | `path` | True if path exists |
| `file.is-file` | `path` | True if path is a file |
| `file.is-dir` | `path` | True if path is a directory |
| `file.is-symlink` | `path` | True if path is a symlink |
| `file.size` | `path` | File size in bytes |
| `file.size.deep` | `path` | Total size of file or directory (recursive) |
| `file.stat` | `path` | Object with `size`, `is_file`, `is_dir`, `is_symlink`, `readonly` |

**Listing:**

| Function | Args | Description |
|----------|------|-------------|
| `file.list` | `path` | List directory contents (names only) |
| `file.list!` | `path` | List directory contents (full paths) |
| `file.glob` | `pattern` | Find files matching glob pattern (full paths) |

---

#### `path.` — Path manipulation

| Function | Args | Description |
|----------|------|-------------|
| `path.name` | `path` | File name with extension |
| `path.stem` | `path` | File name without extension |
| `path.ext` | `path` | File extension |
| `path.parent` | `path` | Parent directory |
| `path.join` | `base component` | Join two path components |
| `path.absolute` | `path` | Convert to absolute path |

```shik
path.name :/home/user/file.txt    ; "file.txt"
path.stem :/home/user/file.txt    ; "file"
path.ext  :/home/user/file.txt    ; "txt"
path.parent :/home/user/file.txt  ; "/home/user"
path.join :./src :main.rs         ; "./src/main.rs"
```

---

#### `shell.` — Shell commands & environment

**Execution:**

| Function | Args | Description |
|----------|------|-------------|
| `shell` | `cmd` | Run command, return stdout as string. Error on non-zero exit. |
| `shell!` | `cmd` | Run command with output shown, return exit code |
| `shell.code` | `cmd` | Run silently, return exit code |
| `shell.full` | `cmd` | Run, return object `{stdout stderr code ok}` |
| `shell?` | `cmd` | Run, return stdout or `null` on failure |
| `shell.ok?` | `cmd` | True if command exits with code 0 |
| `shell.lines` | `cmd` | Run, return stdout as list of lines |

```shik
shell "git status"                ; stdout string
shell.lines "git branch"          ; ["  main" "* feature"]
shell? "ls /nonexistent"          ; null (no error thrown)
shell.full "make test"            ; {:stdout "..." :stderr "..." :code 0 :ok true}
```

**Environment:**

| Function | Args | Description |
|----------|------|-------------|
| `shell.env` | `name` | Get environment variable, or `null` |
| `shell.env.set` | `name val` | Set environment variable |
| `shell.env.remove` | `name` | Remove environment variable |
| `shell.env.all` | — | All environment variables as object |
| `shell.home` | — | Home directory path |
| `shell.cwd` | — | Current working directory |
| `shell.cd` | `path` | Change current working directory |
| `shell.os` | — | OS name: `"linux"`, `"macos"`, `"windows"` |
| `shell.arch` | — | CPU architecture: `"x86_64"`, `"aarch64"`, etc. |
| `shell.has` | `cmd` | True if command exists in PATH |
| `shell.which` | `cmd` | Full path to command, or `null` |
| `shell.ask` | `[prompt]` | Read a line from stdin (optional prompt) |
| `shell.args` | — | All command-line arguments as list |

**Process:**

| Function | Args | Description |
|----------|------|-------------|
| `process.pid` | — | Current process ID |
| `process.file` | — | Name of executing script file, or `null` in REPL |
| `process.args` | — | Command-line arguments (without `shik` and filename) |
| `process.sleep` | `ms` | Sleep for `ms` milliseconds |
| `process.abort` | — | Abnormal process termination |
| `exit` | `code` | Exit with given code |
| `exit!` | — | Exit with code 0 |

---

#### `bool.` — Comparisons & logic

| Function | Args | Description |
|----------|------|-------------|
| `=` | `a b` | Equality (numbers, bools, strings, null) |
| `!=` | `a b` | Inequality |
| `>` | `a b` | `b > a` — "is b greater than a?" |
| `>=` | `a b` | `b >= a` |
| `<` | `a b` | `b < a` — "is b less than a?" |
| `<=` | `a b` | `b <= a` |
| `and` | `a b` | Logical AND (short-circuit) |
| `or` | `a b` | Logical OR (short-circuit) |
| `not` | `a` | Logical negation |
| `bool` | `val` | Convert to bool (0, null, `""`, `[]`, `{}` = false) |

Note the argument order for comparisons: `> a b` asks "is b greater than a?" This is consistent with the modifier-first convention.

```shik
> 5 10        ; true  (is 10 > 5?)
< 10 5        ; true  (is 5 < 10?)
>= 3 3        ; true

; Curried predicates
[1 5 3 8 2] $> list.filter (> 4)     ; [5 8] (elements > 4)
[1 5 3 8 2] $> list.filter (<= 3)    ; [1 3 2] (elements <= 3)
```

---

#### `fn.` — Function utilities

| Function | Args | Description |
|----------|------|-------------|
| `fn.invoke` / `invoke` | `fn` | Call `fn` with no arguments |
| `fn.id` | `val` | Identity function — returns `val` |

```shik
[(fn [] print :a) (fn [] print :b)] $> list.iterate fn.invoke
; a
; b
```

---

#### `var.` — Dynamic variable access

| Function | Args | Description |
|----------|------|-------------|
| `var.get` | `name` | Look up variable by string name, or `null` |

---

#### Polymorphic functions

These functions work across multiple types:

| Function | Types | Description |
|----------|-------|-------------|
| `+` | Number+Number, String+String, String+other | Addition or concatenation |
| `at` | (index, String) or (index, List) | Get element at index |
| `iterate` | (fn, String) or (fn, List) | Iterate over characters or elements |
| `iterate-backward` / `<iterate` | (fn, String) or (fn, List) | Iterate in reverse |
| `print` | any | Print value to stdout |

---

#### Utilities

| Function | Args | Description |
|----------|------|-------------|
| `print` | `val` | Print value with newline |
| `help` | `[topic]` | Show help (no args: overview; `"module."`: module list; `"fn.name"`: function info) |
| `or?` | `val default` | Return `default` if `val` is `null` |


---

## 5. Examples

Runnable scripts are in the [`demo/`](demo/) directory.

### [Sort by size](demo/files-new-demo.shk)

List `.docx` files sorted by size with human-readable byte counts. Shows list destructuring in `list.sort` comparators and `string.bytes`.

### [Pattern matching](demo/match.shk)

Some examples of using pattern matching.

### [XO kata](demo/xo.shk)

[Codewars kata](https://www.codewars.com/kata/55908aad6620c066bc00002a): check whether a string has equal numbers of `x` and `o`. Shows `string.iterate` with mutable closure state and `if` as a multi-branch expression.

### [Dice game](demo/dice-game.shk)

[Codewars kata](https://www.codewars.com/kata/5270d0d18625160ada0000e4): score a five-dice throw by the Greed rules. Shows `list.fold` building a frequency object, `match` dispatching on dice face, and curried scoring helpers.

### [Scoping](demo/nested-scope.shk)

Counter factory: `counter n` returns a `bump` function that closes over its own `x`. Three independent counters, outer `x` untouched. Shows lexical closures and mutable captured state.

### [Chain list](demo/chainer.shk)

Recursion+pattern matching. Format a list as `[a-b-c-d]` via recursion and `match` on length. Includes a one-liner equivalent using `string.join` to show the contrast.

---

## 6. Performance

Shik is written in Rust with a tree-walk interpreter, Rc/RefCell memory management (no tracing GC), and all built-in functions implemented in native Rust. IO-bound workloads are fast. CPU-bound algorithmic work (heavy branching, deep recursion) is slower.

**Benchmark: count lines across ~9,800 lines of Rust source (37 files).** Via `hyperfine --warmup 3 -N`, macOS, Apple Silicon.

**Shik:**

```shik
file.glob :./src/**/*.rs $>
  list.map (file.read-lines #> list.len) $>
  list.sum $> print
```

**Bash:**

```bash
find ./src -name '*.rs' -exec cat {} + | wc -l
```

**Python:**

```python
from pathlib import Path
print(sum(len(f.read_text().splitlines()) for f in Path('./src').rglob('*.rs')))
```

| Tool | Time | Memory |
|------|------|--------|
| **Shik** | 4.4 ms | 2.6 MB |
| Bash | 9.1 ms | 2.1 MB |
| Python | 30.3 ms | 12 MB |

---

For CPU-bound algorithmic work (e.g. a dice game win probability calculator), Shik is roughly **10× slower than Python**. Optimization is planned, but ergonomics come first.

**Shik:** [Dice game](demo/dice-game.shk)

But replaced in the end with

```shik
list.range 1000 $> list.iterate fn [_] '(
  dice-game [5 1 3 4 1]
  dice-game [1 1 1 3 1]
  dice-game [2 4 4 5 4]
)
```

**Python:**

```python
from collections import Counter

def dice_game(throw):
    bucket = Counter(throw)

    def count_accumulative(triple_mod, single_mod, count):
        if count == 3:
            return triple_mod
        elif count > 3:
            return triple_mod + (count % 3) * single_mod
        else:
            return (count % 3) * single_mod

    score = 0
    for dice, count in bucket.items():
        if dice == 1:
            score += count_accumulative(1000, 100, count)
        elif dice == 5:
            score += count_accumulative(500, 50, count)
        elif count >= 3:
            score += dice * 100

    return score

for _ in range(1000):
    dice_game([5, 1, 3, 4, 1])
    dice_game([1, 1, 1, 3, 1])
    dice_game([2, 4, 4, 5, 4])
```

**Result:**

```
Benchmark 1: shik dice-game.shk
  Time (mean ± σ):     335.2 ms ±   5.8 ms    [User: 312.1 ms, System: 17.5 ms]
  Range (min … max):   326.2 ms … 341.6 ms    10 runs
 
Benchmark 2: python3 dice-game.py
  Time (mean ± σ):      30.9 ms ±   3.1 ms    [User: 23.3 ms, System: 5.2 ms]
  Range (min … max):    28.5 ms …  45.1 ms    89 runs

Summary
  python3 dice-game.py ran
   10.84 ± 1.10 times faster than shik dice-game.shk
```

Shik is positioned for IO-bound shell automation, not complex life-time applications or servers.

---

## 7. Roadmap

Current version: **v0.7.1**

Planned, roughly in priority order:

- Shebang support (`#!/usr/bin/env shik`)
- Object destructuring
- Regular expressions
- Multiple statements per line with `,`
- Networking
- Lambda shorthand (`#(- #1 #2)` instead of `fn [a b] - a b`)
- JSON parsing
- User-facing error handling (`try`/`catch` or similar)
- Threading

---

## 8. Contributing

Shik is in active development. The codebase is a Rust project; see [`CLAUDE.md`](CLAUDE.md) (named after one nice fella, he is  a great contributor) for architecture notes and conventions for adding native functions.

Issues and PRs welcome at [github.com/pungy/shik](https://github.com/pungy/shik).

**License:** MIT
