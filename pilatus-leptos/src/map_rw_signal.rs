//! Copy from https://github.com/mskorkowski/leptos-forge/blob/main/libs/utils_leptos/src/rust/main/signal.rs with some modifications

use std::panic::Location;

use leptos::attr::Attribute;
use leptos::attr::AttributeValue;
use leptos::attr::any_attribute::AnyAttribute;
use leptos::prelude::*;
use leptos::tachys::html::property::IntoProperty;
use leptos::tachys::hydration::Cursor;
use leptos::tachys::reactive_graph::RenderEffectState;
use leptos::tachys::reactive_graph::bind::IntoSplitSignal;
use leptos::tachys::renderer::types::Element;
use leptos::tachys::ssr::StreamBuilder;
use leptos::tachys::view::Mountable;
use leptos::tachys::view::Position;
use leptos::tachys::view::PositionState;
use leptos::tachys::view::Render;
use leptos::tachys::view::RenderHtml;
use leptos::tachys::view::add_attr::AddAnyAttr;

use serde::de::DeserializeOwned;
use serde::{Serialize, Serializer};

use crate::LeafRwSignal;
use crate::PilatusPrimitiveValue;

/// Signal which allows reading and writing a value
///
/// You should never, ever create an RwSignal which reads from one value and writes to the other. I consider
/// you to be warned.
#[derive(Debug)]
pub struct MapRwSignal<T>
where
    T: Send + Sync + 'static,
{
    /// Location where the MapRwSignal was created
    defined_at: &'static Location<'static>,
    /// signal from which we can read the value
    read_signal: Signal<T>,
    /// signal from where we can write the value to
    write_signal: SignalSetter<T>,
}

impl<T> Clone for MapRwSignal<T>
where
    T: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for MapRwSignal<T> where T: Send + Sync + 'static {}

impl<T> MapRwSignal<T>
where
    T: Send + Sync + 'static,
{
    /// Creates a new RwSignal with initial value `initial`
    #[track_caller]
    pub fn new(initial: T) -> Self {
        let (read, write) = signal(initial);
        Self {
            defined_at: Location::caller(),
            read_signal: read.into(),
            write_signal: write.into(),
        }
    }
}

impl<T> MapRwSignal<T>
where
    T: Send + Sync + 'static + Clone,
{
    /// Returns a read only part of the RwSignal
    pub fn read_only(&self) -> Signal<T> {
        self.read_signal
    }

    /// Returns a write only part of the RwSignal
    pub fn write_only(&self) -> SignalSetter<T> {
        self.write_signal
    }

    /// Transforms signal of T into signal of A
    ///
    /// Allows consistent read/write with the derived MapRwSignal
    #[track_caller]
    pub fn map<A>(
        &self,
        towards: impl Fn(&T) -> A + Send + Sync + 'static,
        from: impl Fn(&mut T, A) + Send + Sync + 'static,
    ) -> MapRwSignal<A>
    where
        A: Send + Sync + 'static + PartialEq,
    {
        self.map_internal(from, move |read| {
            Memo::new(move |_| read.with(|t| towards(t))).into()
        })
    }

    #[track_caller]
    pub fn map_leaf<A>(
        &self,
        towards: impl Fn(&T) -> PilatusPrimitiveValue<A> + Send + Sync + 'static,
        from: impl Fn(&mut T, PilatusPrimitiveValue<A>) + Send + Sync + 'static,
    ) -> LeafRwSignal<A>
    where
        A: DeserializeOwned + Send + Sync + 'static + PartialEq + Clone,
    {
        let read = self.read_signal;
        let new_read: Signal<PilatusPrimitiveValue<A>> =
            Memo::new(move |_| read.with(|t| towards(t))).into();
        let write: SignalSetter<T> = self.write_signal;
        let new_write = SignalSetter::map(move |value: PilatusPrimitiveValue<A>| {
            let mut t = read.get_untracked();
            from(&mut t, value);
            write.set(t);
        });

        LeafRwSignal::new_with_signals(new_read, new_write)
    }

    #[track_caller]
    pub fn map_uncached<A>(
        &self,
        towards: impl Fn(&T) -> A + Send + Sync + 'static,
        from: impl Fn(&mut T, A) + Send + Sync + 'static,
    ) -> MapRwSignal<A>
    where
        A: Send + Sync + 'static,
    {
        self.map_internal(from, move |read| {
            Signal::derive(move || read.with(|t| towards(t)))
        })
    }

    #[track_caller]
    fn map_internal<A>(
        &self,
        from: impl Fn(&mut T, A) + Send + Sync + 'static,
        create_signal: impl FnOnce(Signal<T>) -> Signal<A> + Send + Sync + 'static,
    ) -> MapRwSignal<A>
    where
        A: Send + Sync + 'static,
    {
        // let read: Signal<T> = self.read_signal;
        // let memo = Memo::new(move |_| read.with(|t| towards(t)));
        // memo.into();
        let read = self.read_signal;
        let new_read: Signal<A> = create_signal(read);

        let write: SignalSetter<T> = self.write_signal;
        let new_write = SignalSetter::map(move |a: A| {
            let mut t = read.get_untracked();
            from(&mut t, a);
            write.set(t);
        });

        MapRwSignal {
            defined_at: Location::caller(),
            read_signal: new_read,
            write_signal: new_write,
        }
    }
}

