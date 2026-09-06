use serde_json::json;

use crate::{
    args::{self, Arguments},
    away, config, envelope, forgetting, hook_runtime, hooks, memory, profile, receipt, run,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn positional<'a>(a: &'a Arguments, index: usize, message: &str) -> Result<&'a str, String> {
    a.positional
        .get(index)
        .map(String::as_str)
        .ok_or_else(|| message.to_owned())
}

fn option<'a>(a: &'a Arguments, name: &str, message: &str) -> Result<&'a str, String> {
    a.options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| message.to_owned())
}

pub fn run(argv: Vec<String>) -> Result<(), String> {
    if argv.is_empty() {
        print_help();
        return Ok(());
    }

    let command = match argv[0].as_str() {
        "--help" => "help",
        "--version" => "version",
        command => command,
    };
    let parsed = if matches!(argv[0].as_str(), "--help" | "--version") {
        args::parse(&argv)?
    } else {
        args::parse(&argv[1..])?
    };

    if command == "version" || parsed.flags.contains_key("version") {
        args::assert_options("version", &parsed, &["version", "help"])?;
        args::assert_positionals("version", &parsed, 0)?;
        println!("{VERSION}");
        return Ok(());
    }
    if command == "help" {
        args::assert_options(command, &parsed, &["help", "version"])?;
        if parsed.positional == ["advanced"] {
            print_advanced_help();
            return Ok(());
        }
        args::assert_positionals(command, &parsed, 0)?;
        print_help();
        return Ok(());
    }
    if parsed.flags.contains_key("help") {
        args::assert_options(command, &parsed, &["help", "version"])?;
        args::assert_positionals(command, &parsed, 0)?;
        print_help();
        return Ok(());
    }
    match command {
        "hook-protocol" => {
            args::assert_options(command, &parsed, &[])?;
            args::assert_positionals(command, &parsed, 0)?;
            println!("{}", hooks::PROTOCOL);
            Ok(())
        }
        "hook-run" => {
            args::assert_options(command, &parsed, &[])?;
            args::assert_positionals(command, &parsed, 0)?;
            hook_runtime::run()
        }
        "init" => crate::project_commands::init(&parsed),
        "bind" => crate::project_commands::bind(&parsed),
        "doctor" => crate::project_commands::doctor(&parsed),
        "hooks" => hooks_command(&parsed),
        "profile" if parsed.positional.first().map(String::as_str) == Some("audit") => {
            profile_audit_command(&parsed)
        }
        command => configured_command(command, &parsed),
    }
}

fn hooks_command(a: &Arguments) -> Result<(), String> {
    args::assert_options("hooks", a, &["hosts", "root", "json"])?;
    args::assert_positionals("hooks", a, 1)?;
    let action = positional(
        a,
        0,
        "hooks requires one action: plan, apply, status, or remove",
    )?;
    let hosts = option(
        a,
        "hosts",
        "hooks requires explicit --hosts (for example --hosts codex,claude)",
    )?;
    let items = hooks::manage(
        action,
        hosts,
        a.options.get("root").map(String::as_str).unwrap_or("."),
    )?;
    if a.flags.contains_key("json") {
        print_json(&json!({ "action": action, "hosts": items }))?;
    } else {
        for item in items {
            println!(
                "{}: {}\n  target: {}",
                item["host"], item["state"], item["targetPath"]
            );
        }
    }
    Ok(())
}

fn configured_command(command: &str, a: &Arguments) -> Result<(), String> {
    if command == "benchmark" {
        return benchmark_command(a);
    }
    if !matches!(
        command,
        "check" | "brief" | "plan" | "verify" | "profile" | "memory" | "run" | "away" | "migrate"
    ) {
        return Err(format!("unknown command '{command}'"));
    }
    let loaded = config::load(a.options.get("config").map(String::as_str))?;
    match command {
        "check" => crate::project_commands::check(&loaded, a),
        "brief" => brief_command(&loaded, a),
        "plan" => plan_command(&loaded, a),
        "verify" => verify_command(&loaded, a),
        "profile" => profile_command(&loaded, a),
        "memory" => memory_command(&loaded, a),
        "run" => run_command(&loaded, a),
        "away" => away_command(&loaded, a),
        "migrate" => migrate_command(&loaded, a),
        _ => Err(format!("unknown command '{command}'")),
    }
}

