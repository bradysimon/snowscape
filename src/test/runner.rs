//! Async test execution helpers used by test state updates.

use std::path::PathBuf;

use crate::test;

/// Merges latest run outcomes into an existing result set.
pub(super) fn merge_run_results(
    previous: Option<Vec<test::Outcome>>,
    updates: Vec<test::Outcome>,
) -> Vec<test::Outcome> {
    let Some(mut existing) = previous else {
        return updates;
    };

    for update in updates {
        if let Some(slot) = existing
            .iter_mut()
            .find(|result| result.name == update.name)
        {
            *slot = update;
        } else {
            existing.push(update);
        }
    }

    existing
}

/// Runs the given test paths in a separate thread to avoid stalling the UI,
/// returning the test results when all are complete.
pub(super) async fn run_tests_in_thread(
    configure: crate::app::ConfigureFn,
    preview_index: usize,
    tests: Vec<PathBuf>,
) -> Vec<test::Outcome> {
    let mut tasks = tokio::task::JoinSet::new();

    for path in tests {
        let configure = configure.clone();
        tasks.spawn(run_test_path(configure, preview_index, path));
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(outcome) => results.push(outcome),
            Err(error) => results.push(test::Outcome::failed(
                "test-runner",
                format!("[Internal error] Test runner failed: {error}"),
            )),
        }
    }

    results
}

/// Runs the test at the given `path` and returns the result.
async fn run_test_path(
    configure: crate::app::ConfigureFn,
    preview_index: usize,
    path: PathBuf,
) -> test::Outcome {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let content = match tokio::fs::read_to_string(&path).await {
        Ok(content) => content,
        Err(e) => {
            return test::Outcome::failed(name, format!("Failed to read test file: {e}"));
        }
    };

    let ice = match super::Ice::parse(&content) {
        Ok(ice) => ice,
        Err(e) => {
            return test::Outcome::failed(name, format!("Failed to parse .ice file: {e}"));
        }
    };

    let mut app = (configure)(crate::App::default());
    if preview_index >= app.descriptors().len() {
        return test::Outcome::failed(name, "Preview index out of range for test run");
    }

    match super::run_single_test(&mut app, preview_index, &ice, &path) {
        Some(error) => test::Outcome::failed(name, error),
        None => test::Outcome::passed(name),
    }
}

#[cfg(test)]
mod tests {
    use super::merge_run_results;
    use crate::test;

    /// Outcomes with matching names should be replaced by the latest update.
    #[test]
    fn merge_run_results_updates_existing_by_name() {
        let previous = Some(vec![
            test::Outcome::passed("alpha"),
            test::Outcome::failed("beta", "old error"),
        ]);

        let updates = vec![test::Outcome::passed("beta")];

        let merged = merge_run_results(previous, updates);
        assert_eq!(
            merged,
            vec![
                test::Outcome::passed("alpha"),
                test::Outcome::passed("beta"),
            ]
        );
    }

    /// New test outcomes should be appended to the end of the results list.
    #[test]
    fn merge_run_results_appends_new_outcomes_to_end() {
        let previous = Some(vec![
            test::Outcome::passed("alpha"),
            test::Outcome::passed("beta"),
        ]);

        let updates = vec![test::Outcome::failed("gamma", "new error")];

        let merged = merge_run_results(previous, updates);

        assert_eq!(
            merged,
            vec![
                test::Outcome::passed("alpha"),
                test::Outcome::passed("beta"),
                test::Outcome::failed("gamma", "new error"),
            ]
        );
    }
}
