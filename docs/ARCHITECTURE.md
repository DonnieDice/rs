# Architecture

## Crate Dependency Graph

```
protondrive-core   ──→  types, errors, opaque IDs (serde + thiserror only)
     │
     ├──→ protondrive-crypto   ──→  OpenPGP insulation over proton-crypto
     │                                   (re-exports, block/node keys)
     │
     ├──→ protondrive-api     ──→  ApiClient, endpoint bindings, retry/backoff
     │
     ├──→ protondrive-auth    ──→  SRP login, Session/SerializedSession, token refresh
     │
     ├──→ protondrive-events  ──→  EventStream (futures::Stream), DriveEvent types
     │
     ├──→ protondrive-sdk     ──→  High-level facade: ProtonDrive, ProtonDriveBuilder
     │
     └──→ protondrive-ffi     ──→  C ABI; **only** crate where `unsafe_code` is permitted
```

Dependency arrows flow from consumer to provider. Each crate consumes only the
crates below it. The rule:

- `protondrive-core` — zero internal deps (outside serde, thiserror, uuid).
- `protondrive-crypto`, `protondrive-api`, `protondrive-auth`, `protondrive-events` — depend on `core` only.
- `protondrive-sdk` — depends on `api`, `auth`, `events`, `crypto`; re-exports `core` types.
- `protondrive-ffi` — depends on `sdk`; the one escape hatch for C linkage.

### External dependency chain for crypto

```
proton-srp (0.8.2)        ─→  SRP-6a proof computation (used by protondrive-auth)
proton-crypto (0.12.1)    ─→  OpenPGP: keys, sign/verify, encrypt/decrypt (used by protondrive-crypto)
proton-crypto-account (0.18.1) ─→  Proton account key structures (AddressKeys, UserKeys)
```

All three are Proton-published crates (not intended for general external usage,
per their READMEs). The `protondrive-crypto` crate provides the SDK's insulation
layer so that upstream API changes only touch one file.

---

## Typical Call Flow

### Builder → Login → List Volumes → List Children

```
ProtonDrive::builder()          ──→  ProtonDriveBuilder::new()
    .app_version("app@1.0")         optional, default "external-drive-linux@0.1.0-alpha"
    .user_agent("MyApp/1.0")        optional, default "ProtonDrive-Rust/0.1.0"
    .session(session)                required (from login())
    .build()                         creates SdkConfig → ApiClient → ProtonDrive

ProtonDrive { client, session }
    .list_volumes()              GET /drive/v2/shares/my-files
        → Vec<Volume>            { id, share_id, state: Active | Deleted | Locked }

    .list_children(&share_id, parent?)
        GET /drive/v2/shares/{share}/links (or ?Page={parent})
        → Vec<Node>

    .get_node(&share_id, &node_id)
        GET /drive/v2/shares/{share}/links/{link}
        → Node
```

### Volume → Share mapping

A `Volume` response contains a `share_id` from which all subsequent node
operations are derived. Callers use:

```rust
let volumes = drive.list_volumes().await?;
let share = &volumes[0].share_id;
let root = drive.list_children(share, None).await?;
```

---

## Auth Flow (SRP-6a)

```
User supplies: username + password + optional 2FA

  1.  GET  /auth/v4/info      ──→  AuthInfoResponse
      { body: { Username } }       { Modulus, ServerEphemeral, Version, Salt, SRPSession }

  2.  proton_srp::SRPAuth::with_pgp(username, password, version, salt, modulus, server_ephemeral)
      ├──  hash password with salt
      ├──  generate client ephemeral (A = g^a mod N)
      └──  compute client proof (M = H(N xor g, H(I:s), A, B, K))
      → SRPProofB64 { client_ephemeral, client_proof }

  3.  POST /auth/v4            ──→  AuthResponse
      { Username, ClientEphemeral, ClientProof, SRPSession }
                                    { AccessToken, RefreshToken, UID, UserID, TwoFactor? }

  4a. IF two_factor.enabled:
      POST /auth/v4/2fa        ──→  200 OK
      { TwoFactorCode }

  4b.  ApiClient::set_access_token(access_token)
       → Session { uid, user_id, access_token (SecretString), refresh_token (SecretString) }

Token refresh (on SessionExpired):
  POST /auth/v4/refresh
  { RefreshToken, UID, ResponseType, GrantType, RedirectURI }
  → new access_token + refresh_token → updates Session + ApiClient

Session serialization:
  Session::serialize() → SerializedSession (JSON-serializable, no SecretString)
  // Consumer MUST encrypt before persisting (e.g. OS keyring).
  resume_session(client, serialized) → Session (re-creates SecretStrings, sets token on client)
```