fn benchmark_command(a: &Arguments) -> Result<(), String> {
    args::assert_options("benchmark", a, &["output", "json"])?;
    args::assert_positionals("benchmark", a, 0)?;
    let value = crate::value_benchmark::run(a.options.get("output").map(String::as_str))?;
    if a.flags.contains_key("json") {
        print_json(&value)
    } else {
        print!("{}", crate::value_benchmark::render(&value)?);
        Ok(())
    }
}

fn migrate_command(l: &config::Loaded, a: &Arguments) -> Result<(), String> {
    args::assert_options("migrate", a, &["config", "apply"])?;
    args::assert_positionals("migrate", a, 1)?;
    let apply = a.flags.contains_key("apply");
    let value = match positional(a, 0, "migrate requires layout or paths")? {
        "layout" => crate::layout_migration::run(l, apply),
        "paths" => crate::layout_migration::prepare_paths(l, apply),
        _ => return Err("migrate requires layout or paths".into()),
    }?;
    print_json(&value)
}

fn away_command(l: &config::Loaded, a: &Arguments) -> Result<(), String> {
    let action = positional(a, 0, "away requires start, list, or show")?;
    match action {
        "start" => {
            args::assert_options(
                "away start",
                a,
                &["config", "name", "require-harness-receipt", "sandbox-mode"],
            )?;
            args::assert_positionals("away start", a, 3)?;
            let result = away::start(
                l,
                positional(a, 1, "away start requires AGENT LEDGER")?,
                positional(a, 2, "away start requires AGENT LEDGER")?,
                a.options.get("name").map(String::as_str).unwrap_or("away"),
                a.flags.contains_key("require-harness-receipt"),
                a.options.get("sandbox-mode").map(String::as_str),
            )?;
            println!(
                "run_id={}\nsocket={}\nsession={}\nstate={}",
                result["runId"].as_str().unwrap_or(""),
                result["socket"].as_str().unwrap_or(""),
                result["session"].as_str().unwrap_or(""),
                result["state"].as_str().unwrap_or("")
            );
            Ok(())
        }
        "list" => {
            args::assert_options("away list", a, &["config"])?;
            args::assert_positionals("away list", a, 1)?;
            let runs = away::list(l)?;
            if runs.is_empty() {
                println!("no Soulmate away runs");
            } else {
                for (run, status) in runs {
                    println!("{run}\t{status}");
                }
            }
            Ok(())
        }
        "show" => {
            args::assert_options("away show", a, &["config"])?;
            args::assert_positionals("away show", a, 2)?;
            for (name, value) in away::show(l, positional(a, 1, "away show requires RUN_ID")?)? {
                println!("{name}={value}");
            }
            Ok(())
        }
        "_run" => {
            args::assert_options("away _run", a, &["config"])?;
            args::assert_positionals("away _run", a, 7)?;
            away::run_child(
                l,
                positional(a, 1, "invalid internal away command")?,
                positional(a, 2, "invalid internal away command")?,
                positional(a, 3, "invalid internal away command")?,
                (
                    positional(a, 4, "invalid internal away command")?,
                    positional(a, 5, "invalid internal away command")?,
                ),
                positional(a, 6, "invalid internal away command")?,
            )
        }
        _ => Err("away requires start, list, or show".into()),
    }
}

fn brief_command(l: &config::Loaded, a: &Arguments) -> Result<(), String> {
    args::assert_options(
        "brief",
        a,
        &["config", "task", "receipt", "harness-manifest", "json"],
    )?;
    args::assert_positionals("brief", a, 1)?;
    let name = positional(a, 0, "brief accepts 1 positional argument")?;
    let task = option(a, "task", "--task requires a non-empty value")?;
    let value = envelope::brief(l, name, task)?;
    if let Some(path) = a.options.get("receipt") {
        receipt::write(
            path,
            l,
            &value,
            a.options.get("harness-manifest").map(String::as_str),
        )?;
    }
    if a.flags.contains_key("json") {
        print_json(&value)?;
    } else {
        print!("{}", envelope::render(&value));
    }
    Ok(())
}

