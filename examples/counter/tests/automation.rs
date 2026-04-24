//! Automation tests for the counter program.
//!
//! These tests demonstrate `snowscape::test::Emulator`, which let you test
//! your UI in a headless environment.

use std::time::Duration;

use counter::{COUNT_TEXT_ID, program};
use snowscape::test::Emulator;
use snowscape::test::automation::{self, Id, select};

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
