# Compliance

Maps each Proton Drive compliance requirement to the code location that enforces it.

| Requirement | Enforcement | Location |
|---|---|---|
| Official Proton endpoints only | `PROTON_API_BASE` is a `const`; no runtime override | `protondrive-api/src/config.rs` |
| `x-pm-appversion` on every request | Header added in `build_request`; regex validated in `SdkConfig::new` | `protondrive-api/src/client.rs`, `config.rs` |
| `x-pm-appversion` regex | Unit tests assert valid/invalid patterns against the canonical regex | `protondrive-api/src/config.rs#[cfg(test)]` |
| Event-based sync only | No `list_all_recursive`; only `subscribe_events` + `latest_event_id` | `protondrive-sdk/src/drive.rs` |
| No password storage | `SecretString` only; zeroized on drop; never written to disk by SDK | `protondrive-auth/src/session.rs` |
| No Proton branding | Code review checklist; no logos or marks in this repo | CI review |
| License obligations | `cargo-deny` with explicit allow-list | `deny.toml`, CI `deny` job |
