//! Copy from https://github.com/mskorkowski/leptos-forge/blob/main/libs/utils_leptos/src/rust/main/signal.rs with some modifications

use std::panic::Location;

use leptos::attr::Attribute;
use leptos::attr::AttributeValue;
use leptos::attr::any_attribute::AnyAttribute;
use leptos::prelude::*;
use leptos::tachys::html::property::IntoProperty;
use leptos::tachys::hydration::Cursor;
use leptos::tachys::reactive_graph::RenderEffectState;
use leptos::tachys::renderer::types::Element;
use leptos::tachys::ssr::StreamBuilder;
use leptos::tachys::view::Mountable;
use leptos::tachys::view::Position;
use leptos::tachys::view::PositionState;
use leptos::tachys::view::Render;
use leptos::tachys::view::RenderHtml;
use leptos::tachys::view::add_attr::AddAnyAttr;

use serde::{Serialize, Serializer};

use crate::PilatusPrimitiveValue;
use crate::ValueKind;

/// Signal which allows reading and writing a value
///
/// You should never, ever create an RwSignal which reads from one value and writes to the other. I consider
/// you to be warned.
#[derive(Debug)]
pub struct LeafRwSignal<T>
where
    T: Send + Sync + 'static,
{
    /// Location where the MapRwSignal was created
    pub(crate) defined_at: &'static Location<'static>,
    /// signal from which we can read the value
    pub(crate) read_signal: Signal<PilatusPrimitiveValue<T>>,
    /// signal from where we can write the value to
    pub(crate) write_signal: SignalSetter<PilatusPrimitiveValue<T>>,
}

impl<T> Clone for LeafRwSignal<T>
where
    T: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> LeafRwSignal<T>
where
    T: Send + Sync + 'static,
{
    /// Create a new LeafRwSignal from raw signals
    /// The write_signal should accept `PilatusPrimitiveValue<T>`
    #[track_caller]
    pub(crate) fn new_with_signals(
        read_signal: Signal<PilatusPrimitiveValue<T>>,
        write_signal: SignalSetter<PilatusPrimitiveValue<T>>,
    ) -> Self {
        Self {
            defined_at: std::panic::Location::caller(),
            read_signal,
            write_signal,
        }
    }
}

impl<T> Copy for LeafRwSignal<T> where T: Send + Sync + 'static {}

impl<T> LeafRwSignal<T>
where
    T: Send + Sync + 'static,
{
    /// Creates a new RwSignal with initial value `initial`
    #[track_caller]
    pub fn new(initial: T) -> Self {
        let (read, write) = signal(PilatusPrimitiveValue::new(initial));
        Self {
            defined_at: Location::caller(),
            read_signal: read.into(),
            write_signal: write.into(),
        }
    }
}

impl<T> LeafRwSignal<T>
where
    T: Send + Sync + 'static + Clone,
{
    /// Returns a read only part of the RwSignal
    pub fn read_only(&self) -> Signal<PilatusPrimitiveValue<T>> {
        self.read_signal
    }

    /// Returns a write only part of the RwSignal
    pub fn write_only(&self) -> SignalSetter<PilatusPrimitiveValue<T>> {
        self.write_signal
    }

    /// Gets the actual value (T), resolving variables from DeviceContext if needed
    pub fn get_value(&self) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        self.read_signal.with(|prim_val| match &prim_val.value {
            ValueKind::Value(v) => v.clone(),
            ValueKind::Variable(var) => {
                let device_ctx = expect_context::<crate::RecipeContext>();
                device_ctx.get_variable::<T>(&**var)
            }
        })
    }

    /// Gets the actual value (T) without tracking, resolving variables from DeviceContext if needed
    pub fn get_value_untracked(&self) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        self.read_signal.with_untracked(|prim_val| {
            match &prim_val.value {
                ValueKind::Value(v) => v.clone(),
                ValueKind::Variable(var) => {
                    // Try to get DeviceContext and resolve the variable
                    let device_ctx = expect_context::<crate::RecipeContext>();
                    device_ctx.get_variable::<T>(&**var)
                }
            }
        })
    }

    /// Sets the value
    /// - If currently a variable reference, writes to that variable in DeviceContext
    /// - If currently a local value, updates the local value
    pub fn set_value(&self, value: T)
    where
        T: serde::Serialize,
    {
        let current_prim = self.read_signal.get_untracked();

        match &current_prim.value {
            ValueKind::Variable(var) => {
                // Write to the variable in DeviceContext
                if let Some(device_ctx) = use_context::<crate::RecipeContext>() {
                    let var_name = var.to_string();
                    device_ctx.set_variable(&var_name, value);
                    leptos::logging::log!("Wrote value to variable '{}'", var_name);
                } else {
                    leptos::logging::error!("Cannot write to variable: DeviceContext not found");
                }
            }
            ValueKind::Value(_) => {
                // Update local value
                let prim_val = PilatusPrimitiveValue::new(value);
                self.write_signal.set(prim_val);
            }
        }
    }

    /// Checks if the current value is a variable reference
    pub fn is_variable(&self) -> bool {
        self.read_signal
            .with(|prim_val| matches!(prim_val.value, ValueKind::Variable(_)))
    }

    /// Checks if the current value is a variable reference without tracking
    pub fn is_variable_untracked(&self) -> bool {
        self.read_signal
            .with_untracked(|prim_val| matches!(prim_val.value, ValueKind::Variable(_)))
    }

    /// Gets the variable name if this is a variable reference
    pub fn get_variable_name(&self) -> Option<String> {
        self.read_signal
            .with(|prim_val| prim_val.variable_name().map(|s| s.to_string()))
    }

    /// Gets the variable name if this is a variable reference without tracking
    pub fn get_variable_name_untracked(&self) -> Option<String> {
        self.read_signal
            .with_untracked(|prim_val| prim_val.variable_name().map(|s| s.to_string()))
    }

    /// Converts the current value to a variable reference
    pub fn convert_to_variable(&self, variable_name: &str) -> Result<(), String> {
        let var_kind = ValueKind::new_variable(variable_name)
            .ok_or_else(|| "Invalid variable name".to_string())?;

        let prim_val = PilatusPrimitiveValue {
            is_explicit: true,
            value: var_kind,
        };
        self.write_signal.set(prim_val);

        Ok(())
    }

    /// Converts a variable reference to a local value
    /// The value parameter is what to set as the new local value
    pub fn convert_to_local(&self, value: T)
    where
        T: serde::Serialize,
    {
        self.set_value(value);
    }
}

