use std::borrow::Cow;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};
use tree_sitter_shik::HIGHLIGHTS_QUERY;

// ANSI color codes
pub const RESET: &str = "\x1b[0m";
const BOLD_BLUE: &str = "\x1b[1;34m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DIM_GRAY: &str = "\x1b[2;37m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const BOLD_CYAN: &str = "\x1b[1;36m";
const BOLD_WHITE: &str = "\x1b[1;37m";

// Prompt constants exported for use in helper.rs
pub const BOLD_GREEN: &str = "\x1b[1;32m";
pub const PROMPT: &str = "> ";
pub const PROMPT_CONTINUATION: &str = "… ";

pub struct ShikHighlighter {
    language: Language,
    query: Query,
}

impl Default for ShikHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl ShikHighlighter {
    pub fn new() -> Self {
        let language: Language = tree_sitter_shik::LANGUAGE.into();
        let query =
            Query::new(&language, HIGHLIGHTS_QUERY).expect("Failed to compile highlights query");
        Self { language, query }
    }

    pub fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        match self.try_highlight(line) {
            Some(highlighted) => Cow::Owned(highlighted),
            None => Cow::Borrowed(line),
        }
    }

    fn try_highlight(&self, input: &str) -> Option<String> {
        if input.is_empty() {
            return None;
        }

        let mut parser = Parser::new();
        parser.set_language(&self.language).ok()?;
        let tree = parser.parse(input, None)?;

        let src = input.as_bytes();
        let mut cursor = QueryCursor::new();
        let mut captures = cursor.captures(&self.query, tree.root_node(), src);

        // Collect (start_byte, end_byte, color) spans.
        // For overlapping captures, only the first one wins (tree-sitter's priority order).
        let mut spans: Vec<(usize, usize, &'static str)> = Vec::new();
        while let Some((m, cap_idx)) = captures.next() {
            let cap = &m.captures[*cap_idx];
            let name = &self.query.capture_names()[cap.index as usize];
            let color = capture_name_to_color(name);
            if !color.is_empty() {
                let range = cap.node.byte_range();
                if range.start < range.end {
                    spans.push((range.start, range.end, color));
                }
            }
        }

        // Sort by start byte; remove spans that overlap with already-covered ranges.
        spans.sort_by_key(|(start, _, _)| *start);

        let mut result = String::with_capacity(input.len() * 2);
        let mut pos: usize = 0; // current byte position in `input`

        for (start, end, color) in &spans {
            if *start < pos {
                // Already covered by a previous (higher-priority) span; skip.
                continue;
            }
            // Emit uncolored gap between previous span and this one.
            result.push_str(&input[pos..*start]);
            // Emit colored span.
            result.push_str(color);
            result.push_str(&input[*start..*end]);
            result.push_str(RESET);
            pos = *end;
        }

        // Emit any remaining text after the last span.
        result.push_str(&input[pos..]);

        Some(result)
    }
}

fn capture_name_to_color(name: &str) -> &'static str {
    match name {
        "comment" => DIM_GRAY,
        "string" | "string.escape" => GREEN,
        "number" => YELLOW,
        "keyword" => BOLD_BLUE,
        "punctuation.bracket" => CYAN,
        "punctuation.special" => BOLD_CYAN,
        "function" => BOLD_WHITE,
        "variable.parameter" => CYAN,
        "boolean" | "constant.builtin" => MAGENTA,
        _ => "",
    }
}
