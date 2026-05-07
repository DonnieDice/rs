# `protondrive-sdk-rs`

> The Rust SDK for Proton Drive. A community-built, Proton-Foundation-coordinated Drive client library, layered on top of Proton's official `proton-crypto-rs` crates. Pure library: no GUI, no filesystem, no persistence. Consumed by [`protondrive-linux`](https://github.com/DonnieDice/protondrive-linux) and any other Rust/FFI client (Swift, Kotlin, C#, headless daemons, NAS firmware).

This directory (`rs/`) lives inside the [`DonnieDice/rs`](https://github.com/DonnieDice/rs) fork of the official Proton Drive SDK repo, alongside the upstream `js/` workspace. The `rs/` directory is its own self-contained Cargo workspace and is the primary deliverable of this project.

---

## Status

| Phase | Scope | Status |
|---|---|---|
| **Phase 0 — Skeleton** | Workspace, all crates, API bindings, login, list volumes/children | ✅ Done |
| **Phase 1 — Read path** | Streaming downloads, event subscription, signature verify | 🔜 Q2 2026 |
| **Phase 2 — Write path** | Streaming uploads, chunking + encryption, rename/move/trash, revisions | 🔜 Q3 2026 |
| **Phase 3 — Parity** | Sharing, photos, full official-SDK feature parity | 🔜 Q3 2026 |
| **Phase 4 — FFI** | C ABI for Swift/Kotlin/C# consumers; recognized SDK status pursued | 🔜 Q4 2026 |
| **Phase 5 — Crypto migration** | Track `proton-crypto`'s new crypto model; update insulation layer | Aligned with Proton |
| **Phase 6 — 1.0** | API freeze, security audit, public crates.io release | 🔜 Q1 2027 |

---

## Mission

Build the Rust SDK for Proton Drive that:

1. Reaches feature parity with the official TypeScript/C# Drive SDK
2. Layers cleanly on top of `proton-crypto`, `proton-crypto-account`, and `proton-srp` — Proton's existing Rust crypto foundation — without reinventing any of it
3. Provides the Drive-specific business logic: API client, events, sharing, photos, revisions, streaming chunked upload/download
4. Fills the auth gap the official Drive SDK explicitly excludes, by composing `proton-srp` with Drive-specific session flows
5. Delivers headless/B2B capabilities (daemon-friendly, NAS-friendly) that the official roadmap deprioritizes
6. Is positioned for formal "recognized community Drive SDK" status with the Proton Foundation

---

## Workspace Layout

```
rs/
├── Cargo.toml                          # workspace root; all shared dependency versions
├── rust-toolchain.toml                 # MSRV pin (1.75)
├── deny.toml                           # cargo-deny license + advisory config
├── crates/
│   ├── protondrive-core/               # types, errors, opaque IDs (NodeId, ShareId, EventId…)
│   ├── protondrive-crypto/             # thin facade over proton-crypto; SDK-owned type aliases
│   ├── protondrive-api/                # HTTP client, endpoint bindings, retry/backoff, rate limit
│   ├── protondrive-auth/               # Drive login flow: wraps proton-srp + 2FA + session mgmt
│   ├── protondrive-events/             # event stream subscription + replay
│   ├── protondrive-sdk/                # umbrella high-level API (re-exports from above)
│   └── protondrive-ffi/                # C ABI for Swift/Kotlin/C# consumers (Phase 4)
├── examples/
│   ├── login_and_list.rs               # SRP login → list volumes → list root
│   ├── upload_stream.rs                # streaming upload (Phase 2)
│   ├── download_stream.rs              # streaming download (Phase 1)
│   └── event_subscription.rs          # subscribe to Drive event stream
├── docs/
│   ├── ARCHITECTURE.md                 # proton-crypto-rs dependency boundary + insulation layer
│   ├── COMPLIANCE.md                   # Proton requirements → code location mapping
│   ├── PROTOCOL.md                     # Drive wire format, event semantics (Phase 1)
│   ├── THREAT_MODEL.md                 # threat model document (Phase 1)
│   ├── LICENSES.md                     # transitive license inventory
│   └── ROADMAP.md                      # extended roadmap details
└── .github/workflows/
    └── ci.yml                          # build / test / clippy / fmt / deny / audit matrix
```

**Every crate is `#![forbid(unsafe_code)]` except `protondrive-ffi`**, which requires unsafe for C ABI boundary work and is explicitly excluded.

---

## Foundation Dependency: `proton-crypto-rs`

The SDK consumes — and does **not** reimplement — the following Proton-published crates:

| Crate | Role in this SDK |
|---|---|
| `proton-crypto` | All Drive OpenPGP operations: node key decryption, block encryption/decryption, signature sign/verify |
| `proton-crypto-account` | User keys, address keys, key management (re-exports `proton-crypto`) |
| `proton-srp` | SRP-6a login handshake against Proton's auth server |
| `proton-crypto-subtle` | AEAD, HKDF primitives where Drive needs them outside OpenPGP framing |
| `proton-rpgp` | Used transitively via `proton-crypto`'s `rustpgp` feature; not depended on directly |
| `gopenpgp-sys` | Used transitively via `proton-crypto`'s `gopgp` feature; not depended on directly |

### Crypto backend selection

`proton-crypto` exposes the GopenPGP-vs-pure-Rust choice via Cargo features. This SDK passes it through unchanged:

| Feature | Backend | Build requirement | Use case |
|---|---|---|---|
| `gopgp` *(default)* | Go-backed GopenPGP | Go toolchain required | Bit-exact Proton parity; B2B builds |
| `rustpgp` | Pure Rust via `proton-rpgp` | No Go toolchain | AppImage / embedded builds |

The SDK does **not** invent its own `CryptoBackend` trait — `proton-crypto`'s API is the abstraction.

### Insulation layer

`crates/protondrive-crypto` re-exports every `proton-crypto` type the SDK depends on under SDK-owned type aliases. If `proton-crypto`'s public API changes (it is explicitly *"not vetted for general usage outside Proton"*), the blast radius is one file, not the entire SDK surface.

---

## Quick Start

```toml
# Cargo.toml
[dependencies]
protondrive-sdk = { git = "https://github.com/DonnieDice/rs", features = ["rustpgp"] }
```

```rust
use protondrive_api::{client::ApiClient, config::SdkConfig};
use protondrive_auth::{login, TwoFactorProvider};
use protondrive_sdk::ProtonDrive;
use secrecy::SecretString;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = SdkConfig::new(
        "external-drive-linux@0.1.0-alpha",
        "ProtonDrive-Rust/0.1.0",
    )?;
    let client = ApiClient::new(config)?;

    let session = login(
        &client,
        "alice@proton.me",
        SecretString::new("hunter2".into()),
        None, // or Some(TwoFactorProvider::Totp(code))
    )
    .await?;

    let drive = ProtonDrive::builder().session(session).build()?;

    for volume in drive.list_volumes().await? {
        let children = drive.list_children(&volume.share_id, None).await?;
        for node in children {
            println!("[{:?}] {}", node.kind, node.name);
        }
    }

    Ok(())
}
```

---

## Public API Surface

The high-level surface in `protondrive-sdk` mirrors the official TS/C# Drive SDK shape so consumers can swap implementations:

```rust
impl ProtonDrive {
    pub fn builder() -> ProtonDriveBuilder;

    // Volumes & navigation
    pub async fn list_volumes(&self) -> Result<Vec<Volume>>;
    pub async fn list_children(&self, share: &ShareId, parent: Option<&NodeId>) -> Result<Vec<Node>>;
    pub async fn get_node(&self, share: &ShareId, id: &NodeId) -> Result<Node>;

    // Streaming file I/O — never buffers whole files in memory
    pub async fn upload<R: AsyncRead + Unpin + Send>(
        &self, parent: &NodeId, name: &str, content: R, opts: UploadOptions,
    ) -> Result<NodeId>;
    pub async fn download(&self, node: &NodeId) -> Result<impl AsyncRead + Unpin + Send>;

    // Mutations
    pub async fn rename(&self, share: &ShareId, node: &NodeId, new_name: &str) -> Result<()>;
    pub async fn move_node(&self, share: &ShareId, node: &NodeId, new_parent: &NodeId, name: &str) -> Result<()>;
    pub async fn trash(&self, share: &ShareId, node: &NodeId) -> Result<()>;
    pub async fn delete(&self, share: &ShareId, node: &NodeId) -> Result<()>;

    // Sharing
    pub async fn create_share(&self, node: &NodeId, opts: ShareOptions) -> Result<Share>;
    pub async fn list_shares(&self) -> Result<Vec<Share>>;

    // Photos
    pub async fn list_photos(&self, share: &ShareId) -> Result<Vec<Photo>>;

    // Revisions
    pub async fn list_revisions(&self, share: &ShareId, node: &NodeId) -> Result<Vec<Revision>>;
    pub async fn restore_revision(&self, share: &ShareId, node: &NodeId, rev: &RevisionId) -> Result<()>;

    // Events — the only sync mechanism; no polling, no recursive walks
    pub async fn latest_event_id(&self, share: &ShareId) -> Result<EventId>;
    pub async fn subscribe_events(&self, share: &ShareId, since: EventId) -> Result<EventStream>;
}
```

---

## Authentication (`protondrive-auth`)

The official Proton Drive SDK explicitly excludes auth. This crate is a thin Drive-specific orchestration layer over `proton-srp` — no SRP reimplementation, no spec re-derivation.

```rust
// SRP-6a login via proton-srp + optional 2FA
pub async fn login(
    client: &ApiClient,
    username: &str,
    password: SecretString,
    two_factor: Option<TwoFactorProvider>,
) -> Result<Session>

// Restore a previously serialized session (consumer handles storage/encryption)
pub fn resume_session(client: &ApiClient, token: SerializedSession) -> Result<Session>

// Refresh an expired access token
pub async fn refresh(client: &ApiClient, session: &mut Session) -> Result<()>

// Revoke session on Proton servers
pub async fn logout(client: &ApiClient, session: Session) -> Result<()>
```

`Session` and all secret material use `secrecy::SecretString` — zeroized on drop, redacted in `Debug`. The SDK **never persists secrets**; `SerializedSession` is handed to the consumer for them to encrypt and store via OS keyring or equivalent.

### 2FA support

| Method | Status |
|---|---|
| TOTP (authenticator app) | ✅ `TwoFactorProvider::Totp(code)` |
| WebAuthn / FIDO2 | 🔜 `TwoFactorProvider::WebAuthn(assertion)` — consumer provides signed bytes; GUI layer drives UX |

---

## Streaming I/O

- **Upload**: chunked at 4 MiB; configurable parallel block concurrency (default 4). Each block is encrypted with `proton-crypto` before transmission. The consumer streams an `AsyncRead` — no buffering of the whole file.
- **Download**: streaming `AsyncRead` with backpressure. Blocks are decrypted on the fly via `proton-crypto`. MAC verified per block against the revision manifest before bytes are yielded.
- **Zero-copy**: `bytes::Bytes` throughout; decrypt-in-place where `proton-crypto` allows.
- **Hardware AES-NI**: arrives automatically through `proton-crypto`'s backends — no SDK-level intrinsics.

---

## Hard Compliance Constraints

These are enforced in code or CI — not just policy:

| Constraint | Enforcement |
|---|---|
| Official Proton endpoints only; no proxy/rewrite | `PROTON_API_BASE` is a `const`; no runtime override path exists |
| `x-pm-appversion` on every request | Added in `ApiClient::build_request`; validated at `SdkConfig` construction |
| `x-pm-appversion` canonical regex unit-tested | `protondrive-api/src/config.rs#[cfg(test)]` |
| Event-based sync only | No `list_all_recursive`; only `subscribe_events` + `latest_event_id` exist |
| No password storage | `SecretString` only; zeroized on drop; SDK never writes secrets to disk |
| No Proton branding in this repo | Code review checklist; CI has no logo/mark assets |
| All license obligations satisfied | `cargo-deny` allow-list in `deny.toml`; blocking CI job |

### Canonical `x-pm-appversion` regex

```
/^(external-drive)+(-[a-z_]+)+@[0-9]+\.[0-9]+\.[0-9]+(\.[0-9]+)?-((stable|beta|RC|alpha)(([.-]?\d+)*)?)?([.-]?dev)?(\+.*)?$/i
```

Valid examples: `external-drive-linux@1.0.0-stable`, `external-drive-linux@0.1.0-alpha.1`

---

## Security

- `#![forbid(unsafe_code)]` on all crates except `protondrive-ffi`
- `secrecy::SecretString` for all passwords, tokens, and key material at the SDK boundary
- `zeroize` on drop for `Session`, `DecryptedNodeKey`, and `SerializedSession`
- Constant-time comparisons via `subtle` where needed
- No key or token material ever appears in `Debug` or `Display` output
- SBOM via `cargo-cyclonedx` on every release
- `cargo-deny` + `cargo-audit` are blocking CI steps
- **Threat model**: server is hostile, network is hostile, only the local Rust process is trusted
- `proton-crypto`'s `SECURITY.md` is the upstream reporting path for crypto vulnerabilities in the foundation layer
- SDK-level security issues: open a GitHub Security Advisory in this repo

---

## Testing

```bash
# Unit tests (no network, no credentials)
cargo test --features rustpgp

# Both backend variants (CI runs both)
cargo test --no-default-features --features gopgp
cargo test --no-default-features --features rustpgp

# Integration tests (requires Proton staging credentials)
PROTON_USERNAME=... PROTON_PASSWORD=... \
  cargo test --features rustpgp,integration-tests

# Fuzz (Phase 1+)
cargo +nightly fuzz run fuzz_api_json
cargo +nightly fuzz run fuzz_event_stream
```

**Testing stack:**
- `wiremock` — HTTP endpoint mocking for unit tests; no real network required
- `proptest` — property-based tests for serialization round-trips and event-stream invariants
- `proton-crypto` test vectors — used directly for crypto verification; we don't maintain our own
- CI matrix: Linux × macOS × Windows, stable × MSRV × nightly (nightly is allowed-to-fail)
- Both `gopgp` and `rustpgp` features tested in every CI run

---

## Building

### Prerequisites

| Toolchain | When needed |
|---|---|
| Rust 1.75+ (MSRV) | Always |
| Go 1.21+ | Only for `--features gopgp` (default) |
| No Go needed | `--no-default-features --features rustpgp` |

```bash
# Pure-Rust build (no Go required)
cd rs
cargo build --no-default-features --features rustpgp

# Full build with GopenPGP (requires Go in PATH)
cargo build --features gopgp

# Check MSRV
rustup override set 1.75
cargo check --no-default-features --features rustpgp
```

---

## Detailed Roadmap

### Phase 0 — Skeleton ✅ Q2 2026

- [x] Cargo workspace with all 7 crates
- [x] `protondrive-core`: opaque IDs, all domain types, `DriveError`
- [x] `protondrive-crypto`: insulation layer; SDK-owned type aliases over `proton-crypto`
- [x] `protondrive-api`: `ApiClient`, `SdkConfig` with `x-pm-appversion` validation, all read/write endpoint bindings, exponential backoff retry
- [x] `protondrive-auth`: SRP-6a login via `proton-srp`, TOTP 2FA, `Session`/`SerializedSession`
- [x] `protondrive-events`: `EventStream` (`futures::Stream` impl), `DriveEvent`, `EventAction`
- [x] `protondrive-sdk`: `ProtonDrive` high-level API, `ProtonDriveBuilder`
- [x] `protondrive-ffi`: cdylib/staticlib skeleton (Phase 4 placeholder)
- [x] Examples: `login_and_list`, `event_subscription`
- [x] CI: build/test/clippy/fmt/deny/audit on Linux×macOS×Windows, both feature sets
- [x] `docs/ARCHITECTURE.md`, `docs/COMPLIANCE.md`

### Phase 1 — Read path 🔜 Q2 2026

- [ ] Streaming download: fetch block token URLs → parallel decrypt via `proton-crypto` → `AsyncRead`
- [ ] Per-block MAC verification against revision manifest
- [ ] Signature verification on node names and revision manifests
- [ ] Event stream polling with configurable backoff; gap detection + re-sync signal
- [ ] `docs/PROTOCOL.md`: Drive wire format, block structure, event semantics
- [ ] `docs/THREAT_MODEL.md`
- [ ] `examples/download_stream.rs`
- [ ] Fuzz targets: API JSON parser, event stream parser

### Phase 2 — Write path 🔜 Q3 2026

- [ ] Streaming upload: encrypt in 4 MiB blocks via `proton-crypto`, parallel block upload, commit revision
- [ ] Node key generation for new files and folders
- [ ] Rename / move with re-encrypted node name
- [ ] Trash / delete
- [ ] Revision management (create, supersede, restore)
- [ ] Name conflict detection and version-vector emission
- [ ] `examples/upload_stream.rs`

### Phase 3 — Feature parity 🔜 Q3 2026

- [ ] Sharing: create public links, share keys, permission management
- [ ] Photos: list, album management, thumbnail handling, `savePhotosToTimeline`
- [ ] Full pagination on all list endpoints
- [ ] Drive scope verification on login (does account have Drive access?)
- [ ] `docs/LICENSES.md` transitive inventory
- [ ] SBOM output via `cargo-cyclonedx`

### Phase 4 — FFI 🔜 Q4 2026

- [ ] C ABI in `protondrive-ffi`: opaque handle types, error codes, async bridge via `tokio::runtime::Handle`
- [ ] Swift bindings (via `swift-bridge` or manual `cbindgen` headers)
- [ ] Kotlin/JNI bindings
- [ ] Pursue "recognized community Drive SDK" status with Proton Foundation

### Phase 5 — Crypto-model migration (Aligned with Proton)

- [ ] Track `proton-crypto`'s adoption of Proton's announced breaking crypto model changes
- [ ] SDK insulation layer (`protondrive-crypto`) absorbs the type changes
- [ ] No consumer-facing API breakage required

### Phase 6 — 1.0 🔜 Q1 2027

- [ ] API freeze
- [ ] External security audit
- [ ] Publish all crates to crates.io (pending foundation blessing on `proton-crypto-rs` consumption)
- [ ] `CHANGELOG.md` semver-tracked from 0.1.0

---

## Proton Foundation Coordination

This project is pursuing the following asks with the Proton Foundation:

| Ask | Priority | Status |
|---|---|---|
| **Explicit blessing to consume `proton-crypto-rs` publicly** — repo states "not intended for general usage outside Proton" | **Critical** | Pending |
| Semver/deprecation stability commitment for `proton-crypto`, `proton-crypto-account`, `proton-srp` | High | Pending |
| Recognized "community Drive SDK" status | High | Pending |
| Drive protocol clarifications (event stream semantics, share key flows, photo metadata format) | High | Pending |
| Bug-report channel for `proton-crypto-rs` issues | Medium | Pending |
| Early access to upcoming breaking crypto-model spec | Medium | Pending |
| Quarterly roadmap coordination to avoid duplicate work | Medium | Pending |

> **The first ask is critical.** Without it, the SDK consumes code Proton has signaled isn't for outside use. This is fragile ground regardless of legality. The insulation layer limits blast radius if API stability isn't granted, and Plan B (clean-room reimplementation via the `srp` crate + `rpgp` directly) remains viable if the blessing is declined.

---

## Out of Scope

The SDK explicitly does **not** ship the following. These belong in the consuming application (`protondrive-linux` or similar):

- SQLite / SQLCipher path mapping
- Filesystem watching (`notify` crate)
- Conflict resolution policy — the SDK emits version vectors and signatures; the app decides
- Daemon / `systemd` / launchd integration
- GUI primitives or framework bridge code
- Password storage / OS keyring integration
- Any user-facing strings — the SDK returns structured `DriveError`s; the app localizes them

---

## Risks

| Risk | Mitigation |
|---|---|
| `proton-crypto-rs` API breaks unannounced | Insulation layer in `protondrive-crypto` limits blast radius to one module |
| Foundation declines public consumption blessing | Plan B: clean-room via `srp` crate + `rpgp` directly |
| `proton-crypto-rs` accepts no external contributions | Bug-report channel via foundation ask; fork-and-patch as last resort |
| Proton's announced breaking crypto change | `proton-crypto` adopts it upstream; SDK insulation layer absorbs type changes |
| `gopgp` cross-compilation friction (Go required) | Default to `rustpgp` for AppImage; offer `gopgp` for B2B |
| API instability during Proton's SDK alpha | Pin against stable endpoints; integration tests gate releases |
| License contamination from reading GPL Proton clients | Default GPLv3 SDK license eliminates the risk |
| Foundation goodwill changes | Compliance constraints already satisfy public terms; goodwill accelerates, isn't required |

---

## License

**GPL-3.0-or-later** — matches Proton's web client, fits the Linux ecosystem, and eliminates any concerns about structural reference to GPL'd Proton code.

`protondrive-ffi` and `protondrive-core` may be **dual-licensed MIT/Apache-2.0** under a foundation carve-out, so closed-source NAS firmware and mobile apps can embed the C ABI without GPL obligations. This is under discussion.

---

## North Star

> The goal is not to ship before Proton — it is to **be** the Drive layer of Proton's Rust ecosystem, sitting cleanly on top of the crypto foundation Proton already provides, by the time anyone notices it was missing.

When this project hits 1.0:

- Proton Drive has a Rust SDK because this community built the Drive layer on top of Proton's crypto stack
- `protondrive-linux` ships a native daemon and GUI that the Linux community treats as the reference client
- The Proton Foundation has a working relationship with this project; "recognized community SDK" status is in hand
- `proton-crypto-rs`'s next breaking change lands cleanly because the insulation layer absorbed it
- The community has a clean, shippable Drive SDK that didn't have to reinvent crypto, SRP, or the Go FFI bridge

---

*This is `rs/README.md` — the Rust SDK workspace readme. For the parent repo (Proton's official JS/C# SDK fork), see [`../README.md`](../README.md).*
