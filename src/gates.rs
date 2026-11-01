//! The gates of a `check` run.
//!
//! The gate names use the same spelling as `GateName` in fallow. The two
//! lists stay equal by convention until the protocol crate owns the names.

use std::collections::BTreeMap;
use std::fs;

use fallow_report_protocol::{GateFile, GateOutcome, GateStatus};
use serde::Serialize;

use crate::cli::Options;
use crate::workspace::{self, Workspace};

/// The rule severity that `error-severity-findings` counts.
const ERROR_SEVERITY: &str = "error";

/// The name of a gate, as it appears in `gate_outcomes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustGateName {
    /// Fails when the run has a finding with the `error` severity.
    ErrorSeverityFindings,
    /// Fails when a target root file does not parse.
    ParseError,
}

/// The gate outcomes of one run, sorted by gate name.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GateOutcomes(BTreeMap<RustGateName, GateOutcome>);

impl GateOutcomes {
    /// Sets the outcome of one gate.
    pub fn insert(&mut self, name: RustGateName, outcome: GateOutcome) {
        self.0.insert(name, outcome);
    }

    /// True when one outcome fails the run.
    pub fn fails_run(&self) -> bool {
        self.0.values().any(GateOutcome::fails_run)
    }

    /// The outcomes, or `None` when no gate reported.
    pub fn into_option(self) -> Option<BTreeMap<RustGateName, GateOutcome>> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0)
        }
    }
}

impl RustGateName {
    /// The wire spelling of the name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ErrorSeverityFindings => "error-severity-findings",
            Self::ParseError => "parse-error",
        }
    }
}

/// Computes every gate for the workspace.
pub fn evaluate(workspace: &Workspace, options: &Options) -> GateOutcomes {
    let mut outcomes = GateOutcomes::default();
    // M0 has no checks, so no finding has the `error` severity.
    outcomes.insert(
        RustGateName::ErrorSeverityFindings,
        GateOutcome::counted(GateStatus::Pass, true, 0.0, ERROR_SEVERITY),
    );
    outcomes.insert(
        RustGateName::ParseError,
        parse_error(workspace, options.fail_on_parse_error),
    );
    outcomes
}

/// Parses the root file of each target. A file that cannot be read counts as
/// a file that does not parse.
fn parse_error(workspace: &Workspace, enforced: bool) -> GateOutcome {
    let mut roots: Vec<&str> = workspace
        .members
        .iter()
        .flat_map(|member| member.targets.iter().map(|target| target.src_path.as_str()))
        .collect();
    roots.sort_unstable();
    roots.dedup();
    let files: Vec<GateFile> = roots
        .into_iter()
        .filter(|path| !parses(&workspace::absolute_path(&workspace.root, path)))
        .map(|path| GateFile {
            path: path.to_owned(),
            error_count: 1,
            panicked: true,
            reason: None,
        })
        .collect();
    let status = if files.is_empty() {
        GateStatus::Pass
    } else {
        GateStatus::Fail
    };
    GateOutcome::with_files(status, enforced, files)
}

fn parses(path: &std::path::Path) -> bool {
    fs::read_to_string(path).is_ok_and(|source| syn::parse_file(&source).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failing(enforced: bool) -> GateOutcome {
        GateOutcome::new(GateStatus::Fail, enforced)
    }

    #[test]
    fn empty_outcomes_do_not_fail_and_give_none() {
        let outcomes = GateOutcomes::default();
        assert!(!outcomes.fails_run());
        assert!(outcomes.into_option().is_none());
    }

    #[test]
    fn only_an_enforced_failure_fails_the_run() {
        let mut outcomes = GateOutcomes::default();
        outcomes.insert(RustGateName::ParseError, failing(false));
        assert!(!outcomes.fails_run());
        outcomes.insert(RustGateName::ErrorSeverityFindings, failing(true));
        assert!(outcomes.fails_run());
    }

    #[test]
    fn insert_replaces_and_into_option_keeps_name_order() {
        let mut outcomes = GateOutcomes::default();
        outcomes.insert(RustGateName::ParseError, failing(true));
        outcomes.insert(
            RustGateName::ParseError,
            GateOutcome::new(GateStatus::Pass, true),
        );
        outcomes.insert(RustGateName::ErrorSeverityFindings, failing(false));
        assert!(!outcomes.fails_run());
        let map = outcomes.into_option().unwrap_or_default();
        let names: Vec<&str> = map.keys().map(|name| name.as_str()).collect();
        assert_eq!(names, ["error-severity-findings", "parse-error"]);
    }

    #[test]
    fn wire_names_match_serde() {
        for name in [
            RustGateName::ErrorSeverityFindings,
            RustGateName::ParseError,
        ] {
            let wire = serde_json::to_value(name).unwrap_or_default();
            assert_eq!(wire, name.as_str());
        }
    }
}
