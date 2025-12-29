use iced::Alignment::Center;
use iced::Length::{FillPortion, Shrink};
use iced::widget::{
    button, column, container, pick_list, responsive, right, row, scrollable, slider, space, svg,
    table, text, text_input,
};
use iced::{Color, Element, Length, Theme, border};

use crate::style;
use crate::{
    app::Message,
    dynamic::{Param, Value},
};

/// The pane containing the list of adjustable dynamic parameters for the preview.
///
/// Dynamic parameters allow the user to modify certain parts of the preview at runtime.
pub fn parameter_pane(params: &[Param]) -> Element<'_, Message> {
    if params.is_empty() {
        text("This preview has no adjustable parameters.")
            .size(16)
            .into()
    } else {
        scrollable(responsive(|size| {
            if size.width < 576.0 {
                vertical_view(params)
            } else {
                table_view(params)
            }
        }))
        .spacing(4)
        .into()
    }
}

/// Displays the parameters in a table layout, typically for larger widths.
pub fn table_view(params: &[Param]) -> Element<'_, Message> {
    let header_style = |theme: &Theme| text::Style {
        color: Some(theme.palette().text.scale_alpha(0.75)),
    };

    let columns = [
        table::column(
            text("Name").size(14).style(header_style),
            |(_, param): (usize, &Param)| text(&param.name).size(14),
        )
        .width(FillPortion(1)),
        table::column(
            row![
                text("Value").size(14).style(header_style),
                space::horizontal(),
                undo_button(),
            ],
            |(index, param): (usize, &Param)| field(param, index),
        )
        .width(FillPortion(3)),
    ];

    table(columns, params.iter().enumerate())
        .separator(0)
        .into()
}

/// Allows the user to undo any changes they've made to the dynamic parameters.
pub fn undo_button<'a>() -> Element<'a, Message> {
    button(
        row![
            crate::icon::undo()
                .width(14)
                .height(14)
                .style(|theme: &Theme, _status| svg::Style {
                    color: Some(theme.palette().text),
                }),
            text("Undo").size(14),
        ]
        .spacing(4)
        .align_y(Center),
    )
    .on_press(Message::ResetParams)
    .style(|theme: &Theme, status| button::Style {
        background: None,
        border: border::rounded(4),
        ..button::text(theme, status)
    })
    .into()
}

/// Displays the parameters in a vertical layout, typically for narrow widths.
pub fn vertical_view(params: &[Param]) -> Element<'_, Message> {
    let fields = params
        .iter()
        .enumerate()
        .map(|(index, param)| labeled(&param.name, field(param, index)));

    // Place the undo button near the top so vertical layouts can reset params.
    column![right(undo_button()), column(fields).spacing(10)]
        .spacing(8)
        .into()
}

/// Displays an editable field for a dynamic `param`.
pub fn field(param: &Param, index: usize) -> Element<'_, Message> {
    match &param.value {
        Value::Bool(active) => boolean_toggle(*active, |active| {
            Message::ChangeParam(index, Value::Bool(active))
        }),
        Value::Text(value) => text_input(&param.name, value)
            .on_input(move |value| Message::ChangeParam(index, Value::Text(value)))
            .style(input_style)
            .into(),
        // TODO: Use a number input once iced's `Component` rework is finished
        Value::I32(number) => text_input(&param.name, &number.to_string())
            .on_input(move |value| {
                if let Ok(num) = value.parse::<i32>() {
                    Message::ChangeParam(index, Value::I32(num))
                } else {
                    Message::Noop
                }
            })
            .style(input_style)
            .into(),
        Value::Select(selected_index, options) => {
            let options_clone = options.clone();
            let selected = options.get(*selected_index).cloned();
            pick_list(options.clone(), selected, move |selected_value| {
                let new_index = options_clone
                    .iter()
                    .position(|o| *o == selected_value)
                    .unwrap_or(0);
                Message::ChangeParam(index, Value::Select(new_index, options_clone.clone()))
            })
            .style(crate::style::pick_list::default)
            .menu_style(crate::style::pick_list::menu)
            .text_size(14)
            .into()
        }
        Value::Slider(value, range) => row![
            slider(range.clone(), *value, move |v| {
                Message::ChangeParam(index, Value::Slider(v, range.clone()))
            })
            .width(Length::Fill),
            text!("{:.1}", value).size(14).width(40),
        ]
        .spacing(8)
        .into(),
        Value::Color(color) => color_picker(index, *color),
    }
}

/// Displays a label above the given `element`.
pub fn labeled<'a>(
    label: &'a str,
    element: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![text(label).size(14), element.into()]
        .spacing(2)
        .into()
}

fn input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let default = text_input::default(theme, status);
    text_input::Style {
        border: default.border.rounded(4),
        ..default
    }
}

