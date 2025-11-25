use std::{num::NonZeroU8, ops::Deref};

use impex::{Impex, ImpexPrimitive, WrapperSettings};
use leptos::prelude::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct PilatusPrimitiveValue<T> {
    pub is_explicit: bool,
    pub(crate) value: ValueKind<T>,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub(crate) enum ValueKind<T> {
    Variable(Variable),
    Value(T),
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub(crate) struct Variable {
    bytes: [u8; 30],
    len: NonZeroU8,
}

impl Variable {
    pub fn new(name: &str) -> Option<Self> {
        let len = name.len();
        if len > 30 {
            return None;
        }
        let len_u8 = NonZeroU8::new(len as u8)?;
        let mut name_bytes = [0u8; 30];
        name_bytes[..len].copy_from_slice(name.as_bytes());
        Some(Self {
            bytes: name_bytes,
            len: len_u8,
        })
    }
}

impl<T: Default> Default for ValueKind<T> {
    fn default() -> Self {
        Self::Value(T::default())
    }
}

impl<T> ValueKind<T> {
    pub fn new_variable(name: &str) -> Option<Self> {
        Some(Self::Variable(Variable::new(name)?))
    }
}

impl std::ops::Deref for Variable {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        std::str::from_utf8(&self.bytes[..self.len.get() as usize]).unwrap()
    }
}

impl<T> PilatusPrimitiveValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            is_explicit: true,
            value: ValueKind::Value(value),
        }
    }

    pub fn make_explicit(&mut self) {
        self.is_explicit = true;
    }
    pub fn is_explicit(&self) -> bool {
        self.is_explicit
    }

    pub fn is_implicit(&self) -> bool {
        !self.is_explicit
    }

    pub fn set_explicit(&mut self, value: T) {
        self.is_explicit = true;
        self.value = ValueKind::Value(value);
    }

    pub fn variable_name(&self) -> Option<&str> {
        match &self.value {
            ValueKind::Variable(variable) => Some(variable),
            ValueKind::Value(_) => None,
        }
    }

    /// Maps the value to a different type, preserving variable references
    /// The code assumes, that the A can always be read from Variable, whereas the opposite might not be the case
    pub fn with_mapped_value<A>(self, transformer: impl FnOnce(T) -> A) -> PilatusPrimitiveValue<A>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
        A: serde::Serialize + serde::de::DeserializeOwned,
    {
        match self.value {
            ValueKind::Variable(variable) => {
                #[cfg(debug_assertions)]
                {
                    let context = expect_context::<crate::RecipeContext>();
                    context.expect_variable::<A>(&variable);
                }
                PilatusPrimitiveValue {
                    is_explicit: self.is_explicit,
                    value: ValueKind::Variable(variable),
                }
            }
            ValueKind::Value(value) => PilatusPrimitiveValue {
                is_explicit: self.is_explicit,
                value: ValueKind::Value(transformer(value)),
            },
        }
    }

    /// Extracts the actual value, or None if it's a variable reference
    pub fn try_get_nonvar_value(&self) -> Option<T>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        match &self.value {
            ValueKind::Value(v) => Some(v.clone()),
            ValueKind::Variable(_) => None,
        }
    }

    /// Gets the actual value, resolving variables from DeviceContext if needed
    pub fn get_value(&self) -> T
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        match &self.value {
            ValueKind::Value(v) => v.clone(),
            ValueKind::Variable(var) => {
                let device_ctx = expect_context::<crate::RecipeContext>();
                device_ctx.expect_variable::<T>(var)
            }
        }
    }
}

// impl<T: Default> std::ops::Deref for PilatusPrimitiveValue<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         match &self.value {
//             ValueKind::Value(v) => v,
//             ValueKind::Variable(_, _) => {
//                 panic!("Cannot dereference a variable reference without resolving it first")
//             }
//         }
//     }
// }

