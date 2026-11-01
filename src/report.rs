//! The report of a `check` run.

use std::collections::BTreeMap;

use fallow_report_protocol::GateOutcome;
use serde::Serialize;

use crate::gates::{GateOutcomes, RustGateName};
use crate::workspace::{Member, Workspace};

/// The version of the report shape. A change that removes a member or
/// changes its type increments it.
pub const SCHEMA_VERSION: u32 = 1;

/// The package version, written to the report.
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A finding. No check emits a finding in M0, so the type has no values.
#[derive(Debug, Clone, Serialize)]
pub enum Finding {}

/// The report of a `check` run. The field order is the wire order.
#[derive(Debug, Clone, Serialize)]
pub struct CheckReport {
    /// The version of the report shape.
    pub schema_version: u32,
    /// The version of cargo-fallow.
    pub version: &'static str,
    /// Always `check`.
    pub kind: &'static str,
    /// Always `rust`.
    pub language: &'static str,
    /// The workspace members.
    pub workspace: WorkspaceSummary,
    /// The number of findings.
    pub total_issues: usize,
    /// The findings.
    pub findings: Vec<Finding>,
    /// The gate outcomes, keyed by gate name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_outcomes: Option<BTreeMap<RustGateName, GateOutcome>>,
}

/// The workspace part of the report.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceSummary {
    /// The members, sorted by name.
    pub members: Vec<Member>,
}

impl CheckReport {
    /// Builds the report of one run.
    pub fn new(workspace: Workspace, outcomes: GateOutcomes) -> Self {
        let findings: Vec<Finding> = Vec::new();
        Self {
            schema_version: SCHEMA_VERSION,
            version: TOOL_VERSION,
            kind: "check",
            language: "rust",
            workspace: WorkspaceSummary {
                members: workspace.members,
            },
            total_issues: findings.len(),
            findings,
            gate_outcomes: outcomes.into_option(),
        }
    }

    /// The number of targets in all members.
    pub fn target_count(&self) -> usize {
        self.workspace
            .members
            .iter()
            .map(|member| member.targets.len())
            .sum()
    }
}
