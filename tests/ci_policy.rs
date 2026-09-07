use std::fs;

const WSL_SCHEDULE: &str = "${{ github.event_name == 'workflow_dispatch' || (github.event_name == 'push' && github.ref == 'refs/heads/main') }}";
const PROOF_BASE_ENV: &str =
    "VALUE_PROOF_BASE: ${{ github.event.pull_request.base.sha || github.event.before }}";
const ZERO_PROOF_BASE: &str =
    "if [ \"$VALUE_PROOF_BASE\" = \"0000000000000000000000000000000000000000\" ]; then";
const EXACT_FETCH: &str = "git fetch --no-tags origin \"$VALUE_PROOF_BASE\"";
const VERIFY_PROOF_BASE: &str = "git cat-file -e \"${VALUE_PROOF_BASE}^{commit}\"";

fn source(path: &str) -> String {
    fs::read_to_string(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path)).unwrap()
}

#[test]
fn primary_ci_and_release_keep_the_accepted_wsl_cadence() {
    let ci = source(".github/workflows/ci.yml");
    assert!(ci.contains("on:\n  push:\n  pull_request:\n  workflow_dispatch:"));

    let rust = ci
        .split_once("  rust:\n")
        .unwrap()
        .1
        .split_once("  macos:\n")
        .unwrap()
        .0;
    let macos = ci
        .split_once("  macos:\n")
        .unwrap()
        .1
        .split_once("  windows-wsl:\n")
        .unwrap()
        .0;
    for block in [rust, macos] {
        assert_eq!(block.matches(PROOF_BASE_ENV).count(), 1);
        assert!(block.contains(ZERO_PROOF_BASE));
        assert!(block.contains("git rev-parse --verify HEAD^"));
        assert_eq!(block.matches(EXACT_FETCH).count(), 1);
        assert_eq!(block.matches(VERIFY_PROOF_BASE).count(), 2);
        assert!(block.find(EXACT_FETCH).unwrap() < block.find("cargo test --locked").unwrap());
        assert!(!block.contains("git fetch origin main"));
        assert!(!block.contains("VALUE_PROOF_BASE=$(git rev-parse --verify HEAD^) ||"));
    }

    let wsl = ci
        .split_once("  windows-wsl:\n")
        .unwrap()
        .1
        .split_once("  audit:\n")
        .unwrap()
        .0;
    assert!(wsl.contains(&format!("    if: {WSL_SCHEDULE}\n")));
    assert!(wsl.contains("    needs: rust\n"));

    let upload = ci
        .split_once("      - name: Upload Linux release candidate for WSL\n")
        .unwrap()
        .1
        .split_once("  macos:\n")
        .unwrap()
        .0;
    assert!(upload.contains(&format!("        if: {WSL_SCHEDULE}\n")));

    let release = source(".github/workflows/release.yml");
    assert!(release.contains("  windows-wsl:\n"));
    assert!(release.contains("  publish:\n    needs: [linux, macos, windows-wsl]\n"));
    assert!(!release.contains(WSL_SCHEDULE));
}