impl<T: serde::Serialize> serde::Serialize for PilatusPrimitiveValue<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if !self.is_explicit {
            return serializer.serialize_none();
        }

        match &self.value {
            ValueKind::Value(v) => v.serialize(serializer),
            ValueKind::Variable(variable) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("__var", variable.deref())?;
                map.end()
            }
        }
    }
}

impl<'de, T: serde::de::DeserializeOwned> serde::Deserialize<'de> for PilatusPrimitiveValue<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let value = serde_json::Value::deserialize(deserializer)?;

        if let serde_json::Value::Object(ref map) = value
            && let Some(serde_json::Value::String(var_name)) = map.get("__var")
        {
            let var_kind = ValueKind::new_variable(var_name)
                .ok_or_else(|| D::Error::custom("Invalid variable name"))?;

            Ok(PilatusPrimitiveValue {
                is_explicit: true,
                value: var_kind,
            })
        } else {
            T::deserialize(value)
                .map(|val| PilatusPrimitiveValue {
                    is_explicit: true,
                    value: ValueKind::Value(val),
                })
                .map_err(D::Error::custom)
        }
    }
}

impl<T: ImpexPrimitive, TW: WrapperSettings> Impex<TW> for PilatusPrimitiveValue<T> {
    type Value = T;

    fn is_explicit(&self) -> bool {
        self.is_explicit
    }

    fn into_value(self) -> Self::Value {
        match self.value {
            ValueKind::Value(v) => v,
            ValueKind::Variable(_) => {
                panic!(
                    "into_value: Cannot convert variable reference to value without resolving it first"
                )
            }
        }
    }
    fn set_impex(&mut self, v: Self::Value, is_explicit: bool) {
        self.is_explicit = is_explicit;
        self.value = ValueKind::Value(v);
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy)]
pub struct PilatusWrapperSettings;

impl<'de> serde::de::Deserialize<'de> for PilatusWrapperSettings {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self)
    }
}

impl serde::Serialize for PilatusWrapperSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_none()
    }
}

impl WrapperSettings for PilatusWrapperSettings {
    type PrimitiveWrapper<T: ImpexPrimitive> = PilatusPrimitiveValue<T>;

    fn create_primitive<T: ImpexPrimitive>(
        value: T,
        is_explicit: bool,
    ) -> Self::PrimitiveWrapper<T> {
        PilatusPrimitiveValue {
            is_explicit,
            value: ValueKind::Value(value),
        }
    }
}

impl<T> ::impex::Visitor<(NonZeroU8, [u8; 30])> for PilatusPrimitiveValue<T> {
    fn visit(&mut self, ctx: &mut (NonZeroU8, [u8; 30])) {
        self.value = ValueKind::Variable(Variable {
            len: ctx.0,
            bytes: ctx.1,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_value() {
        let input = r#""Hello""#;
        let x: PilatusPrimitiveValue<String> = serde_json::from_str(input).unwrap();
        assert_eq!(x.value, ValueKind::Value("Hello".to_string()));
        assert!(x.is_explicit());
        assert!(x.variable_name().is_none());
    }

    #[test]
    fn deserialize_value_with_variable_name() {
        let input = r#"{"__var": "myvar"}"#;
        let x: PilatusPrimitiveValue<String> = serde_json::from_str(input).unwrap();
        assert_eq!(x.value, ValueKind::new_variable("myvar").unwrap(),);
        assert!(x.is_explicit());
        assert_eq!(Some("myvar"), x.variable_name());
    }

    #[test]
    fn deserialize_value_with_invalid_variable_name() {
        #[derive(PartialEq, Clone, Default, Impex)]
        struct Wrapper {
            foo: String,
        }

        let input = r#"{}"#;
        let x: WrapperImpex<PilatusWrapperSettings> = serde_json::from_str(input).unwrap();

        assert!(x.foo.is_implicit());
        assert_eq!(x.foo.value, ValueKind::Value("".to_string()));
    }
}
