use crate::{
    map_crypto_err, DataEncoding, Decryptor, DecryptorSync, DetachedSignatureVariant, Encryptor,
    EncryptorSync, PGPProviderSync, SigningMode, VerifiedData, WritingMode,
};
use bytes::Bytes;
use protondrive_core::Result;
use sha2::{Digest, Sha256};
use std::io::Cursor;

/// SHA-256 hash of the encrypted block bytes used by Proton Drive block checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMac(pub [u8; 32]);

/// Decrypt a single 4 MiB (or smaller last-block) encrypted block.
///
/// Returns `(plaintext, encrypted_block_hash)`. Caller verifies the hash against
/// the revision manifest before trusting the plaintext.
pub fn decrypt_block<P>(
    ciphertext: &[u8],
    provider: &P,
    content_session_key: &P::SessionKey,
) -> Result<(Bytes, BlockMac)>
where
    P: PGPProviderSync,
{
    let encrypted_block_hash = compute_mac(ciphertext);
    let plaintext = provider
        .new_decryptor()
        .with_session_key_ref(content_session_key)
        .decrypt(ciphertext, DataEncoding::Bytes)
        .map_err(map_crypto_err)?
        .into_vec();

    Ok((Bytes::from(plaintext), encrypted_block_hash))
}

/// Encrypt a single plaintext block.
///
/// Returns `(ciphertext, encrypted_block_hash, encrypted_detached_signature)`.
/// Caller includes the encrypted block hash in the upload manifest.
pub fn encrypt_block<P>(
    plaintext: &[u8],
    provider: &P,
    content_session_key: &P::SessionKey,
    signing_key: &P::PrivateKey,
) -> Result<(Bytes, BlockMac, Vec<u8>)>
where
    P: PGPProviderSync,
{
    let mut ciphertext = Vec::new();
    let detached = provider
        .new_encryptor()
        .with_session_key_ref(content_session_key)
        .with_signing_key(signing_key)
        .encrypt_to_writer(
            Cursor::new(plaintext),
            DataEncoding::Bytes,
            SigningMode::Detached(DetachedSignatureVariant::Encrypted),
            WritingMode::All,
            &mut ciphertext,
        )
        .map_err(map_crypto_err)?;
    let signature = detached
        .try_into_detached_signature()
        .map_err(map_crypto_err)?;
    let mac = compute_mac(&ciphertext);

    Ok((Bytes::from(ciphertext), mac, signature))
}

fn compute_mac(data: &[u8]) -> BlockMac {
    let digest = Sha256::digest(data);
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&digest[..32]);
    BlockMac(arr)
}
