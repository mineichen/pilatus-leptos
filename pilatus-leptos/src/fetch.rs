use std::{sync::Arc, time::Duration};

use futures_util::FutureExt;
use gloo_net::http::Response;
use serde::{Serialize, de::DeserializeOwned};
use web_sys::wasm_bindgen::JsValue;

use crate::NotificationContext;

#[derive(Clone, Copy)]
pub struct FetchApi {
    notifications: NotificationContext,
}

pub type FetchResult<T> = Result<T, FetchError>;

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
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

    pub fn get_silent(&self, url: &str) -> impl Future<Output = FetchResult<Response>> + use<> {
        let request = gloo_net::http::Request::get(url);

        async move {
            let response = request.send().await?;
            Self::handle_http_error(response).await
        }
    }
    pub fn get_json_silent<T: DeserializeOwned>(
        &self,
        url: &str,
    ) -> impl Future<Output = FetchResult<T>> + use<T> {
        let fut = self.get_silent(url);
        async move { Ok(fut.await?.json::<T>().await?) }
    }

    pub fn put_json<T: Serialize>(
        &self,
        url: &str,
        payload: T,
    ) -> impl Future<Output = FetchResult<Response>> + use<T> {
        self.put_json_silent(url, payload)
            .map(self.notify_callback())
    }

    pub fn put_json_silent<T: Serialize>(
        &self,
        url: &str,
        payload: T,
    ) -> impl Future<Output = FetchResult<Response>> + use<T> {
        let request = gloo_net::http::Request::put(url).json(&payload);
        async move {
            let response = request?.send().await?;
            Self::handle_http_error(response).await
        }
    }

    pub fn put_body(
        &self,
        url: &str,
        payload: JsValue,
    ) -> impl Future<Output = FetchResult<Response>> + use<> {
        self.put_body_silent(url, payload)
            .map(self.notify_callback())
    }

    pub fn put_body_silent(
        &self,
        url: &str,
        payload: JsValue,
    ) -> impl Future<Output = FetchResult<Response>> + use<> {
        let request = gloo_net::http::Request::put(url).body(payload);
        async move {
            let response = request?.send().await?;
            Self::handle_http_error(response).await
        }
    }
    pub fn put(&self, url: &str) -> impl Future<Output = FetchResult<Response>> + use<> {
        self.put_silent(url).map(self.notify_callback())
    }

    pub fn put_silent(&self, url: &str) -> impl Future<Output = FetchResult<Response>> + use<> {
        let request = gloo_net::http::Request::put(url);
        async move {
            let response = request.send().await?;
            Self::handle_http_error(response).await
        }
    }

    pub fn post_json<T: Serialize>(
        &self,
        url: &str,
        payload: T,
    ) -> impl Future<Output = FetchResult<Response>> + use<T> {
        self.post_json_silent(url, payload)
            .map(self.notify_callback())
    }

    pub fn post_json_silent<T: Serialize>(
        &self,
        url: &str,
        payload: T,
    ) -> impl Future<Output = FetchResult<Response>> + use<T> {
        let request = gloo_net::http::Request::post(url).json(&payload);
        async move {
            let response = request?.send().await?;
            Self::handle_http_error(response).await
        }
    }

    pub fn post_body(
        &self,
        url: &str,
        payload: JsValue,
    ) -> impl Future<Output = FetchResult<Response>> + use<> {
        self.post_body_silent(url, payload)
            .map(self.notify_callback())
    }

    pub fn post_body_silent(
        &self,
        url: &str,
        payload: JsValue,
    ) -> impl Future<Output = FetchResult<Response>> + use<> {
        let request = gloo_net::http::Request::post(url).body(payload);
        async move {
            let response = request?.send().await?;
            Self::handle_http_error(response).await
        }
    }

    pub fn delete(&self, url: &str) -> impl Future<Output = FetchResult<Response>> + use<> {
        self.delete_silent(url).map(self.notify_callback())
    }
    pub fn delete_silent(&self, url: &str) -> impl Future<Output = FetchResult<Response>> + use<> {
        let request = gloo_net::http::Request::delete(url);
        async move {
            let response = request.send().await?;
            Self::handle_http_error(response).await
        }
    }

    fn notify_callback<TOk>(
        &self,
    ) -> impl FnOnce(Result<TOk, FetchError>) -> Result<TOk, FetchError> + use<TOk> {
        let notifications = self.notifications;
        move |result| {
            result.inspect_err(move |e| notifications.error(e.to_string(), Duration::from_secs(3)))
        }
    }

    async fn handle_http_error(r: Response) -> FetchResult<Response> {
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
