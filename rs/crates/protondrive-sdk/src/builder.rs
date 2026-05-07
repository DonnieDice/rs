use crate::drive::ProtonDrive;
use protondrive_api::{client::ApiClient, config::SdkConfig};
use protondrive_auth::Session;
use protondrive_core::error::Result;

pub struct ProtonDriveBuilder {
    app_version: Option<String>,
    user_agent: Option<String>,
    session: Option<Session>,
}

impl ProtonDriveBuilder {
    pub fn new() -> Self {
        Self {
            app_version: None,
            user_agent: None,
            session: None,
        }
    }

    pub fn app_version(mut self, v: impl Into<String>) -> Self {
        self.app_version = Some(v.into());
        self
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn session(mut self, session: Session) -> Self {
        self.session = Some(session);
        self
    }

    pub fn build(self) -> Result<ProtonDrive> {
        let app_version = self
            .app_version
            .unwrap_or_else(|| "external-drive-linux@0.1.0-alpha".into());
        let user_agent = self
            .user_agent
            .unwrap_or_else(|| "ProtonDrive-Rust/0.1.0".into());

        let config = SdkConfig::new(app_version, user_agent)?;
        let client = ApiClient::new(config)?;

        if let Some(session) = self.session {
            client.set_access_token(session.access_token());
            return Ok(ProtonDrive { client, session });
        }

        Err(protondrive_core::error::DriveError::Auth(
            "session required; call .session() before .build()".into(),
        ))
    }
}

impl Default for ProtonDriveBuilder {
    fn default() -> Self {
        Self::new()
    }
}