/// A custom toggle for Booleans that shows true/false labels.
/// Similar to a segmented button but only for two states.
fn boolean_toggle<'a, Message: Clone + 'a>(
    active: bool,
    message: impl Fn(bool) -> Message,
) -> Element<'a, Message> {
    let button_style = |theme: &Theme, status: button::Status, active: bool| {
        let active_pair = if theme.extended_palette().is_dark {
            theme.extended_palette().background.strongest
        } else {
            theme.extended_palette().background.weakest
        };
        button::Style {
            background: active.then(|| active_pair.color.into()),
            border: border::rounded(8),
            text_color: if active {
                active_pair.text
            } else {
                theme.palette().text
            },
            ..button::text(theme, status)
        }
    };

    // Fixed with to ensure the true/false buttons are consistent
    const BUTTON_WIDTH: f32 = 40.0;
    container(
        container(
            row![
                button(text("False").size(14).width(BUTTON_WIDTH).center())
                    .on_press(message(false))
                    .padding([4, 6])
                    .style(move |theme, status| button_style(theme, status, !active)),
                button(text("True").size(14).width(BUTTON_WIDTH).center())
                    .on_press(message(true))
                    .padding([4, 6])
                    .style(move |theme, status| button_style(theme, status, active)),
            ]
            .width(Shrink)
            .spacing(0),
        )
        .padding(2),
    )
    .style(|theme: &Theme| container::Style {
        background: Some(theme.extended_palette().background.weak.color.into()),
        border: border::rounded(10),
        ..Default::default()
    })
    .into()
}

/// A simple color picker with a preview swatch.
fn color_picker<'a>(index: usize, color: Color) -> Element<'a, Message> {
    use iced::{border, widget::container};

    let [r, g, b, a] = color.into_rgba8();

    let color_swatch =
        container(space().width(32).height(32)).style(move |theme: &Theme| container::Style {
            background: Some(color.into()),
            border: border::rounded(4)
                .width(1)
                .color(theme.extended_palette().background.neutral.color),
            ..Default::default()
        });

    let color_slider = |channel: style::ColorChannel, value: u8| {
        let (r, g, b, a) = (r, g, b, a);
        let backgrounds = style::channel_slider_backgrounds(channel, r, g, b, a);

        let color_slider = container(
            slider(0..=255, value, move |v| {
                let new_color = match channel {
                    style::ColorChannel::Red => Color::from_rgba8(v, g, b, a as f32 / 255.0),
                    style::ColorChannel::Green => Color::from_rgba8(r, v, b, a as f32 / 255.0),
                    style::ColorChannel::Blue => Color::from_rgba8(r, g, v, a as f32 / 255.0),
                    style::ColorChannel::Alpha => Color::from_rgba8(r, g, b, v as f32 / 255.0),
                };
                Message::ChangeParam(index, Value::Color(new_color))
            })
            .style(move |theme: &Theme, _status| slider::Style {
                rail: slider::Rail {
                    backgrounds,
                    border: border::rounded(4)
                        .width(1)
                        .color(theme.extended_palette().background.weak.color),
                    width: 6.0,
                },
                handle: slider::Handle {
                    shape: slider::HandleShape::Circle { radius: 8.0 },
                    background: theme.extended_palette().secondary.base.color.into(),
                    border_width: 1.0,
                    border_color: theme.extended_palette().secondary.strong.color,
                },
            })
            .width(Length::Fill),
        )
        .max_width(400);

        let rgb_input = text_input("", &value.to_string())
            .on_input(move |v| {
                if let Ok(num) = v.parse::<u8>() {
                    let clamped = num.clamp(0, 255);
                    let new_color = match channel {
                        style::ColorChannel::Red => {
                            Color::from_rgba8(clamped, g, b, a as f32 / 255.0)
                        }
                        style::ColorChannel::Green => {
                            Color::from_rgba8(r, clamped, b, a as f32 / 255.0)
                        }
                        style::ColorChannel::Blue => {
                            Color::from_rgba8(r, g, clamped, a as f32 / 255.0)
                        }
                        style::ColorChannel::Alpha => {
                            Color::from_rgba8(r, g, b, clamped as f32 / 255.0)
                        }
                    };
                    Message::ChangeParam(index, Value::Color(new_color))
                } else {
                    Message::Noop
                }
            })
            .style(input_style)
            .size(12)
            .width(40);

        row![
            text(channel.letter()).size(12).width(16),
            color_slider,
            rgb_input,
        ]
        .spacing(4)
        .align_y(Center)
    };

    column![
        row![
            color_swatch,
            column![
                color_slider(style::ColorChannel::Red, r),
                color_slider(style::ColorChannel::Green, g),
                color_slider(style::ColorChannel::Blue, b),
                color_slider(style::ColorChannel::Alpha, a),
            ]
            .spacing(2)
            .width(Length::Fill),
        ]
        .spacing(8),
    ]
    .into()
}
