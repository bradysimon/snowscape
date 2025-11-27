mod extract_params;
pub mod param;

use iced::{Element, Task};

use crate::preview::Preview;
pub use extract_params::ExtractParams;
pub use param::{Param, boolean, number, text};

/// A function type that generates a preview from given parameters.
pub type Generate<Params, Preview> = dyn Fn(&Params) -> Preview + Send;

/// A dynamic preview that supports adjustable parameters.
/// This allows users to modify certain aspects of the preview at runtime.
pub struct Dynamic<Params: ExtractParams, P: Preview> {
    /// The dynamic parameters the user can adjust.
    params: Params,
    /// A cached list of params generated from `params` for displaying in the UI.
    cached_params: Vec<Param>,
    /// A function to regenerate the preview from parameters.
    generate: Box<Generate<Params::Values, P>>,
    /// The underlying preview component.
    preview: P,
}

impl<Params: ExtractParams, P: Preview> Preview for Dynamic<Params, P> {
    fn metadata(&self) -> &crate::Metadata {
        self.preview.metadata()
    }

    fn update(&mut self, message: crate::Message) -> Task<crate::Message> {
        if let crate::Message::ChangeParam(index, value) = message {
            self.params.update_index(index, value);
            self.cached_params = self.params.to_params();
            let values = self.params.extract();
            self.preview = (self.generate)(&values);
            Task::none()
        } else {
            self.preview.update(message)
        }
    }

    fn view(&self) -> Element<'_, crate::Message> {
        self.preview.view()
    }

    fn message_count(&self) -> usize {
        self.preview.message_count()
    }

    fn timeline(&self) -> Option<crate::preview::Timeline> {
        self.preview.timeline()
    }

    fn visible_messages(&self) -> &'_ [String] {
        self.preview.visible_messages()
    }

    fn params(&self) -> &[Param] {
        &self.cached_params
    }
}

/// A dynamic parameter value used within [`Param`].
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Text(String),
    I32(i32),
}

/// Create a new dynamic preview with adjustable parameters.
///
/// Pass in parameters implementing [`ExtractParams`] and a function that generates
/// a preview from the extracted parameter values. You typically want to combine this
/// with the [`crate::stateless`] or [`crate::stateful`] preview functions.
///
/// # Example
/// ```
/// use snowscape::{dynamic, stateless};
///
/// #[derive(Debug, Clone)]
/// enum Message {}
///
/// fn label(content: &str) -> iced::Element<'_, Message> {
///    iced::widget::text(content).into()
/// }
///
/// fn main() -> iced::Result {
///     snowscape::run(|app| {
///         let preview = dynamic(dynamic::text("Content", "Editable"), |content| {
///             stateless("Label", move || label(&content))
///                 .description("Shows a label for some given content")
///         });
///         app.preview(preview)
///     })
/// }
/// ```
pub fn dynamic<Params, F, P>(params: Params, generate: F) -> Dynamic<Params, P>
where
    Params: ExtractParams,
    F: Fn(Params::Values) -> P + Send + 'static,
    P: Preview,
{
    let values = params.extract();
    let preview = generate(values);
    let cached_params = params.to_params();

    Dynamic {
        params,
        cached_params,
        generate: Box::new(move |values| generate(values.clone())),
        preview,
    }
}
