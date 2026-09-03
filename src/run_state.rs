use serde_json::{json, Map, Value};

const SHA_LEN: usize = 64;
const ROLES: &[&str] = &["lead", "adviser", "worker", "reviewer"];

pub fn make_event(mut value: Value) -> Value {
    let hash = crate::hash::value(&value);
    value["eventSha256"] = json!(hash);
    value
}

pub fn reduce(events: &[Value]) -> Result<Value, String> {
    if events.is_empty() {
        return Err("run ledger has no start event".into());
    }
    validate_start(&events[0], 1)?;
    let first = &events[0];
    let mut state = json!({
        "runId": first["runId"], "workflow": first["workflow"], "goal": first["goal"],
        "configSha256": first["configSha256"], "plan": first["plan"], "status": "running",
        "currentStage": 1, "attempt": 1, "submissions": [], "events": events
    });
    if let Some(receipt) = first.get("harnessReceipt") {
        state["harnessReceipt"] = receipt.clone();
    }
    for (index, event) in events.iter().enumerate().skip(1) {
        validate_event(event, events.get(index - 1), index + 1)?;
        if event["runId"] != first["runId"] {
            return Err(format!(
                "invalid run ledger line {}: runId changed",
                index + 1
            ));
        }
        apply_submission(&mut state, event)?;
    }
    state["assignments"] = json!(crate::run_assignment::pending(&state));
    Ok(state)
}

pub fn validate_event(event: &Value, previous: Option<&Value>, line: usize) -> Result<(), String> {
    let object = event
        .as_object()
        .ok_or_else(|| format!("invalid run ledger line {line}: event must be an object"))?;
    let version = event["version"].as_u64();
    if !matches!(version, Some(1 | 2)) || event["kind"] != "run" {
        return Err(format!(
            "invalid run ledger line {line}: invalid event header"
        ));
    }
    if previous.is_some_and(|previous| previous["version"] != event["version"]) {
        return Err(format!(
            "invalid run ledger line {line}: mixed event versions"
        ));
    }
    if event
        .get("producer")
        .is_some_and(|producer| !crate::producer::valid(producer))
        || (version == Some(2) && !event.get("producer").is_some_and(crate::producer::valid))
    {
        return Err(format!("invalid run ledger line {line}: invalid producer"));
    }
    if event["action"] != "start" && event["action"] != "submit" {
        return Err(format!("invalid run ledger line {line}: invalid action"));
    }
    if !is_sha(event["runId"].as_str()) {
        return Err(format!("invalid run ledger line {line}: invalid runId"));
    }
    if !is_timestamp(event["timestamp"].as_str()) {
        return Err(format!("invalid run ledger line {line}: invalid timestamp"));
    }
    if !is_sha(event["eventSha256"].as_str()) {
        return Err(format!(
            "invalid run ledger line {line}: invalid event hash"
        ));
    }
    let previous_hash = previous
        .map(|x| x["eventSha256"].clone())
        .unwrap_or(Value::Null);
    if event["previousEventSha256"] != previous_hash {
        return Err(format!(
            "invalid run ledger line {line}: broken event chain"
        ));
    }
    if crate::hash::value(&without(event, "eventSha256")) != event["eventSha256"] {
        return Err(format!(
            "invalid run ledger line {line}: event hash mismatch"
        ));
    }
    if let Some(prev) = previous {
        if timestamp_ms(event["timestamp"].as_str()) < timestamp_ms(prev["timestamp"].as_str()) {
            return Err(format!(
                "invalid run ledger line {line}: timestamp is earlier than previous event"
            ));
        }
    }
    let shape = if event["action"] == "start" {
        validate_start_version(event, line, version.unwrap_or_default())
    } else {
        validate_submission(event, line)
    };
    let action = event["action"]
        .as_str()
        .ok_or_else(|| format!("invalid run ledger line {line}: invalid action"))?;
    shape.and_then(|_| reject_unknown(object, action, version.unwrap_or_default(), line))
}

pub fn validate_start(event: &Value, line: usize) -> Result<(), String> {
    validate_start_version(event, line, event["version"].as_u64().unwrap_or_default())
}

