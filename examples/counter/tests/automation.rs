//! Automation tests for the counter program.
//!
//! These tests demonstrate `snowscape::test::Emulator`, which lets you test
//! your UI in a headless environment.

use std::time::Duration;

use counter::{COUNT_TEXT_ID, program};
use snowscape::test::Emulator;
use snowscape::test::automation::{self, Id, select};

// MARK: - Basic interactions

/// Clicking the increment button several times updates the visible count.
#[test]
fn clicking_increment_increases_count() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    assert!(emulator.exists("Count: 0"), "initial count should be 0");

    for _ in 0..3 {
        emulator.click("Increment")?;
    }

    assert!(emulator.exists("Count: 3"), "count should reach 3");
    Ok(())
}

/// Decrement after several increments returns the count to zero.
#[test]
fn decrement_restores_count() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    emulator.click("Increment")?;
    emulator.click("Increment")?;
    emulator.click("Decrement")?;
    emulator.click("Decrement")?;

    assert!(emulator.exists("Count: 0"));
    Ok(())
}

/// `wait_for_text` polls the UI until an asynchronously-updated value appears.
#[test]
fn wait_for_text_after_delayed_increment() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    emulator.click("Delayed Increment")?;

    emulator.wait_for_text_with_timeout("Count: 1", Duration::from_secs(2))?;
    Ok(())
}

/// `find` locates a widget by its id even when its text content changes.
#[test]
fn find_widget_by_id() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    emulator.click("Increment")?;

    let target = emulator.find(select::id(COUNT_TEXT_ID))?;
    assert!(target.visible_bounds().is_some());
    Ok(())
}

/// The custom builder lets tests configure size and the default timeout, and
/// `Id::new` clicks targets by widget id.
#[test]
fn builder_configures_emulator() -> automation::Result {
    let mut emulator = Emulator::builder(program())
        .size(iced::Size::new(640.0, 480.0))
        .default_timeout(Duration::from_millis(750))
        .build()?;

    assert_eq!(emulator.size(), iced::Size::new(640.0, 480.0));
    assert_eq!(emulator.default_timeout(), Duration::from_millis(750));

    emulator.click(Id::new(counter::INCREMENT_BUTTON_ID))?;
    assert!(emulator.exists("Count: 1"));
    Ok(())
}

// MARK: - Advanced queries

/// `find_all` returns every widget matching a selector.
#[test]
fn find_all_returns_multiple_matches() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    // Find all text widgets in the counter UI.
    let texts = emulator.find_all(|candidate: select::Candidate<'_>| {
        if let select::Candidate::Text { content, .. } = candidate {
            Some(content.to_string())
        } else {
            None
        }
    });
    // There should be at least 4 text widgets: "Count: 0", "Increment", "Decrement", "Delayed Increment".
    assert!(
        texts.len() >= 4,
        "expected >= 4 text widgets, got {}: {:?}",
        texts.len(),
        texts
    );
    Ok(())
}

/// `count` returns how many widgets match a selector.
#[test]
fn count_matches_selector() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    // Count how many text widgets contain "Count:" — there should be exactly one.
    let n = emulator.count(|candidate: select::Candidate<'_>| {
        if let select::Candidate::Text { content, .. } = candidate {
            content.contains("Count:").then(|| content.to_string())
        } else {
            None
        }
    });
    assert_eq!(n, 1, "expected exactly 1 'Count:' text widget");
    Ok(())
}

/// `is_visible` checks visibility within the viewport.
#[test]
fn is_visible_for_visible_widget() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    assert!(emulator.is_visible(select::id(COUNT_TEXT_ID)));
    Ok(())
}

/// `widget_tree` produces a non-empty tree with Display output.
#[test]
fn widget_tree_is_not_empty() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    let tree = emulator.widget_tree();
    let display = format!("{tree}");
    assert!(
        !display.is_empty(),
        "widget tree should have display output"
    );
    // The tree should contain text nodes for "Count: 0" and the button labels.
    assert!(display.contains("Count: 0"), "tree should show Count: 0");
    assert!(display.contains("Increment"), "tree should show Increment");
    assert!(display.contains("Decrement"), "tree should show Decrement");
    Ok(())
}

// MARK: - Assertion helpers