fn plan_command(l: &config::Loaded, a: &Arguments) -> Result<(), String> {
    args::assert_options(
        "plan",
        a,
        &["config", "goal", "receipt", "harness-manifest"],
    )?;
    args::assert_positionals("plan", a, 1)?;
    let workflow = positional(a, 0, "plan accepts 1 positional argument")?;
    let goal = option(a, "goal", "--goal requires a non-empty value")?;
    let value = envelope::plan(l, workflow, goal)?;
    if let Some(path) = a.options.get("receipt") {
        receipt::write(
            path,
            l,
            &value,
            a.options.get("harness-manifest").map(String::as_str),
        )?;
    }
    print_json(&value)?;
    Ok(())
}

fn verify_command(l: &config::Loaded, a: &Arguments) -> Result<(), String> {
    args::assert_options("verify", a, &["config"])?;
    args::assert_positionals("verify", a, 1)?;
    let value = receipt::verify(positional(a, 0, "verify requires a receipt path")?, l)?;
    print_json(&value)?;
    if !value["valid"].as_bool().unwrap_or(false) {
        return Err("receipt verification failed".into());
    }
    Ok(())
}

fn profile_command(l: &config::Loaded, a: &Arguments) -> Result<(), String> {
    let action = a.positional.first().map(String::as_str).unwrap_or("");
    match action {
        "audit" => profile_audit_command(a),
        "import" => {
            args::assert_options("profile import", a, &["config", "purpose", "forbid-term"])?;
            args::assert_positionals("profile import", a, 3)?;
            let result = profile::import(
                l,
                positional(a, 1, "profile import requires NAME SOURCE")?,
                positional(a, 2, "profile import requires NAME SOURCE")?,
                option(a, "purpose", "profile import requires --purpose")?,
                a.options.get("forbid-term").map(String::as_str),
            )?;
            print!("{result}");
            Ok(())
        }
        "" => Err("profile requires AGENT".into()),
        _ => {
            args::assert_options("profile", a, &["config"])?;
            args::assert_positionals("profile", a, 1)?;
            let name = positional(a, 0, "profile requires AGENT")?;
            let agent = l
                .agent(name)
                .ok_or_else(|| format!("unknown agent '{name}'"))?;
            let path = config::file(&l.control_root, &agent.profile)?;
            print!(
                "{}",
                std::fs::read_to_string(path).map_err(|e| e.to_string())?
            );
            Ok(())
        }
    }
}

fn profile_audit_command(a: &Arguments) -> Result<(), String> {
    args::assert_options("profile audit", a, &["config", "forbid-term", "json"])?;
    args::assert_positionals("profile audit", a, 2)?;
    let value = profile::audit(
        positional(a, 1, "profile audit requires SOURCE")?,
        a.options.get("forbid-term").map(String::as_str),
    )?;
    if a.flags.contains_key("json") {
        print_json(&value)?;
    } else {
        println!(
            "Profile audit {}: {}",
            if value["valid"].as_bool().unwrap_or(false) {
                "passed"
            } else {
                "failed"
            },
            value["source"]
        );
    }
    if value["valid"].as_bool().unwrap_or(false) {
        Ok(())
    } else {
        Err("profile audit failed".into())
    }
}

fn memory_command(l: &config::Loaded, a: &Arguments) -> Result<(), String> {
    let action = positional(a, 0, "memory requires an action")?;
    match action {
        "resolve" => {
            args::assert_options("memory resolve", a, &["config", "json"])?;
            args::assert_positionals("memory resolve", a, 2)?;
            let agent = positional(a, 1, "memory resolve requires AGENT")?;
            let value = memory::resolve(l, agent)?;
            if a.flags.contains_key("json") {
                print_json(&value)?;
            } else if let Some(references) = value["references"].as_array() {
                for reference in references {
                    println!(
                        "{} {} {} {}",
                        reference["itemId"].as_str().unwrap_or(""),
                        reference["scope"].as_str().unwrap_or(""),
                        reference["sourcePath"].as_str().unwrap_or(""),
                        reference["sourceSha256"].as_str().unwrap_or("")
                    );
                }
            }
            Ok(())
        }
        "attest-forgotten" => {
            args::assert_options("memory attest-forgotten", a, &["config", "receipt"])?;
            args::assert_positionals("memory attest-forgotten", a, 3)?;
            let actor = positional(a, 1, "memory attest-forgotten requires AGENT LEDGER")?;
            let ledger = positional(a, 2, "memory attest-forgotten requires AGENT LEDGER")?;
            let receipt = option(a, "receipt", "memory attest-forgotten requires --receipt")?;
            let value = forgetting::attest(l, actor, ledger, receipt)?;
            print_json(&value)?;
            Ok(())
        }
        "inspect" => {
            args::assert_options("memory inspect", a, &["config", "json"])?;
            args::assert_positionals("memory inspect", a, 2)?;
            let value = memory::inspect(l, positional(a, 1, "memory inspect accepts LEDGER")?)?;
            if a.flags.contains_key("json") {
                print_json(&value)?;
            } else if let Some(items) = value["items"].as_array() {
                for item in items {
                    println!("{} {} {}", item["itemId"], item["state"], item["scope"]);
                }
            }
            Ok(())
        }
        "propose" => memory_transition(l, a, true),
        _ => memory_transition(l, a, false),
    }
}

