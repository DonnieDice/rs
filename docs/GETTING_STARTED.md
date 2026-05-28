# Getting Started with protondrive-sdk

## Minimal Example: Login + List Volumes

```rust
use protondrive_sdk::ProtonDrive;
use protondrive_sdk::{Session, TwoFactorProvider};
use protondrive_sdk::{DriveError, Result};
use secrecy::SecretString;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Create a bare ApiClient (no session yet)
    let mut builder = ProtonDrive::builder()
        .app_version("my-app@1.0.0")
        .user_agent("MyApp/1.0");

    // 2. Login via SRP-6a
    let api_client = /* not directly exposed — login takes &ApiClient */;
    // See login() below

    // Actually, login() requires an ApiClient. The current API flow is:
    let client = /* ... create ApiClient from SdkConfig ... */;

    // Full login (SRP-6a + optional 2FA):
    let password = SecretString::new("your-password".into());
    let session = protondrive_auth::login(
        &client,
        "your-username",
        password,
        None, // or Some(TwoFactorProvider::Totp(totp_secret))
    )
    .await?;

    // 3. Build ProtonDrive with the authenticated session
    let drive = ProtonDrive::builder()
        .session(session)
        .build()?;

    // 4. List volumes
    let volumes = drive.list_volumes().await?;
    println!("Volumes: {volumes:#?}");

    // 5. Navigate the tree
    if let Some(share) = volumes.first() {
        let root = drive.list_children(&share.share_id, None).await?;
        for node in &root {
            println!("  {} — {:?} ({})", node.name, node.kind, node.size);
        }
    }

    Ok(())
}
```

### Important notes

- The `login` function lives in `protondrive_auth` (re-exported as
  `protondrive_sdk::login` once the facade exposes it directly). Import
  it from `protondrive_auth` or `protondrive_sdk` depending on the
  workspace's current re-export state.
- `password` must be a `SecretString` (from the `secrecy` crate) — never
  a bare `String`.
- `TwoFactorProvider::Totp(SecretString)` — the TOTP secret, not a
  one-time code. The SDK generates the rotating code internally via
  `totp-rs`.

---

## Serializing and Restoring a Session

### Serialize (save)

```rust
use protondrive_sdk::SerializedSession;

let serialized: SerializedSession = session.serialize();
let json = serde_json::to_string(&serialized)?;

// ⚠️  Encrypt before writing to disk!
// Good: OS keyring (secret-service, gnome-keyring, etc.)
// Bad: raw file on disk (contains access_token + refresh_token)
```

### Deserialize (restore)

```rust
use protondrive_sdk::{Session, SerializedSession};
use protondrive_sdk::login::resume_session;

let json = /* read encrypted blob, decrypt to plaintext */;
let serialized: SerializedSession = serde_json::from_str(&json)?;
let session = resume_session(&client, serialized)?;
// resume_session calls client.set_access_token() internally
```

### What's in `SerializedSession`

| Field | Type | Notes |
|---|---|---|
| `uid` | `String` | Session UID from Proton |
| `user_id` | `String` | Proton user identifier |
| `access_token` | `String` | Bearer token for API calls |
| `refresh_token` | `String` | Token for refreshing an expired session |

The in-memory `Session` wraps `access_token` and `refresh_token` in
`SecretString` (zeroized on drop). `SerializedSession` uses plain `String`
so that serde can serialize it — hence the **encrypt-before-storing**
requirement.

---

## Feature Flags: `gopgp` vs `rustpgp`

Add to your `Cargo.toml`:

```toml
[dependencies]
protondrive-sdk = { git = "https://github.com/DonnieDice/rs", features = ["rustpgp"] }
```

| Feature | Backend | Build requirement | Use case |
|---|---|---|---|
| `gopgp` (default) | Go-backed via `proton-crypto/gopgp` | Go toolchain | Bit-exact parity with official Proton clients |
| `rustpgp` | Pure Rust via `proton-rpgp` | None (Rust only) | Self-contained builds (AppImage, WASM, CI without Go) |

### When to pick each

- **Default (`gopgp`)** — use during active development or when running on
  a system with `go` installed. Produces hashes that match the JS SDK exactly.
- **`rustpgp`** — use for distribution builds (Flatpak, AppImage), CI
  environments without Go, or any scenario where adding a Go toolchain is
  impractical. The Rust backend is verified against the Go backend's test
  vectors.

