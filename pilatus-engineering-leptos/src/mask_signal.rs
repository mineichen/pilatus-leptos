use futures_util::TryFutureExt;
use gloo_net::http::Response;
use imask::{NonZeroRange, SortedRanges, SyncRangeWriter};
use leptos::prelude::*;
use pilatus_leptos::FetchError;
use std::future::Future;

use crate::{LeptosPipelineError, RecoverPipelineError};

/// The value held by a [`MaskSignal`]: either the loaded mask or an error
/// describing why there is none yet.
pub type StoredMaskValue = Result<SortedRanges<u32>, LeptosPipelineError>;

/// A mask that is loaded once from a server and stored back to it on every write.
///
/// It packages the four things that make up such a server-persisted mask:
/// - a read [`Signal`] that exposes the current value,
/// - a write [`SignalSetter`] used to edit it,
/// - the [`LocalResource`] that provides the initial value,
/// - the [`Action`] that stores a written value back to the server.
///
/// `MaskSignal` itself implements the reactive traits over the current value
/// ([`Read`], [`ReadUntracked`] and, via blanket impls, [`With`],
/// [`WithUntracked`], [`Get`] and [`GetUntracked`]) by delegating to the read
/// signal, as well as [`Set`] and [`Update`] by delegating to the write
/// signal: a write updates the resource immediately and triggers the store
/// [`Action`] from within the write signal. The resource loading the initial
/// value and the store action are private implementation details, so the
/// consumer never touches them directly.
#[derive(Clone, Copy)]
pub struct MaskSignal {
    read: Signal<StoredMaskValue, LocalStorage>,
    write: SignalSetter<StoredMaskValue, LocalStorage>,
    #[allow(
        dead_code,
        reason = "Owned to keep the loaded state alive for the read signal"
    )]
    resource: LocalResource<StoredMaskValue>,
    #[allow(
        dead_code,
        reason = "Owned to keep the store action alive for the write signal"
    )]
    store: Action<StoredMaskValue, Result<(), LeptosPipelineError>>,
}

impl MaskSignal {
    /// Creates a new mask signal.
    ///
    /// - `subject` describes why the mask is not yet available. Until the
    ///   loader resolves, the read signal yields
    ///   [`LeptosPipelineError::NotAvailableYet`] carrying this reason.
    /// - `loader` produces the initial mask from the server. It can return
    ///   [`Err`] to signal an empty mask ([`LeptosPipelineError::Pipeline`])
    ///   or a fetch failure ([`LeptosPipelineError::MissingInfo`]).
    /// - `store` persists a written mask back to the server.
    pub fn new<F, Fut, S, FutS>(subject: &'static str, loader: F, store: S) -> Self
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = Result<Response, LeptosPipelineError>> + 'static,
        S: Fn(Option<Vec<u8>>) -> FutS + 'static,
        FutS: Future<Output = Result<(), FetchError>> + 'static,
    {
        Self::new_with_vec(
            subject,
            move || {
                loader().and_then(|response| async move {
                    response.binary().await.map_err(|_| {
                        LeptosPipelineError::MissingInfo(FetchError::Other(
                            "Could not read body".to_string(),
                        ))
                    })
                })
            },
            store,
        )
    }

    /// Creates a new mask signal from the already fetched serialized mask.
    ///
    /// Like [`MaskSignal::new`], but the loader provides the raw mask bytes
    /// instead of an HTTP [`Response`]. This makes the reactive behavior
    /// testable outside of a web runtime.
    pub(crate) fn new_with_vec<F, Fut, S, FutS>(subject: &'static str, loader: F, store: S) -> Self
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = Result<Vec<u8>, LeptosPipelineError>> + 'static,
        S: Fn(Option<Vec<u8>>) -> FutS + 'static,
        FutS: Future<Output = Result<(), FetchError>> + 'static,
    {
        let resource = LocalResource::new(move || {
            loader().and_then(|bytes| async move {
                match SortedRanges::<u32>::from_serialized(&bytes) {
                    Ok(m) if m.len() > 0 => Ok(m),
                    _ => Err(LeptosPipelineError::Pipeline(imask::PipelineError::Empty)),
                }
            })
        });
        let store_action = Action::new_local(move |value: &StoredMaskValue| {
            let mask = value
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|x| {
                    let mut buf = Vec::<u8>::new();

                    SyncRangeWriter::new(&mut buf, x.iter_roi::<NonZeroRange<u32>>())
                        .write()
                        .expect("Writing SortedRanges to Vec cannot fail");
                    Ok(buf)
                })
                .map(Some)
                .or_else(|e| e.allow_empty());
            let map_fut = mask.map(|x| store(x));
            async move {
                let x = map_fut?.await?;
                Ok(x)
            }
        });

        // The read signal mirrors whatever the resource currently holds, so the
        // loaded value shows up once the loader has resolved.
        let read = Signal::derive_local(move || {
            resource
                .get()
                .unwrap_or(Err(LeptosPipelineError::NotAvailableYet(subject)))
        });
        // A write updates the resource immediately and stores it back via the
        // hidden action. The resource and the store action stay hidden away.
        let write = SignalSetter::map(move |value: StoredMaskValue| {
            resource.set(Some(value.clone()));
            store_action.dispatch_local(value);
        });

        Self {
            read,
            write,
            resource,
            store: store_action,
        }
    }
}

impl DefinedAt for MaskSignal {
    fn defined_at(&self) -> Option<&'static std::panic::Location<'static>> {
        self.read.defined_at()
    }
}

impl Dispose for MaskSignal {
    fn dispose(self) {
        self.read.dispose();
    }
}

impl ReadUntracked for MaskSignal {
    type Value = <Signal<StoredMaskValue, LocalStorage> as ReadUntracked>::Value;

