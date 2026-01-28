use crate::{
    define_help, define_native,
    eval::{
        evaluator::Interpretator,
        native_functions::{native_result, polymorphic::Print},
        value::{EnvRef, NativeContext, SpecialClosure, SpecialFn, Value},
        EvalResult,
    },
    parser::{Expression, ExpressionKind},
    special_op,
};
use std::rc::Rc;

const GENERAL_HELP: &str = "SHIK language:

To get help on any module of the language: help [module_name]

Example:

```shik
> help number.

number module:

Arithmetic:
- number.+: addition
...
```

MODULES:


-- Type modules
- number.: arithmetic, rounding, comparison, math functions, random
- string.: string manipulation, conversion, iteration
- list.: list operations, higher-order functions
- object.: object access, mutation, creation, higher-order functions
- bool.: comparison, logic, conversion

-- Function modules
- fn.: function utilities (invoke, quote)

-- System modules
- file.: file reading, writing, operations, information
- path.: path manipulation (name, stem, ext, parent, join, absolute)
- shell.: shell execution, environment variables, I/O
- process.: process control (pid, args, sleep, exit, abort)

-- Language features
- branching.: loops, conditionals (if, while, match)
- var.: variable operations (get by name)

-- Misc
- general.: polymorphic operations for any value (+, at, iterate)
- misc.: utility functions (or?)

-- Language concepts
- syntax.: syntax rules, application, operators, identifiers
- let.: pattern matching in let$ and fn
- match.: pattern matching with match expression";

special_op!(Help, "help", args, ctx, {
    let msg: String;
    if args.len() > 0 {
        let arg = &args[0];

        match &arg.kind {
            ExpressionKind::Identifier(ident) => {
                let help_msg = ctx.env.lookup_help(ident);
                match help_msg {
                    Some(help_msg) => msg = help_msg,
                    None => {
                        msg = help_for_literal(&arg, ctx, Some(format!("Identifier {}\n", ident)))
                    }
                }
            }
            _ => msg = help_for_literal(&arg, ctx, None),
        }
    } else {
        msg = GENERAL_HELP.to_string();
    }

    let value = Rc::new(Value::String(msg));

    Print::run(&value)?;

    native_result(Value::Null)
});

fn help_for_literal(expr: &Expression, ctx: &NativeContext, prefix: Option<String>) -> String {
    let val = ctx.inter.eval_expr(expr, &ctx.env);
    match val {
        Ok(val) => format!(
            "{}{:?}: {}",
            prefix.unwrap_or("".to_string()),
            val.get_type(),
            val.to_string()
        ),
        Err(_) => "Unable to get help".to_string(),
    }
}

