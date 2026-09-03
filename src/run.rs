//! Run orchestration for bounded agent workflows.
//!
//! Ledger storage and locking live in `run_ledger`; event validation and state
//! transitions live in `run_state`. This module coordinates those boundaries
//! and checks live configuration/artifact evidence before mutation.

use crate::{
    config::{self, Loaded},
    envelope, hash, run_error, run_ledger, run_state,
};
pub use run_error::DriftError;
use run_ledger::{
    append, claim_path, ledger_path, load, load_at, obtain_claim, predecessor, rollback_claim,
    with_lock,
};
use serde_json::{json, Value};
use std::fs;

pub fn start(
    loaded: &Loaded,
    workflow: &str,
    goal: &str,
    ledger: &str,
    boundary: Option<&str>,
    harness_receipt: Option<&str>,
) -> Result<Value, String> {
    if workflow.trim().is_empty() {
        return Err("workflow is required".into());
    }
    if goal.trim().is_empty() {
        return Err("--goal requires a non-empty value".into());
    }
    let plan = crate::boundary_manifest::apply(
        loaded,
        selected_plan(envelope::plan(loaded, workflow, goal)?)?,
        boundary,
    )?;
    if let Some(receipt) = harness_receipt {
        crate::receipt::for_run(loaded, receipt, &plan)?;
    }
    let timestamp = now();
    let config_sha = hash::text(&loaded.source);
    let run_id = hash::value(&json!({
        "workflow": workflow,
        "configSha256": config_sha,
        "goal": goal,
        "timestamp": timestamp
    }));
    let path = ledger_path(&loaded.state_root, ledger, true)?;
    let targets = [path.path.as_path(), path.lock.as_path()];
    crate::git_preflight::refuse_tracked_targets(&loaded.state_root, &targets)?;
    let event = with_lock(&path, || {
        crate::git_preflight::refuse_tracked_targets(&loaded.state_root, &targets)?;
        let reference = if let Some(receipt) = harness_receipt {
            if fs::read_to_string(&loaded.path).map_err(|error| error.to_string())? != loaded.source
            {
                return Err("configuration changed while starting; reload configuration".into());
            }
            Some(crate::receipt::for_run(loaded, receipt, &plan)?)
        } else {
            None
        };
        let mut value = json!({
            "version": 1,
            "kind": "run",
            "producer": crate::producer::evidence(),
            "action": "start",
            "runId": run_id,
            "workflow": workflow,
            "goal": goal,
            "configSha256": config_sha,
            "plan": plan,
            "previousEventSha256": null,
            "timestamp": timestamp
        });
        if let Some(reference) = reference {
            value["version"] = json!(2);
            value["harnessReceipt"] = reference;
        }
        let event = run_state::make_event(value);
        append(&path, &event, true, "")?;
        Ok(event)
    })?;
    result(&[event])
}

pub fn next(loaded: &Loaded, ledger: &str) -> Result<Value, String> {
    let (_, events, _) = load(loaded, ledger)?;
    let state = run_state::reduce(&events)?;
    assert_no_drift(loaded, &state)?;
    crate::run_artifact::assert_current(loaded, &state)?;
    predecessor(loaded, &events[0])?;
    Ok(json!({
        "valid": true,
        "runId": state["runId"],
        "status": state["status"],
        "workflow": state["workflow"],
        "currentStage": if state["status"] == "running" { state["currentStage"].clone() } else { Value::Null },
        "attempt": if state["status"] == "running" { state["attempt"].clone() } else { Value::Null },
        "assignments": crate::run_assignment::pending(&state)
    }))
}

pub fn submit(
    loaded: &Loaded,
    agent: &str,
    ledger: &str,
    outcome: &str,
    artifact: &str,
    artifact_root: Option<&str>,
) -> Result<Value, String> {
    if agent.trim().is_empty() {
        return Err("agent is required".into());
    }
    let path = ledger_path(&loaded.state_root, ledger, false)?;
    let targets = [path.path.as_path(), path.lock.as_path()];
    crate::git_preflight::refuse_tracked_targets(&loaded.state_root, &targets)?;
    with_lock(&path, || {
        crate::git_preflight::refuse_tracked_targets(&loaded.state_root, &targets)?;
        if claim_path(&path).exists() {
            return Err("run has been superseded; no mutation was made".into());
        }
        let (_, events, source) = load_at(loaded, &path)?;
        let state = run_state::reduce(&events)?;
        assert_no_drift(loaded, &state)?;
        crate::run_artifact::assert_current(loaded, &state)?;
        let assignment = crate::run_assignment::pending(&state)
            .into_iter()
            .find(|item| item["agent"] == agent)
            .ok_or_else(|| format!("agent '{agent}' is not currently pending"))?;
        let artifact_value = crate::run_artifact::evidence(loaded, artifact_root, artifact)?;
        let last = events.last().ok_or("run ledger has no head event")?;
        let version = if state.get("harnessReceipt").is_some() {
            2
        } else {
            1
        };
        let event = run_state::make_event(json!({
            "version": version,
            "kind": "run",
            "producer": crate::producer::evidence(),
            "action": "submit",
            "runId": state["runId"],
            "stage": assignment["stage"],
            "attempt": assignment["attempt"],
            "agent": agent,
            "role": assignment["role"],
            "outcome": outcome,
            "artifact": artifact_value,
            "previousEventSha256": last["eventSha256"],
            "timestamp": nondecreasing(&last["timestamp"])?
        }));
        let mut all = events;
        all.push(event.clone());
        let next_state = run_state::reduce(&all)?;
        append(&path, &event, false, &source)?;
        Ok(json!({
            "valid": true,
            "event": event,
            "status": next_state["status"],
            "currentStage": if next_state["status"] == "running" { next_state["currentStage"].clone() } else { Value::Null },
            "attempt": if next_state["status"] == "running" { next_state["attempt"].clone() } else { Value::Null },
            "assignments": crate::run_assignment::pending(&next_state)
        }))
    })
}

