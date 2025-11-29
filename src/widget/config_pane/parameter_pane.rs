use iced::Length::FillPortion;
use iced::widget::{column, responsive, scrollable, table, text, text_input, toggler};
use iced::{Element, Theme};

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
    match param.value {
        Value::Bool(active) => toggler(active)
            .on_toggle(move |active| Message::ChangeParam(index, Value::Bool(active)))
            .into(),
        Value::Text(ref value) => text_input(&param.name, value)
            .on_input(move |value| Message::ChangeParam(index, Value::Text(value)))
            .into(),
        Value::I32(number) => text_input(&param.name, &number.to_string())
            .on_input(move |value| {
                if let Ok(num) = value.parse::<i32>() {
                    Message::ChangeParam(index, Value::I32(num))
                } else {
                    Message::Noop
                }
            })
            .into(),
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
