use iced::{Element, Task};

use crate::preview::Preview;

pub struct Dynamic<P: Preview> {
    params: ParamSet,
    preview: P,
}

impl<P: Preview> Preview for Dynamic<P> {
    fn metadata(&self) -> &crate::Metadata {
        self.preview.metadata()
    }

    fn update(&mut self, message: crate::Message) -> Task<crate::Message> {
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

    fn params(&self) -> Option<Vec<Param>> {
        self.params.params.clone().into()
    }
}

#[derive(Debug, Clone)]
pub enum Param {
    Text(String),
    I32(i32),
}

impl From<String> for Param {
    fn from(value: String) -> Self {
        Param::Text(value)
    }
}

impl From<i32> for Param {
    fn from(value: i32) -> Self {
        Param::I32(value)
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