fn memory_transition(l: &config::Loaded, a: &Arguments, propose: bool) -> Result<(), String> {
    let allowed = if propose {
        &["config", "ledger", "scope", "expires-at"][..]
    } else {
        &["config"][..]
    };
    args::assert_options("memory", a, allowed)?;
    args::assert_positionals("memory", a, 3)?;
    let actor = positional(a, 1, "memory action requires AGENT LEDGER")?;
    let ledger = if propose {
        option(a, "ledger", "memory propose requires --ledger")?
    } else {
        positional(a, 2, "memory transition requires LEDGER")?
    };
    let value = memory::action(
        l,
        actor,
        positional(a, 0, "memory requires an action")?,
        if propose {
            a.positional.get(2).map(String::as_str)
        } else {
            None
        },
        a.options.get("scope").map(String::as_str),
        ledger,
        a.options.get("expires-at").map(String::as_str),
    )?;
    print_json(&value["event"])?;
    Ok(())
}

fn run_command(l: &config::Loaded, a: &Arguments) -> Result<(), String> {
    let action = positional(a, 0, "run requires an action")?;
    let allowed = match action {
        "start" => &[
            "config",
            "goal",
            "ledger",
            "boundary",
            "harness-receipt",
            "check-command",
            "proof-origin",
        ][..],
        "next" => &["config", "json", "text"][..],
        "inspect" => &["config", "json"][..],
        "submit" => &["config", "outcome", "artifact", "artifact-root", "json", "event-id"][..],
        "record-check" => &[
            "config",
            "target",
            "check-command",
            "exit-code",
            "duration-ms",
            "json",
        ][..],
        "status" => &["config", "json"][..],
        "explain" => &["config", "event", "json"][..],
        "report" => &["config", "json"][..],
        "supersede" => &[
            "config",
            "workflow",
            "goal",
            "ledger",
            "boundary",
            "harness-receipt",
            "check-command",
            "proof-origin",
            "json",
        ][..],
        _ => {
            return Err("run requires one action: start, next, submit, record-check, status, explain, report, inspect, or supersede".into())
        }
    };
    args::assert_options("run", a, allowed)?;
    for alternative in ["event-id", "text"] {
        if a.flags.contains_key(alternative) && a.flags.contains_key("json") {
            return Err(format!("--{alternative} and --json are mutually exclusive"));
        }
    }
    let value = match action {
        "start" => {
            args::assert_positionals("run start", a, 2)?;
            let workflow = positional(a, 1, "run start requires WORKFLOW")?;
            let goal = option(a, "goal", "run start requires --goal")?;
            let ledger = option(a, "ledger", "run start requires --ledger")?;
            let boundary = a.options.get("boundary").map(String::as_str);
            let receipt = a.options.get("harness-receipt").map(String::as_str);
            let check_command = a.options.get("check-command").map(String::as_str);
            let proof_origin = a.options.get("proof-origin").map(String::as_str);
            if check_command.is_none() && proof_origin.is_none() {
                run::start(l, workflow, goal, ledger, boundary, receipt)
            } else {
                run::start_with_policy(
                    l,
                    workflow,
                    goal,
                    ledger,
                    boundary,
                    receipt,
                    check_command,
                    proof_origin,
                )
            }
        }
        "next" => {
            args::assert_positionals("run next", a, 2)?;
            run::next(l, positional(a, 1, "run next requires LEDGER")?)
        }
        "submit" => {
            args::assert_positionals("run submit", a, 3)?;
            run::submit(
                l,
                positional(a, 1, "run submit requires AGENT")?,
                positional(a, 2, "run submit requires LEDGER")?,
                option(a, "outcome", "run submit requires --outcome")?,
                option(a, "artifact", "run submit requires --artifact")?,
                a.options.get("artifact-root").map(String::as_str),
            )
        }
        "inspect" => {
            args::assert_positionals("run inspect", a, 2)?;
            run::inspect(l, positional(a, 1, "run inspect requires LEDGER")?)
        }
        "record-check" => {
            args::assert_positionals("run record-check", a, 2)?;
            run::record_check(
                l,
                positional(a, 1, "run record-check requires LEDGER")?,
                option(a, "target", "run record-check requires --target")?,
                option(
                    a,
                    "check-command",
                    "run record-check requires --check-command",
                )?,
                option(a, "exit-code", "run record-check requires --exit-code")?,
                a.options.get("duration-ms").map(String::as_str),
            )
        }
        "status" => {
            args::assert_positionals("run status", a, 2)?;
            let ledger = positional(a, 1, "run status requires LEDGER")?;
            let json_output = a.flags.contains_key("json");
            let config = if json_output {
                ""
            } else {
                a.options.get("config").map(String::as_str).map_or_else(
                    || l.path.to_str().ok_or("configuration path is not valid UTF-8"),
                    Ok,
                )?
            };
            let value = run::status(l, ledger).map_err(|error| {
                if !json_output && run::inspect(l, ledger).is_ok() {
                    eprintln!(
                        "Inspect: {}",
                        crate::presentation::read_command("inspect", config, ledger)
                    );
                }
                map_run_error(error, json_output)
            })?;
            if a.flags.contains_key("json") {
                print_json(&value)?;
            } else {
                print_status(&value);
                if value["status"] != "running" {
                    println!("Guidance: this run is terminal; inspect its recorded outcome before choosing an explicit successor where supported.");
                    println!("Inspect: {}", crate::presentation::read_command("inspect", config, ledger));
                } else if value["artifact"]["status"] != "current" {
                    println!("Guidance: artifact drift prevents progression. Inspect the recorded references and restore the exact recorded bytes before retrying.");
                    println!("Inspect: {}", crate::presentation::read_command("inspect", config, ledger));
                } else {
                    match run::next(l, ledger) {
                        Ok(next) if next["status"] == "running" => println!(
                            "Next: {}",
                            crate::presentation::read_command("next", config, ledger)
                        ),
                        _ => {
                            println!("Guidance: no validated pending progression is available; inspect the run before recovery.");
                            println!("Inspect: {}", crate::presentation::read_command("inspect", config, ledger));
                        }
                    }
                }
            }
            return Ok(());
        }
        "explain" => {
            args::assert_positionals("run explain", a, 2)?;
            let value = run::explain(
                l,
                positional(a, 1, "run explain requires LEDGER")?,
                a.options.get("event").map(String::as_str),
            )
            .map_err(|error| map_run_error(error, a.flags.contains_key("json")))?;
            if a.flags.contains_key("json") {
                print_json(&value)?;
            } else {
                print_explain(&value);
            }
            return Ok(());
        }
        "report" => {
            if a.positional.len() < 2 {
                return Err("run report requires at least one LEDGER".into());
            }
            let ledgers = a
                .positional
                .iter()
                .skip(1)
                .map(String::as_str)
                .collect::<Vec<_>>();
            let report = run::report(l, &ledgers)?;
            if a.flags.contains_key("json") {
                print_json(&report)?;
            } else {
                print!("{}", run::report_markdown(&report));
            }
            return Ok(());
        }
        "supersede" => {
            args::assert_positionals("run supersede", a, 2)?;
            let old_ledger = positional(a, 1, "run supersede requires OLD_LEDGER")?;
            let workflow = option(a, "workflow", "run supersede requires --workflow")?;
            let goal = option(a, "goal", "run supersede requires --goal")?;
            let ledger = option(a, "ledger", "run supersede requires --ledger")?;
            let boundary = a.options.get("boundary").map(String::as_str);
            let receipt = a.options.get("harness-receipt").map(String::as_str);
            let check_command = a.options.get("check-command").map(String::as_str);
            let proof_origin = a.options.get("proof-origin").map(String::as_str);
            if check_command.is_none() && proof_origin.is_none() {
                run::supersede(l, old_ledger, workflow, goal, ledger, boundary, receipt)
            } else {
                run::supersede_with_policy(
                    l,
                    old_ledger,
                    workflow,
                    goal,
                    ledger,
                    boundary,
                    receipt,
                    check_command,
                    proof_origin,
                )
            }
        }
        _ => return Err("unsupported run action".into()),
    }
    .map_err(|error| map_run_error(error, a.flags.contains_key("json")))?;
    if action == "start" {
        if a.options.contains_key("check-command") {
            eprintln!("Checked run: acceptance requires a caller-reported passing result for each current worker submission, bound to the frozen check command. The host executes the command; Soulmate records the report.");
        } else {
            eprintln!("Unchecked run: no check-result requirement is configured. Start with --check-command to require caller-reported checks before acceptance.");
        }
    }
    if action == "submit" && a.flags.contains_key("event-id") {
        println!(
            "{}",
            value["event"]["eventSha256"]
                .as_str()
                .expect("successful submission has a validated event hash")
        );
    } else if action == "next" && a.flags.contains_key("text") {
        crate::presentation::print_next(&value)?;
    } else {
        print_json(&value)?;
    }
    Ok(())
}

