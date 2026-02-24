//! Snapshot capture helpers for recorded test sessions.

use crate::test::Session;

/// Captures a baseline snapshot for the completed recording session, when enabled.
pub(super) fn capture_snapshot_for_session(
    session: &Session,
    configure: Option<crate::app::ConfigureFn>,
) {
    if !session.config.capture_snapshot {
        return;
    }

    let Some(configure) = configure else {
        eprintln!("Failed to capture snapshot: missing configure callback");
        return;
    };

    let mut app = (configure)(crate::App::default());
    let ice = session.to_ice();

    if let Some(snapshot_path) = session.snapshot_path()
        && let Err(e) = super::capture_baseline_screenshot(
            &mut app,
            session.preview_index,
            &ice,
            &snapshot_path,
        )
    {
        eprintln!("Failed to capture snapshot: {}", e);
    }
}
