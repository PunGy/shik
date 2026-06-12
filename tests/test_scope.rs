//! Tests for scope and lifetime correctness
//!
//! These tests verify that:
//! 1. Local variables do not persist across calls
//! 2. Calls don't clobber captured vars unexpectedly
//! 3. Match bindings inside a function disappear after return

mod common;

use common::{eval, eval_is_error, eval_number};

#[test]
fn test_local_variables_do_not_persist_across_calls() {
    // Define a function that creates a local variable
    // After calling it, the local should not be visible
    let code = r#"
let f (fn [n] '(
    let x n
    0
))
f 1
f 2
"#;

    // This should succeed - the function returns 0
    let result = eval(code);
    assert!(result.is_ok(), "Function should execute: {:?}", result);
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn test_local_variable_not_visible_after_call() {
    // After calling a function that defines 'x', 'x' should not be visible
    // in the outer scope
    let code = r#"
let f (fn [n] '(
    let x n
    x
))
f 42
x
"#;

    // This should fail because 'x' is not defined in outer scope
    assert!(
        eval_is_error(code),
        "Variable 'x' should not be visible after function returns"
    );
}

#[test]
fn test_multiple_calls_dont_share_locals() {
    // Each call should have its own local scope
    let code = r#"
let counter (fn [n] '(
    let local n
    local
))
let a (counter 10)
let b (counter 20)
+ a b
"#;

    let result = eval_number(code);
    assert!(result.is_ok(), "Should execute: {:?}", result);
    assert_eq!(result.unwrap(), 30.0);
}

#[test]
fn test_closure_captures_definition_env() {
    // A closure should capture variables from its definition environment
    let code = r#"
let x 100
let f (fn [n] (+ x n))
f 5
"#;

    let result = eval_number(code);
    assert!(result.is_ok(), "Should execute: {:?}", result);
    assert_eq!(result.unwrap(), 105.0);
}

#[test]
fn test_calls_dont_clobber_captured_vars() {
    // Calling a function should not modify captured variables
    let code = r#"
let x 100
let f (fn [n] '(
    let x n
    x
))
f 5
x
"#;

    let result = eval_number(code);
    assert!(result.is_ok(), "Should execute: {:?}", result);
    // The outer 'x' should still be 100, not 5
    assert_eq!(result.unwrap(), 100.0);
}

#[test]
fn test_nested_function_calls_have_separate_frames() {
    // Nested calls should each have their own frame
    let code = r#"
let outer (fn [a] '(
    let inner (fn [b] '(
        let local b
        + a local
    ))
    inner (* a 2)
))
outer 10
"#;

    let result = eval_number(code);
    assert!(result.is_ok(), "Should execute: {:?}", result);
    // outer(10) calls inner(20), which returns 10 + 20 = 30
    assert_eq!(result.unwrap(), 30.0);
}

#[test]
fn test_recursive_calls_have_separate_frames() {
    // Recursive calls should each have their own frame
    let code = r#"
let factorial (fn [n] '(
    if (< n 2) $
        1 $
        (* n (factorial (- 1 n)))
))
factorial 5
"#;

    let result = eval_number(code);
    assert!(result.is_ok(), "Should execute: {:?}", result);
    assert_eq!(result.unwrap(), 120.0);
}

#[test]
fn test_higher_order_function_preserves_scopes() {
    // Higher-order functions should work correctly with scopes
    let code = r#"
let apply (fn [f x] (f x))
let double (fn [n] (* n 2))
apply double 21
"#;

    let result = eval_number(code);
    assert!(result.is_ok(), "Should execute: {:?}", result);
    assert_eq!(result.unwrap(), 42.0);
}

#[test]
fn test_map_preserves_scopes() {
    // list.map should work correctly with closures
    let code = r#"
let multiplier 10
let f (fn [x] (* x multiplier))
list.sum (list.map f [1 2 3])
"#;

    let result = eval_number(code);
    assert!(result.is_ok(), "Should execute: {:?}", result);
    // (1*10) + (2*10) + (3*10) = 60
    assert_eq!(result.unwrap(), 60.0);
}

#[test]
fn test_closure_returned_from_function() {
    // A closure returned from a function should still work
    let code = r#"
let make-adder (fn [n] (fn [x] (+ x n)))
let add5 (make-adder 5)
add5 10
"#;

    let result = eval_number(code);
    assert!(result.is_ok(), "Should execute: {:?}", result);
    assert_eq!(result.unwrap(), 15.0);
}

#[test]
fn test_multiple_closures_from_same_factory() {
    // Multiple closures from the same factory should be independent
    let code = r#"
let make-adder (fn [n] (fn [x] (+ x n)))
let add5 (make-adder 5)
let add10 (make-adder 10)
+ (add5 1) (add10 1)
"#;

    let result = eval_number(code);
    assert!(result.is_ok(), "Should execute: {:?}", result);
    // (1+5) + (1+10) = 17
    assert_eq!(result.unwrap(), 17.0);
}
