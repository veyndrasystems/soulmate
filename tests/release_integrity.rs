mod support;

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASE_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "REFERENCE.md",
    "CHANGELOG.md",
    "install.sh",
    "plugin.json",
    "systems.veyndra.soulmate/.codex-plugin/plugin.json",
    "systems.veyndra.soulmate/.claude-plugin/plugin.json",
    "scripts/ci-wsl.sh",
];
const SCAN_DIRS: &[&str] = &["docs", "examples", "schema", "scripts", "src"];

struct Fixture(PathBuf);

impl Fixture {
    fn new(label: &str) -> Self {
        Self(support::temp(&format!("release-integrity-{label}")))
    }

    fn release() -> Self {
        let fixture = Self::new("refs");
        for name in RELEASE_FILES {
            let path = fixture.0.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::copy(Path::new(env!("CARGO_MANIFEST_DIR")).join(name), path).unwrap();
        }
        for name in SCAN_DIRS {
            fs::create_dir_all(fixture.0.join(name)).unwrap();
        }
        fixture
    }

    fn replace(&self, file: &str, old: &str, new: &str) {
        let path = self.0.join(file);
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains(old),
            "fixture did not contain {old} in {file}"
        );
        fs::write(path, text.replacen(old, new, 1)).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn gate(name: &str, root: &Path) -> Output {
    gate_args(name, root, &[])
}

fn gate_args(name: &str, root: &Path, arguments: &[&str]) -> Output {
    gate_command(name, root, arguments).output().unwrap()
}

fn gate_command(name: &str, root: &Path, arguments: &[&str]) -> Command {
    let mut command = Command::new("sh");
    command
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("scripts")
                .join(name),
        )
        .args(arguments)
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        // Supported installer overrides must not replace its checked source default.
        .env("SOULMATE_VERSION", "v99.98.97")
        .env("SOULMATE_REPOSITORY", "example/override");
    command
}

fn expect_success(output: Output) {
    assert!(output.status.success(), "{output:?}");
}

fn expect_failure(output: Output, context: &str) {
    assert!(!output.status.success(), "unexpected pass: {context}");
    assert!(
        !output.stderr.is_empty(),
        "missing failure detail: {context}"
    );
}

#[test]
fn current_release_and_historical_changelog_pass_with_installer_overrides() {
    let fixture = Fixture::release();
    let path = fixture.0.join("CHANGELOG.md");
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("\n## 0.1.0\n\nHistorical references such as v0.1.0 remain historical.\n");
    fs::write(path, text).unwrap();
    expect_success(gate("check-release-refs.sh", &fixture.0));
}

#[test]
fn stale_and_malformed_readme_install_commands_are_rejected() {
    let current = format!("curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v{VERSION}/install.sh | sh");
    for replacement in [
        current.replace(&format!("v{VERSION}"), "v0.10.0"),
        current.replace("install.sh", "install.sh.invalid"),
        format!("# {current}"),
        String::new(),
    ] {
        let fixture = Fixture::release();
        fixture.replace("README.md", &current, &replacement);
        expect_failure(gate("check-release-refs.sh", &fixture.0), &replacement);
    }
}

#[test]
fn html_comments_cannot_hide_the_readme_install_instruction() {
    let command = format!("curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v{VERSION}/install.sh | sh");
    let block = format!("```sh\n{command}\nexport PATH=\"$HOME/.local/bin:$PATH\"\n```");
    for hidden in [
        format!("<!--\n{block}\n-->"),
        format!("<!-- {block} -->"),
        format!("Installation: <!-- hidden\n{block}\n-->"),
        format!("<!-- closed --> <!-- hidden\n{block}\n-->"),
        format!("<!--\n{block}"),
        block.replace(&command, &format!("<!-- {command} -->")),
    ] {
        let fixture = Fixture::release();
        fixture.replace("README.md", &block, &hidden);
        expect_failure(
            gate("check-release-refs.sh", &fixture.0),
            "HTML-commented install command",
        );
    }
    for visible in [
        format!("<!-- release installation -->\n{block}"),
        format!("<!-- first --> <!-- second -->\n{block}"),
        format!("<!-- historical note\nclosed here -->\n{block}"),
    ] {
        let fixture = Fixture::release();
        fixture.replace("README.md", &block, &visible);
        expect_success(gate("check-release-refs.sh", &fixture.0));
    }
}

#[test]
fn complete_reference_tokens_reject_prefix_collisions_and_other_majors() {
    for wrong in [
        "v0.10.0".to_owned(),
        format!("v{VERSION}0"),
        "v1.0.0".to_owned(),
        format!("v{VERSION}-rc.1"),
        format!("v{VERSION}+rebuild"),
    ] {
        let fixture = Fixture::release();
        // Leave the valid README command intact to exercise the recursive token scan.
        fs::write(
            fixture.0.join("docs/reference.md"),
            format!("release {wrong}\n"),
        )
        .unwrap();
        expect_failure(gate("check-release-refs.sh", &fixture.0), &wrong);
    }
}

