use crate::{map_crypto_err, DecryptedNodeKey};
use bytes::Bytes;
use protondrive_core::Result;

/// SHA-256 MAC of a decrypted block, used for checksum verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMac(pub [u8; 32]);

/// Decrypt a single 4 MiB (or smaller last-block) encrypted block.
///
/// Returns `(plaintext, mac)`. Caller verifies mac against the manifest.
pub fn decrypt_block(
    ciphertext: &[u8],
    key: &DecryptedNodeKey,
    block_index: usize,
) -> Result<(Bytes, BlockMac)> {
    let plaintext = proton_crypto::crypto::decrypt_block_with_session_key(
        &key.inner,
        ciphertext,
        block_index,
    )
    .map_err(|e| map_crypto_err(e))?;

    let mac = compute_mac(&plaintext);
    Ok((Bytes::from(plaintext), mac))
}

/// Encrypt a single plaintext block.
///
/// Returns `(ciphertext, mac)`. Caller includes mac in the upload manifest.
pub fn encrypt_block(
    plaintext: &[u8],
    key: &DecryptedNodeKey,
    block_index: usize,
) -> Result<(Bytes, BlockMac)> {
    let mac = compute_mac(plaintext);
    let ciphertext = proton_crypto::crypto::encrypt_block_with_session_key(
        &key.inner,
        plaintext,
        block_index,
    )
    .map_err(|e| map_crypto_err(e))?;

    Ok((Bytes::from(ciphertext), mac))
}

fn compute_mac(data: &[u8]) -> BlockMac {
    use std::hash::Hasher;
    // proton-crypto exposes sha256; fallback placeholder until API is confirmed
    let digest = proton_crypto::crypto::sha256(data);
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&digest[..32]);
    BlockMac(arr)
}
