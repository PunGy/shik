use crate::parser::{parse, ParseError};
use rustyline::validate::ValidationResult;

pub struct ShikValidator;

impl ShikValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, input: &str) -> rustyline::Result<ValidationResult> {
        if input.trim().is_empty() {
            return Ok(ValidationResult::Valid(None));
        }

        match parse(input) {
            Ok(_) => Ok(ValidationResult::Valid(None)),
            Err(ParseError::UnexpectedEndOfInput { .. })
            | Err(ParseError::UnterminatedString { .. })
            | Err(ParseError::UnterminatedInterpolationString { .. }) => {
                Ok(ValidationResult::Incomplete)
            }
            // Any other parse error: submit so the evaluator can show the error
            Err(_) => Ok(ValidationResult::Valid(None)),
        }
    }
}