#[test]
fn missing_release_sources_and_scan_roots_are_rejected() {
    for name in RELEASE_FILES.iter().chain(SCAN_DIRS) {
        let fixture = Fixture::release();
        let path = fixture.0.join(name);
        if path.is_dir() {
            fs::remove_dir_all(path).unwrap();
        } else {
            fs::remove_file(path).unwrap();
        }
        expect_failure(gate("check-release-refs.sh", &fixture.0), name);
    }
}

#[test]
fn absent_or_drifted_required_version_values_are_rejected() {
    let package = format!("version = \"{VERSION}\"");
    let lock = format!("name = \"soulmate\"\nversion = \"{VERSION}\"");
    let manifest = format!("\"version\": \"{VERSION}\",");
    let installer = format!("version=\"${{SOULMATE_VERSION:-v{VERSION}}}\"");
    let wsl = format!("test \"$(soulmate version)\" = \"{VERSION}\"");
    let changelog = format!("## {VERSION}\n");
    for (file, marker) in [
        ("Cargo.toml", package.as_str()),
        ("Cargo.lock", lock.as_str()),
        ("plugin.json", manifest.as_str()),
        (
            "systems.veyndra.soulmate/.codex-plugin/plugin.json",
            manifest.as_str(),
        ),
        (
            "systems.veyndra.soulmate/.claude-plugin/plugin.json",
            manifest.as_str(),
        ),
        ("install.sh", installer.as_str()),
        ("scripts/ci-wsl.sh", wsl.as_str()),
        ("CHANGELOG.md", changelog.as_str()),
    ] {
        for replacement in [String::new(), marker.replace(VERSION, "1.2.3")] {
            let fixture = Fixture::release();
            fixture.replace(file, marker, &replacement);
            expect_failure(gate("check-release-refs.sh", &fixture.0), file);
        }
    }
}

#[cfg(unix)]
#[test]
fn recursive_scan_errors_are_not_treated_as_clean_results() {
    let fixture = Fixture::release();
    std::os::unix::fs::symlink("missing-reference", fixture.0.join("docs/broken-link")).unwrap();
    expect_failure(
        gate("check-release-refs.sh", &fixture.0),
        "broken scan link",
    );
}

#[cfg(unix)]
#[test]
fn source_validation_rejects_broken_links_when_grep_suppresses_errors() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = Fixture::release();
    let bin = fixture.0.join("test-bin");
    fs::create_dir(&bin).unwrap();
    let lookup = Command::new("sh")
        .args(["-c", "command -v grep"])
        .output()
        .unwrap();
    assert!(lookup.status.success());
    let real_grep = String::from_utf8(lookup.stdout).unwrap();
    let shim = bin.join("grep");
    fs::write(
        &shim,
        b"#!/bin/sh\ncase \"$1\" in\n  -R*)\n    printf recursive > \"$SOULMATE_TEST_GREP_CALLED\"\n    \"$SOULMATE_TEST_REAL_GREP\" \"$@\" 2>/dev/null\n    status=$?\n    if test \"$status\" -gt 1; then exit 0; fi\n    exit \"$status\"\n    ;;\nesac\nexec \"$SOULMATE_TEST_REAL_GREP\" \"$@\"\n",
    )
    .unwrap();
    fs::set_permissions(&shim, fs::Permissions::from_mode(0o755)).unwrap();
    let old_path = std::env::var_os("PATH").unwrap();
    let path =
        std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(&old_path))).unwrap();
    let called = fixture.0.join("grep-called");
    let run = || {
        gate_command("check-release-refs.sh", &fixture.0, &[])
            .env("PATH", &path)
            .env("SOULMATE_TEST_REAL_GREP", real_grep.trim())
            .env("SOULMATE_TEST_GREP_CALLED", &called)
            .output()
            .unwrap()
    };
    expect_success(run());
    assert!(called.is_file(), "control must reach recursive grep");
    fs::remove_file(&called).unwrap();
    std::os::unix::fs::symlink("missing-reference", fixture.0.join("docs/broken-link")).unwrap();
    expect_failure(run(), "grep may silently skip broken links");
    assert!(
        !called.exists(),
        "validation must fail before recursive grep"
    );
}

