#[cfg(test)]
mod tests {
    use crate::parser::error::ParseError;
    use crate::parser::lexer::Lexer;
    use crate::parser::tokens::TokenType;

    /// Helper function to tokenize input and extract string content from the first token
    fn tokenize_string(input: &str) -> Result<String, ParseError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        match &tokens[0].token_type {
            TokenType::String(s) => Ok(s.clone()),
            _ => panic!("Expected string token, got {:?}", tokens[0].token_type),
        }
    }

    /// Helper function to tokenize and return all tokens
    fn tokenize(input: &str) -> Result<Vec<crate::parser::tokens::Token>, ParseError> {
        let mut lexer = Lexer::new(input);
        lexer.tokenize()
    }

    // ==================== Basic String Tests ====================

    mod block_strings {
        use super::*;

        #[test]
        fn simple_string() {
            let result = tokenize_string(r#""hello world""#).unwrap();
            assert_eq!(result, "hello world");
        }

        #[test]
        fn empty_string() {
            let result = tokenize_string(r#""""#).unwrap();
            assert_eq!(result, "");
        }

        #[test]
        fn string_with_spaces() {
            let result = tokenize_string(r#""  spaces  around  ""#).unwrap();
            assert_eq!(result, "  spaces  around  ");
        }

        #[test]
        fn string_with_numbers() {
            let result = tokenize_string(r#""test123""#).unwrap();
            assert_eq!(result, "test123");
        }
    }

    mod inline_strings {
        use super::*;

        #[test]
        fn simple_inline_string() {
            let result = tokenize_string(":hello").unwrap();
            assert_eq!(result, "hello");
        }

        #[test]
        fn inline_string_with_hyphen() {
            let result = tokenize_string(":hello-world").unwrap();
            assert_eq!(result, "hello-world");
        }

        #[test]
        fn inline_string_terminates_at_space() {
            let tokens = tokenize(":hello world").unwrap();
            match &tokens[0].token_type {
                TokenType::String(s) => assert_eq!(s, "hello"),
                _ => panic!("Expected string token"),
            }
        }

        #[test]
        fn inline_string_terminates_at_newline() {
            let tokens = tokenize(":hello\nworld").unwrap();
            match &tokens[0].token_type {
                TokenType::String(s) => assert_eq!(s, "hello"),
                _ => panic!("Expected string token"),
            }
        }
    }

    // ==================== Escape Sequence Tests ====================

    mod escape_sequences_block_string {
        use super::*;

        #[test]
        fn escape_newline() {
            let result = tokenize_string(r#""hello\nworld""#).unwrap();
            assert_eq!(result, "hello\nworld");
        }

        #[test]
        fn escape_carriage_return() {
            let result = tokenize_string(r#""hello\rworld""#).unwrap();
            assert_eq!(result, "hello\rworld");
        }

        #[test]
        fn escape_tab() {
            let result = tokenize_string(r#""hello\tworld""#).unwrap();
            assert_eq!(result, "hello\tworld");
        }

        #[test]
        fn escape_backslash() {
            let result = tokenize_string(r#""hello\\world""#).unwrap();
            assert_eq!(result, "hello\\world");
        }

        #[test]
        fn escape_double_quote() {
            let result = tokenize_string(r#""hello\"world""#).unwrap();
            assert_eq!(result, "hello\"world");
        }

        #[test]
        fn escape_single_quote() {
            let result = tokenize_string(r#""hello\'world""#).unwrap();
            assert_eq!(result, "hello'world");
        }

        #[test]
        fn escape_null() {
            let result = tokenize_string(r#""hello\0world""#).unwrap();
            assert_eq!(result, "hello\0world");
        }

        #[test]
        fn escape_left_brace() {
            let result = tokenize_string(r#""hello\{world""#).unwrap();
            assert_eq!(result, "hello{world");
        }

        #[test]
        fn escape_right_brace() {
            let result = tokenize_string(r#""hello\}world""#).unwrap();
            assert_eq!(result, "hello}world");
        }

        #[test]
        fn multiple_escapes() {
            let result = tokenize_string(r#""line1\nline2\ttabbed\r\n""#).unwrap();
            assert_eq!(result, "line1\nline2\ttabbed\r\n");
        }

        #[test]
        fn escape_at_start() {
            let result = tokenize_string(r#""\nhello""#).unwrap();
            assert_eq!(result, "\nhello");
        }

        #[test]
        fn escape_at_end() {
            let result = tokenize_string(r#""hello\n""#).unwrap();
            assert_eq!(result, "hello\n");
        }

        #[test]
        fn consecutive_escapes() {
            let result = tokenize_string(r#""\n\n\n""#).unwrap();
            assert_eq!(result, "\n\n\n");
        }
    }

    mod escape_sequences_inline_string {
        use super::*;

        #[test]
        fn escape_newline() {
            let result = tokenize_string(r#":hello\nworld"#).unwrap();
            assert_eq!(result, "hello\nworld");
        }

        #[test]
        fn escape_tab() {
            let result = tokenize_string(r#":hello\tworld"#).unwrap();
            assert_eq!(result, "hello\tworld");
        }

        #[test]
        fn escape_backslash() {
            let result = tokenize_string(r#":hello\\world"#).unwrap();
            assert_eq!(result, "hello\\world");
        }

        #[test]
        fn escape_space_in_inline_string() {
            // Escaping space allows it to be part of the inline string
            let result = tokenize_string(r#":hello\ world"#).unwrap();
            assert_eq!(result, "hello world");
        }

        #[test]
        fn escape_newline_literal_in_inline_string() {
            // Escaping actual newline character allows continuation
            let tokens = tokenize(":hello\\\nworld").unwrap();
            match &tokens[0].token_type {
                TokenType::String(s) => assert_eq!(s, "hello\nworld"),
                _ => panic!("Expected string token"),
            }
        }

        #[test]
        fn escape_left_paren() {
            let result = tokenize_string(r#":hello\(world"#).unwrap();
            assert_eq!(result, "hello(world");
        }

        #[test]
        fn escape_right_paren() {
            let result = tokenize_string(r#":hello\)world"#).unwrap();
            assert_eq!(result, "hello)world");
        }

        #[test]
        fn escape_left_bracket() {
            let result = tokenize_string(r#":hello\[world"#).unwrap();
            assert_eq!(result, "hello[world");
        }

        #[test]
        fn escape_right_bracket() {
            let result = tokenize_string(r#":hello\]world"#).unwrap();
            assert_eq!(result, "hello]world");
        }

        #[test]
        fn escape_left_brace() {
            let result = tokenize_string(r#":hello\{world"#).unwrap();
            assert_eq!(result, "hello{world");
        }

        #[test]
        fn escape_right_brace() {
            let result = tokenize_string(r#":hello\}world"#).unwrap();
            assert_eq!(result, "hello}world");
        }
    }

    mod hex_escape_sequences {
        use super::*;

        #[test]
        fn hex_escape_lowercase() {
            let result = tokenize_string(r#""\x41""#).unwrap();
            assert_eq!(result, "A");
        }

        #[test]
        fn hex_escape_uppercase() {
            let result = tokenize_string(r#""\x4F""#).unwrap();
            assert_eq!(result, "O");
        }

        #[test]
        fn hex_escape_null() {
            let result = tokenize_string(r#""\x00""#).unwrap();
            assert_eq!(result, "\0");
        }

        #[test]
        fn hex_escape_max_byte() {
            let result = tokenize_string(r#""\xff""#).unwrap();
            assert_eq!(result, "\u{ff}");
        }

        #[test]
        fn hex_escape_newline() {
            let result = tokenize_string(r#""\x0a""#).unwrap();
            assert_eq!(result, "\n");
        }

        #[test]
        fn hex_escape_tab() {
            let result = tokenize_string(r#""\x09""#).unwrap();
            assert_eq!(result, "\t");
        }

        #[test]
        fn hex_escape_in_context() {
            let result = tokenize_string(r#""hello\x20world""#).unwrap();
            assert_eq!(result, "hello world");
        }

        #[test]
        fn multiple_hex_escapes() {
            let result = tokenize_string(r#""\x48\x65\x6c\x6c\x6f""#).unwrap();
            assert_eq!(result, "Hello");
        }

        #[test]
        fn hex_escape_inline_string() {
            let result = tokenize_string(r#":hello\x20world"#).unwrap();
            assert_eq!(result, "hello world");
        }
    }

    mod unicode_escape_sequences {
        use super::*;

        #[test]
        fn unicode_escape_basic() {
            let result = tokenize_string(r#""\u{41}""#).unwrap();
            assert_eq!(result, "A");
        }

        #[test]
        fn unicode_escape_emoji() {
            let result = tokenize_string(r#""\u{1F600}""#).unwrap();
            assert_eq!(result, "😀");
        }

        #[test]
        fn unicode_escape_heart() {
            let result = tokenize_string(r#""\u{2764}""#).unwrap();
            assert_eq!(result, "❤");
        }

        #[test]
        fn unicode_escape_chinese() {
            let result = tokenize_string(r#""\u{4E2D}""#).unwrap();
            assert_eq!(result, "中");
        }

        #[test]
        fn unicode_escape_lowercase_hex() {
            let result = tokenize_string(r#""\u{1f600}""#).unwrap();
            assert_eq!(result, "😀");
        }

        #[test]
        fn unicode_escape_mixed_case() {
            let result = tokenize_string(r#""\u{1F60A}""#).unwrap();
            assert_eq!(result, "😊");
        }

        #[test]
        fn unicode_escape_single_digit() {
            let result = tokenize_string(r#""\u{A}""#).unwrap();
            assert_eq!(result, "\n");
        }

        #[test]
        fn unicode_escape_in_context() {
            let result = tokenize_string(r#""I \u{2764} Rust""#).unwrap();
            assert_eq!(result, "I ❤ Rust");
        }

        #[test]
        fn multiple_unicode_escapes() {
            let result = tokenize_string(r#""\u{1F44D}\u{1F44D}""#).unwrap();
            assert_eq!(result, "👍👍");
        }

        #[test]
        fn unicode_escape_inline_string() {
            let result = tokenize_string(r#":hello\u{1F600}"#).unwrap();
            assert_eq!(result, "hello😀");
        }
    }

    // ==================== Error Cases ====================

    mod escape_sequence_errors {
        use super::*;

        #[test]
        fn invalid_escape_character() {
            let result = tokenize(r#""\q""#);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidEscapeSequence { sequence, .. } => {
                    assert_eq!(sequence, "q");
                }
                e => panic!("Expected InvalidEscapeSequence error, got {:?}", e),
            }
        }

        #[test]
        fn invalid_hex_escape_short() {
            let result = tokenize(r#""\x4""#);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidEscapeSequence { sequence, .. } => {
                    assert!(sequence.starts_with("x"));
                }
                e => panic!("Expected InvalidEscapeSequence error, got {:?}", e),
            }
        }

        #[test]
        fn invalid_hex_escape_non_hex() {
            let result = tokenize(r#""\xGG""#);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidEscapeSequence { sequence, .. } => {
                    assert!(sequence.starts_with("x"));
                }
                e => panic!("Expected InvalidEscapeSequence error, got {:?}", e),
            }
        }

        #[test]
        fn invalid_unicode_escape_no_brace() {
            let result = tokenize(r#""\u41""#);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidEscapeSequence { sequence, .. } => {
                    assert_eq!(sequence, "u");
                }
                e => panic!("Expected InvalidEscapeSequence error, got {:?}", e),
            }
        }

        #[test]
        fn invalid_unicode_escape_unclosed() {
            let result = tokenize(r#""\u{41""#);
            assert!(result.is_err());
        }

        #[test]
        fn invalid_unicode_escape_empty() {
            let result = tokenize(r#""\u{}""#);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidEscapeSequence { sequence, .. } => {
                    assert!(sequence.contains("u{"));
                }
                e => panic!("Expected InvalidEscapeSequence error, got {:?}", e),
            }
        }

        #[test]
        fn invalid_unicode_escape_too_long() {
            let result = tokenize(r#""\u{1234567}""#);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidEscapeSequence { sequence, .. } => {
                    assert!(sequence.contains("u{"));
                }
                e => panic!("Expected InvalidEscapeSequence error, got {:?}", e),
            }
        }

        #[test]
        fn invalid_unicode_escape_invalid_codepoint() {
            // U+D800 is a surrogate, not a valid Unicode scalar value
            let result = tokenize(r#""\u{D800}""#);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidEscapeSequence { sequence, .. } => {
                    assert!(sequence.contains("u{"));
                }
                e => panic!("Expected InvalidEscapeSequence error, got {:?}", e),
            }
        }

        #[test]
        fn invalid_unicode_escape_out_of_range() {
            // U+110000 is beyond the valid Unicode range
            let result = tokenize(r#""\u{110000}""#);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidEscapeSequence { sequence, .. } => {
                    assert!(sequence.contains("u{"));
                }
                e => panic!("Expected InvalidEscapeSequence error, got {:?}", e),
            }
        }

        #[test]
        fn escape_at_end_of_string_unterminated() {
            let result = tokenize(r#""hello\"#);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::UnterminatedString { .. } => {}
                e => panic!("Expected UnterminatedString error, got {:?}", e),
            }
        }
    }

    // ==================== String Interpolation Tests ====================

    mod string_interpolation {
        use super::*;

        #[test]
        fn simple_interpolation() {
            let tokens = tokenize(r#""hello {name}""#).unwrap();
            match &tokens[0].token_type {
                TokenType::StringInterpolation(info) => {
                    assert_eq!(info.entries.len(), 1);
                    assert!(info.string.contains('_'));
                }
                _ => panic!("Expected StringInterpolation token"),
            }
        }

        #[test]
        fn interpolation_with_escape_before() {
            let tokens = tokenize(r#""hello\n{name}""#).unwrap();
            match &tokens[0].token_type {
                TokenType::StringInterpolation(info) => {
                    assert!(info.string.starts_with("hello\n"));
                }
                _ => panic!("Expected StringInterpolation token"),
            }
        }

        #[test]
        fn interpolation_with_escape_after() {
            let tokens = tokenize(r#""{name}\nworld""#).unwrap();
            match &tokens[0].token_type {
                TokenType::StringInterpolation(info) => {
                    assert!(info.string.ends_with("\nworld"));
                }
                _ => panic!("Expected StringInterpolation token"),
            }
        }

        #[test]
        fn escaped_brace_not_interpolation() {
            let result = tokenize_string(r#""hello \{name\}""#).unwrap();
            assert_eq!(result, "hello {name}");
        }

        #[test]
        fn mixed_escapes_and_interpolation() {
            let tokens = tokenize(r#""line1\nvalue: {x}\ttab""#).unwrap();
            match &tokens[0].token_type {
                TokenType::StringInterpolation(info) => {
                    assert!(info.string.contains('\n'));
                    assert!(info.string.contains('\t'));
                    assert!(info.string.contains('_'));
                }
                _ => panic!("Expected StringInterpolation token"),
            }
        }
    }

    // ==================== Edge Cases ====================

    mod edge_cases {
        use super::*;

        #[test]
        fn string_with_only_escapes() {
            let result = tokenize_string(r#""\n\t\r""#).unwrap();
            assert_eq!(result, "\n\t\r");
        }

        #[test]
        fn string_with_unicode_and_regular_text() {
            let result = tokenize_string(r#""Hello \u{1F600} World""#).unwrap();
            assert_eq!(result, "Hello 😀 World");
        }

        #[test]
        fn string_with_all_escape_types() {
            let result = tokenize_string(r#""a\nb\tc\x41\u{42}""#).unwrap();
            assert_eq!(result, "a\nb\tcAB");
        }

        #[test]
        fn inline_string_empty_after_colon() {
            // Edge case: colon followed by space
            let tokens = tokenize(": ").unwrap();
            match &tokens[0].token_type {
                TokenType::String(s) => assert_eq!(s, ""),
                _ => panic!("Expected empty string token"),
            }
        }

        #[test]
        fn very_long_escape_sequence() {
            let result = tokenize_string(r#""\n\n\n\n\n\n\n\n\n\n""#).unwrap();
            assert_eq!(result, "\n\n\n\n\n\n\n\n\n\n");
        }

        #[test]
        fn backslash_before_quote_in_block_string() {
            let result = tokenize_string(r#""say \"hello\"""#).unwrap();
            assert_eq!(result, "say \"hello\"");
        }

        #[test]
        fn null_character_in_string() {
            let result = tokenize_string(r#""null\0char""#).unwrap();
            assert_eq!(result, "null\0char");
        }
    }

    // ==================== Lexeme Preservation Tests ====================

    mod lexeme_preservation {
        use super::*;

        #[test]
        fn block_string_lexeme_preserved() {
            let tokens = tokenize(r#""hello\nworld""#).unwrap();
            // The lexeme should contain the original source text
            assert_eq!(tokens[0].lexeme, r#""hello\nworld""#);
            // But the token value should have the processed escape
            match &tokens[0].token_type {
                TokenType::String(s) => assert_eq!(s, "hello\nworld"),
                _ => panic!("Expected string token"),
            }
        }

        #[test]
        fn inline_string_lexeme_preserved() {
            let tokens = tokenize(r#":hello\nworld"#).unwrap();
            assert_eq!(tokens[0].lexeme, r#":hello\nworld"#);
            match &tokens[0].token_type {
                TokenType::String(s) => assert_eq!(s, "hello\nworld"),
                _ => panic!("Expected string token"),
            }
        }
    }

    // ==================== Position Tracking Tests ====================

    mod position_tracking {
        use super::*;

        #[test]
        fn escape_error_reports_correct_position() {
            let result = tokenize(r#""abc\qdef""#);
            match result.unwrap_err() {
                ParseError::InvalidEscapeSequence { column, .. } => {
                    // The backslash is at column 5 (1-indexed: "abc\ is positions 1-4)
                    assert!(column >= 4 && column <= 6, "Column was {}", column);
                }
                e => panic!("Expected InvalidEscapeSequence error, got {:?}", e),
            }
        }

        #[test]
        fn string_token_has_correct_line() {
            let tokens = tokenize("x\n\"hello\"").unwrap();
            // Find the string token
            let string_token = tokens.iter().find(|t| matches!(t.token_type, TokenType::String(_)));
            assert!(string_token.is_some());
            assert_eq!(string_token.unwrap().line, 2);
        }
    }
}