//! The test configuration pane for recording visual tests.

use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    border, padding,
    widget::{
        button, checkbox, column, container, row, rule, scrollable, space, text, text_input,
        tooltip,
    },
};

use crate::{app::App, message::Message, test, widget::stop_recording_button};

use crate::test::discovery::{is_sanitized, sanitize_name};

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
    let test_state = app.test_state();
    let instruction_count = test_state
        .session
        .as_ref()
        .map(|s| s.instructions.len())
        .unwrap_or(0);
    let expect_input = test_state
        .session
        .as_ref()
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
                        .style(crate::style::text_input::default)
                        .on_input(|t| Message::Test(test::Message::ChangeExpectText(t)))
                        .on_submit(Message::Test(test::Message::AddTextExpectation))
                        .width(Fill),
                    button(text("Expect").size(14))
                        .on_press(Message::Test(test::Message::AddTextExpectation))
                        .padding([4, 12]),
                ]
                .spacing(8)
                .align_y(Center),
            ]
            .spacing(4),
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
    let test_state = app.test_state();

    let content = column![
        new_test_section(test_state),
        rule::horizontal(1).style(rule::weak),
        existing_tests_section(test_state),
    ]
    .spacing(16);

    container(scrollable(content).spacing(4).height(Fill)).into()
}

/// Section for creating a new test.
fn new_test_section<'a>(test_state: &'a test::State) -> Element<'a, Message> {
    use std::borrow::Cow;

    let has_size_error = !test_state.width_input.is_valid() || !test_state.height_input.is_valid();
    let size_error = has_size_error.then(|| {
        text("Invalid size")
            .size(14)
            .style(crate::style::text::danger)
    });
    let test_name = test_state.name_input.as_str();
    let saved_test_name: Cow<'_, str> = if is_sanitized(test_name) {
        Cow::Borrowed(test_name)
    } else {
        Cow::Owned(sanitize_name(test_name))
    };
    let saved_name_hint = container((!test_state.name_input.trim().is_empty()).then(|| {
        text(format!("Will be saved as: {}.ice", saved_test_name))
            .size(12)
            .style(crate::style::text::muted)
    }))
    .height(16);

    column![
        row![
            text("New Test"),
            size_error,
            space::horizontal(),
            record_button(test_state.can_record()),
        ]
        .spacing(8)
        .align_y(Center),
        // Test name and window size on same row
        row![
            // Test name input
            column![
                text("Name").size(12).style(crate::style::text::muted),
                text_input(
                    "Enter a test name, e.g. basic-increment",
                    &test_state.name_input
                )
                .style(crate::style::text_input::default)
                .size(14)
                .on_input(|n| Message::Test(test::Message::ChangeTestName(n))),
                saved_name_hint,
            ]
            .spacing(4)
            .width(Fill),
            // Window size configuration
            column![
                text("Size").size(12).style(crate::style::text::muted),
                row![
                    text_input("W", test_state.width_input.display())
                        .style(|theme, status| crate::style::text_input::validated(
                            theme,
                            status,
                            test_state.width_input.is_valid()
                        ))
                        .size(14)
                        .width(60)
                        .on_input(|w| Message::Test(test::Message::ChangeWidth(w))),
                    text("×").size(14),
                    text_input("H", test_state.height_input.display())
                        .style(|theme, status| crate::style::text_input::validated(
                            theme,
                            status,
                            test_state.height_input.is_valid()
                        ))
                        .size(14)
                        .width(60)
                        .on_input(|h| Message::Test(test::Message::ChangeHeight(h))),
                ]
                .spacing(6)
                .align_y(Center),
            ]
            .spacing(4),
        ]
        .spacing(12),
        // Snapshot option
        checkbox(test_state.config.capture_snapshot)
            .label("Capture snapshot at end")
            .on_toggle(|b| Message::Test(test::Message::ToggleSnapshot(b)))
            .text_size(13),
    ]
    .spacing(12)
    .into()
}