fn validate_start_version(event: &Value, line: usize, version: u64) -> Result<(), String> {
    if !matches!(version, 1 | 2) {
        return Err(format!(
            "invalid run ledger line {line}: invalid event version"
        ));
    }
    let object = event
        .as_object()
        .ok_or_else(|| format!("invalid run ledger line {line}: start must be an object"))?;
    reject_unknown(object, "start", version, line)?;
    if event["previousEventSha256"] != Value::Null {
        return Err(format!(
            "invalid run ledger line {line}: start must begin the chain"
        ));
    }
    for field in ["workflow", "goal", "configSha256"] {
        if event[field].as_str().map_or(true, |x| x.trim().is_empty()) {
            return Err(format!(
                "invalid run ledger line {line}: {field} is required"
            ));
        }
    }
    if !is_sha(event["configSha256"].as_str()) {
        return Err(format!(
            "invalid run ledger line {line}: invalid config hash"
        ));
    }
    let plan = &event["plan"];
    if plan["version"] != 1 {
        return Err(format!(
            "invalid run ledger line {line}: invalid workflow plan"
        ));
    }
    let stages = plan["stages"]
        .as_array()
        .filter(|stages| !stages.is_empty());
    let Some(stages) = stages else {
        return Err(format!(
            "invalid run ledger line {line}: invalid workflow plan"
        ));
    };
    if plan["maxParallel"].as_u64().map_or(true, |x| x < 1) {
        return Err(format!(
            "invalid run ledger line {line}: invalid maxParallel"
        ));
    }
    if let Some(boundary) = plan.get("boundaryManifest") {
        if !crate::boundary_manifest::validate_evidence(boundary) {
            return Err(format!(
                "invalid run ledger line {line}: invalid boundary manifest evidence"
            ));
        }
    }
    for (i, stage) in stages.iter().enumerate() {
        let agents = stage["agents"]
            .as_array()
            .filter(|agents| !agents.is_empty());
        let Some(agents) = agents else {
            return Err(format!(
                "invalid run ledger line {line}: invalid stage {}",
                i + 1
            ));
        };
        if stage["stage"] != i + 1 {
            return Err(format!(
                "invalid run ledger line {line}: invalid stage {}",
                i + 1
            ));
        }
        for agent in agents {
            if agent.as_object().is_none()
                || agent["name"].as_str().map_or(true, str::is_empty)
                || !ROLES.contains(&agent["role"].as_str().unwrap_or(""))
                || !display_name(agent["displayName"].as_str())
                || !native_name(agent["nativeTaskName"].as_str())
                || !relative(agent["profile"].as_str())
                || !is_sha(agent["profileSha256"].as_str())
                || !agent["runtime"].is_object()
                || !agent["declaredBoundary"].is_object()
            {
                return Err(format!(
                    "invalid run ledger line {line}: invalid selected agent evidence"
                ));
            }
            if let Some(references) = agent.get("memoryReferences") {
                crate::memory_selection::validate_references(references)
                    .map_err(|error| format!("invalid run ledger line {line}: {error}"))?;
            }
        }
    }
    if let Some(link) = event.get("supersedes") {
        validate_supersession(link, line)?;
    }
    if version == 2 {
        validate_harness_receipt(event.get("harnessReceipt"), line)?;
    } else if event.get("harnessReceipt").is_some() {
        return Err(format!(
            "invalid run ledger line {line}: v1 start must not contain harnessReceipt"
        ));
    }
    Ok(())
}

fn validate_harness_receipt(value: Option<&Value>, line: usize) -> Result<(), String> {
    let Some(value) = value else {
        return Err(format!(
            "invalid run ledger line {line}: v2 start requires harnessReceipt"
        ));
    };
    let object = value.as_object().ok_or_else(|| {
        format!("invalid run ledger line {line}: harnessReceipt must be an object")
    })?;
    if object.len() != 3
        || !object.contains_key("path")
        || !object.contains_key("sha256")
        || !object.contains_key("version")
        || value["version"] != 2
        || !relative(value["path"].as_str())
        || !is_sha(value["sha256"].as_str())
    {
        return Err(format!(
            "invalid run ledger line {line}: invalid harness receipt reference"
        ));
    }
    Ok(())
}