impl<T> Dispose for LeafRwSignal<T>
where
    T: Send + Sync + 'static,
{
    fn dispose(self) {
        self.read_signal.dispose();
    }
}

impl<T> DefinedAt for LeafRwSignal<T>
where
    T: Send + Sync + 'static,
{
    fn defined_at(&self) -> Option<&'static Location<'static>> {
        Some(self.defined_at)
    }
}

impl<T> PartialEq for LeafRwSignal<T>
where
    T: Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.read_signal == other.read_signal
    }
}

impl<T> Eq for LeafRwSignal<T> where T: Send + Sync + 'static {}

impl<T> ReadUntracked for LeafRwSignal<T>
where
    T: Send + Sync + 'static + Clone + serde::de::DeserializeOwned,
{
    type Value = <Signal<PilatusPrimitiveValue<T>> as ReadUntracked>::Value;

    fn try_read_untracked(&self) -> Option<Self::Value> {
        self.read_signal.try_read_untracked()
    }
}

impl<T> Set for LeafRwSignal<T>
where
    T: Send + Sync + 'static + Clone + serde::Serialize,
{
    type Value = T;

    fn set(&self, new_value: Self::Value) {
        self.set_value(new_value);
    }

    fn try_set(&self, value: Self::Value) -> Option<Self::Value> {
        self.set_value(value.clone());
        None // Value was successfully set
    }
}

impl<T> Get for LeafRwSignal<T>
where
    T: Send + Sync + 'static + Clone + serde::de::DeserializeOwned,
{
    type Value = T;

    fn try_get(&self) -> Option<Self::Value> {
        Some(self.get_value())
    }
}

impl<T> IntoProperty for LeafRwSignal<T>
where
    T: 'static + IntoProperty + Clone + Send + Sync,
    <T as IntoProperty>::State: 'static,
    LeafRwSignal<T>: Get<Value = T> + Clone,
{
    type State = RenderEffect<<T as IntoProperty>::State>;
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn build(self, el: &Element, key: &str) -> Self::State {
        (move || self.get()).build(el, key)
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &Element, key: &str) -> Self::State {
        (move || self.get()).hydrate::<FROM_SERVER>(el, key)
    }

    fn rebuild(self, state: &mut Self::State, key: &str) {
        (move || self.get()).rebuild(state, key)
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self
    }
}

impl<T> From<T> for LeafRwSignal<T>
where
    T: Send + Sync + 'static,
{
    #[track_caller]
    fn from(value: T) -> Self {
        LeafRwSignal::new(value)
    }
}

impl<T> Render for LeafRwSignal<T>
where
    T: Send + Sync + 'static + Clone + Render,
    <T as Render>::State: 'static,
    LeafRwSignal<T>: Get<Value = T>,
{
    type State = RenderEffectState<<T as Render>::State>;

    fn build(self) -> Self::State {
        (move || self.get()).build()
    }

    fn rebuild(self, state: &mut Self::State) {
        let new = self.build();
        let mut old = std::mem::replace(state, new);
        old.insert_before_this(state);
        old.unmount();
    }
}