---

## Event Loop

```
1. let latest = drive.latest_event_id(&share).await?;
   GET /drive/v2/shares/{share}/events/latest → { EventID }

2. let stream = drive.subscribe_events(&share, latest).await?;
   → EventStream { client, share_id, cursor, buffer, has_more, fetch }

3.  while let Some(event) = stream.next().await {
        match event? {
            DriveEvent { event_id, share_id, action }
            // action: NodeUpdated{node_id, payload}
            //         NodeDeleted{node_id}
            //         NodeTrashed{node_id}
            //         NodeRestored{node_id}
            //         Unknown{raw_action}
        }
    }

Internal poll loop (futures::Stream):
  ┌─ poll_next()
  │   buffer has items? → yield item
  │   fetch in flight?  → poll it
  │   need new fetch?   → start async GET /drive/v2/shares/{share}/events/{cursor}
  │                       response: { EventID, More, Events[] }
  │                       set cursor = next EventID
  │                       push events onto buffer
  │                       has_more = (more != 0)
  │   buffer empty?     → wake waker, return Pending (caller should re-poll after delay)
  └────────────────────────────────────────────────────────────────

On gap detected: DriveError::EventStreamGap { latest_event_id }
  → consumer must re-sync from the provided latest_event_id.
```

---

## Upload Flow

> **Status: Phase 2 — not yet implemented.** The stub reads and discards
> the input stream to prove the API signature compiles end-to-end.

```
caller:  drive.upload(&parent, "photo.jpg", file_reader, UploadOptions { concurrency: 4, .. })

planned flow:
  ┌─ Read file content in 4 MiB blocks (BLOCK_SIZE = 4 × 1024 × 1024)
  ├─ Create revision via API
  ├─ For each block:
  │   ├─ encrypt block with protondrive-crypto::encrypt_block
  │   └─ POST to storage URL (ApiClient::post_block) with pm-storage-token header
  ├─ (concurrent uploads controlled by UploadOptions.concurrency)
  └─ Commit revision via API
```

UploadOptions supports `concurrency` (parallel block uploads, default 4) and
optional `mime_type`.

---

## Download Flow

> **Status: Phase 1 — not yet implemented.** The stub returns
> `DriveError::Auth("download not yet implemented")`.

```
Type: DownloadStream = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>

caller:  let mut stream = drive.download(&node_id).await?;
         pin_mut!(stream);
         while let Some(chunk) = stream.next().await {
             let bytes = chunk?;
             // write to file / process
         }

planned flow:
  ┌─ Fetch block metadata URLs from API
  ├─ For each block:
  │   ├─ GET from storage URL (ApiClient::get_block) with pm-storage-token
  │   └─ decrypt block with protondrive-crypto::decrypt_block
  └─ Yield decrypted Bytes via Stream
```

---

## Crypto Layer (`protondrive-crypto`)

This crate is a **thin re-export / insulation shim** over `proton-crypto` (0.12.1)
and `proton-crypto-account` (0.18.1).

### What it wraps

