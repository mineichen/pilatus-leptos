use leptos::prelude::*;
use leptos::tachys::reactive_graph::bind::IntoSplitSignal;
use reactive_graph::wrappers::read::SignalTypes;
use serde::{Serialize, Serializer};
use std::panic::Location;

/// Inherits values from it's parent until it writes the data itself. At this point, it will become non-reactive.
/// It's value will remain the old value before the first change. This is useful, if you want to block external changes
/// once you start changing the value, but still propagate your changes.
#[derive(Debug)]
pub struct FrozenSignal<T, S = SyncStorage>
where
    T: 'static,
    S: Storage<T>,
{
    defined_at: &'static Location<'static>,
    write_signal: SignalSetter<T, S>,
    latch: StoredValue<Option<T>, S>, // None until first write → zero construction clone
    read_signal: Signal<T, S>,
}
impl<T, S> Clone for FrozenSignal<T, S>
where
    T: 'static,
    S: Storage<T>,
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, S> Copy for FrozenSignal<T, S>
where
    T: 'static,
    S: Storage<T>,
{
}

impl<T> FrozenSignal<T>
where
    T: Send + Sync + Clone + PartialEq + 'static,
{
    /// Creates a new frozen signal from any source that provides both a reactive read
    /// ([`Get<Value = T>`](Get)) and a write part ([`SignalSetter<T>`]).
    #[track_caller]
    pub fn new<S>(source: S) -> Self
    where
        S: Copy + Send + Sync + 'static + Get<Value = T> + Into<SignalSetter<T>>,
    {
        let write_signal: SignalSetter<T> = source.into();
        let latch = StoredValue::new(None);

        let (l_m, s_m) = (latch, source);
        let read_signal: Signal<T> =
            Memo::new(move |_| l_m.get_value().unwrap_or_else(|| s_m.get())).into();

        Self {
            defined_at: Location::caller(),
            write_signal,
            latch,
            read_signal,
        }
    }
}

impl<T> FrozenSignal<T, LocalStorage>
where
    T: Clone + PartialEq + 'static,
{
    /// Creates a new frozen signal over a local-storage source, for values that are not `Send + Sync`.
    #[track_caller]
    pub fn new_local<S>(source: S) -> Self
    where
        S: Copy + 'static + Get<Value = T> + Into<SignalSetter<T, LocalStorage>>,
    {
        let write_signal: SignalSetter<T, LocalStorage> = source.into();
        let latch = StoredValue::new_local(None);

        let (l_m, s_m) = (latch, source);
        let read_signal: Signal<T, LocalStorage> =
            Signal::derive_local(move || l_m.get_value().unwrap_or_else(|| s_m.get()));

        Self {
            defined_at: Location::caller(),
            write_signal,
            latch,
            read_signal,
        }
    }
}

impl<T, S> FrozenSignal<T, S>
where
    T: Clone + PartialEq + 'static,
    S: Storage<T>
        + Storage<SignalTypes<T, S>>
        + Storage<ArcStoredValue<Option<T>>>
        + Storage<ArcWriteSignal<T>>
        + Storage<Box<dyn Fn(T) + Send + Sync>>,
{
    pub fn write_only(&self) -> SignalSetter<T, S> {
        let this = *self;
        SignalSetter::map(move |value| this.set(value))
    }
    pub fn read_only(&self) -> Signal<T, S> {
        self.read_signal
    }

    pub fn set(&self, new_val: T) {
        let (write_signal, latch, read_signal) = (self.write_signal, self.latch, self.read_signal);
        if latch.read_value().is_some() {
            write_signal.set(new_val); // already frozen: just forward to parent
            return;
        }
        // First write: capture old value, latch, freeze, forward — atomically.
        // batch() defers the Memo's re-eval until frozen=true + latch=Some are visible.
        batch(move || {
            let old = read_signal.get_untracked();
            write_signal.set(new_val);
            latch.set_value(Some(old));
        });
    }

    pub fn frozen(&self) -> bool {
        self.latch.get_value().is_some()
    }
}

impl<T, S> Read for FrozenSignal<T, S>
where
    T: 'static,
    S: Storage<T> + Storage<SignalTypes<T, S>>,
{
    type Value = <Signal<T, S> as Read>::Value;

    fn try_read(&self) -> Option<Self::Value> {
        self.read_signal.try_read()
    }

    fn read(&self) -> Self::Value {
        self.read_signal.read()
    }
}

