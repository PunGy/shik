//! Golden tests: every `tests/cases/**/*.shk` file is evaluated with a fresh
//! interpreter and its final value (or error) is snapshotted with insta.
//!
//! Workflow for a new behavioral test: drop a `.shk` file under `tests/cases/`,
//! run `cargo test --test golden`, then review snapshots with `cargo insta review`
//! (or set `INSTA_UPDATE=always` and inspect the diff).
//!
//! Note: multi-key objects must not appear in snapshotted values — `Display`
//! iterates a `HashMap`, so their key order is non-deterministic.

use shik::eval::evaluator::Interpretator;
use shik::lang::evaluate;

#[test]
fn golden_cases() {
    insta::glob!("cases/**/*.shk", |path| {
        let source = std::fs::read_to_string(path).expect("case file is readable");
        let interpretator = Interpretator::new();
        let rendered = match evaluate(&source, &interpretator) {
            Ok(value) => value.to_string(),
            Err(e) => format!("ERROR: {e}"),
        };
        insta::assert_snapshot!(rendered);
    });
}
