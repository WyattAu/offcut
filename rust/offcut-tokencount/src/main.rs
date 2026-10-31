//! offcut-tokencount: transcript token estimation for LLM cost ceilings.
//!
//! Pins `simd-tokenizer` (WyattAu estate). The selection stage budgets by
//! token cost before sending a transcript window to any model; this gives
//! the Python worker a single measured number instead of a chars/4 guess.
//!
//! Usage: read transcript on stdin (or --file <path>), emit one JSON line:
//!   {"tokens": N, "chars": N, "splits": N, "method": "simd-tokenizer"}
//!
//! Exit codes: 0 ok, 2 usage/IO error.

use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let input = match args.iter().position(|a| a == "--file") {
        Some(p) => match std::fs::read_to_string(args.get(p + 1).map(String::as_str).unwrap_or("")) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read failed: {e}");
                std::process::exit(2);
            }
        },
        None => {
            let mut buf = String::new();
            if std::io::stdin().read_to_string(&mut buf).is_err() {
                eprintln!("stdin read failed");
                std::process::exit(2);
            }
            buf
        }
    };

    let tokenizer = simd_tokenizer::SimdWhitespaceTokenizer::new();
    let bytes = input.as_bytes();
    let splits = tokenizer.count_splits(bytes);
    let tokens = simd_tokenizer::estimate_from_whitespace_splits(splits, bytes.len());

    println!(
        "{}",
        serde_json::json!({
            "tokens": tokens,
            "chars": input.chars().count(),
            "splits": splits,
            "method": "simd-tokenizer",
        })
    );
}