fn map_run_error(error: String, json_output: bool) -> String {
    let Some(machine) = error.strip_prefix("SOULMATE_DRIFT:") else {
        return error;
    };
    if json_output {
        return format!("SOULMATE_JSON:{machine}");
    } else if let Ok(value) = serde_json::from_str::<serde_json::Value>(machine) {
        if value["classification"] == "config_drift" {
            eprintln!(
                "configuration drift detected after run start (expected {}, current {}). Inspect the old run, then use 'soulmate run supersede' to begin an explicit successor.",
                value["expectedConfigSha256"], value["currentConfigSha256"]
            );
        } else if value["classification"] == "profile_drift" {
            eprintln!(
                "profile drift detected after run start for {} (expected {}, current {}). Inspect the old run before choosing a successor.",
                value["agent"], value["expectedProfileSha256"], value["currentProfileSha256"]
            );
        } else if value["classification"] == "boundary_drift" {
            eprintln!(
                "run boundary manifest drift detected after run start (expected {}, current {}). Restore the exact manifest or explicitly supersede the run.",
                value["expectedBoundarySha256"], value["currentBoundarySha256"]
            );
        } else if value["classification"] == "harness_receipt_drift" {
            eprintln!(
                "harness receipt drift detected after run start (expected {}, current {}). Restore the exact receipt and manifest or explicitly supersede the run.",
                value["expectedHarnessReceiptSha256"], value["currentHarnessReceiptSha256"]
            );
        } else {
            eprintln!(
                "memory drift detected after run start for {} (expected set {}, current set {}). Inspect the old run and current memory references, then use 'soulmate run supersede' for an intentional successor.",
                value["agent"], value["expectedMemorySetSha256"], value["currentMemorySetSha256"]
            );
        }
    }
    if machine.contains("\"classification\":\"boundary_drift\"") {
        "run boundary drift".to_string()
    } else if machine.contains("\"classification\":\"harness_receipt_drift\"") {
        "harness receipt drift".to_string()
    } else if machine.contains("\"classification\":\"memory_drift\"") {
        "run memory drift".to_string()
    } else if machine.contains("\"classification\":\"profile_drift\"") {
        "run profile drift".to_string()
    } else {
        "run configuration drift".to_string()
    }
}

