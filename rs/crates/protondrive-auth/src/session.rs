use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// A live authenticated session.
#[derive(ZeroizeOnDrop)]
pub struct Session {
    pub uid: String,
    pub user_id: String,
    #[zeroize(skip)]
    pub access_token: SecretString,
    #[zeroize(skip)]
    pub refresh_token: SecretString,
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("uid", &self.uid)
            .field("user_id", &self.user_id)
            .field("access_token", &"[redacted]")
            .field("refresh_token", &"[redacted]")
            .finish()
    }
}

impl Session {
    pub fn access_token(&self) -> &str {
        self.access_token.expose_secret()
    }

    pub fn refresh_token(&self) -> &str {
        self.refresh_token.expose_secret()
    }

    pub fn serialize(&self) -> SerializedSession {
        SerializedSession {
            uid: self.uid.clone(),
            user_id: self.user_id.clone(),
            access_token: self.access_token.expose_secret().to_owned(),
            refresh_token: self.refresh_token.expose_secret().to_owned(),
        }
    }
}

/// A serializable form of a session for persistence by the consumer.
///
/// The consumer is responsible for encrypting this before storing it
/// (e.g. via the OS keyring). The SDK never persists secrets.
#[derive(Serialize, Deserialize, ZeroizeOnDrop)]
pub struct SerializedSession {
    pub uid: String,
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: String,
}

impl fmt::Debug for SerializedSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SerializedSession")
            .field("uid", &self.uid)
            .field("user_id", &self.user_id)
            .field("access_token", &"[redacted]")
            .field("refresh_token", &"[redacted]")
            .finish()
    }
}

impl From<SerializedSession> for Session {
    fn from(s: SerializedSession) -> Self {
        Self {
            uid: s.uid,
            user_id: s.user_id,
            access_token: SecretString::new(s.access_token.into()),
            refresh_token: SecretString::new(s.refresh_token.into()),
        }
    }
}
