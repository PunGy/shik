use std::borrow::Cow;

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Context;

use super::highlighter::{ShikHighlighter, BOLD_GREEN, PROMPT_CONTINUATION, RESET};
use super::validator::ShikValidator;

pub struct ReplHelper {
    highlighter: ShikHighlighter,
    validator: ShikValidator,
}

impl Default for ReplHelper {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplHelper {
    pub fn new() -> Self {
        Self {
            highlighter: ShikHighlighter::new(),
            validator: ShikValidator::new(),
        }
    }
}

impl rustyline::Helper for ReplHelper {}

impl Completer for ReplHelper {
    type Candidate = Pair;
    // No completions yet — ready to extend.
}

impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            // Main prompt: bold green.
            Cow::Owned(format!("{}{}{}", BOLD_GREEN, prompt, RESET))
        } else {
            // Continuation prompt: same width as "> " (2 chars).
            Cow::Owned(format!("{}{}{}", BOLD_GREEN, PROMPT_CONTINUATION, RESET))
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        // Always re-render for real-time syntax highlighting.
        true
    }
}

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        self.validator.validate(ctx.input())
    }
}
