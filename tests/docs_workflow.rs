//! Gate the local documentation journey, including links to renamed headings.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

fn anchors(markdown: &str) -> BTreeSet<String> {
    let mut seen = BTreeMap::<String, usize>::new();
    let mut result = BTreeSet::new();
    let mut fenced = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced || !line.starts_with('#') {
            continue;
        }
        let text = line.trim_start_matches('#').trim();
        let slug: String = text
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();
        let count = seen.entry(slug.clone()).or_default();
        result.insert(if *count == 0 {
            slug
        } else {
            format!("{slug}-{count}")
        });
        *count += 1;
    }
    result
}

fn check_links(root: &Path, source: &str) -> Result<(), String> {
    let path = root.join(source);
    let markdown = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    for after_label in markdown.split("](").skip(1) {
        let Some((target, _)) = after_label.split_once(')') else {
            continue;
        };
        if target.contains("://") || target.starts_with("mailto:") {
            continue;
        }
        let (file, anchor) = target
            .split_once('#')
            .map_or((target, None), |(f, a)| (f, Some(a)));
        let destination = if file.is_empty() {
            path.clone()
        } else {
            path.parent().unwrap().join(file)
        };
        if !destination.exists() {
            return Err(format!("{source}: missing local destination {target}"));
        }
        if let Some(anchor) = anchor {
            let content = fs::read_to_string(&destination).map_err(|error| error.to_string())?;
            if !anchors(&content).contains(anchor) {
                return Err(format!("{source}: missing heading in {target}"));
            }
        }
    }
    Ok(())
}

#[test]
fn checked_work_journey_links_resolve_in_the_checkout() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for source in [
        "README.md",
        "REFERENCE.md",
        "docs/onboarding.md",
        "docs/first-checked-run.md",
        "docs/value-proof-methodology.md",
        "skills/soulmate/SKILL.md",
    ] {
        check_links(root, source).unwrap();
    }
}

#[test]
fn heading_gate_distinguishes_fenced_examples_and_removed_destinations() {
    let headings = anchors("# Real\n```md\n# Gone\n```\n## `Quoted` heading\n## Real\n");
    assert!(headings.contains("real"));
    assert!(headings.contains("real-1"));
    assert!(headings.contains("quoted-heading"));
    assert!(!headings.contains("gone"));
}
