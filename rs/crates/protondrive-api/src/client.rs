use crate::config::{SdkConfig, PROTON_API_BASE};
use crate::retry::default_backoff;
use backon::Retryable;
use protondrive_core::error::{DriveError, Result};
use reqwest::{header, Method, RequestBuilder, Response, StatusCode};

fn net_err(e: reqwest::Error) -> DriveError {
    DriveError::Network(e.to_string())
}
use reqwest_cookie_store::CookieStoreMutex;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct ApiClient {
    inner: reqwest::Client,
    config: SdkConfig,
    access_token: Arc<std::sync::RwLock<Option<String>>>,
}

impl ApiClient {
    pub fn new(config: SdkConfig) -> Result<Self> {
        let cookie_store = Arc::new(CookieStoreMutex::default());
        let inner = reqwest::Client::builder()
            .cookie_provider(cookie_store)
            .use_rustls_tls()
            .build()
            .map_err(net_err)?;

        Ok(Self {
            inner,
            config,
            access_token: Arc::new(std::sync::RwLock::new(None)),
        })
    }

    pub fn set_access_token(&self, token: impl Into<String>) {
        *self.access_token.write().unwrap() = Some(token.into());
    }

    pub fn clear_access_token(&self) {
        *self.access_token.write().unwrap() = None;
    }

    #[instrument(skip(self), fields(path = %path))]
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let req = self.build_request(Method::GET, path);
        self.execute_with_retry(req).await
    }

    #[instrument(skip(self, body), fields(path = %path))]
    pub async fn post<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let req = self.build_request(Method::POST, path).json(body);
        self.execute_with_retry(req).await
    }

    #[instrument(skip(self, body), fields(path = %path))]
    pub async fn put<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let req = self.build_request(Method::PUT, path).json(body);
        self.execute_with_retry(req).await
    }

    #[instrument(skip(self), fields(path = %path))]
    pub async fn delete(&self, path: &str) -> Result<()> {
        let req = self.build_request(Method::DELETE, path);
        self.execute_with_retry::<serde_json::Value>(req).await?;
        Ok(())
    }

    fn build_request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{PROTON_API_BASE}{path}");
        let mut req = self.inner.request(method, &url);

        req = req.header("x-pm-appversion", &self.config.app_version);
        req = req.header(header::USER_AGENT, &self.config.user_agent);
        req = req.header("x-pm-apiversion", "3");

        if let Some(token) = self.access_token.read().unwrap().as_deref() {
            req = req.bearer_auth(token);
        }

        req
    }

    async fn execute_with_retry<T: DeserializeOwned>(&self, req: RequestBuilder) -> Result<T> {
        let action = || async {
            let response = req
                .try_clone()
                .expect("request must be cloneable for retry")
                .send()
                .await
                .map_err(net_err)?;
            self.parse_response(response).await
        };

        action
            .retry(default_backoff())
            .when(|e| matches!(e, DriveError::RateLimited { .. } | DriveError::Network(_)))
            .await
    }

    async fn parse_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        match response.status() {
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response
                    .headers()
                    .get(header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60);
                Err(DriveError::RateLimited {
                    retry_after_secs: retry_after,
                })
            }
            StatusCode::UNAUTHORIZED => Err(DriveError::SessionExpired),
            StatusCode::NOT_FOUND => Err(DriveError::NotFound),
            StatusCode::FORBIDDEN => Err(DriveError::PermissionDenied),
            s if s.is_success() => {
                response.json::<T>().await.map_err(DriveError::Network)
            }
            _ => {
                #[derive(serde::Deserialize)]
                struct ApiErr {
                    #[serde(rename = "Code")]
                    code: u32,
                    #[serde(rename = "Error")]
                    error: String,
                }
                let status = response.status().as_u16() as u32;
                let body = response
                    .json::<ApiErr>()
                    .await
                    .unwrap_or(ApiErr { code: status, error: "unknown error".into() });
                Err(DriveError::Api {
                    code: body.code,
                    message: body.error,
                })
            }
        }
    }
}
