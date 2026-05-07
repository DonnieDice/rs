//! Thin insulation layer over `proton-crypto`.
//!
//! All `proton-crypto` types consumed by the SDK are re-exported from here
//! under SDK-owned aliases. If `proton-crypto`'s public API changes, this
//! module is the only blast radius.
#![forbid(unsafe_code)]

pub mod keys;
pub mod node;
pub mod blocks;

// SDK-owned type aliases for proton-crypto types we depend on.
// Rename here if upstream renames them.
pub use proton_crypto::crypto::PrivateKey;
pub use proton_crypto::crypto::PublicKey;
pub use proton_crypto::crypto::SessionKey;
pub use proton_crypto::crypto::Signature;
pub use proton_crypto::crypto::VerificationContext;
pub use proton_crypto_account::keys::AddressKeys;
pub use proton_crypto_account::keys::UserKeys;

pub use keys::{DecryptedNodeKey, NodeKeyBundle};
pub use blocks::{decrypt_block, encrypt_block, BlockMac};
pub use node::{decrypt_node_name, encrypt_node_name};

use protondrive_core::error::{DriveError, Result};

pub(crate) fn map_crypto_err(msg: impl std::fmt::Display) -> DriveError {
    DriveError::Crypto(msg.to_string())
}
