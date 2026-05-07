# Threat Model

## Trust Boundary

| Component | Trust level |
|---|---|
| Local Rust process running the SDK | **Trusted** |
| Proton API server | **Untrusted** — server is treated as potentially hostile |
| Network (TLS termination and beyond) | **Untrusted** — network is treated as hostile |
| Consumer application (caller of the SDK) | Trusted — the SDK is a library; the app sets the security posture |
| Proton's `proton-crypto` crate | Trusted — Proton-published; cryptographic correctness delegated to it |

## What the SDK Protects

### Secret material
- Passwords are passed as `SecretString` and never stored, logged, or written to disk by the SDK.
- Access tokens and refresh tokens are held in `SecretString`; zeroized on drop.
- Node key material (`DecryptedNodeKey`) is zeroized on drop.
- No secret appears in `Debug` or `Display` output — all redacted.

### Data integrity
- Every downloaded block's MAC is verified against the revision manifest before bytes are yielded to the caller. A `ChecksumMismatch` error is returned on failure — no corrupted plaintext is ever handed up.
- Node name signatures are verified on read (Phase 1).
- Upload manifests are signed before commit (Phase 2).

### Transport
- All requests go to `PROTON_API_BASE` (a compile-time `const`). There is no runtime override path. A consumer cannot accidentally redirect traffic to a proxy.
- TLS is enforced via `reqwest` with `rustls`; the system trust store is used by default. Custom TLS roots can be injected through the `HttpClient` trait but not by overriding the endpoint.

## What the SDK Does NOT Protect Against

### Compromised consumer process
The SDK is a library. If the process calling it is compromised, all bets are off. Key material is in-memory; a process-level attacker can read it. OS-level isolation (e.g. seccomp, sandbox) is the consuming app's responsibility.

### Malicious `proton-crypto` build
The SDK delegates all OpenPGP operations to `proton-crypto`. A supply-chain compromise of that crate would compromise all cryptographic guarantees. Mitigation: `cargo-deny` pins known-good versions; `cargo-audit` checks advisories in CI.

### Server-side key compromise
If Proton's key server returns a malicious public key, the SDK will encrypt to it. The SDK has no way to detect this without out-of-band key verification (which is a Proton-side feature, not SDK scope).

### Timing attacks on network metadata
The SDK does not pad requests or add dummy traffic. An observer can infer activity patterns from request timing and size. Countermeasures are out of scope for a pure SDK.

### Persistence layer
The SDK never writes to disk. The consuming application is responsible for encrypting `SerializedSession` before storage (e.g. OS keyring, `age`, LUKS). A `SerializedSession` in plaintext on disk is equivalent to a leaked access token.

## Out-of-Scope Threat Vectors

These are intentionally out of scope for the SDK and belong in the consuming application:

- OS keyring / credential storage security
- Filesystem permission hardening
- Daemon privilege separation
- GUI credential entry security (password masking, clipboard clearing)
- Anti-keylogger measures
- Memory forensics resistance beyond `zeroize`

## Reporting

For vulnerabilities in this SDK: open a GitHub Security Advisory in this repository.

For vulnerabilities in `proton-crypto`, `proton-srp`, or the Proton API: follow Proton's `SECURITY.md` and responsible disclosure process at [proton.me/security](https://proton.me/security).
