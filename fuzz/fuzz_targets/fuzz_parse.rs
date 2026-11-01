#![no_main]
//! The bounded parse of a target root file. A workspace under analysis
//! controls the source, so no input may crash the process: deep nesting,
//! a parser panic and a very long input each give a parse outcome.

use libfuzzer_sys::fuzz_target;

// The module uses only std, ra_ap_parser and ra_ap_syntax, so the target includes the production
// source and needs no public library API.
#[path = "../../src/parse.rs"]
mod parse;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let _ = parse::parse_source(source, parse::Edition::Edition2024);
});
