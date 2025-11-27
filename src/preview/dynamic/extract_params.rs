use crate::dynamic::{
    Value,
    param::{DynamicParam, Param},
};

/// A trait for extracting typed values from parameter configurations,
/// which allows dynamic previews to manage adjustable parameters.
///
/// You can extract values from single parameters or tuples of parameters.
/// Tuples are supported up to arity 8.
pub trait ExtractParams: Clone + Send + 'static {
    /// The type of values extracted from these parameters.
    type Values: Clone + Send + 'static;

    /// Convert to a vector of Params for displaying in the UI.
    fn to_params(&self) -> Vec<Param>;

    /// Update parameters from a changed value at the given index.
    fn update_index(&mut self, index: usize, value: Value);

    /// Extract the typed values.
    fn extract(&self) -> Self::Values;
}

impl<T: DynamicParam> ExtractParams for T {
    type Values = T::Value;

    fn to_params(&self) -> Vec<Param> {
        vec![self.to_param()]
    }

    fn update_index(&mut self, index: usize, value: Value) {
        if index == 0 {
            self.update(value);
        }
    }

    fn extract(&self) -> Self::Values {
        DynamicParam::value(self)
    }
}

// Tuple implementations for ExtractParams
macro_rules! impl_extract_params_tuple {
    ($($T:ident : $idx:tt),+) => {
        impl<$($T: DynamicParam),+> ExtractParams for ($($T,)+) {
            type Values = ($($T::Value,)+);

            fn to_params(&self) -> Vec<Param> {
                vec![$(self.$idx.to_param(),)+]
            }

            fn update_index(&mut self, index: usize, value: Value) {
                $(
                    if index == $idx {
                        self.$idx.update(value);
                        return;
                    }
                )+
            }

            fn extract(&self) -> Self::Values {
                ($(self.$idx.extract(),)+)
            }
        }
    };
}

impl_extract_params_tuple!(T0: 0, T1: 1);
impl_extract_params_tuple!(T0: 0, T1: 1, T2: 2);
impl_extract_params_tuple!(T0: 0, T1: 1, T2: 2, T3: 3);
impl_extract_params_tuple!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4);
impl_extract_params_tuple!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5);
impl_extract_params_tuple!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6);
impl_extract_params_tuple!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7);

#[cfg(test)]
mod tests {
    use crate::dynamic::{Value, boolean, number, text};

    use super::*;

    #[test]
    fn single_param_extract() {
        let param = text("name", "value");
        let params = param.to_params();
        assert_eq!(params.len(), 1);
        assert_eq!(param.extract(), "value");
    }

    #[test]
    fn single_param_update() {
        let mut param = number("count", 10);
        param.update_index(0, Value::I32(20));
        assert_eq!(param.extract(), 20);
    }

    #[test]
    fn tuple_2_extract() {
        let params = (text("name", "Alice"), number("age", 30));
        let extracted = params.extract();
        assert_eq!(extracted.0, "Alice");
        assert_eq!(extracted.1, 30);
    }

    #[test]
    fn tuple_2_to_params() {
        let params = (text("name", "Alice"), number("age", 30));
        let params = params.to_params();
        assert_eq!(
            params,
            vec![
                Param::new("name", Value::Text("Alice".to_string())),
                Param::new("age", Value::I32(30))
            ]
        );
    }

    #[test]
    fn tuple_2_update_first() {
        let mut params = (text("name", "Alice"), number("age", 30));
        params.update_index(0, Value::Text("Bob".to_string()));
        let extracted = params.extract();
        assert_eq!(extracted.0, "Bob");
        assert_eq!(extracted.1, 30);
    }

    #[test]
    fn tuple_2_update_second() {
        let mut params = (text("name", "Alice"), number("age", 30));
        params.update_index(1, Value::I32(40));
        let extracted = params.extract();
        assert_eq!(extracted.0, "Alice");
        assert_eq!(extracted.1, 40);
    }

    #[test]
    fn tuple_3_update_middle() {
        let mut params = (
            text("name", "Alice"),
            number("age", 30),
            boolean("active", true),
        );
        params.update_index(1, Value::I32(35));
        let extracted = params.extract();
        assert_eq!(extracted.0, "Alice");
        assert_eq!(extracted.1, 35);
        assert_eq!(extracted.2, true);
    }

    #[test]
    fn tuple_4_extract() {
        let params = (
            text("name", "Alice"),
            number("age", 30),
            boolean("active", true),
            text("city", "NYC"),
        );
        let extracted = params.extract();
        assert_eq!(
            extracted,
            (String::from("Alice"), 30, true, String::from("NYC"))
        );
    }
}
