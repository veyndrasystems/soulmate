mod support;

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn call(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_soulmate"))
        .current_dir(root)
        .args(args)
        .output()
        .expect("soulmate should start")
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("invalid JSON: {error}; output: {}", text(output)))
}

fn run(root: &Path, args: &[&str]) -> Value {
    let output = call(root, args);
    assert!(output.status.success(), "{}", text(&output));
    json(&output)
}

fn checked_worker(label: &str, command: &str) -> (std::path::PathBuf, String, String) {
    let root = support::temp(label);
    let init = call(&root, &["init", "--root", "."]);
    assert!(init.status.success(), "{}", text(&init));
    let ledger = format!(".soulmate/runs/{label}.jsonl");
    let output = call(
        &root,
        &[
            "run",
            "start",
            "change",
            "--goal",
            "observe",
            "--ledger",
            &ledger,
            "--check-command",
            command,
            "--config",
            "soulmate.json",
        ],
    );
    assert!(output.status.success(), "{}", text(&output));
    let artifact = format!(".soulmate/artifacts/{label}.md");
    fs::write(root.join(&artifact), "worker\n").unwrap();
    let output = call(
        &root,
        &[
            "run",
            "submit",
            "lead",
            &ledger,
            "--outcome",
            "scoped",
            "--artifact",
            &artifact,
            "--artifact-root",
            "state",
            "--config",
            "soulmate.json",
        ],
    );
    assert!(output.status.success(), "{}", text(&output));
    let worker_output = call(
        &root,
        &[
            "run",
            "submit",
            "worker",
            &ledger,
            "--outcome",
            "completed",
            "--artifact",
            &artifact,
            "--artifact-root",
            "state",
            "--config",
            "soulmate.json",
        ],
    );
    assert!(worker_output.status.success(), "{}", text(&worker_output));
    let worker = json(&worker_output);
    let target = worker["event"]["eventSha256"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    (root, ledger, target)
}

#[test]
fn observes_frozen_command_in_product_root_and_records_provenance() {
    let root = support::temp("observe-check");
    let config = root.join("soulmate.json");
    let init = call(&root, &["init", "--root", "."]);
    assert!(init.status.success(), "{}", text(&init));
    let ledger = ".soulmate/runs/observed.jsonl";
    run(
        &root,
        &[
            "run",
            "start",
            "change",
            "--goal",
            "observe",
            "--ledger",
            ledger,
            "--check-command",
            "printf child-stdout; printf child-stderr >&2; printf observed > observed.txt",
            "--config",
            "soulmate.json",
        ],
    );
    fs::write(root.join(".soulmate/artifacts/worker.md"), "worker\n").unwrap();
    run(
        &root,
        &[
            "run",
            "submit",
            "lead",
            ledger,
            "--outcome",
            "scoped",
            "--artifact",
            ".soulmate/artifacts/worker.md",
            "--artifact-root",
            "state",
            "--config",
            "soulmate.json",
        ],
    );
    let worker = run(
        &root,
        &[
            "run",
            "submit",
            "worker",
            ledger,
            "--outcome",
            "completed",
            "--artifact",
            ".soulmate/artifacts/worker.md",
            "--artifact-root",
            "state",
            "--config",
            "soulmate.json",
        ],
    );
    let target = worker["event"]["eventSha256"].as_str().unwrap();
    let observed_output = call(
        &root,
        &[
            "run",
            "observe-check",
            ledger,
            "--target",
            target,
            "--config",
            "soulmate.json",
        ],
    );
    assert!(
        observed_output.status.success(),
        "{}",
        text(&observed_output)
    );
    let observed = json(&observed_output);
    assert_eq!(observed["event"]["version"], 4);
    assert_eq!(observed["event"]["acquisition"], "observed");
    assert_eq!(observed["event"]["result"]["kind"], "exit");
    assert_eq!(observed["event"]["result"]["code"], 0);
    assert_eq!(observed["checks"]["targets"][0]["acquisition"], "observed");
    assert_eq!(observed["checks"]["targets"][0]["result"]["code"], 0);
    assert!(!String::from_utf8_lossy(&observed_output.stdout).starts_with("child-stdout"));
    assert!(String::from_utf8_lossy(&observed_output.stderr).contains("child-stdout"));
    assert!(String::from_utf8_lossy(&observed_output.stderr).contains("child-stderr"));
    let human = call(
        &root,
        &["run", "status", ledger, "--config", "soulmate.json"],
    );
    assert!(human.status.success(), "{}", text(&human));
    assert!(text(&human).contains("acquisition=observed"));
    assert!(text(&human).contains("kind\":\"exit\""));
    let report = run(
        &root,
        &[
            "run",
            "report",
            ledger,
            "--json",
            "--config",
            "soulmate.json",
        ],
    );
    assert_eq!(report["groups"]["local_report"]["durationMsReported"], 0);
    assert_eq!(
        report["groups"]["local_report"]["durationMsReportedCount"],
        0
    );
    assert_eq!(
        fs::read_to_string(root.join("observed.txt")).unwrap(),
        "observed"
    );

    let before = fs::read(root.join(ledger)).unwrap();
    let rejected = call(
        &root,
        &[
            "run",
            "observe-check",
            ledger,
            "--target",
            "not-a-target",
            "--config",
            config.to_str().unwrap(),
        ],
    );
    assert!(!rejected.status.success());
    assert_eq!(fs::read(root.join(ledger)).unwrap(), before);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn records_signal_without_fabricating_exit_code() {
    let root = support::temp("observe-signal");
    let init = call(&root, &["init", "--root", "."]);
    assert!(init.status.success(), "{}", text(&init));
    let ledger = ".soulmate/runs/signal.jsonl";
    run(
        &root,
        &[
            "run",
            "start",
            "change",
            "--goal",
            "observe",
            "--ledger",
            ledger,
            "--check-command",
            "kill -TERM $$",
            "--config",
            "soulmate.json",
        ],
    );
    fs::write(root.join(".soulmate/artifacts/worker.md"), "worker\n").unwrap();
    run(
        &root,
        &[
            "run",
            "submit",
            "lead",
            ledger,
            "--outcome",
            "scoped",
            "--artifact",
            ".soulmate/artifacts/worker.md",
            "--artifact-root",
            "state",
            "--config",
            "soulmate.json",
        ],
    );
    let worker = run(
        &root,
        &[
            "run",
            "submit",
            "worker",
            ledger,
            "--outcome",
            "completed",
            "--artifact",
            ".soulmate/artifacts/worker.md",
            "--artifact-root",
            "state",
            "--config",
            "soulmate.json",
        ],
    );
    let target = worker["event"]["eventSha256"].as_str().unwrap();
    let observed = run(
        &root,
        &[
            "run",
            "observe-check",
            ledger,
            "--target",
            target,
            "--config",
            "soulmate.json",
        ],
    );
    assert_eq!(observed["event"]["result"]["kind"], "signal");
    assert_eq!(observed["event"]["result"]["signal"], 15);
    assert!(observed["event"]["result"].get("code").is_none());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn nonzero_observation_is_recorded_and_acceptance_is_refused() {
    let (root, ledger, target) = checked_worker("observe-nonzero", "exit 7");
    let observed = call(
        &root,
        &[
            "run",
            "observe-check",
            &ledger,
            "--target",
            &target,
            "--config",
            "soulmate.json",
        ],
    );
    assert!(observed.status.success(), "{}", text(&observed));
    let value = json(&observed);
    assert_eq!(value["event"]["result"]["code"], 7);

    let reviewer = call(
        &root,
        &[
            "run",
            "submit",
            "reviewer",
            &ledger,
            "--outcome",
            "approved",
            "--artifact",
            ".soulmate/artifacts/observe-nonzero.md",
            "--artifact-root",
            "state",
            "--config",
            "soulmate.json",
        ],
    );
    assert!(reviewer.status.success(), "{}", text(&reviewer));
    let acceptance = call(
        &root,
        &[
            "run",
            "submit",
            "lead",
            &ledger,
            "--outcome",
            "accepted",
            "--artifact",
            ".soulmate/artifacts/observe-nonzero.md",
            "--artifact-root",
            "state",
            "--config",
            "soulmate.json",
        ],
    );
    assert!(!acceptance.status.success(), "{}", text(&acceptance));
    assert!(text(&acceptance).contains("acceptance refused"));
    let status = run(
        &root,
        &[
            "run",
            "status",
            &ledger,
            "--json",
            "--config",
            "soulmate.json",
        ],
    );
    assert_eq!(status["status"], "running");
    assert_eq!(status["acceptance"]["status"], "absent");
    let human = call(
        &root,
        &["run", "status", &ledger, "--config", "soulmate.json"],
    );
    assert!(human.status.success(), "{}", text(&human));
    let human_text = text(&human);
    assert!(human_text.contains("acquisition=observed"));
    assert!(!human_text.contains("acquisition=reported"));
    let inspected = run(
        &root,
        &[
            "run",
            "inspect",
            &ledger,
            "--json",
            "--config",
            "soulmate.json",
        ],
    );
    let protection = inspected["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["action"] == "protect")
        .and_then(|event| event["eventSha256"].as_str())
        .unwrap();
    let explanation = call(
        &root,
        &[
            "run",
            "explain",
            &ledger,
            "--event",
            protection,
            "--config",
            "soulmate.json",
        ],
    );
    assert!(explanation.status.success(), "{}", text(&explanation));
    let explanation_text = text(&explanation);
    assert!(explanation_text.contains("acquisition=observed"));
    assert!(!explanation_text.contains("acquisition=reported"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn passing_observation_does_not_auto_review_or_accept() {
    let (root, ledger, target) = checked_worker("observe-zero", "exit 0");
    let observed = call(
        &root,
        &[
            "run",
            "observe-check",
            &ledger,
            "--target",
            &target,
            "--config",
            "soulmate.json",
        ],
    );
    assert!(observed.status.success(), "{}", text(&observed));
    let status = run(
        &root,
        &[
            "run",
            "status",
            &ledger,
            "--json",
            "--config",
            "soulmate.json",
        ],
    );
    assert_eq!(status["checks"]["status"], "passed");
    assert_eq!(status["review"]["status"], "absent");
    assert_eq!(status["acceptance"]["status"], "absent");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn launch_failure_and_caller_overrides_do_not_append() {
    let (root, ledger, target) = checked_worker("observe-launch", "printf launched > launched.txt");
    let before = fs::read(root.join(&ledger)).unwrap();
    let bin = root.join("no-sh");
    fs::create_dir(&bin).unwrap();
    std::os::unix::fs::symlink("/usr/bin/git", bin.join("git")).unwrap();
    let failed = Command::new(env!("CARGO_BIN_EXE_soulmate"))
        .current_dir(&root)
        .env("PATH", &bin)
        .args([
            "run",
            "observe-check",
            &ledger,
            "--target",
            &target,
            "--config",
            "soulmate.json",
        ])
        .output()
        .unwrap();
    assert!(!failed.status.success());
    assert!(text(&failed).contains("could not be launched"));
    assert_eq!(fs::read(root.join(&ledger)).unwrap(), before);
    assert!(!root.join("launched.txt").exists());

    for extra in [
        vec!["--check-command", "exit 0"],
        vec!["--exit-code", "0"],
        vec!["--duration-ms", "0"],
        vec!["--env", "X=1"],
        vec!["--cwd", "."],
    ] {
        let mut args = vec![
            "run",
            "observe-check",
            &ledger,
            "--target",
            &target,
            "--config",
            "soulmate.json",
        ];
        args.extend(extra);
        let rejected = call(&root, &args);
        assert!(!rejected.status.success(), "{}", text(&rejected));
        assert_eq!(fs::read(root.join(&ledger)).unwrap(), before);
        assert!(!root.join("launched.txt").exists());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn stale_target_after_supersede_and_post_execution_drift_do_not_append() {
    let (root, ledger, target) = checked_worker("observe-stale", "printf launched > launched.txt");
    let successor = ".soulmate/runs/observe-successor.jsonl";
    let supersede = call(
        &root,
        &[
            "run",
            "supersede",
            &ledger,
            "--workflow",
            "change",
            "--goal",
            "new",
            "--ledger",
            successor,
            "--check-command",
            "printf launched > launched.txt",
            "--config",
            "soulmate.json",
        ],
    );
    assert!(supersede.status.success(), "{}", text(&supersede));
    let rejected = call(
        &root,
        &[
            "run",
            "observe-check",
            &ledger,
            "--target",
            &target,
            "--config",
            "soulmate.json",
        ],
    );
    assert!(!rejected.status.success());
    assert!(!root.join("launched.txt").exists());

    let (root, ledger, target) = checked_worker("observe-drift", "printf changed > soulmate.json");
    let before = fs::read(root.join(&ledger)).unwrap();
    let drifted = call(
        &root,
        &[
            "run",
            "observe-check",
            &ledger,
            "--target",
            &target,
            "--config",
            "soulmate.json",
        ],
    );
    assert!(!drifted.status.success());
    assert!(text(&drifted).contains("drift"));
    assert_eq!(fs::read(root.join(&ledger)).unwrap(), before);
    fs::remove_dir_all(root).unwrap();

    let (root, ledger, target) = checked_worker(
        "observe-artifact-drift",
        "printf changed > .soulmate/artifacts/observe-artifact-drift.md",
    );
    let before = fs::read(root.join(&ledger)).unwrap();
    let drifted = call(
        &root,
        &[
            "run",
            "observe-check",
            &ledger,
            "--target",
            &target,
            "--config",
            "soulmate.json",
        ],
    );
    assert!(!drifted.status.success());
    assert!(text(&drifted).contains("artifact"));
    assert_eq!(fs::read(root.join(&ledger)).unwrap(), before);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reviewer_rework_makes_old_target_stale_before_observation_launch() {
    let (root, ledger, old_target) = checked_worker(
        "observe-reviewer-rework",
        "printf launched > reviewer-rework-launched.txt",
    );
    let rework_artifact = ".soulmate/artifacts/observe-reviewer-rework-request.md";
    fs::write(root.join(rework_artifact), "please rework\n").unwrap();
    let reviewer = call(
        &root,
        &[
            "run",
            "submit",
            "reviewer",
            &ledger,
            "--outcome",
            "rework",
            "--artifact",
            rework_artifact,
            "--artifact-root",
            "state",
            "--config",
            "soulmate.json",
        ],
    );
    assert!(reviewer.status.success(), "{}", text(&reviewer));
    let fresh_artifact = ".soulmate/artifacts/observe-reviewer-rework-fresh.md";
    fs::write(root.join(fresh_artifact), "fresh completion\n").unwrap();
    let worker = call(
        &root,
        &[
            "run",
            "submit",
            "worker",
            &ledger,
            "--outcome",
            "completed",
            "--artifact",
            fresh_artifact,
            "--artifact-root",
            "state",
            "--config",
            "soulmate.json",
        ],
    );
    assert!(worker.status.success(), "{}", text(&worker));
    let before = fs::read(root.join(&ledger)).unwrap();
    let rejected = call(
        &root,
        &[
            "run",
            "observe-check",
            &ledger,
            "--target",
            &old_target,
            "--config",
            "soulmate.json",
        ],
    );
    assert!(!rejected.status.success(), "{}", text(&rejected));
    assert!(text(&rejected).contains("current worker completion"));
    assert_eq!(fs::read(root.join(&ledger)).unwrap(), before);
    assert!(!root.join("reviewer-rework-launched.txt").exists());
    fs::remove_dir_all(root).unwrap();
}