pub fn inspect(loaded: &Loaded, ledger: &str) -> Result<Value, String> {
    let (_, events, _) = load(loaded, ledger)?;
    let state = run_state::reduce(&events)?;
    predecessor(loaded, &events[0])?;
    Ok(json!({
        "valid": true,
        "runId": state["runId"],
        "workflow": state["workflow"],
        "status": state["status"],
        "currentStage": if state["status"] == "running" { state["currentStage"].clone() } else { Value::Null },
        "attempt": if state["status"] == "running" { state["attempt"].clone() } else { Value::Null },
        "events": events,
        "submissions": state["submissions"]
    }))
}

/// Start a fresh bounded run while atomically claiming one running or blocked predecessor.
pub fn supersede(
    loaded: &Loaded,
    old_ledger: &str,
    workflow: &str,
    goal: &str,
    new_ledger: &str,
    boundary: Option<&str>,
    harness_receipt: Option<&str>,
) -> Result<Value, String> {
    if workflow.trim().is_empty() {
        return Err("workflow is required".into());
    }
    if goal.trim().is_empty() {
        return Err("--goal requires a non-empty value".into());
    }
    if fs::read_to_string(&loaded.path).map_err(|error| error.to_string())? != loaded.source {
        return Err("configuration changed while superseding; reload configuration".into());
    }
    let old = ledger_path(&loaded.state_root, old_ledger, false)?;
    let new = ledger_path(&loaded.state_root, new_ledger, true)?;
    let targets = [
        old.path.as_path(),
        old.lock.as_path(),
        new.path.as_path(),
        new.lock.as_path(),
    ];
    crate::git_preflight::refuse_tracked_targets(&loaded.state_root, &targets)?;
    with_lock(&old, || {
        crate::git_preflight::refuse_tracked_targets(&loaded.state_root, &targets)?;
        let (_, old_events, old_source) = load_at(loaded, &old)?;
        let old_state = run_state::reduce(&old_events)?;
        if !matches!(old_state["status"].as_str(), Some("running" | "blocked")) {
            return Err("only a running or blocked run can be superseded".into());
        }
        if fs::read_to_string(&loaded.path).map_err(|error| error.to_string())? != loaded.source {
            return Err("configuration changed while superseding; reload configuration".into());
        }
        let plan = crate::boundary_manifest::apply(
            loaded,
            selected_plan(envelope::plan(loaded, workflow, goal)?)?,
            boundary,
        )?;
        let old_rel = config::rel(&loaded.state_root, &old.expected)?;
        let old_sha = hash::text(&old_source);
        let head = old_events
            .last()
            .and_then(|event| event["eventSha256"].as_str())
            .ok_or("old ledger has no event head")?
            .to_owned();
        let config_sha = hash::text(&loaded.source);
        let timestamp = now();
        let run_id = hash::value(&json!({
            "workflow": workflow,
            "configSha256": config_sha,
            "goal": goal,
            "timestamp": timestamp
        }));
        let harness_reference = harness_receipt
            .map(|receipt| crate::receipt::for_run(loaded, receipt, &plan))
            .transpose()?;
        let wanted_claim = json!({
            "version": 1,
            "oldLedgerPath": old_rel,
            "oldLedgerSha256": old_sha,
            "oldRunId": old_state["runId"],
            "oldHeadEventSha256": head,
            "oldConfigSha256": old_state["configSha256"],
            "newLedgerPath": config::rel(&loaded.state_root, &new.expected)?,
            "workflow": workflow,
            "goalSha256": hash::text(goal),
            "configSha256": config_sha,
            "newRunId": run_id,
            "timestamp": timestamp
        });
        let claim_path = claim_path(&old);
        if new.path.exists() && !claim_path.exists() {
            return Err("successor ledger exists without a matching predecessor claim".into());
        }
        let claim = obtain_claim(&claim_path, &wanted_claim)?;
        let supersedes = json!({
            "ledgerPath": claim.value["oldLedgerPath"],
            "ledgerSha256": claim.value["oldLedgerSha256"],
            "runId": claim.value["oldRunId"],
            "headEventSha256": claim.value["oldHeadEventSha256"],
            "configSha256": claim.value["oldConfigSha256"]
        });
        let mut event_value = json!({
            "version": if harness_reference.is_some() { 2 } else { 1 },
            "kind": "run",
            "producer": crate::producer::evidence(),
            "action": "start",
            "runId": claim.value["newRunId"],
            "workflow": workflow,
            "goal": goal,
            "configSha256": config_sha,
            "plan": plan,
            "previousEventSha256": null,
            "timestamp": claim.value["timestamp"],
            "supersedes": supersedes
        });
        if let Some(reference) = harness_reference {
            event_value["harnessReceipt"] = reference;
        }
        let event = run_state::make_event(event_value);
        let successor = if new.path.exists() {
            match load_at(loaded, &new) {
                Ok((_, existing, _)) if existing.first() == Some(&event) => Ok(()),
                Ok(_) => Err("successor ledger already exists with different provenance".into()),
                Err(error) => Err(error),
            }
        } else {
            append(&new, &event, true, "")
        };
        if let Err(error) = successor {
            rollback_claim(&claim_path, &claim);
            return Err(error);
        }
        result(&[event])
    })
}