pub fn validate_supersession(value: &Value, line: usize) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("invalid run ledger line {line}: invalid supersession evidence"))?;
    let fields = [
        "ledgerPath",
        "ledgerSha256",
        "runId",
        "headEventSha256",
        "configSha256",
    ];
    if object.len() != fields.len()
        || fields.iter().any(|x| !object.contains_key(*x))
        || !relative(value["ledgerPath"].as_str())
        || !is_sha(value["ledgerSha256"].as_str())
        || !is_sha(value["runId"].as_str())
        || !is_sha(value["headEventSha256"].as_str())
        || !is_sha(value["configSha256"].as_str())
    {
        return Err(format!(
            "invalid run ledger line {line}: invalid supersession evidence"
        ));
    }
    Ok(())
}

pub fn validate_submission(event: &Value, line: usize) -> Result<(), String> {
    for field in ["stage", "attempt"] {
        if event[field].as_u64().map_or(true, |x| x < 1) {
            return Err(format!(
                "invalid run ledger line {line}: invalid stage or attempt"
            ));
        }
    }
    if event["agent"]
        .as_str()
        .map_or(true, |x| x.trim().is_empty())
        || !ROLES.contains(&event["role"].as_str().unwrap_or(""))
    {
        return Err(format!(
            "invalid run ledger line {line}: invalid submission actor"
        ));
    }
    if event["outcome"]
        .as_str()
        .map_or(true, |x| x.trim().is_empty())
    {
        return Err(format!(
            "invalid run ledger line {line}: outcome is required"
        ));
    }
    let artifact = &event["artifact"];
    if artifact.as_object().map_or(true, |x| {
        !matches!(x.len(), 2 | 3)
            || !x.contains_key("path")
            || !x.contains_key("sha256")
            || (x.len() == 3 && !x.contains_key("root"))
    }) || artifact
        .get("root")
        .is_some_and(|root| !matches!(root.as_str(), Some("product" | "state")))
        || !relative(artifact["path"].as_str())
        || !is_sha(artifact["sha256"].as_str())
    {
        return Err(format!(
            "invalid run ledger line {line}: invalid artifact evidence"
        ));
    }
    Ok(())
}

fn apply_submission(state: &mut Value, event: &Value) -> Result<(), String> {
    if state["status"] != "running" {
        return Err("run has already reached a terminal state".into());
    }
    let candidate = crate::run_assignment::pending(state)
        .into_iter()
        .find(|x| x["agent"] == event["agent"]);
    let Some(assignment) = candidate else {
        return Err(format!(
            "agent '{}' is not currently pending",
            event["agent"]
        ));
    };
    if event["stage"] != assignment["stage"]
        || event["attempt"] != assignment["attempt"]
        || event["role"] != assignment["role"]
    {
        return Err("submission is out of order".into());
    }
    let role = event["role"].as_str().ok_or("submission role is invalid")?;
    let outcome = event["outcome"]
        .as_str()
        .ok_or("submission outcome is invalid")?;
    let all = match role {
        "lead" => &["scoped", "blocked", "accepted", "rework", "rejected"][..],
        "adviser" | "worker" => &["completed", "blocked"][..],
        "reviewer" => &["approved", "rework", "blocked"][..],
        _ => &[],
    };
    if !all.contains(&event["outcome"].as_str().unwrap_or("")) {
        return Err(format!(
            "outcome '{}' is not allowed for role '{role}'",
            event["outcome"]
        ));
    }
    if role == "lead" && state["currentStage"] == 1 && !["scoped", "blocked"].contains(&outcome) {
        return Err(format!(
            "outcome '{}' is not allowed for this stage",
            event["outcome"]
        ));
    }
    state["submissions"]
        .as_array_mut()
        .ok_or("run state submissions are invalid")?
        .push(json!({"stage":event["stage"],"attempt":event["attempt"],"agent":event["agent"],"role":event["role"],"outcome":event["outcome"],"artifact":event["artifact"],"eventSha256":event["eventSha256"]}));
    if ["accepted", "rejected", "blocked"].contains(&outcome) {
        state["status"] = json!(outcome);
        return Ok(());
    }
    if outcome == "rework" {
        let stages = state["plan"]["stages"]
            .as_array()
            .ok_or("run state plan stages are invalid")?;
        let worker = stages
            .iter()
            .find(|s| {
                s["agents"]
                    .as_array()
                    .is_some_and(|agents| agents.iter().any(|a| a["role"] == "worker"))
            })
            .ok_or("rework requires a worker stage")?;
        state["currentStage"] = worker["stage"].clone();
        let attempt = state["attempt"].as_u64().ok_or("run attempt is invalid")?;
        state["attempt"] = json!(attempt + 1);
        return Ok(());
    }
    let stages = state["plan"]["stages"]
        .as_array()
        .ok_or("run state plan stages are invalid")?;
    let required = stages
        .iter()
        .find(|s| s["stage"] == state["currentStage"])
        .ok_or("run current stage is invalid")?["agents"]
        .as_array()
        .ok_or("run current stage agents are invalid")?
        .len();
    let completed = state["submissions"]
        .as_array()
        .ok_or("run state submissions are invalid")?
        .iter()
        .filter(|x| x["stage"] == state["currentStage"] && x["attempt"] == state["attempt"])
        .count();
    if completed == required && state["currentStage"] != stages.len() {
        let current = state["currentStage"]
            .as_u64()
            .ok_or("run current stage is invalid")?;
        state["currentStage"] = json!(current + 1);
    }
    Ok(())
}

