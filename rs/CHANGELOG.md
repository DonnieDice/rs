# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- Cargo workspace with 7 crates: `protondrive-core`, `protondrive-crypto`, `protondrive-api`,
  `protondrive-auth`, `protondrive-events`, `protondrive-sdk`, `protondrive-ffi`
- Opaque ID types: `VolumeId`, `ShareId`, `NodeId`, `RevisionId`, `EventId`
- `DriveError` with structured variants for all API and crypto failure modes
- `ApiClient` with `x-pm-appversion` validation, exponential backoff retry, rate-limit handling
- All read/write endpoint bindings: volumes, nodes, shares, events, revisions, auth
- SRP-6a login flow via `proton-srp`; TOTP 2FA support
- `Session` / `SerializedSession` with `SecretString` zeroized on drop
- `EventStream` — `futures::Stream` impl polling the Drive event API
- `ProtonDrive` high-level API with `ProtonDriveBuilder`
- Examples: `login_and_list`, `event_subscription`, `upload_stream` (stub), `download_stream` (stub)
- CI matrix: build / test / clippy / fmt / cargo-deny / cargo-audit on Linux × macOS × Windows
- `deny.toml` license and advisory configuration
- `docs/ARCHITECTURE.md`, `docs/COMPLIANCE.md`

---

## [0.1.0] — TBD

First tagged release. Target: Phase 0 complete + Phase 1 read path.