/// `assert_exists` passes for widgets that exist.
#[test]
fn assert_exists_passes() -> automation::Result {
    let mut emulator = Emulator::new(program())?;
    emulator.assert_exists("Count: 0")
}

/// `assert_exists` fails for widgets that don't exist.
#[test]
fn assert_exists_fails_for_missing() {
    let mut emulator = Emulator::new(program()).unwrap();
    let err = emulator.assert_exists("nonexistent text").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("assertion failed"),
        "error should mention assertion: {msg}"
    );
    // The error should include the widget tree for debugging.
    assert!(
        msg.contains("Widget tree:"),
        "error should include widget tree: {msg}"
    );
}

/// `assert_not_exists` passes for absent widgets.
#[test]
fn assert_not_exists_passes() -> automation::Result {
    let mut emulator = Emulator::new(program())?;
    emulator.assert_not_exists("Count: 99")
}

/// `assert_not_exists` fails for present widgets.
#[test]
fn assert_not_exists_fails_for_present() {
    let mut emulator = Emulator::new(program()).unwrap();
    let err = emulator.assert_not_exists("Count: 0").unwrap_err();
    assert!(err.to_string().contains("assertion failed"));
}

/// `assert_count` verifies the exact number of matches.
#[test]
fn assert_count_passes() -> automation::Result {
    let mut emulator = Emulator::new(program())?;
    // There should be exactly 1 "Count: 0" text widget.
    emulator.assert_count("Count: 0", 1)
}

/// `assert_count` fails when the count is wrong.
#[test]
fn assert_count_fails_on_mismatch() {
    let mut emulator = Emulator::new(program()).unwrap();
    let err = emulator.assert_count("Count: 0", 5).unwrap_err();
    assert!(err.to_string().contains("expected 5 matches, found 1"));
}

/// `wait_until_gone` succeeds when a selector disappears.
#[test]
fn wait_until_gone_succeeds_after_change() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    assert!(emulator.exists("Count: 0"));

    emulator.click("Increment")?;
    // "Count: 0" should now be gone immediately after the sync click.
    emulator.wait_until_gone_with_timeout("Count: 0", Duration::from_millis(500))?;
    Ok(())
}

/// `wait_until_gone` times out when the selector stays.
#[test]
fn wait_until_gone_times_out() {
    let mut emulator = Emulator::new(program()).unwrap();
    let err = emulator
        .wait_until_gone_with_timeout("Count: 0", Duration::from_millis(100))
        .unwrap_err();
    assert!(err.to_string().contains("timed out"));
}

// MARK: - Screenshots

/// `screenshot` returns non-empty RGBA data.
#[test]
fn screenshot_produces_data() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    let shot = emulator.screenshot(1.0);
    assert!(shot.width > 0);
    assert!(shot.height > 0);
    assert!(!shot.rgba.is_empty());
    Ok(())
}

/// `save_screenshot` writes a PNG file.
#[test]
fn save_screenshot_writes_file() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    let dir = std::env::temp_dir().join("snowscape_test_screenshots");
    let path = dir.join("counter_test.png");
    // Clean up from any previous run.
    let _ = std::fs::remove_file(&path);

    emulator.save_screenshot(&path, 1.0)?;
    assert!(
        path.exists(),
        "screenshot file should exist at {}",
        path.display()
    );

    // Clean up.
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
    Ok(())
}

/// `try_with_screenshot` saves a screenshot when configured and an error occurs.
#[test]
fn try_with_screenshot_saves_on_failure() -> automation::Result {
    let dir = std::env::temp_dir().join("snowscape_test_failure_shots");
    let _ = std::fs::remove_dir_all(&dir);

    let mut emulator = Emulator::builder(program())
        .screenshot_on_failure(&dir)
        .build()?;

    let result = emulator.try_with_screenshot("test_failure", |e| e.click("nonexistent widget"));
    assert!(result.is_err());

    let expected_path = dir.join("test_failure.png");
    assert!(
        expected_path.exists(),
        "screenshot should be saved at {}",
        expected_path.display()
    );

    // Clean up.
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

// MARK: - Composite interactions

/// `double_click` sends two click events.
#[test]
fn double_click_increments_twice() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    emulator.double_click("Increment")?;
    assert!(
        emulator.exists("Count: 2"),
        "double click should increment twice"
    );
    Ok(())
}
