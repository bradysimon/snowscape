//! Test discovery and file management.

use std::path::{Path, PathBuf};

/// Information about a discovered test.
#[derive(Debug, Clone)]
pub struct TestInfo {
    /// The name of the test (derived from filename without extension).
    pub name: String,
    /// The full path to the `.ice` file.
    pub path: PathBuf,
    /// The preview this test belongs to (folder name).
    pub preview: String,
    /// Whether a snapshot exists for this test.
    pub has_snapshot: bool,
}

/// Whether the given `name` is already sanitized.
pub fn is_sanitized(name: &str) -> bool {
    name.chars().eq(sanitized_chars(name))
}

/// Sanitizes a name for use as a folder or filename.
/// Replaces non-alphanumeric characters with hyphens and lowercases.
pub fn sanitize_name(name: &str) -> String {
    sanitized_chars(name).collect()
}

/// Sanitizes the `name` and returns an iterator of characters for comparison.
/// Avoids allocating a new string when just checking if a name is sanitized.
fn sanitized_chars(name: &str) -> impl Iterator<Item = char> + '_ {
    name.trim().chars().flat_map(|c| {
        let sanitized = if c.is_alphanumeric() || c == '-' {
            c
        } else {
            '-'
        };

        sanitized.to_lowercase()
    })
}

/// Discovers all tests for a given preview in the tests directory.
///
/// Tests are expected to be in `{tests_dir}/{sanitized_preview_name}/*.ice`.
pub fn discover_tests(tests_dir: &Path, preview_name: &str) -> Vec<TestInfo> {
    let sanitized_preview = sanitize_name(preview_name);
    let preview_dir = tests_dir.join(&sanitized_preview);

    if !preview_dir.exists() || !preview_dir.is_dir() {
        return Vec::new();
    }

    let entries = match std::fs::read_dir(&preview_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut tests = Vec::new();

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        // Only look at .ice files
        if path.extension().and_then(|e| e.to_str()) != Some("ice") {
            continue;
        }

        let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        // Determine whether an associated snapshot exists.
        let has_snapshot = has_snapshot(&preview_dir, file_stem);

        tests.push(TestInfo {
            name: file_stem.to_string(),
            path,
            preview: sanitized_preview.clone(),
            has_snapshot,
        });
    }

    // Sort by name for consistent ordering
    tests.sort_by(|a, b| a.name.cmp(&b.name));
    tests
}

/// Returns whether a snapshot file is associated with a test.
fn has_snapshot(dir: &Path, test_name: &str) -> bool {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries.filter_map(Result::ok).any(|e| {
                let path = e.path();
                path.extension().and_then(|e| e.to_str()) == Some("png")
                    && path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|stem| is_associated_snapshot_stem(test_name, stem))
            })
        })
        .unwrap_or(false)
}

/// Deletes a test and all associated snapshot files.
///
/// Returns `Ok(())` if the test was deleted successfully.
pub fn delete_test(path: &Path) -> std::io::Result<()> {
    let Some(parent) = path.parent() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Test file has no parent directory",
        ));
    };

    let Some(test_name) = path.file_stem().and_then(|s| s.to_str()) else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid test filename",
        ));
    };

    // Delete the .ice file
    std::fs::remove_file(path)?;

    // Delete associated snapshots
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.filter_map(Result::ok) {
            let file_path = entry.path();
            let is_png = file_path.extension().and_then(|e| e.to_str()) == Some("png");
            let is_associated = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|stem| is_associated_snapshot_stem(test_name, stem));

            if is_png && is_associated {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    // Remove the preview folder if it's now empty
    if let Ok(remaining) = std::fs::read_dir(parent)
        && remaining.count() == 0
    {
        let _ = std::fs::remove_dir(parent);
    }

    Ok(())
}

fn is_associated_snapshot_stem(test_name: &str, stem: &str) -> bool {
    if stem == test_name {
        return true;
    }

    stem.strip_prefix(test_name)
        .and_then(|rest| rest.chars().next())
        .is_some_and(|separator| matches!(separator, '.' | '-' | '_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Leading and trailing whitespace should be trimmed.
    #[test]
    fn sanitize_name_trims_content() {
        assert_eq!(sanitize_name("  counter  "), "counter");
    }

    /// All test names should be converted to lowercase for consistency.
    #[test]
    fn sanitize_name_lowercases_text() {
        assert_eq!(sanitize_name("Counter"), "counter");
    }

    /// Non-alphanumeric characters should be replaced with hyphens.
    #[test]
    fn sanitize_name_replaces_non_alphanumeric_characters() {
        assert_eq!(sanitize_name("A/B Test"), "a-b-test");
    }

    #[test]
    fn is_sanitized_true_for_valid_kebab_name() {
        assert!(is_sanitized("basic-increment"));
    }

    #[test]
    fn is_sanitized_false_for_name_needing_changes() {
        assert!(!is_sanitized("Basic Increment"));
        assert!(!is_sanitized("basic_increment"));
        assert!(!is_sanitized("basic increment"));
        assert!(!is_sanitized(" basic-increment "));
    }

    #[test]
    fn associated_snapshot_stem_matches_current_and_legacy_names() {
        assert!(is_associated_snapshot_stem("test", "test"));
        assert!(is_associated_snapshot_stem("test", "test.failed"));
        assert!(is_associated_snapshot_stem("test", "test-wgpu"));
        assert!(is_associated_snapshot_stem("test", "test_failed"));
        assert!(is_associated_snapshot_stem("test", "test.failed-wgpu"));

        assert!(!is_associated_snapshot_stem("test", "testcase"));
        assert!(!is_associated_snapshot_stem("test", "other-test"));
    }
}