#[cfg(unix)]
#[test]
fn readable_source_links_are_scanned_and_git_directories_stay_excluded() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::release();
    let targets = fixture.0.join("source-targets");
    fs::create_dir(&targets).unwrap();
    let reference = targets.join("reference.md");
    fs::write(&reference, format!("release v{VERSION}\n")).unwrap();
    symlink(
        "../source-targets/reference.md",
        fixture.0.join("docs/readable-file"),
    )
    .unwrap();
    symlink(
        "../source-targets",
        fixture.0.join("examples/readable-directory"),
    )
    .unwrap();
    let ignored = fixture.0.join("docs/.git");
    fs::create_dir(&ignored).unwrap();
    symlink("missing-reference", ignored.join("broken-link")).unwrap();
    fs::write(ignored.join("history"), "release v1.0.0\n").unwrap();
    expect_success(gate("check-release-refs.sh", &fixture.0));

    fs::write(reference, "release v1.0.0\n").unwrap();
    expect_failure(
        gate("check-release-refs.sh", &fixture.0),
        "readable linked source still participates in the scan",
    );
}

fn git(root: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .args([
            "-c",
            "user.name=Soulmate Test",
            "-c",
            "user.email=soulmate-test@users.noreply.github.com",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .output()
        .unwrap()
}

#[test]
fn published_checkpoint_and_descendants_pass_but_rewritten_parentage_fails() {
    let fixture = Fixture::new("ancestry");
    let checkout = fixture.0.join("checkout");
    expect_success(git(
        &fixture.0,
        &[
            "clone",
            "--no-hardlinks",
            "--quiet",
            env!("CARGO_MANIFEST_DIR"),
            "checkout",
        ],
    ));
    expect_success(gate("check-public-history.sh", &checkout));
    expect_success(git(
        &checkout,
        &[
            "commit",
            "--allow-empty",
            "--quiet",
            "-m",
            "Public fixture descendant",
        ],
    ));
    expect_success(gate("check-public-history.sh", &checkout));

    expect_success(git(&checkout, &["checkout", "--orphan", "rewritten"]));
    expect_success(git(
        &checkout,
        &["commit", "--quiet", "-m", "Public fixture orphan"],
    ));
    expect_failure(gate("check-public-history.sh", &checkout), "orphan commit");

    // A replacement may invent a parent view, but is not stored commit ancestry.
    expect_success(git(
        &checkout,
        &[
            "replace",
            "--graft",
            "HEAD",
            "7f1d0146694d6b049f60e6bfba968e2a8c3a104d",
        ],
    ));
    expect_failure(
        gate("check-public-history.sh", &checkout),
        "replacement parent view",
    );
}

#[test]
fn absent_checkpoint_and_shallow_git_evidence_are_rejected() {
    let fixture = Fixture::new("missing-history");
    expect_failure(
        gate("check-public-history.sh", &fixture.0),
        "no Git worktree",
    );
    expect_success(git(&fixture.0, &["init", "--quiet"]));
    expect_success(git(
        &fixture.0,
        &[
            "commit",
            "--allow-empty",
            "--quiet",
            "-m",
            "Unrelated public fixture",
        ],
    ));
    expect_failure(
        gate("check-public-history.sh", &fixture.0),
        "missing checkpoint",
    );

    let shallow = Fixture::new("shallow");
    expect_success(git(
        &shallow.0,
        &[
            "clone",
            "--no-local",
            "--depth",
            "1",
            "--quiet",
            env!("CARGO_MANIFEST_DIR"),
            "checkout",
        ],
    ));
    expect_failure(
        gate("check-public-history.sh", &shallow.0.join("checkout")),
        "shallow checkout",
    );
}

#[test]
fn synthetic_merge_cannot_mask_a_rewritten_or_missing_candidate() {
    let fixture = Fixture::new("merge-candidate");
    let checkout = fixture.0.join("checkout");
    expect_success(git(
        &fixture.0,
        &[
            "clone",
            "--no-hardlinks",
            "--quiet",
            env!("CARGO_MANIFEST_DIR"),
            "checkout",
        ],
    ));
    expect_success(git(&checkout, &["branch", "preserved"]));
    expect_success(git(&checkout, &["checkout", "--orphan", "rewritten"]));
    expect_success(git(
        &checkout,
        &["commit", "--quiet", "-m", "Public fixture rewritten head"],
    ));
    let output = git(&checkout, &["rev-parse", "HEAD"]);
    assert!(output.status.success());
    let rewritten = String::from_utf8(output.stdout).unwrap();
    expect_success(git(&checkout, &["checkout", "preserved"]));
    expect_success(git(
        &checkout,
        &[
            "merge",
            "--allow-unrelated-histories",
            "--no-edit",
            "rewritten",
        ],
    ));
    expect_success(gate("check-public-history.sh", &checkout));
    for candidate in [rewritten.trim(), "", "unavailable-candidate"] {
        expect_failure(
            gate_args("check-public-history.sh", &checkout, &[candidate]),
            "merge must not hide missing candidate ancestry",
        );
    }
    // Supplying a valid revision cannot bypass the mandatory native HEAD check.
    expect_success(git(&checkout, &["checkout", "rewritten"]));
    expect_failure(
        gate_args("check-public-history.sh", &checkout, &["preserved"]),
        "native HEAD remains mandatory",
    );
}
