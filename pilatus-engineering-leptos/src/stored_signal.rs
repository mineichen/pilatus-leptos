use futures_util::TryFutureExt;
use gloo_net::http::Response;
use imask::{IncompatibleSizeError, NonZeroRange, PipelineError, SortedRanges, SyncRangeWriter};
use leptos::prelude::*;
use pilatus_leptos::FetchError;
use std::future::Future;

/// Errors that can occur while a mask is loaded from or stored to the server.
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum LeptosPipelineError {
    /// The mask has not been loaded yet.
    #[error("not available yet: {0}")]
    NotAvailableYet(&'static str),
    /// The mask information could not be fetched.
    #[error("missing info: {0}")]
    MissingInfo(#[source] FetchError),
    /// The mask pipeline failed, e.g. because the mask is empty.
    #[error("pipeline error: {0}")]
    Pipeline(#[from] PipelineError),
}

impl From<IncompatibleSizeError> for LeptosPipelineError {
    fn from(value: IncompatibleSizeError) -> Self {
        PipelineError::from(value).into()
    }
}

impl From<FetchError> for LeptosPipelineError {
    fn from(value: FetchError) -> Self {
        match value {
            FetchError::StatusCode(404, _) => {
                LeptosPipelineError::Pipeline(imask::PipelineError::Empty)
            }
            e => LeptosPipelineError::MissingInfo(e),
        }
    }
}

pub trait RecoverPipelineError<TOk> {
    fn recover_pipeline_error(
        self,
        recoverer: impl Fn(PipelineError) -> Result<Option<TOk>, PipelineError>,
    ) -> Result<Option<TOk>, LeptosPipelineError>;
}
impl<TOk> RecoverPipelineError<TOk> for Result<TOk, LeptosPipelineError> {
    fn recover_pipeline_error(
        self,
        recoverer: impl Fn(PipelineError) -> Result<Option<TOk>, PipelineError>,
    ) -> Result<Option<TOk>, LeptosPipelineError> {
        match self {
            Ok(x) => Ok(Some(x)),
            Err(e) => e.recover_pipeline_error(recoverer),
        }
    }
}
impl<TOk> RecoverPipelineError<TOk> for LeptosPipelineError {
    fn recover_pipeline_error(
        self,
        recoverer: impl Fn(PipelineError) -> Result<Option<TOk>, PipelineError>,
    ) -> Result<Option<TOk>, LeptosPipelineError> {
        match self {
            LeptosPipelineError::NotAvailableYet(x) => Err(LeptosPipelineError::NotAvailableYet(x)),
            LeptosPipelineError::MissingInfo(x) => Err(LeptosPipelineError::MissingInfo(x)),
            LeptosPipelineError::Pipeline(x) => recoverer(x).map_err(LeptosPipelineError::Pipeline),
        }
    }
}

/// The value held by a [`StoredSignal`]: either the loaded mask or an error
/// describing why there is none yet.
pub type StoredMaskValue = Result<SortedRanges<u32>, LeptosPipelineError>;

/// A value that is loaded once from a server and stored back to it on every write.
///
/// It packages the four things that make up such a server-persisted mask:
/// - the read [`Signal`] that exposes the current value,
/// - the write [`SignalSetter`] used to edit it,
/// - the [`LocalResource`] that provides the initial value,
/// - the [`Action`] that stores a written value back to the server.
///
/// An editor only ever reads the [`Signal`] or writes into the [`SignalSetter`]:
/// a write updates the resource immediately and triggers the store
/// [`Action`] from within the write signal. The resource loading the initial
/// value and the store action are private implementation details, so the
/// consumer never touches them directly.
#[derive(Clone, Copy)]
pub struct StoredSignal {
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

impl StoredSignal {
    /// Creates a new stored signal.
    ///
    /// - `reason` describes why the mask is not yet available. Until the
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
                            "Could not read occlusions body".to_string(),
                        ))
                    })
                })
            },
            store,
        )
    }

    /// Creates a new stored signal from the already fetched serialized mask.
    ///
    /// Like [`StoredSignal::new`], but the loader provides the raw mask bytes
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
                match SortedRanges::<u32, u32>::from_serialized(&bytes) {
                    Ok(m) if m.len() > 0 => Ok(m),
                    _ => Err(LeptosPipelineError::Pipeline(imask::PipelineError::Empty)),
                }
            })
        });
        let store_action = Action::new_local(move |value: &StoredMaskValue| {
            let occlusions = value
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
                .or_else(|e| e.recover_pipeline_error(|e| Ok(e.allow_empty()?)));
            let map_fut = occlusions.map(|x| store(x));
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

    /// The read-only [`Signal`] of the current value.
    pub fn signal(&self) -> Signal<StoredMaskValue, LocalStorage> {
        self.read
    }

    /// The write-only [`SignalSetter`]. Writing to it updates the value and
    /// stores it back to the server.
    pub fn writer(&self) -> SignalSetter<StoredMaskValue, LocalStorage> {
        self.write
    }

    /// Sets the value, updating it immediately and storing it back to the
    /// server.
    pub fn set(&self, value: StoredMaskValue) {
        self.write.set(value);
    }

    /// Reactively reads the current value.
    pub fn get(&self) -> StoredMaskValue {
        self.read.get()
    }

    /// Reads the current value without tracking dependencies.
    pub fn get_untracked(&self) -> StoredMaskValue {
        self.read.get_untracked()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use any_spawner::Executor;
    use imask::{ImaskSet, Rect};
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
        SortedRanges::<u32>::try_from_ordered_iter(ranges.with_roi(TEST_BOUNDS))
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

                let stored = StoredSignal::new_with_vec(
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
                stored.writer().set(Ok(written.clone()));
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
                let stored = StoredSignal::new_with_vec(
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
                let stored = StoredSignal::new(
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
                let stored = StoredSignal::new(
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

    async fn await_loaded(stored: &StoredSignal, expected: impl Fn(&StoredMaskValue) -> bool) {
        for _ in 0..64 {
            if expected(&stored.get()) {
                return;
            }
            Executor::tick().await;
        }
        panic!("stored signal did not reach the expected value");
    }
}
