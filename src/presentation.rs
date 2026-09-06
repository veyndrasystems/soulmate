//! Read-only CLI presentation of already validated run packets.

use serde_json::Value;

pub(crate) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn read_command(action: &str, config: &str, ledger: &str) -> String {
    let format = if action == "next" { " --text" } else { "" };
    format!(
        "soulmate run {action}{format} --config={} -- {}",
        shell_quote(config),
        shell_quote(ledger)
    )
}

pub(crate) fn print_next(value: &Value) -> Result<(), String> {
    let assignments = value["assignments"]
        .as_array()
        .ok_or("next packet must contain an assignments array")?;
    println!(
        "Run {}: {}\nWorkflow: {}",
        value["runId"], value["status"], value["workflow"]
    );
    if assignments.is_empty() {
        println!("No pending assignments.");
        return Ok(());
    }
    println!("Pending assignments: {}", assignments.len());
    for assignment in assignments {
        // JSON string escaping keeps embedded control characters visibly inert.
        println!(
            "\nAgent: {} (role {})\nStage: {} / Attempt: {}\nGoal: {}\nPurpose: {}\nDisplay name: {}\nNative task name: {}\nProfile: {}\nRequested runtime: {}\nDeclared boundary: {}\nResult path: {}\nResult root: {}",
            assignment["agent"], assignment["role"],
            assignment["stage"], assignment["attempt"],
            assignment["goal"], assignment["purpose"],
            assignment["displayName"], assignment["nativeTaskName"],
            assignment["profile"], assignment["runtime"], assignment["declaredBoundary"],
            assignment["artifactPathHint"], assignment["artifactRootHint"]
        );
        if let Some(policy) = assignment.get("checkPolicy") {
            println!(
                "Checked run: caller-reported result required for each current worker submission.\nFrozen check command: {}\nCheck command SHA-256: {}\nCheck origin: {}",
                policy["command"], policy["commandSha256"], policy["origin"]
            );
        } else {
            println!("Unchecked run: no check-result requirement is configured.");
        }
        println!("Upstream artifacts (immutable; keep the exact recorded bytes):");
        if let Some(upstream) = assignment["upstreamArtifacts"].as_array() {
            if upstream.is_empty() {
                println!("  none");
            }
            for artifact in upstream {
                println!(
                    "  {} attempt {} / stage {}: agent={} role={} root={} path={} sha256={}",
                    artifact["attemptStatus"],
                    artifact["attempt"],
                    artifact["stage"],
                    artifact["agent"],
                    artifact["role"],
                    artifact["root"],
                    artifact["path"],
                    artifact["sha256"]
                );
            }
        }
        if let Some(references) = assignment.get("memoryReferences") {
            println!("Role-scoped memory references (no contents): {references}");
        }
        if let Some(receipt) = assignment.get("harnessReceipt") {
            println!("Harness receipt reference: {receipt}");
        }
    }
    println!("\nThe host performs the assignment and executes checks; this command only reads the validated run context.");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn missing_assignments_fail_at_the_presentation_boundary() {
        assert!(super::print_next(&serde_json::json!({"valid": true})).is_err());
    }
}