/// Section showing existing tests for the current preview.
fn existing_tests_section<'a>(test_state: &'a test::State) -> Element<'a, Message> {
    let has_tests = !test_state.discovered_tests.is_empty();
    let run_mode = test_state.run_mode.as_ref();
    let is_running = test_state.is_running();

    let run_all_button: Element<'a, Message> = if has_tests {
        let button = button(text("Run All").size(12))
            .padding([5, 12])
            .style(crate::style::button::subtle)
            .on_press_maybe((!is_running).then(|| Message::Test(test::Message::RunAll)));

        button.into()
    } else {
        space().into()
    };

    let running_indicator: Element<'a, Message> = if is_running {
        container(text("Running...").size(12).style(crate::style::text::muted))
            .padding(padding::left(8))
            .into()
    } else {
        space::horizontal().into()
    };

    let header = row![
        text("Tests").size(15),
        running_indicator,
        space::horizontal(),
        run_all_button,
    ]
    .align_y(Center);

    let test_list: Element<'a, Message> = if has_tests {
        container(
            column(
                test_state
                    .discovered_tests
                    .iter()
                    .enumerate()
                    .map(|(i, info)| {
                        test_row(
                            info,
                            &test_state.last_run_results,
                            i % 2 == 1,
                            is_running,
                            run_mode,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(0),
        )
        .style(|theme: &iced::Theme| container::Style {
            border: border::rounded(6)
                .width(1)
                .color(theme.extended_palette().background.weak.color),
            ..Default::default()
        })
        .into()
    } else {
        container(
            text("No tests recorded yet")
                .size(13)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        )
        .padding(padding::top(8))
        .into()
    };

    column![header, test_list].spacing(10).into()
}

/// Test run status for display.
enum TestStatus<'a> {
    /// Test has not been run.
    NotRun,
    /// Test passed.
    Passed,
    /// Test failed with an optional error message.
    Failed(Option<&'a str>),
}

/// Gets the status of a test from the last run results.
fn get_test_status<'a>(test_name: &str, results: &'a Option<Vec<test::Outcome>>) -> TestStatus<'a> {
    match results {
        Some(results) => {
            if let Some(result) = results.iter().find(|r| r.name == test_name) {
                if result.is_success() {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed(result.error.as_deref())
                }
            } else {
                TestStatus::NotRun
            }
        }
        None => TestStatus::NotRun,
    }
}

/// Creates a status icon element based on test status.
fn status_icon<'a>(status: TestStatus<'a>) -> Element<'a, Message> {
    let icon: Element<'a, Message> = match status {
        TestStatus::NotRun => crate::icon::circle_slash()
            .width(16)
            .height(16)
            .style(crate::style::svg::strong_background)
            .into(),
        TestStatus::Passed => crate::icon::checkmark()
            .width(16)
            .height(16)
            .style(crate::style::svg::success)
            .into(),
        TestStatus::Failed(error) => {
            let icon = crate::icon::xmark()
                .width(16)
                .height(16)
                .style(crate::style::svg::danger);

            let error_text = error.unwrap_or("Test failed");
            tooltip(
                icon,
                container(text(error_text).size(14)).padding(6),
                tooltip::Position::Top,
            )
            .style(crate::style::container::tooltip_background)
            .into()
        }
    };

    container(icon).width(16).height(16).into()
}

