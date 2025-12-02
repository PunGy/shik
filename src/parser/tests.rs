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
        let input = "'(x y z)";
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Block(exprs) => {
                assert_eq!(exprs.len(), 3);
                match &exprs[0] {
                    Expression::Identifier(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier in block"),
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_parse_lazy() {
        let input = "#(x y)";
        let result = parse(input).unwrap();
        assert_eq!(result.statements.len(), 1);

        match &result.statements[0].expression {
            Expression::Lazy(exprs) => {
                assert_eq!(exprs.len(), 2);
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
}