impl<T> RenderHtml for LeafRwSignal<T>
where
    T: Send + Sync + 'static + Clone + RenderHtml,
    <T as Render>::State: 'static,
    LeafRwSignal<T>: Get<Value = T>,
{
    type AsyncOutput = Self;
    type Owned = Self;

    const MIN_LENGTH: usize = 0;

    fn dry_resolve(&mut self) {}

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }

    fn html_len(&self) -> usize {
        T::MIN_LENGTH
    }

    fn to_html_with_buf(
        self,
        buf: &mut String,
        position: &mut Position,
        escape: bool,
        mark_branches: bool,
        extra_attrs: Vec<AnyAttribute>,
    ) {
        let value = self.get();
        value.to_html_with_buf(buf, position, escape, mark_branches, extra_attrs)
    }

    fn to_html_async_with_buf<const OUT_OF_ORDER: bool>(
        self,
        buf: &mut StreamBuilder,
        position: &mut Position,
        escape: bool,
        mark_branches: bool,
        extra_attrs: Vec<AnyAttribute>,
    ) where
        Self: Sized,
    {
        let value = self.get();
        value.to_html_async_with_buf::<OUT_OF_ORDER>(
            buf,
            position,
            escape,
            mark_branches,
            extra_attrs,
        );
    }

    fn hydrate<const FROM_SERVER: bool>(
        self,
        cursor: &Cursor,
        position: &PositionState,
    ) -> Self::State {
        (move || self.get()).hydrate::<FROM_SERVER>(cursor, position)
    }

    fn into_owned(self) -> Self::Owned {
        self
    }
}

impl<T> AddAnyAttr for LeafRwSignal<T>
where
    T: Send + Sync + 'static + Clone + RenderHtml,
    <T as Render>::State: 'static,
    LeafRwSignal<T>: Get<Value = T>,
{
    type Output<SomeNewAttr: Attribute> = Self;

    fn add_any_attr<NewAttr: Attribute>(self, _attr: NewAttr) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: RenderHtml,
    {
        todo!()
    }
}

impl<T> AttributeValue for LeafRwSignal<T>
where
    T: Send + Sync + 'static + Clone + AttributeValue,
    <T as AttributeValue>::State: 'static,
    LeafRwSignal<T>: Get<Value = T>,
{
    type AsyncOutput = Self;
    type State = RenderEffect<<T as AttributeValue>::State>;
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn html_len(&self) -> usize {
        0
    }

    fn to_html(self, key: &str, buf: &mut String) {
        let value = self.get();
        value.to_html(key, buf);
    }

    fn to_template(_key: &str, _buf: &mut String) {}

    fn hydrate<const FROM_SERVER: bool>(self, key: &str, el: &Element) -> Self::State {
        (move || self.get()).hydrate::<FROM_SERVER>(key, el)
    }

    fn build(self, el: &Element, key: &str) -> Self::State {
        (move || self.get()).build(el, key)
    }

    fn rebuild(self, key: &str, state: &mut Self::State) {
        (move || self.get()).rebuild(key, state)
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self
    }

    fn dry_resolve(&mut self) {}

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }
}

impl<T> Serialize for LeafRwSignal<T>
where
    T: Send + Sync + 'static + Serialize,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        self.read_signal.serialize(serializer)
    }
}

// Note: LeafRwSignal cannot be directly converted to Signal<T> or split into Signal<T>
// because it internally holds Signal<PilatusPrimitiveValue<T>>, not Signal<T>.
// If needed, use .get_value() or create a derived signal.
/* Skipped, we don't use stores
impl<Inner, Prev, T> From<Subfield<Inner, Prev, T>> for MapRwSignal<T>
where
    Inner: StoreField<Value = Prev> + Track + Send + Sync + 'static + Clone,
    Prev: 'static,
    T: Send + Sync + Clone + 'static,
{
    #[track_caller]
    fn from(value: Subfield<Inner, Prev, T>) -> Self {
        let r: Signal<T> = value.clone().into();
        let w: SignalSetter<T> = SignalSetter::map(move |t| {
            value.update(|v| {
                *v = t;
            });
        });

        MapRwSignal {
            defined_at: Location::caller(),
            read_signal: r,
            write_signal: w,
        }
    }
}
*/

// Note: LeafRwSignal cannot be directly converted to SignalSetter<T>
// because it internally holds SignalSetter<PilatusPrimitiveValue<T>>, not SignalSetter<T>.
// If needed, use .write_only() to get SignalSetter<PilatusPrimitiveValue<T>> or .set_value()

impl<T: Send + Sync + 'static + Default> Default for LeafRwSignal<T> {
    fn default() -> Self {
        LeafRwSignal::new(T::default())
    }
}

// Thaw Model support
impl<T> From<LeafRwSignal<T>> for thaw_utils::Model<T>
where
    T: Send + Sync + 'static + Clone + PartialEq + serde::de::DeserializeOwned + serde::Serialize,
{
    fn from(leaf: LeafRwSignal<T>) -> Self {
        // Create a derived signal that extracts T from PilatusPrimitiveValue<T>
        let read_signal = Signal::derive(move || leaf.get_value());

        // Create a SignalSetter that writes back to the leaf
        let write_signal = SignalSetter::map(move |value: T| {
            leaf.set_value(value);
        });

        (read_signal, write_signal).into()
    }
}
