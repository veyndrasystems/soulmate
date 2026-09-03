mod support;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const SOULMATE_SKILL: &[u8] = include_bytes!("../skills/soulmate/SKILL.md");
const AWAY_GUIDE: &str = include_str!("../docs/codex-tmux-away.md");
const REFERENCE: &str = include_str!("../REFERENCE.md");

fn temp(label: &str) -> PathBuf {
    support::temp(&format!("skill-refresh-{label}"))
}

fn invoke(arguments: &[&str], bindings: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_soulmate"))
        .env("SOULMATE_BINDINGS_DIR", bindings)
        .args(arguments)
        .output()
        .unwrap()
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn local_project(label: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let base = temp(label);
    let product = base.join("product");
    let control = base.join("control");
    let state = base.join("state");
    let bindings = base.join("bindings");
    for path in [&product, &control, &state, &bindings] {
        fs::create_dir(path).unwrap();
    }
    let initialized = invoke(
        &[
            "init",
            "--mode",
            "local",
            "--project-id",
            "skill_refresh_fixture",
            "--root",
            product.to_str().unwrap(),
            "--control-root",
            control.to_str().unwrap(),
            "--state-root",
            state.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(
        initialized.status.success(),
        "{}",
        output_text(&initialized)
    );
    (base, product, control, bindings)
}

#[test]
fn attended_work_uses_native_spawn_without_away_fallback() {
    let (base, _product, control, _bindings) = local_project("native-spawn");
    let agents_projection =
        fs::read_to_string(control.join(".agents/skills/soulmate/SKILL.md")).unwrap();
    let claude_projection =
        fs::read_to_string(control.join(".claude/skills/soulmate/SKILL.md")).unwrap();
    let source = std::str::from_utf8(SOULMATE_SKILL).unwrap();
    let normalize = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");

    for skill in [source, &agents_projection, &claude_projection] {
        let skill = normalize(skill);
        assert!(skill.contains("every implementation worker and reviewer must use the host's native subagent spawn with the assignment's exact `nativeTaskName`"));
        assert!(skill.contains("native spawn is unavailable, stop and return the pending assignment to the operator; do not fall back to shell `codex exec` or `soulmate away`"));
        assert!(skill.contains("`openai/codex#31894`"));
        assert!(skill.contains(
            "`soulmate away` remains reserved for an explicit operator-away/disconnect handoff"
        ));
        assert!(skill.contains("soulmate away start AGENT LEDGER"));
    }

    for document in [AWAY_GUIDE, REFERENCE] {
        let document = normalize(document);
        assert!(document.contains("openai/codex#31894"));
        assert!(document.contains("a strong external symptom match"));
        assert!(document.contains("not a proven root cause"));
        assert!(document.contains("exclude affected `codex exec` no-result samples from provider-native completion and token-efficiency baselines"));
        assert!(document.contains(
            "Historical evidence remains in place; quarantine does not delete or rewrite it"
        ));
        assert!(document.contains("soulmate away start implementation_worker"));
    }

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn refresh_restores_missing_skills_and_reports_each_state() {
    let (base, product, control, bindings) = local_project("states");
    let product_before = fs::read_dir(&product).unwrap().count();
    let selected = [
        (
            control.join(".agents/skills/soulmate/SKILL.md"),
            SOULMATE_SKILL,
        ),
        (
            control.join(".claude/skills/soulmate/SKILL.md"),
            SOULMATE_SKILL,
        ),
    ];
    for (path, _) in &selected {
        fs::remove_file(path).unwrap();
    }

    let first = invoke(
        &[
            "init",
            "--refresh-skills",
            "--root",
            control.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(first.status.success(), "{}", output_text(&first));
    let first_text = output_text(&first);
    for (path, expected) in &selected {
        assert_eq!(fs::read(path).unwrap(), *expected);
        assert!(first_text.contains(&format!("created {}", relative_skill(path, &control))));
    }
    assert_eq!(fs::read_dir(&product).unwrap().count(), product_before);

    let second = invoke(
        &[
            "init",
            "--refresh-skills",
            "--root",
            control.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(second.status.success(), "{}", output_text(&second));
    let second_text = output_text(&second);
    for (path, _) in &selected {
        assert!(second_text.contains(&format!("unchanged {}", relative_skill(path, &control))));
    }

    fs::OpenOptions::new()
        .append(true)
        .open(&selected[0].0)
        .unwrap()
        .write_all(b"\n")
        .unwrap();
    let third = invoke(
        &[
            "init",
            "--refresh-skills",
            "--root",
            control.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(third.status.success(), "{}", output_text(&third));
    assert!(output_text(&third).contains(&format!(
        "refreshed {}",
        relative_skill(&selected[0].0, &control)
    )));
    assert_eq!(fs::read(&selected[0].0).unwrap(), SOULMATE_SKILL);

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn refresh_conflict_preflight_prevents_earlier_missing_creation() {
    let (base, _product, control, bindings) = local_project("conflict");
    let missing = control.join(".agents/skills/soulmate/SKILL.md");
    let conflict = control.join(".claude/skills/soulmate/SKILL.md");
    fs::remove_file(&missing).unwrap();
    fs::write(&conflict, "operator-owned skill\n").unwrap();

    let refused = invoke(
        &[
            "init",
            "--refresh-skills",
            "--root",
            control.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(!refused.status.success());
    assert!(output_text(&refused).contains("refusing to overwrite existing project skill"));
    assert!(!missing.exists());
    assert_eq!(
        fs::read_to_string(conflict).unwrap(),
        "operator-owned skill\n"
    );

    fs::remove_dir_all(base).unwrap();
}

fn relative_skill(path: &Path, control: &Path) -> String {
    path.strip_prefix(control)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}
