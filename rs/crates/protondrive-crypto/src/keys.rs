use crate::{map_crypto_err, AddressKeys, PrivateKey, SessionKey};
use protondrive_core::Result;
use zeroize::ZeroizeOnDrop;

/// A decrypted node key, ready for use in block operations.
#[derive(ZeroizeOnDrop)]
pub struct DecryptedNodeKey {
    pub(crate) inner: SessionKey,
}

impl std::fmt::Debug for DecryptedNodeKey {
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
    /// Decrypt this bundle using the share key or parent node key.
    pub fn decrypt(&self, decryption_key: &PrivateKey) -> Result<DecryptedNodeKey> {
        let session_key = proton_crypto::crypto::decrypt_session_key(
            decryption_key,
            &self.key_passphrase,
        )
        .map_err(|e| map_crypto_err(e))?;

        Ok(DecryptedNodeKey { inner: session_key })
    }

    /// Decrypt using address keys (for share-level roots).
    pub fn decrypt_with_address_keys(&self, keys: &AddressKeys) -> Result<DecryptedNodeKey> {
        let private = keys
            .primary_private_key()
            .ok_or_else(|| map_crypto_err("no primary address key"))?;
        self.decrypt(private)
    }
}
