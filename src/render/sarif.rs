//! The SARIF 2.1.0 log.

use std::io;

use serde_json::{Value, json};

use crate::report::{CheckReport, TOOL_VERSION};

const SARIF_SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";
const TOOL_NAME: &str = "cargo-fallow";
const INFORMATION_URI: &str = "https://github.com/fallow-rs/cargo-fallow";

/// The report as a SARIF log. M0 has no findings, so `results` is empty.
pub fn render(report: &CheckReport) -> io::Result<String> {
    serde_json::to_string_pretty(&log(report)).map_err(io::Error::other)
}

fn log(report: &CheckReport) -> Value {
    json!({
        "$schema": SARIF_SCHEMA,
        "version": SARIF_VERSION,
        "runs": [{
            "tool": {
                "driver": {
                    "name": TOOL_NAME,
                    "version": TOOL_VERSION,
                    "informationUri": INFORMATION_URI,
                    "rules": [],
                }
            },
            "results": report.findings,
        }],
    })
}