fn result(events: &[Value]) -> Result<Value, String> {
    let state = run_state::reduce(events)?;
    Ok(json!({
        "valid": true,
        "runId": state["runId"],
        "status": state["status"],
        "currentStage": state["currentStage"],
        "attempt": state["attempt"],
        "assignments": crate::run_assignment::pending(&state)
    }))
}

fn selected_plan(mut plan: Value) -> Result<Value, String> {
    let object = plan
        .as_object_mut()
        .ok_or("workflow plan must be an object")?;
    object.remove("goal");
    object.remove("notice");
    Ok(plan)
}

fn assert_no_drift(loaded: &Loaded, state: &Value) -> Result<(), String> {
    crate::boundary_manifest::assert_current(loaded, &state["plan"])?;
    let expected = state["configSha256"].as_str().unwrap_or("");
    let current = fs::read_to_string(&loaded.path)
        .map_err(|error| format!("configuration cannot be read: {error}"))?;
    let current_sha = hash::text(&current);
    if current_sha != expected {
        return Err(run_error::machine_drift(DriftError::config(
            expected.to_owned(),
            current_sha,
        )));
    }

    let mut seen = std::collections::BTreeSet::new();
    let stages = state["plan"]["stages"]
        .as_array()
        .ok_or("validated run state has no plan stages")?;
    for stage in stages {
        let agents = stage["agents"]
            .as_array()
            .ok_or("validated run stage has no agents")?;
        for selected in agents {
            let name = selected["name"].as_str().unwrap_or("");
            if !seen.insert(name) {
                continue;
            }
            let configured = loaded
                .agent(name)
                .ok_or_else(|| format!("profile selection changed: agent '{name}' is missing"))?;
            let path = config::file(&loaded.control_root, &configured.profile)
                .map_err(|error| format!("profile cannot be read for '{name}': {error}"))?;
            let profile_sha = hash::text(
                &fs::read_to_string(&path)
                    .map_err(|error| format!("profile cannot be read for '{name}': {error}"))?,
            );
            if config::rel(&loaded.control_root, &path)? != selected["profile"]
                || profile_sha != selected["profileSha256"]
            {
                return Err(run_error::machine_drift(DriftError::profile(
                    name.to_owned(),
                    selected["profileSha256"].as_str().unwrap_or("").to_owned(),
                    profile_sha,
                )));
            }
            if let Some(references) = selected.get("memoryReferences") {
                let expected = hash::value(references);
                let current = match crate::memory_selection::resolve(loaded, name) {
                    Ok(current) => {
                        let current = Value::Array(current);
                        if &current == references {
                            continue;
                        }
                        hash::value(&current)
                    }
                    Err(error) => hash::text(&format!("memory evidence unavailable: {error}")),
                };
                return Err(run_error::machine_drift(DriftError::memory(
                    name.to_owned(),
                    expected,
                    current,
                )));
            }
        }
    }
    if let Some(reference) = state.get("harnessReceipt") {
        crate::receipt::assert_current(loaded, reference, &state["plan"])?;
    }
    Ok(())
}

fn nondecreasing(previous: &Value) -> Result<String, String> {
    let candidate = now();
    let candidate_time = chrono::DateTime::parse_from_rfc3339(&candidate)
        .map_err(|_| "generated run timestamp is invalid".to_owned())?;
    let previous = previous
        .as_str()
        .ok_or("previous run timestamp is invalid")?;
    let previous_time = chrono::DateTime::parse_from_rfc3339(previous)
        .map_err(|_| "previous run timestamp is invalid".to_owned())?;
    if candidate_time < previous_time {
        Ok(previous.to_owned())
    } else {
        Ok(candidate)
    }
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
