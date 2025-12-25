#[cfg(test)]
mod tests {
    use crate::parser::{parse, Expression};

    #[test]
    fn test_parse_number() {
        let input = "42";
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Number(n) => assert_eq!(*n, 42.0),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_parse_string() {
        let input = r#""hello world""#;
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::String(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_symbol_string() {
        let input = ":symbol";
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::String(s) => assert_eq!(s, "symbol"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_identifier() {
        let input = "variable-name";
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Identifier(name) => assert_eq!(name, "variable-name"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_parse_list() {
        let input = "[1 2 3]";
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::List(items) => {
                assert_eq!(items.len(), 3);
                match &items[0] {
                    Expression::Number(n) => assert_eq!(*n, 1.0),
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

        match &result.statements[0].expression {
            Expression::Object(items) => {
                assert_eq!(items.len(), 2);
                match &items[0].key {
                    Expression::String(s) => assert_eq!(s, "x"),
                    _ => panic!("Expected string key"),
                }
                match &items[0].value {
                    Expression::Number(n) => assert_eq!(*n, 10.0),
                    _ => panic!("Expected number value"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_let() {
        let input = "let x 10";
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Let { pattern, value } => {
                match pattern {
                    crate::parser::LetPattern::Identifier(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier pattern"),
                }
                match &**value {
                    Expression::Number(n) => assert_eq!(*n, 10.0),
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

        match &result.statements[0].expression {
            Expression::Lambda {
                parameters,
                rest,
                body,
            } => {
                assert_eq!(parameters.len(), 2);
                assert!(rest.is_none());
                // Body should be an application expression
                match &**body {
                    Expression::Application { .. } => {}
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

        match &result.statements[0].expression {
            Expression::Pipe { left, right } => {
                match &**left {
                    Expression::Identifier(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier on left"),
                }
                match &**right {
                    Expression::Identifier(name) => assert_eq!(name, "f"),
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

        match &result.statements[0].expression {
            Expression::Flow { left, right } => {
                match &**left {
                    Expression::Identifier(name) => assert_eq!(name, "f"),
                    _ => panic!("Expected identifier on left"),
                }
                match &**right {
                    Expression::Identifier(name) => assert_eq!(name, "g"),
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

        match &result.statements[0].expression {
            Expression::Application { function, argument } => {
                match &**function {
                    Expression::Identifier(name) => assert_eq!(name, "f"),
                    _ => panic!("Expected identifier as function"),
                }
                match &**argument {
                    Expression::Identifier(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier as argument"),
                }
            }
            _ => panic!("Expected application"),
        }
    }

    #[test]
    fn test_parse_block() {
        // '(x y z) should be a single application chain: App(App(x, y), z)
        let input = "'(x y z)";
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Block(exprs) => {
                assert_eq!(exprs.len(), 1);
                match &exprs[0] {
                    Expression::Application { function, argument } => {
                        // argument should be z
                        match &**argument {
                            Expression::Identifier(name) => assert_eq!(name, "z"),
                            _ => panic!("Expected identifier z as argument"),
                        }
                        // function should be App(x, y)
                        match &**function {
                            Expression::Application {
                                function: inner_fn,
                                argument: inner_arg,
                            } => {
                                match &**inner_fn {
                                    Expression::Identifier(name) => assert_eq!(name, "x"),
                                    _ => panic!("Expected identifier x"),
                                }
                                match &**inner_arg {
                                    Expression::Identifier(name) => assert_eq!(name, "y"),
                                    _ => panic!("Expected identifier y"),
                                }
                            }
                            _ => panic!("Expected nested application"),
                        }
                    }
                    _ => panic!("Expected application in block"),
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_parse_lazy() {
        // #(x y) should be a single application: App(x, y)
        let input = "#(x y)";
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Lazy(exprs) => {
                assert_eq!(exprs.len(), 1);
                match &exprs[0] {
                    Expression::Application { function, argument } => {
                        match &**function {
                            Expression::Identifier(name) => assert_eq!(name, "x"),
                            _ => panic!("Expected identifier x"),
                        }
                        match &**argument {
                            Expression::Identifier(name) => assert_eq!(name, "y"),
                            _ => panic!("Expected identifier y"),
                        }
                    }
                    _ => panic!("Expected application in lazy block"),
                }
            }
            _ => panic!("Expected lazy block"),
        }
    }

    #[test]
    fn test_parse_parenthesized() {
        let input = "(+ 1 2)";
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Parenthesized(inner) => match &**inner {
                Expression::Application { .. } => {}
                _ => panic!("Expected application inside parentheses"),
            },
            _ => panic!("Expected parenthesized expression"),
        }
    }

    #[test]
    fn test_precedence_pipe_over_application() {
        let input = "x $> f y";
        let result = parse(input).unwrap();

        // Should parse as: x $> (f y)
        match &result.statements[0].expression {
            Expression::Pipe { left, right } => {
                match &**left {
                    Expression::Identifier(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier on left"),
                }
                match &**right {
                    Expression::Application { .. } => {}
                    _ => panic!("Expected application on right"),
                }
            }
            _ => panic!("Expected pipe expression"),
        }
    }

    #[test]
    fn test_precedence_flow_over_pipe() {
        let input = "a #> b $> c";
        let result = parse(input).unwrap();

        // Should parse as: (a #> b) $> c
        match &result.statements[0].expression {
            Expression::Pipe { left, right } => {
                match &**left {
                    Expression::Flow { .. } => {}
                    _ => panic!("Expected flow on left"),
                }
                match &**right {
                    Expression::Identifier(name) => assert_eq!(name, "c"),
                    _ => panic!("Expected identifier on right"),
                }
            }
            _ => panic!("Expected pipe expression"),
        }
    }

    #[test]
    fn test_complex_expression() {
        let input = "let result (fn [x] x $> double #> add 10)";
        let result = parse(input).unwrap();

        match &result.statements[0].expression {
            Expression::Let { pattern, value } => {
                match pattern {
                    crate::parser::LetPattern::Identifier(name) => assert_eq!(name, "result"),
                    _ => panic!("Expected identifier pattern"),
                }
                match &**value {
                    Expression::Parenthesized(inner) => match &**inner {
                        Expression::Lambda { .. } => {}
                        _ => panic!("Expected lambda"),
                    },
                    _ => panic!("Expected parenthesized lambda"),
                }
            }
            _ => panic!("Expected let expression"),
        }
    }

    #[test]
    fn test_multiple_statements_with_newlines() {
        let input = "let x 10\nlet y 20\nx";
        let result = parse(input).unwrap();

        assert_eq!(result.statements.len(), 3);

        // First statement: let x 10
        match &result.statements[0].expression {
            Expression::Let { .. } => {}
            _ => panic!("Expected let expression"),
        }

        // Second statement: let y 20
        match &result.statements[1].expression {
            Expression::Let { .. } => {}
            _ => panic!("Expected let expression"),
        }

        // Third statement: x
        match &result.statements[2].expression {
            Expression::Identifier(name) => assert_eq!(name, "x"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_block_with_newlines() {
        let input = "'(\n  add 1 2\n  mul 3 4\n  sub 5 6\n)";
        let result = parse(input).unwrap();

        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Block(exprs) => {
                assert_eq!(exprs.len(), 3);

                // Each line should be parsed as an application
                for expr in exprs {
                    match expr {
                        Expression::Application { .. } => {}
                        _ => panic!("Expected application in block, got {:?}", expr),
                    }
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_lazy_with_newlines() {
        let input = "#(\n  x\n  y\n  z\n)";
        let result = parse(input).unwrap();

        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Lazy(exprs) => {
                assert_eq!(exprs.len(), 3);

                // Each line should be a single identifier
                match &exprs[0] {
                    Expression::Identifier(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier"),
                }
                match &exprs[1] {
                    Expression::Identifier(name) => assert_eq!(name, "y"),
                    _ => panic!("Expected identifier"),
                }
                match &exprs[2] {
                    Expression::Identifier(name) => assert_eq!(name, "z"),
                    _ => panic!("Expected identifier"),
                }
            }
            _ => panic!("Expected lazy block"),
        }
    }

    #[test]
    fn test_empty_lines_ignored() {
        let input = "x\n\n\ny\n\nz";
        let result = parse(input).unwrap();

        assert_eq!(result.statements.len(), 3);

        match &result.statements[0].expression {
            Expression::Identifier(name) => assert_eq!(name, "x"),
            _ => panic!("Expected identifier"),
        }
        match &result.statements[1].expression {
            Expression::Identifier(name) => assert_eq!(name, "y"),
            _ => panic!("Expected identifier"),
        }
        match &result.statements[2].expression {
            Expression::Identifier(name) => assert_eq!(name, "z"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_operators_allow_continuation() {
        // When operator is at end of line, next line is continuation
        let input = "x $>\nf\ny";
        let result = parse(input).unwrap();

        // Should parse as: (x $> f) and y
        assert_eq!(result.statements.len(), 2);

        match &result.statements[0].expression {
            Expression::Pipe { left, right } => {
                match &**left {
                    Expression::Identifier(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier x"),
                }
                match &**right {
                    Expression::Identifier(name) => assert_eq!(name, "f"),
                    _ => panic!("Expected identifier f"),
                }
            }
            _ => panic!(
                "Expected pipe expression, got {:?}",
                result.statements[0].expression
            ),
        }
        match &result.statements[1].expression {
            Expression::Identifier(name) => assert_eq!(name, "y"),
            _ => panic!("Expected identifier y"),
        }
    }

    #[test]
    fn test_operators_break_without_continuation() {
        // When operator is NOT at end of line, newline breaks the statement
        let input = "x\n$> f";

        // Should parse as two separate statements: x and ($> f) which is invalid
        // This should actually fail to parse since $> needs a left operand
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_block_with_pipe_on_same_line() {
        let input = "'(\n  x $> f\n  y $> g\n)";
        let result = parse(input).unwrap();

        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Block(exprs) => {
                assert_eq!(exprs.len(), 2);

                // First line: x $> f
                match &exprs[0] {
                    Expression::Pipe { left, right } => {
                        match &**left {
                            Expression::Identifier(name) => assert_eq!(name, "x"),
                            _ => panic!("Expected identifier x"),
                        }
                        match &**right {
                            Expression::Identifier(name) => assert_eq!(name, "f"),
                            _ => panic!("Expected identifier f"),
                        }
                    }
                    _ => panic!("Expected pipe expression"),
                }

                // Second line: y $> g
                match &exprs[1] {
                    Expression::Pipe { left, right } => {
                        match &**left {
                            Expression::Identifier(name) => assert_eq!(name, "y"),
                            _ => panic!("Expected identifier y"),
                        }
                        match &**right {
                            Expression::Identifier(name) => assert_eq!(name, "g"),
                            _ => panic!("Expected identifier g"),
                        }
                    }
                    _ => panic!("Expected pipe expression"),
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_complex_nested_structure() {
        let input = "let process (fn [data] '(\n  let cleaned trim data\n  save cleaned\n))";
        let result = parse(input).unwrap();

        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Let { pattern: _, value } => match &**value {
                Expression::Parenthesized(inner) => match &**inner {
                    Expression::Lambda { body, .. } => match &**body {
                        Expression::Block(exprs) => {
                            assert_eq!(exprs.len(), 2);
                        }
                        _ => panic!("Expected block in lambda body"),
                    },
                    _ => panic!("Expected lambda"),
                },
                _ => panic!("Expected parenthesized expression"),
            },
            _ => panic!("Expected let expression"),
        }
    }

    #[test]
    fn test_block_chain_operator_binds_to_full_application() {
        // This tests the bug fix: inside a block, the chain operator ($) should
        // bind to the entire application chain built so far, not just the last primary.
        //
        // `if true $ print 1` should parse as:
        // Chain { left: Application(if, true), right: Application(print, 1) }
        //
        // NOT as:
        // Application { function: if, argument: Chain { left: true, right: Application(print, 1) } }

        let input = "'(\n  if true $\n    print 1\n)";
        let result = parse(input).unwrap();

        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Block(exprs) => {
                assert_eq!(exprs.len(), 1);

                match &exprs[0] {
                    Expression::Chain { left, right } => {
                        // Left side should be: Application(if, true)
                        match &**left {
                            Expression::Application { function, argument } => {
                                match &**function {
                                    Expression::Identifier(name) => assert_eq!(name, "if"),
                                    _ => panic!("Expected 'if' identifier as function"),
                                }
                                match &**argument {
                                    Expression::Identifier(name) => assert_eq!(name, "true"),
                                    _ => panic!("Expected 'true' identifier as argument"),
                                }
                            }
                            _ => {
                                panic!("Expected Application on left side of Chain, got {:?}", left)
                            }
                        }

                        // Right side should be: Application(print, 1)
                        match &**right {
                            Expression::Application { function, argument } => {
                                match &**function {
                                    Expression::Identifier(name) => assert_eq!(name, "print"),
                                    _ => panic!("Expected 'print' identifier as function"),
                                }
                                match &**argument {
                                    Expression::Number(n) => assert_eq!(*n, 1.0),
                                    _ => panic!("Expected number 1 as argument"),
                                }
                            }
                            _ => panic!(
                                "Expected Application on right side of Chain, got {:?}",
                                right
                            ),
                        }
                    }
                    _ => panic!("Expected Chain expression in block, got {:?}", exprs[0]),
                }
            }
            _ => panic!("Expected Block expression"),
        }
    }

    #[test]
    fn test_block_chain_single_line() {
        // Same test but on a single line within the block
        let input = "'(\n  a b $ c d\n)";
        let result = parse(input).unwrap();

        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Block(exprs) => {
                assert_eq!(exprs.len(), 1);

                match &exprs[0] {
                    Expression::Chain { left, right } => {
                        // Left: Application(a, b)
                        match &**left {
                            Expression::Application { function, argument } => {
                                match &**function {
                                    Expression::Identifier(name) => assert_eq!(name, "a"),
                                    _ => panic!("Expected 'a' identifier"),
                                }
                                match &**argument {
                                    Expression::Identifier(name) => assert_eq!(name, "b"),
                                    _ => panic!("Expected 'b' identifier"),
                                }
                            }
                            _ => panic!("Expected Application on left"),
                        }

                        // Right: Application(c, d)
                        match &**right {
                            Expression::Application { function, argument } => {
                                match &**function {
                                    Expression::Identifier(name) => assert_eq!(name, "c"),
                                    _ => panic!("Expected 'c' identifier"),
                                }
                                match &**argument {
                                    Expression::Identifier(name) => assert_eq!(name, "d"),
                                    _ => panic!("Expected 'd' identifier"),
                                }
                            }
                            _ => panic!("Expected Application on right"),
                        }
                    }
                    _ => panic!("Expected Chain expression"),
                }
            }
            _ => panic!("Expected Block expression"),
        }
    }

    #[test]
    fn test_lazy_chain_operator_binds_to_full_application() {
        // Same test for lazy blocks
        let input = "#(\n  if true $\n    print 1\n)";
        let result = parse(input).unwrap();

        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Lazy(exprs) => {
                assert_eq!(exprs.len(), 1);

                match &exprs[0] {
                    Expression::Chain { left, right } => {
                        match &**left {
                            Expression::Application { .. } => {}
                            _ => panic!("Expected Application on left side of Chain"),
                        }
                        match &**right {
                            Expression::Application { .. } => {}
                            _ => panic!("Expected Application on right side of Chain"),
                        }
                    }
                    _ => panic!("Expected Chain expression in lazy block"),
                }
            }
            _ => panic!("Expected Lazy expression"),
        }
    }

    #[test]
    fn test_block_multiple_operators_same_line() {
        // Test: a b $ c d $> e f
        // Should parse as: Pipe { left: Chain { left: App(a,b), right: App(c,d) }, right: App(e,f) }
        let input = "'(\n  a b $ c d $> e f\n)";
        let result = parse(input).unwrap();

        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Block(exprs) => {
                assert_eq!(exprs.len(), 1);

                match &exprs[0] {
                    Expression::Pipe { left, right } => {
                        // Left should be Chain { left: App(a,b), right: App(c,d) }
                        match &**left {
                            Expression::Chain { .. } => {}
                            _ => panic!("Expected Chain on left side of Pipe, got {:?}", left),
                        }
                        // Right should be App(e, f)
                        match &**right {
                            Expression::Application { .. } => {}
                            _ => panic!("Expected Application on right side of Pipe"),
                        }
                    }
                    _ => panic!("Expected Pipe expression, got {:?}", exprs[0]),
                }
            }
            _ => panic!("Expected Block expression"),
        }
    }
}