fn print_help() {
    println!(
        "Soulmate {VERSION}\n\nUsage: soulmate <command> [options]\n\nCore: init, brief, run, check\nRun actions: start, next, submit, record-check, status, explain, report, inspect, supersede.\nUse 'run next LEDGER --text' for readable pending assignments and 'run submit AGENT LEDGER --event-id' to capture the submitted event hash. Each output flag conflicts with --json; default JSON is unchanged.\n\nRun 'soulmate help advanced' for lifecycle, recovery, migration, hooks, receipts, and optional execution convenience."
    );
}

fn print_advanced_help() {
    println!(
        "Soulmate {VERSION}\n\nAdvanced: bind, doctor, plan, verify, profile, migrate, memory (resolve/inspect/lifecycle), away, hooks, hook-protocol, hook-run, version\n\nRun value proof: the host executes the configured check, then reports its actual result with 'run record-check'; use 'run status', 'run explain', and 'run report' for bounded evidence views.\n\nRun 'soulmate migrate layout --config CONFIG' to inspect a legacy profile migration, then repeat with --apply. Use 'migrate paths' for canonical harness and state directories.\nRun 'soulmate run supersede OLD_LEDGER --workflow WORKFLOW --goal GOAL --ledger NEW_LEDGER' to create an explicit successor after configuration, profile, memory, boundary, or harness-receipt drift."
    );
}

