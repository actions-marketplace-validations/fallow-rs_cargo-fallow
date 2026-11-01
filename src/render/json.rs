//! The JSON report.

use std::io;

use crate::report::CheckReport;

/// The report as pretty JSON.
pub fn render(report: &CheckReport) -> io::Result<String> {
    serde_json::to_string_pretty(report).map_err(io::Error::other)
}