The feature is passed through to `proton-crypto` — you don't configure it
on individual items.

---

## Subscribing to Events

```rust
use futures::StreamExt;
use protondrive_sdk::EventAction;

// 1. Get the latest event ID as a starting cursor
let share = &volumes[0].share_id;
let since = drive.latest_event_id(share).await?;

// 2. Subscribe
let mut stream = drive.subscribe_events(share, since).await?;

// 3. Poll in a loop
while let Some(event) = stream.next().await {
    match event {
        Ok(evt) => match evt.action {
            EventAction::NodeUpdated { node_id, payload } => {
                println!("Node {node_id} was created or updated");
            }
            EventAction::NodeDeleted { node_id } => {
                println!("Node {node_id} was deleted");
            }
            EventAction::NodeTrashed { node_id } => {
                println!("Node {node_id} was trashed");
            }
            EventAction::NodeRestored { node_id } => {
                println!("Node {node_id} was restored from trash");
            }
            EventAction::Unknown { raw_action } => {
                println!("Unknown event type {raw_action} (forward-compat)");
            }
        },
        Err(e) => {
            eprintln!("Event error: {e}");
            // DriveError::EventStreamGap → re-subscribe from latest_event_id
            break;
        }
    }
}
```

### Polling semantics

- `EventStream` implements `futures::Stream<Item = Result<DriveEvent>>`.
- Each `poll_next` fetches a page from the API if the internal buffer is
  empty, advances the cursor, and yields events one at a time.
- When no more events are available, it returns `Poll::Pending` and wakes
  the caller to re-poll later (you should sleep between polls).
- If the server detects a gap: `Err(DriveError::EventStreamGap {
  latest_event_id })` — call `drive.latest_event_id()` and re-subscribe.

### Re-sync pattern after a gap

```rust
match event {
    Err(DriveError::EventStreamGap { latest_event_id }) => {
        println!("Re-syncing from {latest_event_id}");
        stream = drive.subscribe_events(share, latest_event_id.parse().unwrap()).await?;
    }
    Err(e) => { /* unrecoverable */ break; }
    Ok(evt) => { /* process */ }
}
```

---

## Streaming a Download

> **Status: Phase 1 — not yet implemented.** The stub returns
> `DriveError::Auth("download not yet implemented (Phase 1)")`. The signature
> below reflects the planned API.

```rust
use futures::StreamExt;
use tokio_stream::StreamExt as _;
use std::pin::pin;

// node_id from list_children or get_node
let mut stream = drive.download(&node_id).await?;
let mut stream = pin!(stream);

let mut file = tokio::fs::File::create("downloaded_file.bin").await?;

while let Some(chunk) = stream.next().await {
    let bytes = chunk?;
    file.write_all(&bytes).await?;
}
```

### Planned implementation

When implemented, `download()` will:
1. Fetch block URLs and storage tokens from the API.
2. For each block: GET from storage URL with `pm-storage-token` header,
   then decrypt via `protondrive-crypto::decrypt_block`.
3. Yield decrypted `Bytes` through a `Stream` with backpressure (the
   consumer reads at its own pace; no unbounded buffering).

The `DownloadStream` type alias:

```rust
pub type DownloadStream = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;
```

---

## Token Refresh

When an API call returns `Err(DriveError::SessionExpired)` (HTTP 401):

```rust
use protondrive_sdk::DriveError;
use protondrive_sdk::refresh;

let mut session = /* your Session */;

match drive.list_volumes().await {
    Err(DriveError::SessionExpired) => {
        refresh(&client, &mut session).await?;
        // session now has new access + refresh tokens
        // client.set_access_token() was called internally by refresh()
        // retry the original operation
        let volumes = drive.list_volumes().await?;
    }
    result => { /* handle normally */ }
}
```

---

## Feature Checklist by Phase

| Feature | Phase | Status |
|---|---|---|
| SRP-6a login | 0 | Done |
| Session serialization/restore | 0 | Done |
| List volumes | 0 | Done |
| List children (tree navigation) | 0 | Done |
| Get node | 0 | Done |
| Rename, move, trash, delete | 0 | Done |
| List shares | 0 | Done |
| List revisions | 0 | Done |
| Restore revision | 0 | Done |
| Event subscription (polling) | 0 | Done |
| Download stream | 1 | **Stub** |
| Upload (chunked + encrypted) | 2 | **Stub** |
| Photos API | 3 | **Stub** |