fn print_json(value: &serde_json::Value) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn print_status(value: &serde_json::Value) {
    println!(
        "Run {}: {} (stage {}, attempt {})",
        text_field(value, "runId"),
        text_field(value, "status"),
        scalar_field(value, "stage"),
        scalar_field(value, "attempt")
    );
    let claim = &value["claim"];
    println!(
        "Claim: {} event={} artifact={}",
        text_field(claim, "status"),
        text_field(claim, "eventSha256"),
        text_field(claim, "artifactSha256")
    );
    println!("Artifact: {}", text_field(&value["artifact"], "status"));
    let checks = &value["checks"];
    println!(
        "Checks: {} (origin {}, observed {}, failed {}, missing {})",
        text_field(checks, "status"),
        text_field(checks, "origin"),
        scalar_field(checks, "observedCount"),
        scalar_field(checks, "failedCount"),
        scalar_field(checks, "missingCount")
    );
    if let Some(targets) = checks["targets"].as_array() {
        for target in targets {
            println!(
                "  target {}: {} check={} exit={}",
                text_field(target, "targetEventSha256"),
                text_field(target, "status"),
                text_field(target, "checkEventSha256"),
                scalar_field(target, "exitCode")
            );
        }
    }
    print_review_or_acceptance("Review", &value["review"]);
    print_review_or_acceptance("Acceptance", &value["acceptance"]);
    if value["status"] != "running" || value["artifact"]["status"] != "current" {
        return;
    }
    match checks["status"].as_str() {
        Some("not_observed") => println!(
            "Guidance: run the configured check in its host and report the actual result for every worker target with run record-check."
        ),
        Some("blocked") => println!(
            "Guidance: repair or rework as needed, rerun the configured check in its host, report the actual result, then request review and lead acceptance."
        ),
        Some("passed") => println!(
            "Guidance: checks passed; reviewer approval and lead acceptance remain separate authority steps."
        ),
        _ => {}
    }
}

fn print_explain(value: &serde_json::Value) {
    println!("Run {} explanation", text_field(value, "runId"));
    let status = &value["status"];
    println!(
        "Status: {} (artifact {})",
        text_field(status, "status"),
        text_field(&status["artifact"], "status")
    );
    let protection = &value["protection"];
    if protection.is_null() {
        println!("Protection: none selected");
    } else {
        println!(
            "Protection: {} reason={} actor={} event={}",
            text_field(protection, "attemptedOutcome"),
            text_field(protection, "reason"),
            text_field(protection, "actor"),
            text_field(protection, "eventSha256")
        );
        if let Some(evidence) = protection["checkEvidence"].as_array() {
            for item in evidence {
                println!(
                    "  evidence target={} status={} check={} exit={}",
                    text_field(item, "targetEventSha256"),
                    text_field(item, "status"),
                    text_field(item, "checkEventSha256"),
                    scalar_field(item, "exitCode")
                );
            }
        }
    }
    println!("Guidance: {}", text_field(value, "guidance"));
}

fn print_review_or_acceptance(label: &str, value: &serde_json::Value) {
    println!(
        "{label}: {} event={} artifact={}",
        text_field(value, "status"),
        text_field(value, "eventSha256"),
        text_field(value, "artifactSha256")
    );
}

fn text_field<'a>(value: &'a serde_json::Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or("unknown")
}

fn scalar_field(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .filter(|value| !value.is_null())
        .map_or_else(|| "unknown".to_owned(), ToString::to_string)
}
