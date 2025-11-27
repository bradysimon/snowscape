use iced::{Element, Task};

use crate::preview::Preview;

/// A dynamic preview that supports adjustable parameters.
/// This allows users to modify certain aspects of the preview at runtime.
pub struct Dynamic<P: Preview> {
    /// The parameters the user can adjust.
    params: ParamSet,
    /// The underlying preview component.
    preview: P,
}

impl<P: Preview> Preview for Dynamic<P> {
    fn metadata(&self) -> &crate::Metadata {
        self.preview.metadata()
    }

    fn update(&mut self, message: crate::Message) -> Task<crate::Message> {
        if let crate::Message::ChangeParam(index, value) = message {
            if let Some(param) = self.params.params.get_mut(index) {
                param.value = value;
            }
            return Task::none();
        }
        self.preview.update(message)
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
        &self.params.params
    }
}

pub fn text(name: impl Into<String>, value: impl Into<String>) -> Param {
    Param::new(name, Value::Text(value.into()))
}

/// A dynamic parameter value used within [`Param`].
#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Text(String),
    I32(i32),
}

/// A dynamic parameter that can be adjusted in the configuration pane.
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub value: Value,
}

impl Param {
    pub fn new<N, V>(name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<Value>,
    {
        Param {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::I32(value)
    }
}

#[derive(Debug, Clone)]
pub struct ParamSet {
    params: Vec<Param>,
}

impl<T> From<T> for ParamSet
where
    T: Into<Param>,
{
    fn from(value: T) -> Self {
        ParamSet {
            params: vec![value.into()],
        }
    }
}

macro_rules! impl_paramset_from_tuple {
    ($($T:ident),+) => {
        impl<$($T),+> From<($($T,)+)> for ParamSet
        where
            $($T: Into<Param>,)+
        {
            fn from(value: ($($T,)+)) -> Self {
                #[allow(non_snake_case)]
                let ($($T,)+) = value;
                ParamSet {
                    params: vec![$($T.into(),)+],
                }
            }
        }
    };
}

impl_paramset_from_tuple!(T1, T2);
impl_paramset_from_tuple!(T1, T2, T3);
impl_paramset_from_tuple!(T1, T2, T3, T4);
impl_paramset_from_tuple!(T1, T2, T3, T4, T5);
impl_paramset_from_tuple!(T1, T2, T3, T4, T5, T6);
impl_paramset_from_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_paramset_from_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);

pub fn dynamic<Params, F, P>(params: Params, generate: F) -> Dynamic<P>
where
    Params: Into<ParamSet> + Clone + Send,
    F: Fn(Params) -> P + Send + 'static,
    P: Preview,
{
    let preview = generate(params.clone());
    let params = params.into();

    Dynamic { params, preview }
}
