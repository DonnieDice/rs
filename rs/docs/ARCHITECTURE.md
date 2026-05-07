# Architecture

## Dependency Boundary: `proton-crypto-rs`

The SDK consumes `proton-crypto`, `proton-crypto-account`, and `proton-srp` from Proton's
official Rust crates. These crates are noted as "not intended for general usage outside Proton"
in their README.

**Insulation layer** (`crates/protondrive-crypto`): All `proton-crypto` types the SDK depends
on are re-exported as SDK-owned type aliases from this single module. If `proton-crypto`'s API
changes, the blast radius is limited to `protondrive-crypto/src/lib.rs` rather than the full
SDK surface.

## Crate Layout

```
protondrive-core     — types, errors, opaque IDs (no external deps beyond serde/thiserror)
protondrive-crypto   — insulation layer over proton-crypto
protondrive-api      — HTTP client, all endpoint bindings, retry/backoff
protondrive-auth     — login/session orchestration over proton-srp
protondrive-events   — event stream subscription (only sync mechanism)
protondrive-sdk      — high-level API surface (consumers import only this)
protondrive-ffi      — C ABI; the one crate that permits unsafe_code
```

## Backend Feature Flags

`protondrive-sdk` (and the workspace) expose two mutually exclusive features:

- `gopgp` (default) — uses Proton's Go-backed OpenPGP implementation via `proton-crypto/gopgp`.
  Requires a Go toolchain to build. Bit-exact parity with official Proton clients.
- `rustpgp` — pure-Rust via `proton-rpgp` through `proton-crypto/rustpgp`. No Go toolchain.
  Used for AppImage builds in `protondrive-linux`.

The SDK does not define its own `CryptoBackend` trait — `proton-crypto`'s API *is* the
abstraction. The feature flag passthrough is the only knob.

## Threading Model

All public methods are async (`tokio`). Upload uses a configurable concurrency window (default 4
parallel block uploads). Download is streaming `AsyncRead` with backpressure. No blocking calls
in the async path.

## Security Invariants

- `SecretString` (zeroized on drop) for all passwords, tokens, and key material.
- No key material in `Debug` or `Display` output.
- `#![forbid(unsafe_code)]` on every crate except `protondrive-ffi`.
- The server and network are treated as hostile; only the local Rust process is trusted.
