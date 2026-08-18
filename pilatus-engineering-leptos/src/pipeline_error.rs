use imask::{IncompatibleSizeError, PipelineError};
use pilatus_leptos::FetchError;

/// Errors that can occur while a value is loaded from or stored to a server.
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum LeptosPipelineError {
    /// The value has not been loaded yet.
    #[error("not available yet: {0}")]
    NotAvailableYet(&'static str),
    /// The value could not be fetched.
    #[error("missing info: {0}")]
    MissingInfo(#[source] FetchError),
    /// The pipeline failed, e.g. because the mask is empty.
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
