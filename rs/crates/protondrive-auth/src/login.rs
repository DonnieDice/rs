use crate::session::Session;
use crate::two_factor::TwoFactorProvider;
use protondrive_api::client::ApiClient;
use protondrive_api::endpoints::auth::{AuthInfoRequest, AuthRequest, TwoFactorRequest};
use protondrive_core::error::{DriveError, Result};
use secrecy::{ExposeSecret, SecretString};
use tracing::instrument;

/// Login to Proton Drive using SRP-6a + optional 2FA.
///
/// The SRP handshake is performed via `proton-srp`; this function is the
/// Drive-specific orchestration layer on top.
#[instrument(skip(password, two_factor), fields(username = %username))]
pub async fn login(
    client: &ApiClient,
    username: &str,
    password: SecretString,
    two_factor: Option<TwoFactorProvider>,
) -> Result<Session> {
    // Step 1: get SRP parameters from Proton
    let info = client
        .get_auth_info(&AuthInfoRequest {
            username: username.to_owned(),
        })
        .await?;

    // Step 2: SRP-6a handshake via proton-srp
    let srp_version = proton_srp::SrpHashVersion::try_from(info.version as u8)
        .map_err(|e| DriveError::Auth(e.to_string()))?;
    let srp_proof: proton_srp::SRPProofB64 = proton_srp::SRPAuth::with_pgp(
        Some(username),
        password.expose_secret(),
        srp_version,
        &info.salt,
        &info.modulus,
        &info.server_ephemeral,
    )
    .and_then(|srp| srp.generate_proofs())
    .map(proton_srp::SRPProofB64::from)
    .map_err(|e| DriveError::Auth(e.to_string()))?;

    // Step 3: send proof, get tokens
    let auth_resp = client
        .authenticate(&AuthRequest {
            username: username.to_owned(),
            client_ephemeral: srp_proof.client_ephemeral,
            client_proof: srp_proof.client_proof,
            srp_session: info.srp_session,
        })
        .await?;

    // Step 4: 2FA if required
    if let Some(tf_info) = &auth_resp.two_factor {
        if tf_info.enabled != 0 {
            let code = match two_factor.ok_or_else(|| DriveError::TwoFactorRequired)? {
                TwoFactorProvider::Totp(secret) => secret.expose_secret().to_owned(),
                TwoFactorProvider::WebAuthn(_) => {
                    return Err(DriveError::Auth("WebAuthn 2FA not yet implemented".into()))
                }
            };
            client.submit_two_factor(&TwoFactorRequest { code }).await?;
        }
    }

    client.set_access_token(&auth_resp.access_token);

    Ok(Session {
        uid: auth_resp.uid,
        user_id: auth_resp.user_id,
        access_token: SecretString::new(auth_resp.access_token.into()),
        refresh_token: SecretString::new(auth_resp.refresh_token.into()),
    })
}

/// Restore a previously serialized session.
pub fn resume_session(
    client: &ApiClient,
    session: crate::session::SerializedSession,
) -> Result<Session> {
    let session: Session = session.into();
    client.set_access_token(session.access_token());
    Ok(session)
}

/// Refresh an expired access token using the refresh token.
pub async fn refresh(client: &ApiClient, session: &mut Session) -> Result<()> {
    use protondrive_api::endpoints::auth::RefreshRequest;

    let resp = client
        .refresh_token(&RefreshRequest {
            refresh_token: session.refresh_token().to_owned(),
            uid: session.uid.clone(),
            response_type: "token".into(),
            grant_type: "refresh_token".into(),
            redirect_uri: "https://proton.me".into(),
        })
        .await?;

    session.access_token = SecretString::new(resp.access_token.clone().into());
    session.refresh_token = SecretString::new(resp.refresh_token.into());
    client.set_access_token(&resp.access_token);
    Ok(())
}

/// Revoke the session on the Proton servers.
pub async fn logout(client: &ApiClient, session: Session) -> Result<()> {
    client.logout().await?;
    client.clear_access_token();
    drop(session);
    Ok(())
}
