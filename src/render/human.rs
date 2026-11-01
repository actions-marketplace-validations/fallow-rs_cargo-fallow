//! The summary for a person.

use std::fmt::Write;

use fallow_report_protocol::{GateOutcome, GateStatus};

use crate::report::CheckReport;

/// The report as a short summary.
pub fn render(report: &CheckReport) -> String {
    let members = report.workspace.members.len();
    let targets = report.target_count();
    let mut text = format!(
        "cargo fallow check: {members} {}, {targets} {}\n",
        plural(members, "member", "members"),
        plural(targets, "target", "targets"),
    );
    if report.total_issues == 0 {
        text.push_str("No issues found\n");
    }
    text.push_str("Gates:");
    for (name, outcome) in report.gate_outcomes.iter().flatten() {
        // Writing to a String does not fail.
        let _ = write!(text, "\n  {}: {}", name.as_str(), gate_line(outcome));
        for file in &outcome.files {
            let _ = write!(text, "\n    {}", file.path);
        }
    }
    text
}

fn gate_line(outcome: &GateOutcome) -> String {
    let status = match outcome.status {
        GateStatus::Pass => "pass",
        GateStatus::Warn => "warn",
        GateStatus::Fail => "fail",
        GateStatus::Skipped => "skipped",
    };
    let mut line = status.to_owned();
    if !outcome.files.is_empty() {
        let count = outcome.files.len();
        let _ = write!(line, " ({count} {})", plural(count, "file", "files"));
    }
    if !outcome.enforced {
        line.push_str(", not enforced");
    }
    line
}

const fn plural<'a>(count: usize, one: &'a str, many: &'a str) -> &'a str {
    if count == 1 { one } else { many }
}
