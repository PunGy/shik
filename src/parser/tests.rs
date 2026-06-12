use crate::parser::{parse, ExpressionKind, Statement};
use insta::assert_debug_snapshot;

/// Parse and return the statements (panics on parse error).
fn ast(input: &str) -> Vec<Statement> {
    parse(input).unwrap().statements
}

#[test]
fn test_parse_number() {
    let input = "42";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::Number(n) => assert_eq!(*n, 42.0),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_parse_string() {
    let input = r#""hello world""#;
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::String(s) => assert_eq!(s, "hello world"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_parse_symbol_string() {
    let input = ":symbol";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::String(s) => assert_eq!(s, "symbol"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_parse_identifier() {
    let input = "variable-name";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::Identifier(name) => assert_eq!(name, "variable-name"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_parse_list() {
    let input = "[1 2 3]";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::List(items) => {
            assert_eq!(items.len(), 3);
            match &items[0].kind {
                ExpressionKind::Number(n) => assert_eq!(*n, 1.0),
                _ => panic!("Expected number in list"),
            }
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_parse_object() {
    let input = "{:x 10 :y 20}";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::Object(items) => {
            assert_eq!(items.len(), 2);
            match &items[0].key.kind {
                ExpressionKind::String(s) => assert_eq!(s, "x"),
                _ => panic!("Expected string key"),
            }
            match &items[0].value.kind {
                ExpressionKind::Number(n) => assert_eq!(*n, 10.0),
                _ => panic!("Expected number value"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_parse_let() {
    // `let$` is the keyword form; plain `let` is a native function.
    let input = "let$ x 10";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::Let { pattern, value } => {
            match pattern {
                crate::parser::MatchPattern::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier pattern"),
            }
            match &value.kind {
                ExpressionKind::Number(n) => assert_eq!(*n, 10.0),
                _ => panic!("Expected number value"),
            }
        }
        _ => panic!("Expected let expression"),
    }
}

#[test]
fn test_parse_lambda() {
    let input = "fn [x y] + x y";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::Lambda {
            parameters,
            rest,
            body,
        } => {
            assert_eq!(parameters.len(), 2);
            assert!(rest.is_none());
            // Body should be an application expression
            match &body.kind {
                ExpressionKind::Application { .. } => {}
                _ => panic!("Expected application in lambda body"),
            }
        }
        _ => panic!("Expected lambda"),
    }
}

#[test]
fn test_parse_pipe() {
    let input = "x $> f";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::Pipe { left, right } => {
            match &left.kind {
                ExpressionKind::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier on left"),
            }
            match &right.kind {
                ExpressionKind::Identifier(name) => assert_eq!(name, "f"),
                _ => panic!("Expected identifier on right"),
            }
        }
        _ => panic!("Expected pipe expression"),
    }
}

#[test]
fn test_parse_flow() {
    let input = "f #> g";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::Flow { left, right } => {
            match &left.kind {
                ExpressionKind::Identifier(name) => assert_eq!(name, "f"),
                _ => panic!("Expected identifier on left"),
            }
            match &right.kind {
                ExpressionKind::Identifier(name) => assert_eq!(name, "g"),
                _ => panic!("Expected identifier on right"),
            }
        }
        _ => panic!("Expected flow expression"),
    }
}

#[test]
fn test_parse_application() {
    let input = "f x";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::Application { function, argument } => {
            match &function.kind {
                ExpressionKind::Identifier(name) => assert_eq!(name, "f"),
                _ => panic!("Expected identifier as function"),
            }
            match &argument.kind {
                ExpressionKind::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier as argument"),
            }
        }
        _ => panic!("Expected application"),
    }
}

#[test]
fn test_parse_parenthesized() {
    let input = "(+ 1 2)";
    let result = parse(input).unwrap();
    assert_eq!(result.statements.len(), 1);

    match &result.statements[0].expression.kind {
        ExpressionKind::Parenthesized(inner) => match &inner.kind {
            ExpressionKind::Application { .. } => {}
            _ => panic!("Expected application inside parentheses"),
        },
        _ => panic!("Expected parenthesized expression"),
    }
}

#[test]
fn test_empty_lines_ignored() {
    let input = "x\n\n\ny\n\nz";
    let result = parse(input).unwrap();

    assert_eq!(result.statements.len(), 3);

    match &result.statements[0].expression.kind {
        ExpressionKind::Identifier(name) => assert_eq!(name, "x"),
        _ => panic!("Expected identifier"),
    }
    match &result.statements[1].expression.kind {
        ExpressionKind::Identifier(name) => assert_eq!(name, "y"),
        _ => panic!("Expected identifier"),
    }
    match &result.statements[2].expression.kind {
        ExpressionKind::Identifier(name) => assert_eq!(name, "z"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_operators_break_without_continuation() {
    // When operator is NOT at end of line, newline breaks the statement:
    // `$> f` on its own line has no left operand and must fail to parse.
    let input = "x\n$> f";
    let result = parse(input);
    assert!(result.is_err());
}

// ── AST snapshots ──────────────────────────────────────────────────────
// Structural expectations live in `snapshots/`; review with `cargo insta
// review` (or INSTA_UPDATE=always + git diff) after parser changes.

#[test]
fn test_parse_block() {
    // '(x y z) is a single application chain: App(App(x, y), z)
    assert_debug_snapshot!(ast("'(x y z)"));
}

#[test]
fn test_parse_lazy() {
    // #(x y) is a single application: App(x, y)
    assert_debug_snapshot!(ast("#(x y)"));
}

#[test]
fn test_precedence_pipe_over_application() {
    // x $> f y parses as: x $> (f y)
    assert_debug_snapshot!(ast("x $> f y"));
}

#[test]
fn test_precedence_flow_over_pipe() {
    // a #> b $> c parses as: (a #> b) $> c
    assert_debug_snapshot!(ast("a #> b $> c"));
}

#[test]
fn test_complex_expression() {
    assert_debug_snapshot!(ast("let$ result (fn [x] x $> double #> add 10)"));
}

#[test]
fn test_multiple_statements_with_newlines() {
    assert_debug_snapshot!(ast("let$ x 10\nlet$ y 20\nx"));
}

#[test]
fn test_block_with_newlines() {
    // Each line of a block is its own expression
    assert_debug_snapshot!(ast("'(\n  add 1 2\n  mul 3 4\n  sub 5 6\n)"));
}

#[test]
fn test_lazy_with_newlines() {
    assert_debug_snapshot!(ast("#(\n  x\n  y\n  z\n)"));
}

#[test]
fn test_operators_allow_continuation() {
    // Operator at end of line continues onto the next line: (x $> f), then y
    assert_debug_snapshot!(ast("x $>\nf\ny"));
}

#[test]
fn test_block_with_pipe_on_same_line() {
    assert_debug_snapshot!(ast("'(\n  x $> f\n  y $> g\n)"));
}

#[test]
fn test_complex_nested_structure() {
    assert_debug_snapshot!(ast(
        "let$ process (fn [data] '(\n  let$ cleaned trim data\n  save cleaned\n))"
    ));
}

#[test]
fn test_block_chain_operator_binds_to_full_application() {
    // Inside a block, `$` binds to the entire application chain built so
    // far: `if true $ print 1` is Chain { App(if, true), App(print, 1) },
    // NOT Application { if, Chain { true, App(print, 1) } }.
    assert_debug_snapshot!(ast("'(\n  if true $\n    print 1\n)"));
}

#[test]
fn test_block_chain_single_line() {
    // a b $ c d is Chain { App(a, b), App(c, d) }
    assert_debug_snapshot!(ast("'(\n  a b $ c d\n)"));
}

#[test]
fn test_lazy_chain_operator_binds_to_full_application() {
    assert_debug_snapshot!(ast("#(\n  if true $\n    print 1\n)"));
}

#[test]
fn test_block_multiple_operators_same_line() {
    // a b $ c d $> e f is Pipe { Chain { App(a,b), App(c,d) }, App(e,f) }
    assert_debug_snapshot!(ast("'(\n  a b $ c d $> e f\n)"));
}
