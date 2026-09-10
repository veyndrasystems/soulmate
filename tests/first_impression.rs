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
fn conversation_first_readme_keeps_setup_and_proof_safe_inside_git() {
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
    assert!(readme
        .starts_with("# Soulmate\n\nIt verifies what you asked an agent to do and what came back, in the same record."));
    let conversation_start = readme.find("## Start with your existing agent\n").unwrap();
    let conversation_end = readme[conversation_start + 1..]
        .find("\n## ")
        .map(|offset| conversation_start + 1 + offset)
        .unwrap();
    let conversation = &readme[conversation_start..conversation_end];
    assert!(conversation.contains("existing Codex or Claude lead"));
    assert!(conversation.contains("ordinary\nlanguage"));
    assert!(conversation.contains("Ordinary reversible work stays direct"));
    assert!(conversation.contains("silence is never approval"));
    assert!(conversation.contains("After approval, manage the"));
    assert!(conversation.contains("Ask before either installation or a host-permission change."));
    let post_setup_marker = "After setup, a normal-language request can remain simple:\n\n";
    let post_setup_start = conversation.find(post_setup_marker).unwrap() + post_setup_marker.len();
    let post_setup_end = conversation[post_setup_start..]
        .find("\n\n")
        .map(|offset| post_setup_start + offset)
        .unwrap();
    let post_setup_prompt = &conversation[post_setup_start..post_setup_end];
    assert!(post_setup_prompt.starts_with("> Please update the theme, run the existing checks"));
    assert!(post_setup_prompt.contains("what still needs doing"));
    assert!(!post_setup_prompt.contains("Soulmate"));
    assert!(readme.contains("current preview, `v0.14.0-rc.4`"));
    assert!(readme.contains("stable release documentation"));
    assert!(readme.contains("349b662574b29a2b0366f53aac12d97f268bc84c"));
    assert!(readme.contains("Stable release"));
    let boundary = readme.find("Before installing or using it:").unwrap();
    let install = readme
        .find("curl -fsSL https://raw.githubusercontent.com/")
        .unwrap();
    assert!(conversation_start < boundary && boundary < install);
    assert!(readme
        .contains("The default benchmark removes its temporary project and records after showing"));
    assert!(readme.contains("A real running task may leave a pending review or"));
    assert!(readme.contains("assignment for your existing host"));
    assert!(!readme.contains("that pending review"));
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
    assert!(output.contains("Bounded setup facts for your existing root agent"));
    assert!(!output.contains("Replace TASK"));
    assert!(output.contains("Setup does not start agents or grant host permissions."));
    assert_eq!(
        fs::read(project.join("work.txt")).unwrap(),
        b"unfinished user work\n"
    );
    fs::remove_dir_all(base).unwrap();
}
