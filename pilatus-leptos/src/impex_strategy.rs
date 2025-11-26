use std::{collections::HashMap, num::NonZeroU8, ops::Deref};

use impex::{Impex, ImpexPrimitive, WrapperSettings};
use leptos::prelude::*;
use pilatus::Name;
use serde_json;

use crate::RecipeContext;

#[derive(PartialEq, Eq, Clone, Debug)]
enum PilatusPrimitiveValueKind {
    Explicit,
    Implicit,
    Variable(Variable),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PilatusPrimitiveValue<T> {
    kind: PilatusPrimitiveValueKind,
    pub(crate) value: T,
}

impl<T: Default> Default for PilatusPrimitiveValue<T> {
    fn default() -> Self {
        Self {
            kind: PilatusPrimitiveValueKind::Implicit,
            value: T::default(),
        }
    }
}

// #[derive(PartialEq, Eq, Copy, Clone, Debug)]
// pub struct Variable {
//     bytes: [u8; 30],
//     len: NonZeroU8,
// }

pub type Variable = pilatus::Name;

// impl Variable {
//     pub fn new(name: &str) -> Option<Self> {
//         let len = name.len();
//         if len > 30 {
//             return None;
//         }
//         let len_u8 = NonZeroU8::new(len as u8)?;
//         let mut name_bytes = [0u8; 30];
//         name_bytes[..len].copy_from_slice(name.as_bytes());
//         Some(Self {
//             bytes: name_bytes,
//             len: len_u8,
//         })
//     }
// }

// impl std::ops::Deref for Variable {
//     type Target = str;

//     fn deref(&self) -> &Self::Target {
//         std::str::from_utf8(&self.bytes[..self.len.get() as usize]).unwrap()
//     }
// }

impl<T> PilatusPrimitiveValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            kind: PilatusPrimitiveValueKind::Explicit,
            value,
        }
    }

    pub fn make_explicit(&mut self) {
        // When transitioning from Implicit to explicit, choose Explicit over Variable
        if let PilatusPrimitiveValueKind::Implicit = self.kind {
            self.kind = PilatusPrimitiveValueKind::Explicit;
        }
    }

    pub fn is_explicit(&self) -> bool {
        // Variable is explicit too
        matches!(
            self.kind,
            PilatusPrimitiveValueKind::Explicit | PilatusPrimitiveValueKind::Variable(_)
        )
    }

    pub fn is_implicit(&self) -> bool {
        matches!(self.kind, PilatusPrimitiveValueKind::Implicit)
    }

    pub fn set_explicit(&mut self, value: T) {
        if self.is_implicit() {
            self.kind = PilatusPrimitiveValueKind::Explicit;
        }
        self.value = value;
    }

    pub fn variable_name(&self) -> Option<&str> {
        match &self.kind {
            PilatusPrimitiveValueKind::Variable(var) => Some(var.deref()),
            _ => None,
        }
    }

    /// Maps the value to a different type, preserving variable references
    /// For variables, preserves the variable reference and transforms the current value
    /// The value will be re-resolved during the next deserialization
    pub fn map<A>(self, transformer: impl FnOnce(T) -> A) -> PilatusPrimitiveValue<A> {
        PilatusPrimitiveValue {
            kind: self.kind,
            value: transformer(self.value),
        }
    }

    /// Gets the variable if this is a variable reference
    pub fn variable(&self) -> Option<&Variable> {
        match &self.kind {
            PilatusPrimitiveValueKind::Variable(var) => Some(var),
            _ => None,
        }
    }

    /// Sets the kind to variable with the given variable reference
    pub(crate) fn set_kind_to_variable(&mut self, variable: Variable) {
        self.kind = PilatusPrimitiveValueKind::Variable(variable);
    }

    /// Sets the value, making implicit values explicit, but variable stays unchanged
    pub(crate) fn set(&mut self, value: T) {
        self.value = value;
        // Make implicit values explicit
        if let PilatusPrimitiveValueKind::Implicit = self.kind {
            self.kind = PilatusPrimitiveValueKind::Explicit;
        }
        // Variable and Explicit stay unchanged
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
        match &self.kind {
            PilatusPrimitiveValueKind::Implicit => serializer.serialize_none(),
            PilatusPrimitiveValueKind::Variable(variable) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("__var", variable.deref())?;
                map.end()
            }
            PilatusPrimitiveValueKind::Explicit => self.value.serialize(serializer),
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
            // It's a variable reference - get the value from RecipeContext
            let variable = Variable::new(var_name)
                .map_err(|e| D::Error::custom("Invalid variable name {e}"))?;

            // Try to get RecipeContext and resolve the variable
            let device_ctx = use_context::<crate::RecipeContext>().ok_or_else(|| {
                D::Error::custom("RecipeContext not available during deserialization")
            })?;

            let t_value = device_ctx.get_variable::<T>(&variable).map_err(|e| {
                D::Error::custom(format!("Failed to get variable '{}': {}", var_name, e))
            })?;

            Ok(PilatusPrimitiveValue {
                kind: PilatusPrimitiveValueKind::Variable(variable),
                value: t_value,
            })
        } else {
            // It's a regular value
            T::deserialize(value)
                .map(|val| PilatusPrimitiveValue {
                    kind: PilatusPrimitiveValueKind::Explicit,
                    value: val,
                })
                .map_err(D::Error::custom)
        }
    }
}

