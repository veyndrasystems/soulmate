use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};

pub fn file(root: &Path, requested: &str) -> Result<PathBuf, String> {
    if requested.contains('\0') || Path::new(requested).is_absolute() {
        return Err(format!("path escapes project root: {requested}"));
    }
    let real_root = fs::canonicalize(root)
        .map_err(|_| format!("project root does not exist: {}", root.display()))?;
    let candidate = normalize_lexical(&root.join(requested));
    let real_candidate = fs::canonicalize(candidate).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            format!("declared file does not exist: {requested}")
        } else {
            error.to_string()
        }
    })?;
    if !real_candidate.starts_with(real_root) {
        return Err(format!("path escapes project root: {requested}"));
    }
    if !real_candidate.is_file() {
        return Err(format!("declared path is not a regular file: {requested}"));
    }
    Ok(real_candidate)
}

pub fn rel(root: &Path, path: &Path) -> Result<String, String> {
    let root = fs::canonicalize(root)
        .map_err(|error| format!("project root cannot be canonicalized: {error}"))?;
    let path = match fs::canonicalize(path) {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            match fs::symlink_metadata(path) {
                Ok(_) => return Err("path cannot be canonicalized".into()),
                Err(missing) if missing.kind() == std::io::ErrorKind::NotFound => {}
                Err(other) => return Err(other.to_string()),
            }
            let parent = path.parent().ok_or("path has no parent")?;
            let parent = fs::canonicalize(parent)
                .map_err(|error| format!("path parent cannot be canonicalized: {error}"))?;
            parent.join(path.file_name().ok_or("path has no file name")?)
        }
        Err(error) => return Err(error.to_string()),
    };
    let relative = path
        .strip_prefix(&root)
        .map_err(|_| "path escapes project root".to_string())?;
    let rendered = relative
        .to_str()
        .ok_or("project-relative path is not valid UTF-8")?
        .replace('\\', "/");
    if rendered.is_empty() || rendered == "." || rendered == ".." || rendered.starts_with("../") {
        return Err("path escapes project root".into());
    }
    Ok(rendered)
}

pub fn absolute(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(normalize_lexical(path))
    } else {
        Ok(normalize_lexical(
            &std::env::current_dir()
                .map_err(|error| format!("current directory cannot be resolved: {error}"))?
                .join(path),
        ))
    }
}

/// Read one project-relative regular file through no-follow directory handles.
#[cfg(unix)]
pub fn secure_bytes(root: &Path, requested: &str, label: &str) -> Result<Vec<u8>, String> {
    use std::ffi::CString;
    use std::fs::{File, OpenOptions};
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::{AsRawFd, FromRawFd};

    if requested.trim().is_empty() || requested.contains('\0') || Path::new(requested).is_absolute()
    {
        return Err(format!("path escapes project root: {label}"));
    }
    let components = Path::new(requested)
        .components()
        .filter_map(|component| match component {
            Component::CurDir => None,
            Component::Normal(value) => Some(Ok(value)),
            _ => Some(Err(format!("path escapes project root: {label}"))),
        })
        .collect::<Result<Vec<_>, String>>()?;
    let (file_name, parents) = components
        .split_last()
        .ok_or_else(|| format!("{label} path must name a project file"))?;
    let real_root = fs::canonicalize(root)
        .map_err(|_| format!("project root does not exist: {}", root.display()))?;
    let mut directory = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(real_root)
        .map_err(|error| format!("{label}: {error}"))?;
    for component in parents {
        let component = CString::new(component.as_bytes())
            .map_err(|_| format!("path escapes project root: {label}"))?;
        let descriptor = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                component.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if descriptor < 0 {
            return Err(format!("{label}: {}", std::io::Error::last_os_error()));
        }
        directory = unsafe { File::from_raw_fd(descriptor) };
    }
    let file_name = CString::new(file_name.as_bytes())
        .map_err(|_| format!("path escapes project root: {label}"))?;
    let descriptor = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            file_name.as_ptr(),
            libc::O_RDONLY | libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if descriptor < 0 {
        return Err(format!("{label}: {}", std::io::Error::last_os_error()));
    }
    let mut file = unsafe { File::from_raw_fd(descriptor) };
    if !file
        .metadata()
        .map_err(|error| format!("{label}: {error}"))?
        .is_file()
    {
        return Err(format!("{label} must be a regular file"));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("{label}: {error}"))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|error| format!("{label}: {error}"))?;
    let mut confirmation = Vec::new();
    file.read_to_end(&mut confirmation)
        .map_err(|error| format!("{label}: {error}"))?;
    if bytes != confirmation {
        return Err(format!("{label} changed while reading"));
    }
    Ok(bytes)
}

#[cfg(not(unix))]
pub fn secure_bytes(_: &Path, _: &str, label: &str) -> Result<Vec<u8>, String> {
    Err(format!(
        "{label} secure reading requires Unix no-follow support"
    ))
}

fn normalize_lexical(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_allows_only_a_missing_leaf_under_a_real_parent() {
        let root =
            std::env::temp_dir().join(format!("soulmate-project-path-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        assert_eq!(
            rel(&root, &root.join("future.jsonl")).unwrap(),
            "future.jsonl"
        );
        assert!(rel(&root, &root.join("missing/future.jsonl")).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn relative_path_rejects_non_utf8_evidence_names() {
        use std::os::unix::ffi::OsStringExt;

        let root =
            std::env::temp_dir().join(format!("soulmate-project-path-utf8-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let path = root.join(std::ffi::OsString::from_vec(vec![b'a', 0xff]));
        assert!(rel(&root, &path).unwrap_err().contains("not valid UTF-8"));
        fs::remove_dir_all(root).unwrap();
    }
}
