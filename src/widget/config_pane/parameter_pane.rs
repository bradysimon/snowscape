use iced::Element;
use iced::widget::{column, text, text_input, toggler};

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
        let fields = params
            .iter()
            .enumerate()
            .map(|(index, param)| field(param, index));

        column(fields).spacing(10).into()
    }
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
