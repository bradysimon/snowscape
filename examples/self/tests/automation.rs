//! Automation tests for the Snowscape application itself.
//!
//! These tests use `snowscape::test::Emulator` to drive the real Snowscape UI
//! in a headless environment, exercising features like the search input,
//! preview selection, and composite interactions.

use app::program;
use snowscape::test::Emulator;
use snowscape::test::automation::{self, Id};

// MARK: - Search input (fill)

/// Typing into the search input filters the preview sidebar list.
#[test]
fn fill_search_filters_previews() -> automation::Result {
    let mut emulator = Emulator::new(program())?;

    // The sidebar should show all previews initially.
    assert!(
        emulator.exists("Performance Pane"),
        "Performance Pane should be visible"
    );
    assert!(emulator.exists("Dialog"), "Dialog should be visible");

    // Type "Dialog" into the search input to filter.
    emulator.fill(Id::new(snowscape::widget::SEARCH_INPUT_ID), "Dialog")?;

    // "Dialog" previews should still be visible.
    assert!(
        emulator.exists("Dialog"),
        "Dialog should still be visible after search"
    );
    // "Performance Pane" should be filtered out — it only appeared in the sidebar.
    assert!(
        !emulator.exists("Performance Pane"),
        "Performance Pane should be filtered out by search"
    );

    Ok(())
}

// MARK: - Time travel (drag)

/// Generating messages in a stateful preview and clicking the timeline
/// slider to rewind causes earlier messages to disappear from the message pane.
#[test]
fn time_travel_filters_visible_messages() -> automation::Result {
    let mut emulator = Emulator::builder(program())
        .size(iced::Size::new(1280.0, 960.0))
        .build()?;

    // Select the "Dialog Without Animation" preview
    emulator.click("Dialog Without Animation")?;

    // Open the dialog.
    emulator.click("Open Dialog")?;

    // Click "+" twice inside the dialog to generate messages.
    emulator.click("+")?;
    emulator.click("+")?;

    // Switch to the Messages tab to see the emitted messages.
    emulator.click("Messages")?;

    // Verify messages are visible: the timeline should be "Live" and messages should show.
    assert!(emulator.exists("Live"), "Live button should be visible");
    assert!(
        !emulator.exists("No messages emitted."),
        "should have messages after interacting with the dialog"
    );

    // Find the "Live" button's position to anchor the slider click.
    // The timeline row is: [position badge] [slider 200px] [Live button]
    // Clicking on the far left of the slider rewinds to position 0.
    let live_target = emulator.find("Live")?;
    let live_bounds = live_target.bounds();

    // The slider is 200px wide and ends 4px before the Live button.
    let slider_start_x = live_bounds.x - 4.0 - 200.0;

    // Click the far-left edge of the slider to rewind to position 0.
    let rewind_point = iced::Point::new(slider_start_x + 1.0, live_bounds.center_y());
    emulator.click(rewind_point)?;

    // After rewinding to position 0, the message pane should be empty.
    assert!(
        emulator.exists("No messages emitted."),
        "message pane should show 'No messages emitted.' after rewinding to position 0"
    );

    // Click "Live" to return to the present.
    emulator.click("Live")?;

    // Messages should reappear.
    assert!(
        !emulator.exists("No messages emitted."),
        "messages should reappear after returning to Live"
    );

    Ok(())
}

// MARK: - Formatting (temporary)

/// Temporary test to verify that assertion failures render with readable
/// multi-line output instead of escaped `\n` characters. Delete after confirming.
#[test]
fn assert_exists_shows_readable_tree_on_failure() -> automation::Result {
    let mut emulator = Emulator::new(program())?;
    emulator.assert_exists("this text does not exist anywhere")
}
