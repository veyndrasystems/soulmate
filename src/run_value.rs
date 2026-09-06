//! Typed value-proof records derived from a run ledger.
//!
//! This module deliberately does not execute a check command.  A host (or a
//! caller using the CLI) reports the result and the run reducer verifies that
//! the report names the configured command and the exact current worker
//! submission.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

pub(crate) const CHECK_POLICY_VERSION: u64 = 1;
pub(crate) const VALUE_REPORT_VERSION: u64 = 1;

const SHA_LEN: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CheckPolicy {
    pub(crate) command: String,
    pub(crate) command_sha256: String,
    pub(crate) origin: ProofOrigin,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProofOrigin {
    LocalReport,
    Synthetic,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CheckAction {
    Check,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ProtectionReason {
    CheckMissing,
    CheckFailed,
}

impl ProtectionReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::CheckMissing => "check_missing",
            Self::CheckFailed => "check_failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum EvidenceStatus {
    Missing,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckTargetStatus {
    Missing,
    Passed,
    Failed,
}

impl CheckTargetStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckTarget {
    target_event_sha256: String,
    status: CheckTargetStatus,
    check_event_sha256: Option<String>,
    exit_code: Option<u64>,
}

impl CheckTarget {
    fn value(&self) -> Value {
        let mut value = json!({
            "targetEventSha256": self.target_event_sha256,
            "status": self.status.as_str(),
        });
        if let Some(check_event_sha256) = &self.check_event_sha256 {
            value["checkEventSha256"] = json!(check_event_sha256);
        }
        if let Some(exit_code) = self.exit_code {
            value["exitCode"] = json!(exit_code);
        }
        value
    }

    fn is_missing(&self) -> bool {
        self.status == CheckTargetStatus::Missing
    }

    fn is_failed(&self) -> bool {
        self.status == CheckTargetStatus::Failed
    }

    fn is_observed(&self) -> bool {
        self.check_event_sha256.is_some()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckObservation {
    version: u64,
    kind: String,
    producer: Value,
    action: CheckAction,
    #[serde(rename = "runId")]
    run_id: String,
    #[serde(rename = "targetEventSha256")]
    target_event_sha256: String,
    #[serde(rename = "checkCommand")]
    check_command: String,
    #[serde(rename = "checkCommandSha256")]
    check_command_sha256: String,
    origin: ProofOrigin,
    #[serde(rename = "exitCode")]
    _exit_code: u64,
    #[serde(rename = "durationMs")]
    duration_ms: Option<u64>,
    #[serde(rename = "previousEventSha256")]
    previous_event_sha256: Option<String>,
    timestamp: String,
    #[serde(rename = "eventSha256")]
    event_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckEvidence {
    #[serde(rename = "targetEventSha256")]
    target_event_sha256: String,
    status: EvidenceStatus,
    #[serde(rename = "checkEventSha256")]
    check_event_sha256: Option<String>,
    #[serde(rename = "exitCode")]
    exit_code: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtectionRecord {
    version: u64,
    kind: String,
    producer: Value,
    action: ProtectionAction,
    #[serde(rename = "runId")]
    run_id: String,
    stage: u64,
    attempt: u64,
    actor: String,
    role: String,
    #[serde(rename = "attemptedOutcome")]
    attempted_outcome: String,
    reason: ProtectionReason,
    #[serde(rename = "checkEvidence")]
    check_evidence: Vec<CheckEvidence>,
    #[serde(rename = "origin")]
    _origin: ProofOrigin,
    #[serde(rename = "previousEventSha256")]
    previous_event_sha256: Option<String>,
    timestamp: String,
    #[serde(rename = "eventSha256")]
    event_sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ProtectionAction {
    Protect,
}

impl ProofOrigin {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "local_report" => Ok(Self::LocalReport),
            "synthetic" => Ok(Self::Synthetic),
            _ => Err("proof origin must be local_report or synthetic".into()),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::LocalReport => "local_report",
            Self::Synthetic => "synthetic",
        }
    }
}

impl CheckPolicy {
    pub(crate) fn value(&self) -> Value {
        json!({
            "version": CHECK_POLICY_VERSION,
            "command": self.command,
            "commandSha256": self.command_sha256,
            "origin": self.origin.as_str(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckPolicyRecord {
    version: u64,
    command: String,
    #[serde(rename = "commandSha256")]
    command_sha256: String,
    origin: ProofOrigin,
}

/// Parse the optional start-time policy.  A proof origin without a command is
/// intentionally rejected so `--proof-origin` cannot opt a run into a weaker
/// or ambiguous checked state.
pub(crate) fn policy_from_cli(
    command: Option<&str>,
    origin: Option<&str>,
) -> Result<Option<CheckPolicy>, String> {
    let Some(command) = command else {
        if origin.is_some() {
            return Err("--proof-origin requires --check-command".into());
        }
        return Ok(None);
    };
    if command.trim().is_empty() {
        return Err("--check-command requires a non-empty value".into());
    }
    if command.contains('\0') {
        return Err("--check-command must not contain NUL bytes".into());
    }
    let origin = origin
        .map(ProofOrigin::parse)
        .transpose()?
        .unwrap_or(ProofOrigin::LocalReport);
    Ok(Some(CheckPolicy {
        command: command.to_owned(),
        command_sha256: crate::hash::text(command),
        origin,
    }))
}

pub(crate) fn policy_from_value(value: &Value, line: usize) -> Result<CheckPolicy, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("invalid run ledger line {line}: checkPolicy must be an object"))?;
    let allowed = ["version", "command", "commandSha256", "origin"];
    reject_unknown(object, &allowed, line, "checkPolicy")?;
    let record: CheckPolicyRecord = serde_json::from_value(value.clone())
        .map_err(|_| format!("invalid run ledger line {line}: malformed checkPolicy"))?;
    if object.len() != allowed.len()
        || record.version != CHECK_POLICY_VERSION
        || record.command.trim().is_empty()
        || record.command.contains('\0')
        || !is_sha(Some(&record.command_sha256))
    {
        return Err(format!(
            "invalid run ledger line {line}: malformed checkPolicy"
        ));
    }
    if crate::hash::text(&record.command) != record.command_sha256 {
        return Err(format!(
            "invalid run ledger line {line}: checkPolicy command hash mismatch"
        ));
    }
    Ok(CheckPolicy {
        command: record.command,
        command_sha256: record.command_sha256,
        origin: record.origin,
    })
}

pub(crate) fn validate_check_event(event: &Value, line: usize) -> Result<(), String> {
    let record: CheckObservation = serde_json::from_value(event.clone())
        .map_err(|_| format!("invalid run ledger line {line}: malformed check event"))?;
    let object = event
        .as_object()
        .ok_or_else(|| format!("invalid run ledger line {line}: check event must be an object"))?;
    let allowed = [
        "version",
        "kind",
        "producer",
        "action",
        "runId",
        "targetEventSha256",
        "checkCommand",
        "checkCommandSha256",
        "origin",
        "exitCode",
        "durationMs",
        "previousEventSha256",
        "timestamp",
        "eventSha256",
    ];
    reject_unknown(object, &allowed, line, "check")?;
    let required = [
        "version",
        "kind",
        "producer",
        "action",
        "runId",
        "targetEventSha256",
        "checkCommand",
        "checkCommandSha256",
        "origin",
        "exitCode",
        "previousEventSha256",
        "timestamp",
        "eventSha256",
    ];
    if !required.iter().all(|key| object.contains_key(*key))
        || record.version != 3
        || record.kind != "run"
        || record.action != CheckAction::Check
        || !crate::producer::valid(&record.producer)
        || !is_sha(Some(&record.run_id))
        || !is_sha(Some(&record.target_event_sha256))
        || record.check_command.trim().is_empty()
        || record.check_command.contains('\0')
        || !is_sha(Some(&record.check_command_sha256))
        || crate::hash::text(&record.check_command) != record.check_command_sha256
        || !valid_timestamp(&record.timestamp)
        || !is_sha(Some(&record.event_sha256))
    {
        return Err(format!(
            "invalid run ledger line {line}: malformed check event"
        ));
    }
    if record.origin != ProofOrigin::LocalReport && record.origin != ProofOrigin::Synthetic {
        return Err(format!(
            "invalid run ledger line {line}: proof origin is invalid"
        ));
    }
    if record.previous_event_sha256.is_none() && event["previousEventSha256"] != Value::Null {
        return Err(format!(
            "invalid run ledger line {line}: previous event hash is malformed"
        ));
    }
    if let Some(duration) = event.get("durationMs") {
        if !duration.is_null() && record.duration_ms.is_none() {
            return Err(format!(
                "invalid run ledger line {line}: durationMs must be a non-negative integer or null"
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_protection_event(event: &Value, line: usize) -> Result<(), String> {
    let record: ProtectionRecord = serde_json::from_value(event.clone())
        .map_err(|_| format!("invalid run ledger line {line}: malformed protection event"))?;
    let object = event.as_object().ok_or_else(|| {
        format!("invalid run ledger line {line}: protection event must be an object")
    })?;
    let allowed = [
        "version",
        "kind",
        "producer",
        "action",
        "runId",
        "stage",
        "attempt",
        "actor",
        "role",
        "attemptedOutcome",
        "reason",
        "checkEvidence",
        "origin",
        "previousEventSha256",
        "timestamp",
        "eventSha256",
    ];
    reject_unknown(object, &allowed, line, "protection")?;
    if object.len() != allowed.len()
        || record.version != 3
        || record.kind != "run"
        || record.action != ProtectionAction::Protect
        || !crate::producer::valid(&record.producer)
        || !is_sha(Some(&record.run_id))
        || record.stage < 1
        || record.attempt < 1
        || record.actor.trim().is_empty()
        || record.role != "lead"
        || record.attempted_outcome != "accepted"
        || record.check_evidence.is_empty()
        || !valid_timestamp(&record.timestamp)
        || !is_sha(Some(&record.event_sha256))
    {
        return Err(format!(
            "invalid run ledger line {line}: malformed protection event"
        ));
    }
    if record.previous_event_sha256.is_none() && event["previousEventSha256"] != Value::Null {
        return Err(format!(
            "invalid run ledger line {line}: previous event hash is malformed"
        ));
    }
    let evidence = event["checkEvidence"]
        .as_array()
        .ok_or_else(|| format!("invalid run ledger line {line}: malformed protection evidence"))?;
    for item in evidence {
        validate_check_evidence(item, line)?;
    }
    let has_missing = record
        .check_evidence
        .iter()
        .any(|item| item.status == EvidenceStatus::Missing);
    let has_failed = record
        .check_evidence
        .iter()
        .any(|item| item.status == EvidenceStatus::Failed);
    let reason = record.reason.as_str();
    if (!has_missing && reason == "check_missing") || (!has_failed && reason == "check_failed") {
        return Err(format!(
            "invalid run ledger line {line}: protection reason does not match evidence"
        ));
    }
    Ok(())
}

fn validate_check_evidence(value: &Value, line: usize) -> Result<(), String> {
    let record: CheckEvidence = serde_json::from_value(value.clone())
        .map_err(|_| format!("invalid run ledger line {line}: malformed protection evidence"))?;
    let object = value
        .as_object()
        .ok_or_else(|| format!("invalid run ledger line {line}: malformed protection evidence"))?;
    if !is_sha(Some(&record.target_event_sha256)) {
        return Err(format!(
            "invalid run ledger line {line}: malformed protection evidence"
        ));
    }
    if record.status == EvidenceStatus::Missing {
        if object.len() != 2 {
            return Err(format!(
                "invalid run ledger line {line}: missing check evidence has extra fields"
            ));
        }
    } else if object.len() != 4
        || !is_sha(record.check_event_sha256.as_deref())
        || record.exit_code.is_none()
    {
        return Err(format!(
            "invalid run ledger line {line}: failed check evidence is malformed"
        ));
    }
    Ok(())
}

/// A checked run can only accept after every current-attempt worker completion
/// has a latest matching zero exit-code result.  This function is pure over
/// reduced state so replay and live mutation share exactly the same guard.
pub(crate) fn check_guard(state: &Value) -> Result<CheckAssessment, String> {
    let Some(policy_value) = state.get("checkPolicy") else {
        return Ok(CheckAssessment::unconfigured());
    };
    let policy =
        policy_from_value(policy_value, 0).map_err(|error| error.replacen("line 0", "state", 1))?;
    let expected_workers = state["plan"]["stages"]
        .as_array()
        .ok_or("run state plan stages are invalid")?
        .iter()
        .map(|stage| {
            stage["agents"].as_array().map_or(0, |agents| {
                agents
                    .iter()
                    .filter(|agent| agent["role"] == "worker")
                    .count()
            })
        })
        .sum::<usize>();
    let attempt = state["attempt"]
        .as_u64()
        .ok_or("run state attempt is invalid")?;
    let submissions = state["submissions"]
        .as_array()
        .ok_or("run state submissions are invalid")?;
    let empty_checks = Vec::new();
    let checks = state["checks"].as_array().unwrap_or(&empty_checks);
    let targets: Vec<CheckTarget> = submissions
        .iter()
        .filter(|submission| {
            submission["attempt"] == attempt
                && submission["role"] == "worker"
                && submission["outcome"] == "completed"
        })
        .map(|submission| {
            let target = submission["eventSha256"].clone();
            let latest = checks
                .iter()
                .rfind(|check| check["targetEventSha256"] == target);
            let target_event_sha256 = target
                .as_str()
                .map(str::to_owned)
                .ok_or("worker submission event hash is invalid")?;
            Ok::<CheckTarget, String>(match latest {
                None => CheckTarget {
                    target_event_sha256,
                    status: CheckTargetStatus::Missing,
                    check_event_sha256: None,
                    exit_code: None,
                },
                Some(check) if check["exitCode"] == 0 => CheckTarget {
                    target_event_sha256,
                    status: CheckTargetStatus::Passed,
                    check_event_sha256: check["eventSha256"].as_str().map(str::to_owned),
                    exit_code: check["exitCode"].as_u64(),
                },
                Some(check) => CheckTarget {
                    target_event_sha256,
                    status: CheckTargetStatus::Failed,
                    check_event_sha256: check["eventSha256"].as_str().map(str::to_owned),
                    exit_code: check["exitCode"].as_u64(),
                },
            })
        })
        .collect::<Result<_, _>>()?;
    let incomplete = expected_workers == 0 || targets.len() < expected_workers;
    Ok(CheckAssessment::configured(policy, targets, incomplete))
}

#[derive(Clone, Debug)]
pub(crate) struct CheckAssessment {
    pub(crate) policy: Option<CheckPolicy>,
    targets: Vec<CheckTarget>,
    pub(crate) incomplete: bool,
}

impl CheckAssessment {
    fn unconfigured() -> Self {
        Self {
            policy: None,
            targets: Vec::new(),
            incomplete: false,
        }
    }

    fn configured(policy: CheckPolicy, targets: Vec<CheckTarget>, incomplete: bool) -> Self {
        Self {
            policy: Some(policy),
            targets,
            incomplete,
        }
    }

    pub(crate) fn reason(&self) -> Option<&'static str> {
        if self.targets.iter().any(CheckTarget::is_missing) {
            Some("check_missing")
        } else if self.targets.iter().any(CheckTarget::is_failed) {
            Some("check_failed")
        } else {
            None
        }
    }

    pub(crate) fn is_blocked(&self) -> bool {
        self.incomplete || self.reason().is_some()
    }

    pub(crate) fn has_refusal_evidence(&self) -> bool {
        !self.incomplete && self.reason().is_some()
    }
}

pub(crate) fn validate_check_target(state: &Value, target: &str) -> Result<Value, String> {
    if !is_sha(Some(target)) {
        return Err("--target must be a lowercase SHA-256 event hash".into());
    }
    let attempt = state["attempt"].as_u64().ok_or("run attempt is invalid")?;
    state["submissions"]
        .as_array()
        .ok_or("run state submissions are invalid")?
        .iter()
        .find(|submission| {
            submission["eventSha256"].as_str() == Some(target)
                && submission["attempt"] == attempt
                && submission["role"] == "worker"
                && submission["outcome"] == "completed"
        })
        .cloned()
        .ok_or_else(|| "check target is not a current worker completion".into())
}

pub(crate) fn validate_check_against_state(
    state: &Value,
    event: &Value,
    line: usize,
) -> Result<(), String> {
    let policy_value = state
        .get("checkPolicy")
        .ok_or_else(|| format!("invalid run ledger line {line}: check policy is not configured"))?;
    let policy = policy_from_value(policy_value, line)?;
    let target = event["targetEventSha256"]
        .as_str()
        .ok_or_else(|| format!("invalid run ledger line {line}: check target is malformed"))?;
    validate_check_target(state, target)
        .map_err(|error| format!("invalid run ledger line {line}: {error}"))?;
    if event["checkCommand"] != policy.command
        || event["checkCommandSha256"] != policy.command_sha256
        || event["origin"] != policy.origin.as_str()
    {
        return Err(format!(
            "invalid run ledger line {line}: check report does not match configured policy"
        ));
    }
    Ok(())
}

pub(crate) fn validate_protection_against_state(
    state: &Value,
    event: &Value,
    line: usize,
) -> Result<(), String> {
    if state["status"] != "running" {
        return Err(format!(
            "invalid run ledger line {line}: protection event requires a running run"
        ));
    }
    let assignments = crate::run_assignment::pending(state);
    let assignment = assignments
        .iter()
        .find(|assignment| assignment["agent"] == event["actor"])
        .ok_or_else(|| {
            format!("invalid run ledger line {line}: protection actor is not currently pending")
        })?;
    if assignment["stage"] != event["stage"]
        || assignment["attempt"] != event["attempt"]
        || assignment["role"] != event["role"]
    {
        return Err(format!(
            "invalid run ledger line {line}: protection target is out of order"
        ));
    }
    let assessment =
        check_guard(state).map_err(|error| format!("invalid run ledger line {line}: {error}"))?;
    if !assessment.has_refusal_evidence() {
        return Err(format!(
            "invalid run ledger line {line}: protection has no current complete check refusal"
        ));
    }
    let expected = assessment
        .targets
        .iter()
        .filter(|target| target.is_missing() || target.is_failed())
        .map(CheckTarget::value)
        .collect::<Vec<_>>();
    if event["reason"] != assessment.reason().unwrap_or_default()
        || event["origin"]
            != assessment.policy.as_ref().map_or(Value::Null, |policy| {
                Value::String(policy.origin.as_str().to_owned())
            })
        || event["checkEvidence"] != Value::Array(expected)
    {
        return Err(format!(
            "invalid run ledger line {line}: protection evidence does not match current checks"
        ));
    }
    Ok(())
}

/// Build the factual refusal event.  It contains only the failing/missing
/// check references; it never claims avoided loss or invents human time.
pub(crate) fn protection_event(
    state: &Value,
    actor: &str,
    stage: &Value,
    attempt: &Value,
    previous_event: &Value,
    timestamp: &str,
) -> Result<Value, String> {
    let assessment = check_guard(state)?;
    let reason = assessment
        .reason()
        .ok_or("acceptance is not blocked by a configured check")?;
    let origin = assessment
        .policy
        .as_ref()
        .ok_or("acceptance is not blocked by a configured check")?
        .origin
        .as_str();
    Ok(json!({
        "version": 3,
        "kind": "run",
        "producer": crate::producer::evidence(),
        "action": "protect",
        "runId": state["runId"],
        "stage": stage,
        "attempt": attempt,
        "actor": actor,
        "role": "lead",
        "attemptedOutcome": "accepted",
        "reason": reason,
        "checkEvidence": assessment.targets.iter().filter(|target| target.is_missing() || target.is_failed()).map(CheckTarget::value).collect::<Vec<_>>(),
        "origin": origin,
        "previousEventSha256": previous_event["eventSha256"],
        "timestamp": timestamp,
    }))
}

/// A status view intentionally reports only ledger-derived facts.  Disk
/// revalidation is performed by `run`; callers that have not revalidated bytes
/// must not label an artifact "current".
pub(crate) fn status(state: &Value, artifact_current: Option<bool>) -> Result<Value, String> {
    let assessment = check_guard(state)?;
    let submissions = state["submissions"]
        .as_array()
        .ok_or("run state submissions are invalid")?;
    let current_attempt = state["attempt"].as_u64().unwrap_or_default();
    let claim = submissions.iter().rev().find(|submission| {
        submission["attempt"] == current_attempt && submission["role"] == "worker"
    });
    let review = submissions.iter().rev().find(|submission| {
        submission["attempt"] == current_attempt && submission["role"] == "reviewer"
    });
    let acceptance = submissions.iter().rev().find(|submission| {
        submission["attempt"] == current_attempt && submission["outcome"] == "accepted"
    });
    let mut checks = json!({
        "configured": assessment.policy.is_some(),
        "origin": assessment
            .policy
            .as_ref()
            .map_or("none", |policy| policy.origin.as_str()),
        "status": if assessment.policy.is_none() {
            "not_configured"
        } else if assessment.incomplete
            || assessment
                .targets
                .iter()
            .any(CheckTarget::is_missing)
            || assessment.targets.is_empty()
        {
            "not_observed"
        } else if assessment.is_blocked() {
            "blocked"
        } else {
            "passed"
        },
        "targets": assessment.targets.iter().map(CheckTarget::value).collect::<Vec<_>>(),
    });
    if let Some(object) = checks.as_object_mut() {
        object.insert("targetCount".into(), json!(assessment.targets.len()));
        object.insert(
            "observedCount".into(),
            json!(assessment
                .targets
                .iter()
                .filter(|target| target.is_observed())
                .count()),
        );
        object.insert(
            "failedCount".into(),
            json!(assessment
                .targets
                .iter()
                .filter(|target| target.is_failed())
                .count()),
        );
        object.insert(
            "missingCount".into(),
            json!(assessment
                .targets
                .iter()
                .filter(|target| target.is_missing())
                .count()),
        );
    }
    Ok(json!({
        "version": VALUE_REPORT_VERSION,
        "runId": state["runId"],
        "workflow": state["workflow"],
        "status": state["status"],
        "stage": state["currentStage"],
        "attempt": state["attempt"],
        "claim": claim.map_or_else(|| json!({"status":"absent"}), |submission| json!({
            "status": submission["outcome"],
            "agent": submission["agent"],
            "eventSha256": submission["eventSha256"],
            "artifactSha256": submission["artifact"]["sha256"],
        })),
        "artifact": artifact_current.map_or_else(|| json!({"status":"not_revalidated"}), |current| json!({"status": if current {"current"} else {"drifted"}})),
        "checks": checks,
        "review": review.map_or_else(|| json!({"status":"absent"}), |submission| json!({
            "status": submission["outcome"],
            "eventSha256": submission["eventSha256"],
            "artifactSha256": submission["artifact"]["sha256"],
        })),
        "acceptance": acceptance.map_or_else(|| json!({"status":"absent"}), |submission| json!({
            "status": "accepted",
            "eventSha256": submission["eventSha256"],
            "artifactSha256": submission["artifact"]["sha256"],
        })),
        "evidence": {
            "eventCount": state["events"].as_array().map_or(0, Vec::len),
            "submissionCount": submissions.len(),
            "checkCount": state["checks"].as_array().map_or(0, Vec::len),
            "protectionCount": state["protections"].as_array().map_or(0, Vec::len),
        },
    }))
}

/// Return a concise explanation for a run.  The caller may optionally identify
/// a protection event; unknown IDs remain an explicit unknown instead of being
/// silently attributed to a nearby event.
pub(crate) fn explain_with_artifact(
    state: &Value,
    event_id: Option<&str>,
    artifact_current: bool,
) -> Result<Value, String> {
    let status = status(state, Some(artifact_current))?;
    let empty_protections = Vec::new();
    let protections = state["protections"]
        .as_array()
        .unwrap_or(&empty_protections);
    let selected = event_id.and_then(|wanted| {
        protections
            .iter()
            .find(|event| event["eventSha256"].as_str() == Some(wanted))
    });
    if event_id.is_some() && selected.is_none() {
        return Err("explanation target is not a protection event in this run".into());
    }
    Ok(json!({
        "version": VALUE_REPORT_VERSION,
        "runId": state["runId"],
        "status": status,
        "protection": selected.map_or(Value::Null, Clone::clone),
        "guidance": if selected.is_some() {
            "rerun the configured check in its host and report the actual result for every current worker completion; repair or rework if needed, then request review and acceptance again; a passing report still requires authority"
        } else {
            "inspect the exact evidence references before choosing repair, rework, or acceptance"
        },
    }))
}

/// Aggregate redacted run views.  Commands, goals, prompts, profiles, paths,
/// and artifact content are intentionally absent.  Synthetic and unclassified
/// runs are grouped separately and no incident or avoided-loss count is
/// inferred from them.  Missing policy metadata remains unclassified rather
/// than being inferred as a local report.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReportGroup {
    runs: u64,
    checks: u64,
    protections: u64,
    failed_checks: u64,
    missing_checks: u64,
    duration_ms_reported: u64,
    duration_ms_reported_count: u64,
}

#[derive(Debug, Default, Serialize)]
struct ReportGroups {
    local_report: ReportGroup,
    synthetic: ReportGroup,
    unclassified: ReportGroup,
}

impl ReportGroups {
    fn get_mut(&mut self, origin: &str) -> Option<&mut ReportGroup> {
        match origin {
            "local_report" => Some(&mut self.local_report),
            "synthetic" => Some(&mut self.synthetic),
            "unclassified" => Some(&mut self.unclassified),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReportMetrics {
    synthetic_incident_count: &'static str,
    user_confirmed_avoided_loss_count: &'static str,
    human_time_ms: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReportRedaction {
    goals: &'static str,
    commands: &'static str,
    prompts: &'static str,
    profiles: &'static str,
    paths: &'static str,
    artifact_content: &'static str,
    hashes_are_not_anonymization: bool,
}

#[derive(Debug, Serialize)]
struct ValueReport {
    version: u64,
    runs: u64,
    groups: ReportGroups,
    metrics: ReportMetrics,
    redaction: ReportRedaction,
}

impl ValueReport {
    fn new() -> Self {
        Self {
            version: VALUE_REPORT_VERSION,
            runs: 0,
            groups: ReportGroups::default(),
            metrics: ReportMetrics {
                synthetic_incident_count: "unknown",
                user_confirmed_avoided_loss_count: "unknown",
                human_time_ms: "unknown",
            },
            redaction: ReportRedaction {
                goals: "omitted",
                commands: "omitted",
                prompts: "omitted",
                profiles: "omitted",
                paths: "omitted",
                artifact_content: "omitted",
                hashes_are_not_anonymization: true,
            },
        }
    }
}

pub(crate) fn aggregate(states: &[Value]) -> Result<Value, String> {
    let mut seen_runs = std::collections::BTreeMap::new();
    let mut report = ValueReport::new();
    for state in states {
        let run_id = state["runId"]
            .as_str()
            .ok_or("report run is missing a runId")?;
        let state_hash = crate::hash::value(state);
        if let Some(previous) = seen_runs.insert(run_id.to_owned(), state_hash.clone()) {
            if previous != state_hash {
                return Err("duplicate runId has conflicting ledger evidence".into());
            }
            continue;
        }
        let origin = state["checkPolicy"]["origin"]
            .as_str()
            .unwrap_or("unclassified");
        if !matches!(origin, "local_report" | "synthetic" | "unclassified") {
            return Err("invalid report origin".into());
        }
        let empty_checks = Vec::new();
        let checks = state["checks"].as_array().unwrap_or(&empty_checks);
        let empty_protections = Vec::new();
        let protections = state["protections"]
            .as_array()
            .unwrap_or(&empty_protections);
        let assessment = check_guard(state)?;
        let group = report
            .groups
            .get_mut(origin)
            .ok_or("report groups are invalid")?;
        add_counter(&mut group.runs, 1)?;
        add_counter(&mut group.checks, checks.len() as u64)?;
        add_counter(&mut group.protections, protections.len() as u64)?;
        add_counter(
            &mut group.failed_checks,
            checks
                .iter()
                .filter(|check| check["exitCode"].as_u64().is_some_and(|code| code != 0))
                .count() as u64,
        )?;
        add_counter(
            &mut group.missing_checks,
            assessment
                .targets
                .iter()
                .filter(|target| target.is_missing())
                .count() as u64,
        )?;
        for check in checks {
            if let Some(duration) = check["durationMs"].as_u64() {
                add_counter(&mut group.duration_ms_reported, duration)?;
                add_counter(&mut group.duration_ms_reported_count, 1)?;
            }
        }
    }
    report.runs = seen_runs.len() as u64;
    serde_json::to_value(report).map_err(|error| error.to_string())
}

pub(crate) fn markdown(report: &Value) -> String {
    let groups = &report["groups"];
    format!(
        "Value proof report v{}\n\nRuns: {}\n\nLocal reports: {} runs, {} checks, {} protections\nSynthetic scenarios: {} runs, {} checks, {} protections\nUnclassified runs: {} runs, {} checks, {} protections\n\nSynthetic incident frequency, user-confirmed avoided loss, and human time are unknown.\nGoals, commands, prompts, profiles, paths, and artifact content are omitted. Hashes are not anonymization.\n",
        report["version"],
        report["runs"],
        groups["local_report"]["runs"],
        groups["local_report"]["checks"],
        groups["local_report"]["protections"],
        groups["synthetic"]["runs"],
        groups["synthetic"]["checks"],
        groups["synthetic"]["protections"],
        groups["unclassified"]["runs"],
        groups["unclassified"]["checks"],
        groups["unclassified"]["protections"],
    )
}

fn add_counter(current: &mut u64, amount: u64) -> Result<(), String> {
    let total = (*current)
        .checked_add(amount)
        .ok_or("report metric overflow")?;
    *current = total;
    Ok(())
}

fn reject_unknown(
    object: &Map<String, Value>,
    allowed: &[&str],
    line: usize,
    kind: &str,
) -> Result<(), String> {
    object
        .keys()
        .find(|key| !allowed.contains(&key.as_str()))
        .map_or(Ok(()), |key| {
            Err(format!(
                "invalid run ledger line {line}: unknown {kind} field '{key}'"
            ))
        })
}

fn is_sha(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        value.len() == SHA_LEN
            && value.bytes().all(|byte| byte.is_ascii_hexdigit())
            && value.bytes().all(|byte| !byte.is_ascii_uppercase())
    })
}

fn valid_timestamp(value: &str) -> bool {
    value.contains('T') && chrono::DateTime::parse_from_rfc3339(value).is_ok()
}
