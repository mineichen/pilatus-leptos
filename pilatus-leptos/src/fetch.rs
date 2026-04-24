use std::{sync::Arc, time::Duration};

use serde::{Serialize, de::DeserializeOwned};

use crate::NotificationContext;

#[derive(Clone, Copy)]
pub struct FetchApi {
    notifications: NotificationContext,
}

pub type FetchResult<T> = Result<T, FetchError>;

#[derive(Debug, thiserror::Error, Clone)]
pub enum FetchError {
    #[error("Http {0}: {1}")]
    StatusCode(u16, String),
    #[error("Invalid data: {0}")]
    Deserialize(Arc<serde_json::Error>),
    #[error("{0}")]
    Other(String),
}

impl FetchApi {
    pub(super) fn new(notifications: NotificationContext) -> Self {
        Self { notifications }
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> FetchResult<T> {
        self.notify(self.get_silent(url).await)
    }
    pub async fn get_silent<T: DeserializeOwned>(&self, url: &str) -> FetchResult<T> {
        let response = gloo_net::http::Request::get(url).build()?.send().await?;
        let response = self.handle_http_error(response).await?;
        Ok(response.json::<T>().await?)
    }
    pub async fn put_json(&self, url: &str, payload: impl Serialize) -> FetchResult<()> {
        self.notify(self.put_json_silent(url, payload).await)
    }

    pub async fn put_json_silent(&self, url: &str, payload: impl Serialize) -> FetchResult<()> {
        async {
            let response = gloo_net::http::Request::put(url)
                .json(&payload)?
                .send()
                .await?;
            self.handle_http_error(response).await?;
            Ok(())
        }
        .await
    }

    pub async fn post_json(&self, url: &str, payload: impl Serialize) -> FetchResult<()> {
        self.notify(self.post_json_silent(url, payload).await)
    }

    pub async fn post_json_silent(&self, url: &str, payload: impl Serialize) -> FetchResult<()> {
        let response = gloo_net::http::Request::post(url)
            .json(&payload)?
            .send()
            .await?;
        self.handle_http_error(response).await?;
        Ok(())
    }

    fn notify<TOk>(&self, result: Result<TOk, FetchError>) -> Result<TOk, FetchError> {
        result.inspect_err(|e| {
            self.notifications
                .error(e.to_string(), Duration::from_secs(3))
        })
    }

    async fn handle_http_error(
        &self,
        r: gloo_net::http::Response,
    ) -> FetchResult<gloo_net::http::Response> {
        if r.ok() {
            Ok(r)
        } else {
            Err(FetchError::StatusCode(
                r.status(),
                r.text().await.unwrap_or(r.status_text()),
            ))
        }
    }
}

impl From<gloo_net::Error> for FetchError {
    fn from(value: gloo_net::Error) -> Self {
        match value {
            gloo_net::Error::JsError(e) => FetchError::Other(e.to_string()),
            gloo_net::Error::SerdeError(e) => FetchError::Deserialize(Arc::new(e)),
            gloo_net::Error::GlooError(e) => FetchError::Other(e.to_string()),
        }
    }
}
