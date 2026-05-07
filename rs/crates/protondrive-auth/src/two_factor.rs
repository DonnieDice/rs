use secrecy::SecretString;

/// A 2FA challenge response.
pub enum TwoFactorProvider {
    /// Time-based one-time password (TOTP / authenticator app).
    Totp(SecretString),
    /// FIDO2 / WebAuthn — the consumer provides the signed assertion bytes.
    /// The GUI/app layer performs the actual user interaction.
    WebAuthn(Vec<u8>),
}

impl std::fmt::Debug for TwoFactorProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Totp(_) => f.debug_tuple("Totp").field(&"[redacted]").finish(),
            Self::WebAuthn(b) => f.debug_tuple("WebAuthn").field(&b.len()).finish(),
        }
    }
}