    fn try_read_untracked(&self) -> Option<Self::Value> {
        self.read.try_read_untracked()
    }
}

impl Read for MaskSignal {
    type Value = <Signal<StoredMaskValue, LocalStorage> as Read>::Value;

    fn try_read(&self) -> Option<Self::Value> {
        self.read.try_read()
    }
}

impl Set for MaskSignal {
    type Value = StoredMaskValue;

    fn set(&self, value: Self::Value) {
        self.write.set(value);
    }

    fn try_set(&self, value: Self::Value) -> Option<Self::Value> {
        self.write.try_set(value)
    }
}

impl Update for MaskSignal {
    type Value = StoredMaskValue;

    fn try_maybe_update<U>(&self, fun: impl FnOnce(&mut Self::Value) -> (bool, U)) -> Option<U> {
        let mut current = self.read.try_get_untracked()?;
        let (should_update, ret) = fun(&mut current);
        if should_update {
            self.write.set(current);
        }
        Some(ret)
    }
}

impl From<MaskSignal> for Signal<StoredMaskValue, LocalStorage> {
    fn from(val: MaskSignal) -> Self {
        val.read
    }
}

impl From<MaskSignal> for SignalSetter<StoredMaskValue, LocalStorage> {
    fn from(val: MaskSignal) -> Self {
        val.write
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use any_spawner::Executor;
    use imask::{ImaskSet, PipelineError, Rect};
    use leptos::prelude::{Get, Set};
    use reactive_graph::owner::Owner;
    use std::cell::RefCell;
    use std::num::NonZero;
    use std::rc::Rc;

    const TEST_BOUNDS: Rect<u32> = Rect::new(
        0,
        0,
        NonZero::new(1000u32).unwrap(),
        NonZero::new(1000u32).unwrap(),
    );

    fn mask_ranges(ranges: Vec<std::ops::Range<u32>>) -> SortedRanges<u32> {
        SortedRanges::try_from_ordered_iter(ranges.with_roi(TEST_BOUNDS))
            .expect("Sorted, non-empty ranges")
    }
    fn mask_bytes(ranges: Vec<std::ops::Range<u32>>) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        SyncRangeWriter::new(
            &mut buf,
            mask_ranges(ranges).iter_roi_owned::<NonZeroRange<u32>>(),
        )
        .write()
        .unwrap();
        buf
    }

    #[tokio::test(flavor = "current_thread")]
    async fn loads_and_stores() {
        _ = Executor::init_tokio();
        let owner = Owner::new();
        tokio::task::LocalSet::new()
            .run_until(owner.with(|| async move {
                let stored_values = Rc::new(RefCell::new(Vec::<Option<Vec<u8>>>::new()));
                let store_values = stored_values.clone();

                let stored = MaskSignal::new_with_vec(
                    "test mask",
                    || async { Ok(mask_bytes(vec![0..10])) },
                    move |value| {
                        let store_values = store_values.clone();
                        async move {
                            store_values.borrow_mut().push(value);

                            Ok(())
                        }
                    },
                );

                await_loaded(
                    &stored,
                    |v| matches!(v, Ok(m) if m == &mask_ranges(vec![0..10])),
                )
                .await;
                assert!(stored_values.borrow().is_empty());

                let written = mask_ranges(vec![5..20]);
                stored.set(Ok(written.clone()));
                Executor::tick().await;

                assert!(matches!(stored.get(), Ok(ref m) if m == &written));
                assert_eq!(vec![Some(mask_bytes(vec![5..20]))], *stored_values.borrow());
            }))
            .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn not_loaded_yet_falls_back_to_not_available() {
        _ = Executor::init_tokio();
        let owner = Owner::new();
        tokio::task::LocalSet::new()
            .run_until(owner.with(|| async move {
                let stored = MaskSignal::new_with_vec(
                    "test mask",
                    || async { Ok(mask_bytes(vec![0..7])) },
                    |_value| async { Ok(()) },
                );

                // Not yet resolved: reads as not available.
                assert!(matches!(
                    stored.get(),
                    Err(LeptosPipelineError::NotAvailableYet("test mask"))
                ));

                await_loaded(
                    &stored,
                    |v| matches!(v, Ok(m) if m == &mask_ranges(vec![0..7])),
                )
                .await;
            }))
            .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn empty_mask_reads_as_pipeline_empty() {
        _ = Executor::init_tokio();
        let owner = Owner::new();
        tokio::task::LocalSet::new()
            .run_until(owner.with(|| async move {
                let stored = MaskSignal::new(
                    "test mask",
                    || async { Err(PipelineError::Empty.into()) },
                    |_value| async { Ok(()) },
                );

                await_loaded(&stored, |v| {
                    matches!(v, Err(LeptosPipelineError::Pipeline(PipelineError::Empty)))
                })
                .await;
            }))
            .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_error_reads_as_missing_info() {
        _ = Executor::init_tokio();
        let owner = Owner::new();
        tokio::task::LocalSet::new()
            .run_until(owner.with(|| async move {
                let stored = MaskSignal::new(
                    "test mask",
                    || async { Err(FetchError::Other("boom".into()).into()) },
                    |_value| async { Ok(()) },
                );

                await_loaded(&stored, |v| {
                    matches!(v, Err(LeptosPipelineError::MissingInfo(_)))
                })
                .await;
            }))
            .await;
    }

    async fn await_loaded(stored: &MaskSignal, expected: impl Fn(&StoredMaskValue) -> bool) {
        for _ in 0..64 {
            if expected(&stored.get()) {
                return;
            }
            Executor::tick().await;
        }
        panic!("stored signal did not reach the expected value");
    }
}