impl<T, S> ReadUntracked for FrozenSignal<T, S>
where
    T: 'static,
    S: Storage<T> + Storage<SignalTypes<T, S>>,
{
    type Value = <Signal<T, S> as ReadUntracked>::Value;

    fn try_read_untracked(&self) -> Option<Self::Value> {
        self.read_signal.try_read_untracked()
    }
}

impl<T, S> Set for FrozenSignal<T, S>
where
    SignalSetter<T, S>: Set<Value = T>,
    S: Storage<T>,
{
    type Value = T;

    fn set(&self, new_value: Self::Value) {
        self.write_signal.set(new_value)
    }

    fn try_set(&self, value: Self::Value) -> Option<Self::Value> {
        self.write_signal.try_set(value);
        None
    }
}

impl<T, S> Update for FrozenSignal<T, S>
where
    T: Clone + PartialEq + 'static,
    S: Storage<T>
        + Storage<SignalTypes<T, S>>
        + Storage<ArcStoredValue<Option<T>>>
        + Storage<ArcWriteSignal<T>>
        + Storage<Box<dyn Fn(T) + Send + Sync>>,
{
    type Value = T;

    fn try_maybe_update<U>(&self, fun: impl FnOnce(&mut Self::Value) -> (bool, U)) -> Option<U> {
        let mut current = self.read_signal.get_untracked();
        let (should_update, ret) = fun(&mut current);
        if should_update {
            self.set(current);
        }
        Some(ret)
    }
}

impl<T, S> Dispose for FrozenSignal<T, S>
where
    T: 'static,
    S: Storage<T>,
{
    fn dispose(self) {
        self.read_signal.dispose();
    }
}

impl<T, S> DefinedAt for FrozenSignal<T, S>
where
    T: 'static,
    S: Storage<T>,
{
    fn defined_at(&self) -> Option<&'static Location<'static>> {
        Some(self.defined_at)
    }
}

impl<T, S> PartialEq for FrozenSignal<T, S>
where
    T: 'static,
    S: Storage<T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.read_signal == other.read_signal
    }
}

impl<T, S> Eq for FrozenSignal<T, S>
where
    T: 'static,
    S: Storage<T>,
{
}

impl<T, S> From<FrozenSignal<T, S>> for Signal<T, S>
where
    T: 'static,
    S: Storage<T>,
{
    fn from(val: FrozenSignal<T, S>) -> Self {
        val.read_signal
    }
}

impl<T> IntoSplitSignal for FrozenSignal<T>
where
    T: Send + Sync + Clone + PartialEq + 'static,
{
    type Value = T;
    type Read = Signal<T>;
    type Write = SignalSetter<T>;

    fn into_split_signal(self) -> (Self::Read, Self::Write) {
        (self.read_only(), self.write_only())
    }
}

impl<T> IntoSplitSignal for FrozenSignal<T, LocalStorage>
where
    T: Clone + PartialEq + 'static,
{
    type Value = T;
    type Read = Signal<T, LocalStorage>;
    type Write = SignalSetter<T, LocalStorage>;

    fn into_split_signal(self) -> (Self::Read, Self::Write) {
        (self.read_only(), self.write_only())
    }
}

impl<T> From<FrozenSignal<T>> for SignalSetter<T>
where
    T: Send + Sync + Clone + PartialEq + 'static,
{
    fn from(val: FrozenSignal<T>) -> Self {
        val.write_only()
    }
}

impl<T> From<FrozenSignal<T, LocalStorage>> for SignalSetter<T, LocalStorage>
where
    T: Clone + PartialEq + 'static,
{
    fn from(val: FrozenSignal<T, LocalStorage>) -> Self {
        val.write_only()
    }
}

impl<T, S> Serialize for FrozenSignal<T, S>
where
    T: Send + Sync + Serialize + 'static,
    S: Storage<T> + Storage<SignalTypes<T, S>>,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        self.read_signal.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let owner = Owner::new();
        owner.with(|| {
            let rw_signal = RwSignal::new(21);
            let frozen = FrozenSignal::new(rw_signal);
            let effect_count = StoredValue::new(0);

            Effect::new(move || effect_count.set_value(effect_count.get_value() + 1));
            assert_eq!(1, *effect_count.read_value());
            frozen.set(42);
            assert_eq!(42, *rw_signal.read());
            assert_eq!(21, *frozen.read());
            assert_eq!(1, *effect_count.read_value());
        });
    }
}
