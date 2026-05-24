use crate::config::{
    SdkConfig, DEFAULT_STORAGE_TIMEOUT_MS, DEFAULT_TIMEOUT_MS, DRIVE_SDK_VERSION, PROTON_API_BASE,
};
use crate::retry::default_backoff;
use backon::Retryable;
use bytes::Bytes;
use protondrive_core::error::{DriveError, Result};
use reqwest::{header, Method, RequestBuilder, Response, StatusCode};

fn net_err(e: reqwest::Error) -> DriveError {
    DriveError::Network(e.to_string())
}
use reqwest_cookie_store::CookieStoreMutex;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
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
        let url = metadata_url(path);
        let mut req = self.inner.request(method, &url);

        req = req.timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS));
        req = req.header("x-pm-appversion", &self.config.app_version);
        req = req.header(header::USER_AGENT, &self.config.user_agent);
        req = req.header("x-pm-apiversion", "3");
        req = req.header("x-pm-drive-sdk-version", DRIVE_SDK_VERSION);
        req = req.header(header::ACCEPT, "application/vnd.protonmail.v1+json");

        if let Some(token) = self.access_token.read().unwrap().as_deref() {
            req = req.bearer_auth(token);
        }

        req
    }

    pub async fn get_block(&self, base_url: &str, token: &str) -> Result<Bytes> {
        let req = self
            .inner
            .request(Method::GET, base_url)
            .timeout(Duration::from_millis(DEFAULT_STORAGE_TIMEOUT_MS))
            .header("pm-storage-token", token)
            .header("x-pm-appversion", &self.config.app_version)
            .header(header::USER_AGENT, &self.config.user_agent)
            .header("x-pm-drive-sdk-version", DRIVE_SDK_VERSION);

        let response = req.send().await.map_err(net_err)?;
        self.parse_bytes_response(response).await
    }

    pub async fn post_block<B>(&self, base_url: &str, token: &str, body: B) -> Result<()>
    where
        B: Into<reqwest::Body>,
    {
        let req = self
            .inner
            .request(Method::POST, base_url)
            .timeout(Duration::from_millis(DEFAULT_STORAGE_TIMEOUT_MS))
            .header("pm-storage-token", token)
            .header("x-pm-appversion", &self.config.app_version)
            .header(header::USER_AGENT, &self.config.user_agent)
            .header("x-pm-drive-sdk-version", DRIVE_SDK_VERSION)
            .body(body);

        let response = req.send().await.map_err(net_err)?;
        self.parse_empty_response(response).await
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
            .retry(&default_backoff())
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
            s if s.is_success() => response.json::<T>().await.map_err(net_err),
            _ => {
                #[derive(serde::Deserialize)]
                struct ApiErr {
                    #[serde(rename = "Code")]
                    code: u32,
                    #[serde(rename = "Error")]
                    error: String,
                }
                let status = response.status().as_u16() as u32;
                let body = response.json::<ApiErr>().await.unwrap_or(ApiErr {
                    code: status,
                    error: "unknown error".into(),
                });
                Err(DriveError::Api {
                    code: body.code,
                    message: body.error,
                })
            }
        }
    }

    async fn parse_empty_response(&self, response: Response) -> Result<()> {
        match response.status() {
            s if s.is_success() => Ok(()),
            _ => self
                .parse_response::<serde_json::Value>(response)
                .await
                .map(|_| ()),
        }
    }

    async fn parse_bytes_response(&self, response: Response) -> Result<Bytes> {
        match response.status() {
            s if s.is_success() => response.bytes().await.map_err(net_err),
            _ => self
                .parse_response::<serde_json::Value>(response)
                .await
                .map(|_| Bytes::new()),
        }
    }
}

pub(crate) fn metadata_url(path: &str) -> String {
    format!("{PROTON_API_BASE}/{}", path.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DEFAULT_STORAGE_TIMEOUT_MS, DEFAULT_TIMEOUT_MS};

    fn client() -> ApiClient {
        ApiClient::new(
            SdkConfig::new("external-drive-linux@1.0.0-stable", "protondrive-test").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn metadata_request_sets_official_headers_and_timeout() {
        let req = client()
            .build_request(Method::GET, "drive/v2/shares/my-files")
            .build()
            .unwrap();

        assert_eq!(
            req.url().as_str(),
            "https://drive.proton.me/api/drive/v2/shares/my-files"
        );
        assert_eq!(
            req.headers()["x-pm-appversion"],
            "external-drive-linux@1.0.0-stable"
        );
        assert_eq!(req.headers()["user-agent"], "protondrive-test");
        assert_eq!(req.headers()["x-pm-apiversion"], "3");
        assert_eq!(req.headers()["x-pm-drive-sdk-version"], DRIVE_SDK_VERSION);
        assert_eq!(
            req.headers()["accept"],
            "application/vnd.protonmail.v1+json"
        );
        assert_eq!(
            req.timeout().copied(),
            Some(Duration::from_millis(DEFAULT_TIMEOUT_MS))
        );
    }

    #[test]
    fn storage_upload_request_sets_official_headers_and_timeout() {
        let req = client()
            .inner
            .request(Method::POST, "https://storage.example.test/block")
            .timeout(Duration::from_millis(DEFAULT_STORAGE_TIMEOUT_MS))
            .header("pm-storage-token", "token")
            .header("x-pm-appversion", "external-drive-linux@1.0.0-stable")
            .header(header::USER_AGENT, "protondrive-test")
            .header("x-pm-drive-sdk-version", DRIVE_SDK_VERSION)
            .build()
            .unwrap();

        assert_eq!(req.headers()["pm-storage-token"], "token");
        assert_eq!(req.headers()["x-pm-drive-sdk-version"], DRIVE_SDK_VERSION);
        assert_eq!(
            req.timeout().copied(),
            Some(Duration::from_millis(DEFAULT_STORAGE_TIMEOUT_MS))
        );
    }
}
