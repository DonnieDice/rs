use crate::{map_crypto_err, DecryptedNodeKey};
use protondrive_core::Result;

/// Decrypt an encrypted node name using the node's decrypted key.
pub fn decrypt_node_name(encrypted_name: &str, key: &DecryptedNodeKey) -> Result<String> {
    proton_crypto::crypto::decrypt_message_with_session_key(&key.inner, encrypted_name)
        .map_err(|e| map_crypto_err(e))
}

/// Encrypt a node name for upload.
pub fn encrypt_node_name(name: &str, key: &DecryptedNodeKey) -> Result<String> {
    proton_crypto::crypto::encrypt_message_with_session_key(&key.inner, name)
        .map_err(|e| map_crypto_err(e))
}
