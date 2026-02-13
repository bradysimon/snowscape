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
    /// Number of associated snapshot files.
    pub snapshot_count: usize,
}

/// Sanitizes a name for use as a folder or filename.
/// Replaces non-alphanumeric characters with underscores and lowercases.
pub fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase()
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

        // Count associated snapshots (files matching {test_name}_*.png)
        let snapshot_count = count_snapshots(&preview_dir, file_stem);

        tests.push(TestInfo {
            name: file_stem.to_string(),
            path,
            preview: sanitized_preview.clone(),
            snapshot_count,
        });
    }

    // Sort by name for consistent ordering
    tests.sort_by(|a, b| a.name.cmp(&b.name));
    tests
}

/// Counts the number of snapshot files associated with a test.
fn count_snapshots(dir: &Path, test_name: &str) -> usize {
    let prefix = format!("{}_", test_name);

    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|e| {
                    let name = e.file_name();
                    let name_str = name.to_string_lossy();
                    name_str.starts_with(&prefix) && name_str.ends_with(".png")
                })
                .count()
        })
        .unwrap_or(0)
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
    let prefix = format!("{}_", test_name);
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(&prefix) && name_str.ends_with(".png") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    // Remove the preview folder if it's now empty
    if let Ok(remaining) = std::fs::read_dir(parent) {
        if remaining.count() == 0 {
            let _ = std::fs::remove_dir(parent);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Counter"), "counter");
        assert_eq!(sanitize_name("my button"), "my_button");
        assert_eq!(sanitize_name("Test-Name"), "test-name");
        assert_eq!(sanitize_name("A/B Test"), "a_b_test");
    }
}
