use leptos::prelude::*;
use std::panic::Location;

/// Inherits values from it's parent until it writes the data itself. At this point, it will become non-reactive.
/// It's value will remain the old value before the first change. This is useful, if you want to block external changes
/// once you start changing the value, but still propagate your changes.
#[derive(Debug)]
pub struct FrozenSignal<T: Send + Sync + 'static> {
    defined_at: &'static Location<'static>,
    source: RwSignal<T>,
    latch: StoredValue<Option<T>>, // None until first write → zero construction clone
    read_signal: Signal<T>,
}
impl<T: Send + Sync + 'static> Clone for FrozenSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Send + Sync + 'static> Copy for FrozenSignal<T> {}

impl<T: Send + Sync + Clone + PartialEq + 'static> FrozenSignal<T> {
    #[track_caller]
    pub fn new(source: RwSignal<T>) -> Self {
        let latch = StoredValue::new(None);

        let (l_m, s_m) = (latch, source);
        let read_signal: Signal<T> =
            Memo::new(move |_| l_m.get_value().unwrap_or_else(|| s_m.get())).into();

        Self {
            defined_at: Location::caller(),
            source,
            latch,
            read_signal,
        }
    }

    pub fn read_only(&self) -> Signal<T> {
        self.read_signal
    }

    pub fn set(&self, mut new_val: T) {
        let (source, latch) = (self.source, self.latch);
        if latch.read_value().is_some() {
            source.set(new_val); // already frozen: just forward to parent
            return;
        }
        // First write: capture old by MOVE (no clone), latch, freeze, forward — atomically.
        // batch() defers the Memo's re-eval until frozen=true + latch=Some are visible.

        batch(move || {
            source.update(|v: &mut T| {
                std::mem::swap(v, &mut new_val);
            }); // source := new_val, returns old
            latch.set_value(Some(new_val)); // Some(old), moved — no clone
        });
    }

    pub fn frozen(&self) -> bool {
        self.latch.get_value().is_some()
    }
}

impl<T: Send + Sync> DefinedAt for FrozenSignal<T> {
    fn defined_at(&self) -> Option<&'static Location<'static>> {
        Some(self.defined_at)
    }
}
