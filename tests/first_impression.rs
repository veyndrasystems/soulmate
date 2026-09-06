//! Exercise the literal README entry in an existing Git project, not a copied recipe.
#![cfg(unix)]

mod support;
use std::{fs, os::unix::fs::symlink, path::Path, process::Command};

fn readme_command(prefix: &str) -> String {
    let readme = include_str!("../README.md");
    let blocks: Vec<_> = readme
        .split("```sh\n")
        .skip(1)
        .filter_map(|block| block.split_once("```"))
        .map(|(command, _)| command)
        .filter(|command| command.starts_with(prefix))
        .collect();
    assert_eq!(blocks.len(), 1, "one primary {prefix} entry is required");
    blocks[0].to_owned()
}

fn run_readme(command: &str, root: &Path, bin: &Path, temporary: &Path) -> String {
    let path = std::env::join_paths(
        std::iter::once(bin.to_owned())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let output = Command::new("sh")
        .args(["-eu", "-c", command])
        .current_dir(root)
        .env("PATH", path)
        .env("TMPDIR", temporary)
        .env("SOULMATE_BINDINGS_DIR", temporary.join("bindings"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "README command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn literal_readme_proof_precedes_setup_and_works_inside_git_without_touching_it() {
    let base = support::temp("first-impression");
    let project = base.join("project with spaces");
    let bin = base.join("bin");
    let temporary = base.join("temporary");
    for path in [&project, &bin, &temporary] {
        fs::create_dir(path).unwrap();
    }
    symlink(env!("CARGO_BIN_EXE_soulmate"), bin.join("soulmate")).unwrap();
    assert!(Command::new("git")
        .args(["init", "-q"])
        .arg(&project)
        .status()
        .unwrap()
        .success());
    fs::write(project.join("work.txt"), b"unfinished user work\n").unwrap();
    let git_status = || {
        let out = Command::new("git")
            .args(["status", "--porcelain", "--untracked-files=all"])
            .current_dir(&project)
            .output()
            .unwrap();
        assert!(out.status.success());
        out.stdout
    };
    let before = git_status();
    let proof = readme_command("soulmate benchmark");
    let init = readme_command("soulmate init ");
    let readme = include_str!("../README.md");
    assert!(readme.find(&proof).unwrap() < readme.find(&init).unwrap());
    let output = run_readme(&proof, &project, &bin, &temporary);
    assert!(output.contains("False-completion proof passed (14/14 assertions)."));
    for line in readme
        .split("```text\n")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap()
        .lines()
    {
        assert!(
            output.lines().any(|actual| actual == line),
            "README outcome absent: {line}"
        );
    }
    assert_eq!(git_status(), before);
    assert_eq!(
        fs::read(project.join("work.txt")).unwrap(),
        b"unfinished user work\n"
    );
    assert!(!project.join("soulmate.json").exists());
    assert_eq!(
        fs::read_dir(&temporary).unwrap().count(),
        0,
        "proof must clean up"
    );

    let output = run_readme(&init, &project, &bin, &temporary);
    let skill = project.join(".agents/skills/soulmate/SKILL.md");
    let config = project.join("soulmate.json");
    assert!(config.is_file());
    assert!(skill.is_file());
    assert!(output.contains(skill.to_str().unwrap()));
    assert!(output.contains(config.to_str().unwrap()));
    assert!(output.contains("TASK") && output.contains("TEST_COMMAND"));
    assert!(output.contains("Setup does not start agents or grant host permissions."));
    assert_eq!(
        fs::read(project.join("work.txt")).unwrap(),
        b"unfinished user work\n"
    );
    fs::remove_dir_all(base).unwrap();
}