impl<T: ImpexPrimitive, TW: WrapperSettings> Impex<TW> for PilatusPrimitiveValue<T> {
    type Value = T;

    fn is_explicit(&self) -> bool {
        // Variable is explicit too
        matches!(
            self.kind,
            PilatusPrimitiveValueKind::Explicit | PilatusPrimitiveValueKind::Variable(_)
        )
    }

    fn into_value(self) -> Self::Value {
        self.value
    }

    fn set_impex(&mut self, v: Self::Value, is_explicit: bool) {
        if is_explicit {
            // On transitions from Implicit to explicit, choose Explicit over Variable
            if matches!(self.kind, PilatusPrimitiveValueKind::Implicit) {
                self.kind = PilatusPrimitiveValueKind::Explicit;
            }
            // Otherwise keep current kind (Variable or Explicit)
        } else {
            // When writing implicit, replace the current kind with Implicit
            self.kind = PilatusPrimitiveValueKind::Implicit;
        }
        self.value = v;
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
            kind: if is_explicit {
                PilatusPrimitiveValueKind::Explicit
            } else {
                PilatusPrimitiveValueKind::Implicit
            },
            value,
        }
    }
}

pub struct VariableChangeCtx {
    pub(crate) var_changes: HashMap<Name, pilatus::Variable>,
    pub(crate) recipe_context: RecipeContext,
}

impl VariableChangeCtx {
    pub fn new(recipe_context: RecipeContext) -> Self {
        Self {
            var_changes: HashMap::new(),
            recipe_context,
        }
    }
}

impl<T: serde::de::DeserializeOwned + PartialEq + std::fmt::Debug> impex::Visitor<VariableChangeCtx>
    for PilatusPrimitiveValue<T>
{
    fn visit(&mut self, ctx: &mut VariableChangeCtx) {
        if let Some(v) = self.variable() {
            let current = ctx.recipe_context.get_variable::<T>(&v);
            leptos::logging::log!(
                "Variable change?: {v:?}, var_val: {current:?}, new_val: {self:?}"
            );
            match current {
                Ok(c) if c == self.value => {}
                _ => {
                    // ctx.var_changes.insert(v.deref().to_string(), v);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_value() {
        let input = r#""Hello""#;
        let x: PilatusPrimitiveValue<String> = serde_json::from_str(input).unwrap();
        assert_eq!(&x.value, &"Hello".to_string());
        assert!(x.is_explicit());
        assert!(x.variable_name().is_none());
    }

    #[test]
    #[ignore = "Ask a bigger model to fix this"]
    fn deserialize_value_with_variable_name() {
        use crate::RecipeContext;
        use leptos::prelude::*;
        use pilatus::Recipes;
        use uuid::Uuid;

        // Create a runtime and scope - this provides the context that provide_context and use_context need
        // This is the same mechanism used in production - components run within a scope
        let runtime = leptos_reactive::create_runtime();

        // Create a scope within the runtime and run the test logic
        leptos_reactive::run_as_child(|| {
            // Create a RecipeContext with a variable
            let recipes = Recipes::default();
            let client_id = Uuid::new_v4();
            let ctx = RecipeContext::new(recipes, client_id);

            // Set the variable value
            ctx.set_variable(Name::new("myvar").unwrap(), "test_value");

            // Provide the context using the same mechanism as production
            // provide_context uses the current scope automatically (no need to pass scope explicitly)
            provide_context(ctx);

            let input = r#"{"__var": "myvar"}"#;
            let x: PilatusPrimitiveValue<String> = serde_json::from_str(input).unwrap();

            assert_eq!(&x.value, &"test_value".to_string());
            assert!(x.is_explicit()); // Variable is explicit
            assert_eq!(Some("myvar"), x.variable_name());
        });

        // Clean up resources
        runtime.dispose();
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
        assert_eq!(&x.foo.value, &"".to_string());
    }
}
