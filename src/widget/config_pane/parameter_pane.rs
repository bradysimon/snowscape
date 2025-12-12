use iced::Length::{FillPortion, Shrink};
use iced::widget::{
    button, column, container, pick_list, responsive, row, scrollable, slider, space, table, text,
    text_input,
};
use iced::{Element, Length, Theme, border};

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
            text("Value").size(14).style(header_style),
            |(index, param): (usize, &Param)| field(param, index),
        )
        .width(FillPortion(3)),
    ];

    table(columns, params.iter().enumerate())
        .separator(0)
        .into()
}

/// Displays the parameters in a vertical layout, typically for narrow widths.
pub fn vertical_view(params: &[Param]) -> Element<'_, Message> {
    let fields = params
        .iter()
        .enumerate()
        .map(|(index, param)| labeled(&param.name, field(param, index)));

    column(fields).spacing(10).into()
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
            text_color: active
                .then(|| active_pair.text)
                .unwrap_or(theme.palette().text),
            ..button::text(theme, status)
        }
    };

    // Fixed with to ensure the true/false buttons are consistent
    const BUTTON_SIZE: f32 = 40.0;
    container(
        container(
            row![
                button(text("False").size(14).width(BUTTON_SIZE).center())
                    .on_press(message(false))
                    .style(move |theme, status| button_style(theme, status, !active)),
                button(text("True").size(14).width(BUTTON_SIZE).center())
                    .on_press(message(true))
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
fn color_picker(index: usize, color: iced::Color) -> Element<'static, Message> {
    use iced::widget::container;
    use iced::{Color, border};

    let (r, g, b, a) = (color.r, color.g, color.b, color.a);

    let color_swatch =
        container(space().width(32).height(32)).style(move |theme: &Theme| container::Style {
            background: Some(color.into()),
            border: border::rounded(4)
                .width(1)
                .color(theme.extended_palette().background.neutral.color),
            ..Default::default()
        });

    let color_slider = |label: &'static str, value: f32| {
        let (r, g, b, a) = (r, g, b, a);
        row![
            text(label).size(12).width(16),
            slider(0.0..=1.0, value, move |v| {
                let new_color = match label {
                    "R" => Color::from_rgba(v, g, b, a),
                    "G" => Color::from_rgba(r, v, b, a),
                    "B" => Color::from_rgba(r, g, v, a),
                    "A" => Color::from_rgba(r, g, b, v),
                    _ => Color::from_rgba(r, g, b, a),
                };
                Message::ChangeParam(index, Value::Color(new_color))
            })
            .step(0.01)
            .width(Length::Fill),
            text!("{:.0}", value * 255.0).size(12).width(28),
        ]
        .spacing(4)
    };

    column![
        row![
            color_swatch,
            column![
                color_slider("R", r),
                color_slider("G", g),
                color_slider("B", b),
                color_slider("A", a),
            ]
            .spacing(2)
            .width(Length::Fill),
        ]
        .spacing(8),
    ]
    .into()
}
