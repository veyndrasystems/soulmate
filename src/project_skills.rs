use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const SOULMATE: &str = include_str!("../skills/soulmate/SKILL.md");
const COFFEE: &str = include_str!("../skills/coffee/SKILL.md");
const SKILL_MARKER: &str = "<!-- soulmate-managed-skill:v1 -->";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkillRefreshState {
    Created,
    Refreshed,
    Unchanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SkillRefreshStatus {
    pub(crate) path: String,
    pub(crate) state: SkillRefreshState,
}

struct SkillDestination {
    path: PathBuf,
    content: String,
    label: String,
    state: SkillRefreshState,
}

pub(crate) fn activate(control: &Path, coffee: bool) -> Result<(), String> {
    let destinations = selected_destinations(control, coffee)?;
    for destination in destinations {
        if destination.path.exists() {
            continue;
        }
        crate::managed_files::write_exclusive(&destination.path, destination.content.as_bytes())?;
    }
    Ok(())
}

pub(crate) fn refresh(control: &Path, coffee: bool) -> Result<Vec<SkillRefreshStatus>, String> {
    let destinations = selected_destinations(control, coffee)?;
    for destination in &destinations {
        match destination.state {
            SkillRefreshState::Created => crate::managed_files::write_exclusive(
                &destination.path,
                destination.content.as_bytes(),
            )?,
            SkillRefreshState::Refreshed => refresh_owned(&destination.path, &destination.content)?,
            SkillRefreshState::Unchanged => {}
        }
    }
    Ok(destinations
        .into_iter()
        .map(|destination| SkillRefreshStatus {
            path: destination.label,
            state: destination.state,
        })
        .collect())
}

fn selected_destinations(control: &Path, coffee: bool) -> Result<Vec<SkillDestination>, String> {
    let mut assets = vec![("soulmate", "SKILL.md", SOULMATE)];
    if coffee {
        assets.push(("coffee", "SKILL.md", COFFEE));
    }
    let mut destinations = Vec::new();
    for base in [".agents/skills", ".claude/skills"] {
        for (name, relative, content) in &assets {
            let path = control.join(base).join(name).join(relative);
            let directory = path.parent().ok_or("project skill asset has no parent")?;
            crate::managed_files::ensure_managed_directory(control, directory)?;
            let state = inspect_skill(&path, content)?;
            destinations.push(SkillDestination {
                path,
                content: content.to_string(),
                label: format!("{base}/{name}/{relative}"),
                state,
            });
        }
    }
    Ok(destinations)
}

fn inspect_skill(path: &Path, content: &str) -> Result<SkillRefreshState, String> {
    match fs::symlink_metadata(path) {
        Ok(info) => {
            if info.file_type().is_symlink() {
                return Err(format!(
                    "project skill must not be a symlink: {}",
                    path.display()
                ));
            }
            if !info.is_file() {
                return Err(format!(
                    "project skill path must be a regular file: {}",
                    path.display()
                ));
            }
            let existing = fs::read_to_string(path).map_err(|e| e.to_string())?;
            if existing == content {
                return Ok(SkillRefreshState::Unchanged);
            }
            if !existing.lines().any(|line| line == SKILL_MARKER) {
                return Err(format!(
                    "refusing to overwrite existing project skill: {}",
                    path.display()
                ));
            }
            Ok(SkillRefreshState::Refreshed)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(SkillRefreshState::Created)
        }
        Err(error) => Err(error.to_string()),
    }
}

fn refresh_owned(path: &Path, content: &str) -> Result<(), String> {
    let info = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    let old = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("managed skill path must name a UTF-8 file")?;
    let temp = path.with_file_name(format!(
        ".{}.soulmate-refresh-{}.tmp",
        name,
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|e| e.to_string())?;
    file.sync_all().ok();
    drop(file);
    let result = (|| {
        let current = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if current.file_type().is_symlink()
            || !current.is_file()
            || current.len() != info.len()
            || fs::read_to_string(path).map_err(|e| e.to_string())? != old
        {
            return Err(format!(
                "project skill changed during refresh: {}",
                path.display()
            ));
        }
        fs::rename(&temp, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