impl<T> Dispose for MapRwSignal<T>
where
    T: Send + Sync + 'static,
{
    fn dispose(self) {
        self.read_signal.dispose();
    }
}

impl<T> DefinedAt for MapRwSignal<T>
where
    T: Send + Sync + 'static,
{
    fn defined_at(&self) -> Option<&'static Location<'static>> {
        Some(self.defined_at)
    }
}

impl<T> PartialEq for MapRwSignal<T>
where
    T: Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.read_signal == other.read_signal
    }
}

impl<T> Eq for MapRwSignal<T> where T: Send + Sync + 'static {}

impl<T> ReadUntracked for MapRwSignal<T>
where
    T: Send + Sync + 'static + Clone,
{
    type Value = <Signal<T> as ReadUntracked>::Value;

    fn try_read_untracked(&self) -> Option<Self::Value> {
        self.read_signal.try_read_untracked()
    }
}

impl<T> Set for MapRwSignal<T>
where
    T: Send + Sync + 'static,
{
    type Value = T;

    fn set(&self, new_value: Self::Value) {
        self.write_signal.set(new_value);
    }

    fn try_set(&self, value: Self::Value) -> Option<Self::Value> {
        self.write_signal.try_set(value)
    }
}

impl<T> Update for MapRwSignal<T>
where
    T: Send + Sync + 'static + Clone,
{
    type Value = T;

    fn try_maybe_update<U>(&self, fun: impl FnOnce(&mut Self::Value) -> (bool, U)) -> Option<U> {
        let mut current = self.read_signal.get_untracked();
        let (should_update, ret) = fun(&mut current);
        if should_update {
            self.write_signal.set(current);
        }
        Some(ret)
    }
}

impl<T> Get for MapRwSignal<T>
where
    T: Send + Sync + 'static + Clone,
{
    type Value = T;

    fn try_get(&self) -> Option<Self::Value> {
        self.read_signal.try_get()
    }
}

impl<T> IntoProperty for MapRwSignal<T>
where
    T: 'static + IntoProperty + Clone + Send + Sync,
    <T as IntoProperty>::State: 'static,
    MapRwSignal<T>: Get<Value = T> + Clone,
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

impl<T> From<T> for MapRwSignal<T>
where
    T: Send + Sync + 'static,
{
    #[track_caller]
    fn from(value: T) -> Self {
        MapRwSignal::new(value)
    }
}

impl<T> Render for MapRwSignal<T>
where
    T: Send + Sync + 'static + Clone + Render,
    <T as Render>::State: 'static,
    MapRwSignal<T>: Get<Value = T>,
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

impl<T> RenderHtml for MapRwSignal<T>
where
    T: Send + Sync + 'static + Clone + RenderHtml,
    <T as Render>::State: 'static,
    MapRwSignal<T>: Get<Value = T>,
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

impl<T> AddAnyAttr for MapRwSignal<T>
where
    T: Send + Sync + 'static + Clone + RenderHtml,
    <T as Render>::State: 'static,
    MapRwSignal<T>: Get<Value = T>,
{
    type Output<SomeNewAttr: Attribute> = Self;

    fn add_any_attr<NewAttr: Attribute>(self, _attr: NewAttr) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: RenderHtml,
    {
        todo!()
    }
}

impl<T> AttributeValue for MapRwSignal<T>
where
    T: Send + Sync + 'static + Clone + AttributeValue,
    <T as AttributeValue>::State: 'static,
    MapRwSignal<T>: Get<Value = T>,
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

impl<T> Serialize for MapRwSignal<T>
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

impl<T> From<MapRwSignal<T>> for Signal<T>
where
    T: Send + Sync + 'static,
{
    #[track_caller]
    fn from(val: MapRwSignal<T>) -> Self {
        val.read_signal
    }
}
impl<T> IntoSplitSignal for MapRwSignal<T>
where
    T: Send + Sync + 'static + Clone,
{
    type Value = T;
    type Read = Signal<T>;
    type Write = SignalSetter<T>;

    fn into_split_signal(self) -> (Self::Read, Self::Write) {
        (self.read_signal, self.write_signal)
    }
}
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

impl<T> From<MapRwSignal<T>> for SignalSetter<T>
where
    T: Send + Sync + 'static,
{
    fn from(value: MapRwSignal<T>) -> Self {
        value.write_signal
    }
}

impl<T: Send + Sync + 'static + Default> Default for MapRwSignal<T> {
    fn default() -> Self {
        MapRwSignal::new(T::default())
    }
}

// Thaw Model support
impl<T: Send + Sync + 'static + Clone> From<MapRwSignal<T>> for thaw_utils::Model<T> {
    fn from(value: MapRwSignal<T>) -> Self {
        (value.read_signal, value.write_signal).into()
    }
}
