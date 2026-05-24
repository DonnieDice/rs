use crate::{
    map_crypto_err, DataEncoding, Decryptor, DecryptorSync, PGPMessage, PGPProviderSync,
    VerifiedData,
};
use protondrive_core::Result;

/// A decrypted node key, ready for use in block operations.
pub struct DecryptedNodeKey<P: PGPProviderSync> {
    pub key: P::PrivateKey,
    pub passphrase_session_key: P::SessionKey,
}

impl<P: PGPProviderSync> std::fmt::Debug for DecryptedNodeKey<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecryptedNodeKey").finish_non_exhaustive()
    }
}

/// The on-wire bundle for a node key: the encrypted key + the passphrase session key.
#[derive(Debug, Clone)]
pub struct NodeKeyBundle {
    pub encrypted_key: String,
    pub key_passphrase: String,
    pub key_passphrase_signature: String,
}

impl NodeKeyBundle {
    /// Decrypt this bundle using the same OpenPGP flow as the JavaScript SDK's
    /// `DriveCrypto.decryptKey`.
    pub fn decrypt<P>(
        &self,
        provider: &P,
        decryption_keys: &[P::PrivateKey],
        verification_keys: &[P::PublicKey],
    ) -> Result<DecryptedNodeKey<P>>
    where
        P: PGPProviderSync,
    {
        let passphrase_message = provider
            .pgp_message_import(self.key_passphrase.as_bytes(), DataEncoding::Armor)
            .map_err(map_crypto_err)?;
        let passphrase_session_key = provider
            .new_decryptor()
            .with_decryption_keys(decryption_keys.iter())
            .decrypt_session_key(passphrase_message.as_key_packets())
            .map_err(map_crypto_err)?;

        let mut decryptor = provider
            .new_decryptor()
            .with_session_key_ref(&passphrase_session_key);
        if !verification_keys.is_empty() {
            decryptor = decryptor.with_verification_keys(verification_keys.iter());
        }
        if !self.key_passphrase_signature.is_empty() {
            decryptor = decryptor.with_detached_signature_ref(
                self.key_passphrase_signature.as_bytes(),
                crate::DetachedSignatureVariant::Encrypted,
                true,
            );
        }

        let passphrase = decryptor
            .decrypt(self.key_passphrase.as_bytes(), DataEncoding::Armor)
            .map_err(map_crypto_err)?;
        let passphrase = String::from_utf8(passphrase.into_vec()).map_err(map_crypto_err)?;
        let key = provider
            .private_key_import(
                self.encrypted_key.as_bytes(),
                passphrase.as_bytes(),
                DataEncoding::Armor,
            )
            .map_err(map_crypto_err)?;

        Ok(DecryptedNodeKey {
            key,
            passphrase_session_key,
        })
    }
}
