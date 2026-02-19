# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

**Shik** is a functional, dynamically-typed scripting language for shell automation, implemented as a Rust tree-walk interpreter. Syntax is inspired by Haskell and Lisp: expression-oriented, whitespace-based function application, automatic currying.

## Commands

```bash
cargo build --release      # build (binary: target/release/shik)
cargo test                 # run all tests
cargo test test_parser     # run a single test file
cargo test my_test_name    # run a single test by name
cargo run -- demo/factorial.shk  # run a script
cargo run                        # start REPL
cargo run --ast      # REPL with AST debug output
```

## Architecture

Classic interpreter pipeline: **Lexer → Parser → AST → Evaluator**

```
src/parser/
  lexer.rs      – tokenization; handles string interpolation {…}, comments (;;)
  parser.rs     – Pratt parser producing AST
  ast.rs        – AST node types (Expr enum)
  tokens.rs     – TokenType definitions
  error.rs      – parse errors with Span

src/eval/
  evaluator.rs  – tree-walk interpreter (Interpretator struct)
  value.rs      – runtime Value enum (Null, Bool, Number, String, List, Object, Lambda, NativeClosure, SpecialForm)
  error.rs      – runtime errors with Span
  utils.rs      – pattern matching helpers (define_match, pattern_match)
  native_functions/  – 16 modules: bool, number, string, list, object, file, shell,
                       keywords, branching, variables, polymorphic, function, misc, help, macros

src/lang.rs     – high-level API: evaluate(), print(), run_repl(), eval_file()
src/main.rs     – CLI entry point (file mode or REPL)
src/cli.rs      – CLI argument parsing (parse_args → Command enum), help/version printers
src/repl/
  mod.rs        – REPL entry point: run(ReplConfig); rustyline Editor setup, history (~/.shik_history), main read-eval loop
  helper.rs     – ReplHelper: wires together Highlighter, Validator, Completer for rustyline
  highlighter.rs – ShikHighlighter: tree-sitter-based syntax highlighting via highlights.scm
  validator.rs  – ShikValidator: returns Incomplete on UnexpectedEndOfInput/UnterminatedString for multi-line input
```

Integration tests live in `tests/`, demo scripts in `demo/*.shk`.

## Key Language Semantics

**Operator precedence** (lowest → highest):
1. `$>` – pipe (passes left result as last arg to right)
2. `$` – right-associative chain/application
3. whitespace – function application
4. `#>` – function composition (highest)

**Argument order conventions** (important when adding native functions):
- *Mutation/write*: destination first (`list.set INDEX LIST VALUE`)
- *Read*: specifier first, then target (`list.at INDEX LIST`)
- Non-associative math operators reverse arg order: `(- 1 5)` = 4, `(/ 2 10)` = 5

**Value model**: all values are `Rc<Value>`; environments use `Rc<RefCell<Env>>` with parent chains for lexical scoping. Lists use a view-based `ListRepr` for O(1) head/tail without copying.

**Special forms** (`let`, `fn`, `match`, `if`) are not evaluated eagerly; they are passed as `SpecialForm` values and handled in `apply_special_fn()` inside `evaluator.rs`.

## Adding Native Functions

1. Implement in the appropriate module under `src/eval/native_functions/`.
2. Use the `paste!` macro patterns already present for defining curried functions.
3. Register the function in the module's `bind_*` function that is called from `Interpretator::new()`.
4. Follow the argument order conventions above.