pub fn bind_help_module(env: &EnvRef, inter: Rc<Interpretator>) {
    define_native!(Help, env, inter);
    define_help!(Help, env, "[value?]: print help information about anything. Enter help without arguments to get general information about the language.");

    // Language concept documentation
    env.define_help("syntax.".to_string(), r#"SHIK Syntax Guide

EXPRESSIONS & APPLICATION

Everything in SHIK is an expression. Function application uses whitespace:

    function arg1 arg2    ; applies arg2 to (function arg1)

Application is left-associative and has high precedence:

    f a b c    ; equivalent to: ((f a) b) c

OPERATORS (from lowest to highest precedence)

    $>   Pipe: passes left value as last argument to right
         x $> f a    ; equivalent to: f a x

    $    Chain: groups right side, then applies
         f $ g x     ; equivalent to: f (g x)

    (space)  Application: standard function application
         f x y       ; equivalent to: (f x) y

    #>   Flow: function composition
         f #> g      ; creates: fn [x] g (f x)

IDENTIFIERS

Valid identifier characters:
- Letters (a-z, A-Z) and digits (0-9)
- Special: ' ! @ % ^ & * - = _ + | ? < > . $ /

Examples: my-var, string.upper, list?, +, file.read

LITERALS

    42, 3.14, -17       ; numbers
    "hello world"       ; block string (supports escapes and interpolation)
    :hello              ; inline string (no spaces, terminates at whitespace)
    [1 2 3]             ; list
    {:x 10 :y 20}       ; object

STRING INTERPOLATION

    "Hello, {name}!"              ; simple variable
    "Sum is {+ a b}"              ; expression
    "Upper: {string.upper name}"  ; function call

BLOCKS

    '(expr1 expr2 ...)   ; block: evaluates all, returns last
    #(expr1 expr2 ...)   ; lazy block: deferred evaluation
    (expr)               ; grouping: changes precedence

COMMENTS

    ; single line comment
    {* block comment
       can span multiple lines *}
"#.to_string());

    env.define_help("let$.".to_string(), r#"Pattern Matching with let$ and fn

OVERVIEW

Pattern matching destructures values and binds parts to variables.
Used in `let$` for variable binding and `fn` for function parameters.

PATTERNS

1. IDENTIFIER - binds the entire value
   let$ x 42           ; x = 42
   fn [x] x            ; identity function

2. WILDCARD _ - matches anything, discards value
   let$ _ (side-effect)    ; ignore result
   fn [_ y] y              ; ignore first argument

3. LIST PATTERN - destructures lists
   let$ [a b c] [1 2 3]  ; a=1, b=2, c=3
   let left fn [[x y]] x ; x=1, y=2
   left [1 2] ; x


REST PATTERN

Use # to capture remaining elements:

   let$ [first #rest] [1 2 3 4]
   ; first = 1
   ; rest = [2 3 4]

   let p fn [[fst snd #rest]] + fst snd  ; passed list to function must be at least two elements long
   p [1 2 3 4] ; result: 3, rest: [3 4]

EXAMPLES

; Destructure a list
let$ [x y z] [10 20 30]
print x   ; 10

; Ignore elements
let$ [_ second _] [1 2 3]
print second   ; 2

; Head and tail
let$ [head #tail] [1 2 3 4 5]
print head   ; 1
print tail   ; [2 3 4 5]

; Function with pattern matching
let sum-pair fn [[a b]] + a b
sum-pair [3 7]   ; 10

NOTES

    Variadic functions are not supported, since everything is automatically curried.
    The only exceptions are `special-lambda`, such as `if`, `or?`. They are limited and embed to the language, and cannot be curried.
"#.to_string());

    env.define_help("match.".to_string(), r#"Pattern Matching with match

SYNTAX

    match value {
      pattern1 result1
      pattern2 result2
      ...
    }

Evaluates `value`, then tests each pattern in order.
Returns the result of the first matching pattern.

PATTERN TYPES

1. LITERAL - exact value match
   match x {
     0 :zero
     1 :one
     2 :two
   }

2. IDENTIFIER - matches by the value under identifier
   let n 10
   match x {
     n :matched    ; would be matched only if `= n x`
   }

3. WILDCARD _ - matches anything (catch-all)
   match x {
     0 :zero
     _ :other     ; default case
   }

4. NAMED WILDCARD #name - matches anything, binds to name
   match x {
     10   "you got ten!"
     #val "you got just {val}" ; captures and returns the value
   }

5. LIST PATTERN - destructures lists
   match lst {
     [] :empty
     [x] :single
     [x y] :pair
     [x y #rest] "pair {x}-{y}, and the rest is: {rest}"
   }

EXAMPLES

; List destructuring
let first-two fn [lst] match lst {
  [] :empty
  [x] x
  [x y _] [x y]
}

first-two [1 2 3 4]   ; [1 2]

; Platform detection
let platform match "{shell.os}-{shell.arch}" {
  :Darwin-arm64 :macos-aarch64
  :Darwin-x86_64 :macos-x86_64
  :Linux-x86_64 :linux-x86_64
  #other other
}

; Extracting values
let get-coords fn [point] match point {
  [x y] "2D: {x}, {y}"
  [x y z] "3D: {x}, {y}, {z}"
  _ :invalid
}

NOTES

- Patterns are tested in order; first match wins
- String patterns match exact string values
- Identifiers matched against the value it has
"#.to_string());
}
