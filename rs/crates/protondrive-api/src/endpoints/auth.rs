use crate::client::ApiClient;
use protondrive_core::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AuthInfoRequest {
    #[serde(rename = "Username")]
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthInfoResponse {
    #[serde(rename = "Modulus")]
    pub modulus: String,
    #[serde(rename = "ServerEphemeral")]
    pub server_ephemeral: String,
    #[serde(rename = "Version")]
    pub version: u32,
    #[serde(rename = "Salt")]
    pub salt: String,
    #[serde(rename = "SRPSession")]
    pub srp_session: String,
}

#[derive(Debug, Serialize)]
pub struct AuthRequest {
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "ClientEphemeral")]
    pub client_ephemeral: String,
    #[serde(rename = "ClientProof")]
    pub client_proof: String,
    #[serde(rename = "SRPSession")]
    pub srp_session: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    #[serde(rename = "AccessToken")]
    pub access_token: String,
    #[serde(rename = "RefreshToken")]
    pub refresh_token: String,
    #[serde(rename = "UID")]
    pub uid: String,
    #[serde(rename = "UserID")]
    pub user_id: String,
    #[serde(rename = "TokenType")]
    pub token_type: String,
    #[serde(rename = "Scope")]
    pub scope: String,
    #[serde(rename = "TwoFactor")]
    pub two_factor: Option<TwoFactorInfo>,
}

#[derive(Debug, Deserialize)]
pub struct TwoFactorInfo {
    #[serde(rename = "Enabled")]
    pub enabled: u32,
    #[serde(rename = "TOTP")]
    pub totp: u32,
}

#[derive(Debug, Serialize)]
pub struct TwoFactorRequest {
    #[serde(rename = "TwoFactorCode")]
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshRequest {
    #[serde(rename = "RefreshToken")]
    pub refresh_token: String,
    #[serde(rename = "UID")]
    pub uid: String,
    #[serde(rename = "ResponseType")]
    pub response_type: String,
    #[serde(rename = "GrantType")]
    pub grant_type: String,
    #[serde(rename = "RedirectURI")]
    pub redirect_uri: String,
}

impl ApiClient {
    pub async fn get_auth_info(&self, req: &AuthInfoRequest) -> Result<AuthInfoResponse> {
        self.post("/auth/v4/info", req).await
    }

    pub async fn authenticate(&self, req: &AuthRequest) -> Result<AuthResponse> {
        self.post("/auth/v4", req).await
    }

    pub async fn submit_two_factor(&self, req: &TwoFactorRequest) -> Result<()> {
        self.post::<_, serde_json::Value>("/auth/v4/2fa", req)
            .await?;
        Ok(())
    }

    pub async fn refresh_token(&self, req: &RefreshRequest) -> Result<AuthResponse> {
        self.post("/auth/v4/refresh", req).await
    }

    pub async fn logout(&self) -> Result<()> {
        self.delete("/auth/v4").await
    }
}
