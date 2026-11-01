//! The gates of a `check` run.
//!
//! The gate names use the same spelling as `GateName` in fallow. The two
//! lists stay equal by convention until the protocol crate owns the names.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

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

/// The gate outcomes of a run and the target root files that no gate judged.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Evaluation {
    /// The gate outcomes.
    pub outcomes: GateOutcomes,
    /// The target root files that could not be read, sorted by path.
    pub unreadable: Vec<UnreadableFile>,
}

/// A target root file that could not be read as UTF-8 text.
///
/// The `parse-error` gate does not judge such a file. The gate contract has
/// no reason for it, and fallow reports the same case as a
/// `source-read-failure` diagnostic, not as a gate file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnreadableFile {
    /// The file path, relative to the workspace root, with `/` separators.
    pub path: String,
    /// The I/O or UTF-8 error.
    pub error: String,
}

/// The result of the parse of one target root file.
#[derive(Debug, Clone, PartialEq, Eq)]
enum RootParse {
    /// The file parsed.
    Parsed,
    /// The file could not be read as UTF-8 text.
    Unreadable(String),
    /// The parser stopped at an error.
    Failed {
        /// The number of parser errors.
        error_count: u32,
    },
}

/// Computes every gate for the workspace.
pub fn evaluate(workspace: &Workspace, options: &Options) -> Evaluation {
    let mut outcomes = GateOutcomes::default();
    // M0 has no checks, so no finding has the `error` severity.
    outcomes.insert(
        RustGateName::ErrorSeverityFindings,
        GateOutcome::counted(GateStatus::Pass, true, 0.0, ERROR_SEVERITY),
    );
    let (parse_outcome, unreadable) = parse_error(workspace, options.fail_on_parse_error);
    outcomes.insert(RustGateName::ParseError, parse_outcome);
    Evaluation {
        outcomes,
        unreadable,
    }
}

/// Parses the root file of each target.
fn parse_error(workspace: &Workspace, enforced: bool) -> (GateOutcome, Vec<UnreadableFile>) {
    let mut roots: Vec<&str> = workspace
        .members
        .iter()
        .flat_map(|member| member.targets.iter().map(|target| target.src_path.as_str()))
        .collect();
    roots.sort_unstable();
    roots.dedup();
    let mut files = Vec::new();
    let mut unreadable = Vec::new();
    for path in roots {
        match parse_root(&workspace::absolute_path(&workspace.root, path)) {
            RootParse::Unreadable(error) => unreadable.push(UnreadableFile {
                path: path.to_owned(),
                error,
            }),
            parsed => files.extend(gate_file(path, &parsed)),
        }
    }
    let status = if files.is_empty() {
        GateStatus::Pass
    } else {
        GateStatus::Fail
    };
    (GateOutcome::with_files(status, enforced, files), unreadable)
}

/// The `parse-error` gate item of one file, or `None` when the gate does not
/// count the file.
fn gate_file(path: &str, parsed: &RootParse) -> Option<GateFile> {
    match *parsed {
        RootParse::Parsed | RootParse::Unreadable(_) => None,
        // syn stops at the first error and does not recover, so the parse
        // always stopped in the file.
        RootParse::Failed { error_count } => Some(GateFile {
            path: path.to_owned(),
            error_count,
            panicked: true,
            reason: None,
        }),
    }
}

fn parse_root(path: &Path) -> RootParse {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => return RootParse::Unreadable(err.to_string()),
    };
    match syn::parse_file(&source) {
        Ok(_) => RootParse::Parsed,
        Err(err) => RootParse::Failed {
            error_count: u32::try_from(err.into_iter().count()).unwrap_or(u32::MAX),
        },
    }
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
    fn gate_file_follows_the_contract_for_each_parse_result() {
        assert_eq!(gate_file("a.rs", &RootParse::Parsed), None);
        assert_eq!(
            gate_file("a.rs", &RootParse::Unreadable("denied".to_owned())),
            None
        );
        assert_eq!(
            gate_file("a.rs", &RootParse::Failed { error_count: 2 }),
            Some(GateFile {
                path: "a.rs".to_owned(),
                error_count: 2,
                panicked: true,
                reason: None,
            })
        );
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