fn reject_unknown(
    object: &Map<String, Value>,
    action: &str,
    version: u64,
    line: usize,
) -> Result<(), String> {
    let allowed: &[&str] = if action == "start" {
        let mut allowed = vec![
            "version",
            "kind",
            "producer",
            "action",
            "runId",
            "workflow",
            "goal",
            "configSha256",
            "plan",
            "previousEventSha256",
            "timestamp",
            "eventSha256",
            "supersedes",
        ];
        if version == 2 {
            allowed.push("harnessReceipt");
        }
        return object
            .keys()
            .find(|key| !allowed.contains(&key.as_str()))
            .map_or(Ok(()), |key| {
                Err(format!(
                    "invalid run ledger line {line}: unknown field '{key}'"
                ))
            });
    } else {
        &[
            "version",
            "kind",
            "producer",
            "action",
            "runId",
            "stage",
            "attempt",
            "agent",
            "role",
            "outcome",
            "artifact",
            "previousEventSha256",
            "timestamp",
            "eventSha256",
        ]
    };
    object
        .keys()
        .find(|key| !allowed.contains(&key.as_str()))
        .map_or(Ok(()), |key| {
            Err(format!(
                "invalid run ledger line {line}: unknown field '{key}'"
            ))
        })
}
fn without(value: &Value, key: &str) -> Value {
    let mut copy = value.clone();
    if let Some(object) = copy.as_object_mut() {
        object.remove(key);
    }
    copy
}
fn relative(value: Option<&str>) -> bool {
    let Some(v) = value else { return false };
    if v.contains('\\') {
        return false;
    }
    !v.trim().is_empty()
        && !v.contains('\0')
        && !v.starts_with('/')
        && !v.contains(":/")
        && !v.split('/').any(|x| x.is_empty() || x == "." || x == "..")
}
fn is_sha(value: Option<&str>) -> bool {
    value.is_some_and(|x| {
        x.len() == SHA_LEN
            && x.bytes().all(|b| b.is_ascii_hexdigit())
            && x.bytes().all(|b| !b.is_ascii_uppercase())
    })
}
fn display_name(value: Option<&str>) -> bool {
    value.is_some_and(|x| {
        !x.is_empty() && x.len() <= 80 && x.trim() == x && !x.bytes().any(|b| b < 0x20 || b == 0x7f)
    })
}
fn native_name(value: Option<&str>) -> bool {
    value.is_some_and(|x| {
        !x.is_empty()
            && x.len() <= 64
            && x.bytes()
                .enumerate()
                .all(|(i, b)| b.is_ascii_lowercase() || b.is_ascii_digit() || (b == b'_' && i > 0))
    })
}
fn is_timestamp(value: Option<&str>) -> bool {
    value.is_some_and(|x| x.contains('T') && timestamp_ms(Some(x)) != i64::MIN)
}
fn timestamp_ms(value: Option<&str>) -> i64 {
    value
        .and_then(|x| chrono::DateTime::parse_from_rfc3339(x).ok())
        .map(|x| x.timestamp_millis())
        .unwrap_or(i64::MIN)
}
