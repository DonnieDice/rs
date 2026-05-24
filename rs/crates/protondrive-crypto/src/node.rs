use crate::{
    map_crypto_err, DataEncoding, Decryptor, DecryptorSync, Encryptor, EncryptorSync,
    PGPProviderSync, VerifiedData,
};
use protondrive_core::Result;

/// Decrypt an encrypted node name using the node's decrypted key.
pub fn decrypt_node_name<P>(
    encrypted_name: &str,
    provider: &P,
    node_key: &P::PrivateKey,
) -> Result<String>
where
    P: PGPProviderSync,
{
    let decrypted = provider
        .new_decryptor()
        .with_decryption_key(node_key)
        .decrypt(encrypted_name.as_bytes(), DataEncoding::Armor)
        .map_err(map_crypto_err)?
        .into_vec();
    String::from_utf8(decrypted).map_err(map_crypto_err)
}

/// Encrypt a node name for upload.
pub fn encrypt_node_name<P>(
    name: &str,
    provider: &P,
    encryption_key: &P::PrivateKey,
    signing_key: &P::PrivateKey,
) -> Result<String>
where
    P: PGPProviderSync,
{
    let public_key = provider
        .private_key_to_public_key(encryption_key)
        .map_err(map_crypto_err)?;
    let encrypted = provider
        .new_encryptor()
        .with_encryption_key(&public_key)
        .with_signing_key(signing_key)
        .encrypt_raw(name.as_bytes(), DataEncoding::Armor)
        .map_err(map_crypto_err)?;
    String::from_utf8(encrypted).map_err(map_crypto_err)
}
