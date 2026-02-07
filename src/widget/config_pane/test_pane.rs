//! The test configuration pane for recording visual tests.

use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    border, padding,
    widget::{button, checkbox, column, container, row, space, text, text_input},
};

use crate::{app::App, message::Message, widget::stop_recording_button};

/// The test configuration pane content.
pub fn test_pane<'a>(app: &'a App) -> Element<'a, Message> {
    if app.is_recording() {
        recording_view(app)
    } else {
        configuration_view(app)
    }
}

/// View shown when recording is in progress.
fn recording_view<'a>(app: &'a App) -> Element<'a, Message> {
    let instruction_count = app
        .test_session()
        .map(|s| s.instructions.len())
        .unwrap_or(0);
    let expect_input = app
        .test_session()
        .map(|s| s.expect_text_input.as_str())
        .unwrap_or("");

    container(
        column![
            text("Recording in progress...").size(16),
            text(format!("{} instructions recorded", instruction_count)).size(14),
            // Expectation input
            column![
                text("Add Expectation:").size(14),
                row![
                    text_input("Expected text...", expect_input)
                        .on_input(Message::ChangeExpectText)
                        .width(Fill),
                    button(text("Expect").size(14))
                        .on_press(Message::AddTextExpectation)
                        .padding([4, 12]),
                ]
                .spacing(8)
                .align_y(Center),
            ]
            .spacing(4),
            // Snapshot button
            button(text("Capture Snapshot").size(14))
                .on_press(Message::CaptureSnapshot)
                .padding([8, 16]),
            // Stop recording
            container(stop_recording_button()).padding(padding::top(8)),
        ]
        .spacing(12)
        .align_x(Center),
    )
    .padding(8)
    .into()
}

/// View shown when not recording, allowing configuration.
fn configuration_view<'a>(app: &'a App) -> Element<'a, Message> {
    container(
        column![
            row![
                text("Test Configuration"),
                space::horizontal(),
                container(record_button()).padding(padding::top(8)),
            ]
            .align_y(Center),
            // Window size configuration
            row![
                text("Window Size:").size(14),
                text_input("Width", app.test_width_input())
                    .width(80)
                    .on_input(Message::ChangeTestWidth),
                text("×").size(14),
                text_input("Height", app.test_height_input())
                    .width(80)
                    .on_input(Message::ChangeTestHeight),
            ]
            .spacing(8)
            .align_y(Center),
            // Snapshot option
            checkbox(app.test_config().capture_snapshot)
                .label("Capture snapshot at end of test")
                .on_toggle(Message::ToggleTestSnapshot)
                .text_size(14),
        ]
        .spacing(12),
    )
    .padding(8)
    .into()
}

fn circle<'a>(size: f32) -> Element<'a, Message> {
    container(space())
        .width(size)
        .height(size)
        .style(move |theme: &iced::Theme| container::Style {
            background: Some(theme.palette().text.into()),
            border: border::rounded(size / 2.0),
            ..container::Style::default()
        })
        .into()
}

/// A button to start recording a test.
fn record_button<'a>() -> Element<'a, Message> {
    button(
        row![circle(10.0), text("Start Recording").size(14)]
            .align_y(Center)
            .spacing(6)
            .align_y(Center),
    )
    .on_press(Message::StartTestRecording)
    .style(|theme: &iced::Theme, status| {
        let pair = match status {
            button::Status::Hovered => theme.extended_palette().danger.weak,
            button::Status::Pressed => theme.extended_palette().danger.strong,
            button::Status::Disabled => theme.extended_palette().background.weakest,
            _ => theme.extended_palette().danger.base,
        };
        button::Style {
            background: Some(pair.color.into()),
            text_color: pair.text,
            border: iced::border::rounded(4),
            ..button::primary(theme, status)
        }
    })
    .into()
}
