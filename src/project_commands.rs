//! CLI commands that initialize, bind, and diagnose a project layout.

use crate::{args, args::Arguments, config, onboarding, project_skills};
use serde_json::json;

pub(crate) fn init(arguments: &Arguments) -> Result<(), String> {
    args::assert_options(
        "init",
        arguments,
        &[
            "root",
            "with-coffee",
            "refresh-skills",
            "mode",
            "project-id",
            "control-root",
            "state-root",
        ],
    )?;
    args::assert_positionals("init", arguments, 0)?;
    let root = arguments
        .options
        .get("root")
        .map(String::as_str)
        .unwrap_or(".");
    if arguments.flags.contains_key("refresh-skills") {
        let statuses = onboarding::refresh(root, arguments.flags.contains_key("with-coffee"))?;
        println!(
            "Refreshed project skills:\n{}",
            statuses
                .iter()
                .map(|status| {
                    let state = match status.state {
                        project_skills::SkillRefreshState::Created => "created",
                        project_skills::SkillRefreshState::Refreshed => "refreshed",
                        project_skills::SkillRefreshState::Unchanged => "unchanged",
                    };
                    format!("  {state} {}", status.path)
                })
                .collect::<Vec<_>>()
                .join("\n")
        );
        return Ok(());
    }
    let path = onboarding::init_with_options(
        root,
        arguments.flags.contains_key("with-coffee"),
        arguments.options.get("mode").map(String::as_str),
        arguments.options.get("project-id").map(String::as_str),
        arguments.options.get("control-root").map(String::as_str),
        arguments.options.get("state-root").map(String::as_str),
    )?;
    let coffee = if arguments.flags.contains_key("with-coffee") {
        " + opt-in Coffee"
    } else {
        ""
    };
    println!(
        "Created {}\nActivated project skills for Codex and Claude: Soulmate{coffee}.\nNext:\n  soulmate brief worker --task \"Describe the change you want to make\" --config {}\n  soulmate run start change --goal \"Describe the bounded change\" --ledger .soulmate/runs/run.jsonl --config {}\n  soulmate check --config {}",
        path.display(), path.display(), path.display(), path.display()
    );
    Ok(())
}

pub(crate) fn bind(arguments: &Arguments) -> Result<(), String> {
    args::assert_options("bind", arguments, &["config", "root", "state-root"])?;
    args::assert_positionals("bind", arguments, 0)?;
    let config_path = required(arguments, "config", "bind requires --config CONFIG")?;
    let product = required(arguments, "root", "bind requires --root PRODUCT")?;
    let state = required(arguments, "state-root", "bind requires --state-root STATE")?;
    let path = std::path::PathBuf::from(config_path);
    let source = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let config: serde_json::Value = serde_json::from_str(&source)
        .map_err(|error| format!("invalid JSON in {config_path}: {error}"))?;
    let errors = config::validate(&config);
    if !errors.is_empty() {
        return Err(format!("invalid configuration:\n- {}", errors.join("\n- ")));
    }
    let binding = crate::project_layout::bind_from_config(
        &std::fs::canonicalize(path).map_err(|error| error.to_string())?,
        &config,
        std::path::Path::new(product),
        std::path::Path::new(state),
    )?;
    println!("Local project binding is current: {}", binding.display());
    Ok(())
}

pub(crate) fn doctor(arguments: &Arguments) -> Result<(), String> {
    args::assert_options("doctor", arguments, &["config"])?;
    args::assert_positionals("doctor", arguments, 0)?;
    let checks = onboarding::doctor(arguments.options.get("config").map(String::as_str));
    let required_failed = checks.iter().any(|item| {
        !item["ok"].as_bool().unwrap_or(false)
            && matches!(item["name"].as_str(), Some("binary" | "config"))
    });
    for item in checks {
        println!(
            "{} {}: {}",
            if item["ok"].as_bool().unwrap_or(false) {
                "ok"
            } else {
                "--"
            },
            item["name"],
            item["detail"]
        );
    }
    if required_failed {
        Err("doctor required checks failed".into())
    } else {
        Ok(())
    }
}

pub(crate) fn check(loaded: &config::Loaded, arguments: &Arguments) -> Result<(), String> {
    args::assert_options("check", arguments, &["config", "json"])?;
    args::assert_positionals("check", arguments, 0)?;
    for agent in loaded.agents.values() {
        config::file(&loaded.control_root, &agent.profile)?;
    }
    let warnings = crate::boundary_manifest::warnings(&loaded.config);
    let mode = match loaded.mode {
        crate::project_layout::Mode::Local => "local",
        crate::project_layout::Mode::Portable => "portable",
    };
    if arguments.flags.contains_key("json") {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "valid": true,
                "mode": mode,
                "projectId": loaded.project_id,
                "warnings": warnings
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!("Soulmate configuration is valid ({mode} mode).");
        for warning in warnings {
            println!(
                "warning: agents.{}.{} entry '{}' is descriptive or unsupported for exact run narrowing",
                warning["agent"], warning["field"], warning["entry"]
            );
        }
    }
    Ok(())
}

fn required<'a>(arguments: &'a Arguments, name: &str, message: &str) -> Result<&'a str, String> {
    arguments
        .options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| message.to_owned())
}
