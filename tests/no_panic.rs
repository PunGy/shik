//! Property test: lexer + parser must never panic, whatever the input.
//!
//! Catches the `"{}"` / `"(".repeat(100_000)` class of crashes permanently.
//! Failures found by proptest are persisted in `proptest-regressions/` and
//! replayed on every run — commit that directory if it appears.

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn parse_never_panics_on_arbitrary_input(input in any::<String>()) {
        let _ = shik::parser::parse(&input);
    }

    #[test]
    fn parse_never_panics_on_ascii_soup(input in "[ -~\\n]{0,128}") {
        // Printable ASCII is much denser in shik punctuation ((), [], {}, $, #, ", :)
        // than fully arbitrary unicode, so this hits parser edge cases harder.
        let _ = shik::parser::parse(&input);
    }
}
