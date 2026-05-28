# License Inventory

This SDK is licensed **GPL-3.0-or-later**. This file tracks the licenses of direct and significant transitive dependencies.

`cargo-deny` enforces the allow-list in `deny.toml` on every CI run. This document is the human-readable companion.

---

## Direct Dependencies

| Crate | License | Notes |
|---|---|---|
| `proton-crypto` | Proton-internal / TBD | Foundation dependency; blessing required |
| `proton-crypto-account` | Proton-internal / TBD | Foundation dependency |
| `proton-srp` | Proton-internal / TBD | Foundation dependency |
| `tokio` | MIT | Async runtime |
| `reqwest` | MIT OR Apache-2.0 | HTTP client |
| `reqwest_cookie_store` | MIT OR Apache-2.0 | Cookie jar for token refresh |
| `serde` | MIT OR Apache-2.0 | Serialization |
| `serde_json` | MIT OR Apache-2.0 | JSON |
| `secrecy` | MIT OR Apache-2.0 | Secret string / zeroize wrapper |
| `zeroize` | MIT OR Apache-2.0 | Memory zeroing |
| `thiserror` | MIT OR Apache-2.0 | Error derive |
| `anyhow` | MIT OR Apache-2.0 | Error context |
| `tracing` | MIT | Structured logging |
| `backon` | MIT | Retry / backoff |
| `totp-rs` | MIT | TOTP 2FA |
| `bytes` | MIT | Zero-copy byte buffers |
| `tokio-util` | MIT | IO utilities |
| `futures` | MIT OR Apache-2.0 | Stream traits |
| `subtle` | BSD-3-Clause | Constant-time comparisons |
| `url` | MIT OR Apache-2.0 | URL parsing |
| `uuid` | MIT OR Apache-2.0 | UUID generation |

## Dev / Test Dependencies

| Crate | License | Notes |
|---|---|---|
| `wiremock` | MIT | HTTP mock server |
| `proptest` | MIT OR Apache-2.0 | Property-based testing |
| `tokio-test` | MIT | Async test utilities |
| `regex` | MIT OR Apache-2.0 | App-version regex tests |

## Transitive Highlights

| Crate | License | Why it's here |
|---|---|---|
| `rustls` | MIT OR Apache-2.0 OR ISC | TLS via reqwest |
| `ring` | ISC + BoringSSL (OpenSSL) | Crypto primitives via rustls |
| `proton-rpgp` | MIT OR Apache-2.0 | Via `proton-crypto/rustpgp` feature |
| `gopenpgp-sys` | MIT | Via `proton-crypto/gopgp` feature; links Go runtime |

---

## Notes

- The `proton-crypto-*` and `proton-srp` crates do not currently publish a license in their crates.io metadata (they are Proton-internal). The foundation coordination ask includes a formal license clarification. Until resolved, these are treated as Proton-proprietary; consuming them requires the explicit blessing described in `docs/ARCHITECTURE.md`.
- `ring` bundles BoringSSL (Google fork of OpenSSL). Its license is permissive but requires preserving copyright notices. `cargo-deny` tracks this.
- The `gopgp` feature pulls in a Go runtime via CGo. The Go standard library is BSD-3-Clause.

---

*Last updated: 2026-05-07. Re-run `cargo deny check licenses` to verify current state.*