/// A single row for an existing test.
fn test_row<'a>(
    info: &'a test::TestInfo,
    results: &'a Option<Vec<test::Outcome>>,
    alternate: bool,
    is_running: bool,
    run_mode: Option<&'a test::state::RunMode>,
) -> Element<'a, Message> {
    let status = get_test_status(&info.name, results);

    let status_icon = if is_running {
        match run_mode {
            Some(test::state::RunMode::All) => running_status_icon(),
            Some(test::state::RunMode::Single(path)) if path == &info.path => running_status_icon(),
            _ => status_icon(status),
        }
    } else {
        status_icon(status)
    };

    let path = info.path.clone();
    let path_delete = info.path.clone();

    let name_text = if is_running {
        text(&info.name).size(13).style(crate::style::text::muted)
    } else {
        text(&info.name).size(13)
    };

    let run_button = button(
        crate::icon::play()
            .style(crate::style::svg::text)
            .width(12)
            .height(12),
    )
    .padding([4, 8])
    .style(icon_button_style)
    .on_press_maybe((!is_running).then(|| Message::Test(test::Message::RunSingle(path))));

    let delete_button = button(
        crate::icon::trash()
            .style(crate::style::svg::text)
            .width(12)
            .height(12),
    )
    .padding([4, 8])
    .style(delete_button_style)
    .on_press_maybe((!is_running).then(|| Message::Test(test::Message::Delete(path_delete))));

    let row_content = row![
        status_icon,
        name_text,
        space::horizontal(),
        run_button,
        delete_button,
    ]
    .spacing(8)
    .align_y(Center);

    container(row_content)
        .width(Fill)
        .padding([8, 10])
        .style(move |theme: &iced::Theme| {
            let background = if alternate {
                Some(
                    theme
                        .extended_palette()
                        .background
                        .weak
                        .color
                        .scale_alpha(0.3)
                        .into(),
                )
            } else {
                None
            };

            if is_running {
                container::Style {
                    text_color: Some(
                        theme
                            .extended_palette()
                            .background
                            .weakest
                            .text
                            .scale_alpha(0.55),
                    ),
                    background: background.or_else(|| {
                        Some(
                            theme
                                .extended_palette()
                                .background
                                .weak
                                .color
                                .scale_alpha(0.15)
                                .into(),
                        )
                    }),
                    ..Default::default()
                }
            } else if let Some(background) = background {
                container::Style {
                    background: Some(background),
                    ..Default::default()
                }
            } else {
                container::Style::default()
            }
        })
        .into()
}

fn running_status_icon<'a>() -> Element<'a, Message> {
    let icon = crate::icon::refresh()
        .width(16)
        .height(16)
        .style(crate::style::svg::strong_background);

    container(icon).width(16).height(16).into()
}

/// Icon button style for action buttons in rows.
fn icon_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    match status {
        button::Status::Active => button::Style {
            background: None,
            text_color: palette.background.base.text.scale_alpha(0.7),
            border: border::rounded(4),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(palette.primary.weak.color.into()),
            text_color: palette.primary.weak.text,
            border: border::rounded(4),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(palette.primary.base.color.into()),
            text_color: palette.primary.base.text,
            border: border::rounded(4),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: None,
            text_color: palette.background.base.text.scale_alpha(0.3),
            border: border::rounded(4),
            ..Default::default()
        },
    }
}

/// Delete button style with danger color on hover.
fn delete_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    match status {
        button::Status::Active => button::Style {
            background: None,
            text_color: palette.background.base.text.scale_alpha(0.7),
            border: border::rounded(4),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(palette.danger.weak.color.into()),
            text_color: palette.danger.weak.text,
            border: border::rounded(4),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(palette.danger.base.color.into()),
            text_color: palette.danger.base.text,
            border: border::rounded(4),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: None,
            text_color: palette.background.base.text.scale_alpha(0.3),
            border: border::rounded(4),
            ..Default::default()
        },
    }
}

/// A button to start recording a test.
fn record_button<'a>(enabled: bool) -> Element<'a, Message> {
    let btn = button(text("Record").size(14)).style(|theme: &iced::Theme, status| {
        let pair = match status {
            button::Status::Hovered => theme.extended_palette().danger.weak,
            button::Status::Pressed => theme.extended_palette().danger.strong,
            button::Status::Disabled => theme.extended_palette().danger.weak,
            _ => theme.extended_palette().danger.base,
        };

        let opacity = if status == button::Status::Disabled {
            0.6
        } else {
            1.0
        };

        button::Style {
            background: Some(pair.color.scale_alpha(opacity).into()),
            text_color: pair.text.scale_alpha(opacity),
            border: iced::border::rounded(4),
            ..button::primary(theme, status)
        }
    });

    btn.on_press_maybe(enabled.then(|| Message::Test(test::Message::StartRecording)))
        .into()
}
