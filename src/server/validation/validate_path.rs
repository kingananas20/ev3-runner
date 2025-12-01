use super::PathStatus;
use std::env;
use std::path::{Component, Path, PathBuf};
use tracing::info;

/// Validates and sanitizes a client-provided path to prevent directory traversal attacks.
///
/// This function ensures that:
/// 1. The path is relative (not absolute)
/// 2. The path doesn't contain parent directory references (..)
/// 3. The resolved path stays within the working directory
///
/// Returns the sanitized relative path if valid, or an error describing why it's invalid.
pub fn validate_path(path: &Path) -> Result<PathBuf, PathStatus> {
    // Reject absolute paths immediately
    if path.is_absolute() {
        return Err(PathStatus::AbsolutePath);
    }

    // Check for dangerous path components
    if has_parent_components(path) {
        return Err(PathStatus::InvalidComponents);
    }

    let working_dir = env::current_dir().map_err(|_| PathStatus::CanonicalizationFailed)?;

    let safe_path = resolve_and_validate(path, &working_dir)?;

    info!(
        "Validated path: {} -> {}",
        path.display(),
        safe_path.display()
    );
    Ok(safe_path)
}

/// Check if path contains any parent directory (..) components
fn has_parent_components(path: &Path) -> bool {
    path.components().any(|c| matches!(c, Component::ParentDir))
}

/// Resolve the path and ensure it stays within the working directory
fn resolve_and_validate(path: &Path, working_dir: &Path) -> Result<PathBuf, PathStatus> {
    let target_path = working_dir.join(path);

    let canonical = if target_path.exists() {
        canonicalize_existing(&target_path)?
    } else {
        canonicalize_nonexistent(&target_path, working_dir)?
    };

    if !canonical.starts_with(working_dir) {
        return Err(PathStatus::EscapesWorkingDir);
    }

    canonical
        .strip_prefix(working_dir)
        .map(PathBuf::from)
        .map_err(|_| PathStatus::EscapesWorkingDir)
}

/// Canonicalize a path that exists on the filesystem
fn canonicalize_existing(path: &Path) -> Result<PathBuf, PathStatus> {
    path.canonicalize()
        .map_err(|_| PathStatus::CanonicalizationFailed)
}

/// Validate a path that doesn't exist yet by checking its parent directory
fn canonicalize_nonexistent(path: &Path, working_dir: &Path) -> Result<PathBuf, PathStatus> {
    let parent = path.parent().unwrap_or(working_dir);

    if !parent.exists() {
        return Ok(working_dir.join(path));
    }

    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| PathStatus::CanonicalizationFailed)?;

    let file_name = path.file_name().ok_or(PathStatus::InvalidComponents)?;

    Ok(canonical_parent.join(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejects_absolute_paths() {
        let result = validate_path(Path::new("/etc/passwd"));
        assert!(matches!(result, Err(PathStatus::AbsolutePath)));
    }

    #[test]
    fn test_rejects_parent_directory() {
        let result = validate_path(Path::new("../etc/passwd"));
        assert!(matches!(result, Err(PathStatus::InvalidComponents)));
    }

    #[test]
    fn test_accepts_simple_filename() {
        let result = validate_path(Path::new("program.bin"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("program.bin"));
    }

    #[test]
    fn test_accepts_subdirectory() {
        let result = validate_path(Path::new("bin/program"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_rejects_hidden_traversal() {
        // Even valid-looking paths that resolve outside working dir
        let result = validate_path(Path::new("subdir/../../etc/passwd"));
        assert!(result.is_err());
    }
}