| Upstream crate | Key exports forwarded |
|---|---|
| `proton_crypto::crypto` | `PGPMessage`, `PrivateKey`, `PublicKey`, `SessionKey`, `SessionKeyAlgorithm` |
|  | `Encryptor`/`EncryptorSync`, `Decryptor`/`DecryptorSync` |
|  | `PGPProviderSync`, `DataEncoding`, `DetachedSignatureVariant` |
|  | `SigningMode`, `VerificationContext`, `VerifiedData`, `WritingMode` |
| `proton_crypto_account::keys` | `AddressKeys`, `UserKeys` |

### SDK-provided operations

- **`NodeKeyBundle::decrypt<P>(provider, decryption_keys, verification_keys)`**
  Decrypts the node key passphrase (PGP message → session key → decrypted
  passphrase → import private key). Mirrors the JS SDK's `DriveCrypto.decryptKey`.

- **`encrypt_block` / `decrypt_block` + `BlockMac`**
  Per-block encryption with MAC verification for storage integrity.

- **`encrypt_node_name` / `decrypt_node_name`**
  Node name obfuscation at rest.

### Why separate from `proton-crypto`?

- If `proton-crypto`'s API changes, only `protondrive-crypto/src/lib.rs` needs
  updating — not every consumer in `protondrive-sdk`.
- SDK-owned aliases prevent type confusion if two downstream crates pin
  different `proton-crypto` patch versions.

### Backend selection (feature flags)

- `gopgp` **(default)** — Go-backed OpenPGP via `proton-crypto/gopgp`.
  Requires Go toolchain. Bit-exact with official Proton clients.
- `rustpgp` — pure Rust via `proton-rpgp` through `proton-crypto/rustpgp`.
  No Go dependency. Used for self-contained builds (e.g. AppImage).

The SDK has **no** custom `CryptoBackend` trait — `proton-crypto`'s trait API
is the abstraction. The feature flag passthrough is the only knob.

---

## Error Propagation Model

```
protondrive_core::error::DriveError
```

A single error enum for the entire SDK:

| Variant | Trigger | Retryable? |
|---|---|---|
| `Auth(String)` | SRP handshake failure, bad password | no |
| `Api { code, message }` | Non-2xx API response (unrecognised status) | no |
| `Network(String)` | reqwest transport error (DNS, TLS, timeout) | yes (backon) |
| `Serialization` | JSON parse error (`#[from] serde_json::Error`) | no |
| `Crypto(String)` | PGP / key operation failure | no |
| `NotFound` | 404 | no |
| `PermissionDenied` | 403 | no |
| `RateLimited { retry_after_secs }` | 429 | yes (backon) |
| `EventStreamGap { latest_event_id }` | Event stream has holes | consumer re-sync |
| `ChecksumMismatch { block_index }` | Block integrity failure | no |
| `InvalidAppVersion` | Bad app version string in SdkConfig | no |
| `SessionExpired` | 401 | yes (refresh token) |
| `TwoFactorRequired` | Login needs 2FA but none provided | no |
| `Other(anyhow::Error)` | Catch-all for non-critical errors | varies |

The `Result<T, E = DriveError>` alias is the standard return type across all
public API surfaces.

### Retry policy (ApiClient)

`execute_with_retry` wraps every metadata request:
- Exponential backoff via `backon` crate.
- Only retries on `RateLimited` or `Network` errors.
- Storage requests (`get_block`, `post_block`) do **not** retry at this layer.

---

## Threading Model

- All public methods are **async** (tokio runtime).
- Upload uses a configurable **concurrency window** (default 4 parallel block
  uploads).
- Download returns a **streaming** `DownloadStream` — backpressure via
  `futures::Stream`.
- No blocking calls in the async paths.

---

## Security Invariants

- `SecretString` (from `secrecy` crate, zeroized on drop) for all passwords,
  tokens, and key material.
- No secrets in `Debug` or `Display` output.
- `#![forbid(unsafe_code)]` on every crate except `protondrive-ffi`.
- Server and network are treated as hostile; only the local Rust process is
  trusted.
- `Session::serialize()` produces `SerializedSession` (plaintext). The
  **consumer is responsible** for encrypting before persisting.
