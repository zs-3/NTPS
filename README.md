# NTPS Server 0.1

Terminal-first NTPS hosting node prototype. It creates an Ed25519 node identity, talks to the companion PHP registry API, and serves static files over a length-prefixed NTPS/0.1 TCP framing protocol.

## Important scope and security status

This is a runnable **MVP foundation**, not yet the complete P2P network in the architecture blueprint. The listener currently uses direct TCP. Iroh/QUIC, NAT hole punching, relay fallback, browser integration, signed registry-record verification on the client, range requests, and swarm caching are not implemented in this release. Do not expose this prototype to untrusted public traffic without additional review. Registry URLs should use HTTPS in production.

## Build

Requires Rust stable and Cargo.

```sh
cargo build --release
```

## First run

```sh
ntps init --registry https://your-registry.example
# Edit ntps.toml and set registry_token if your API requires it.
ntps identity
ntps register-node
ntps register-domain example.ntp
ntps serve ./public --listen 0.0.0.0:7443 --domain example.ntp
```

`NTPS_REGISTRY_TOKEN` can override the token in `ntps.toml`.

## Wire protocol

Each TCP connection carries one request:
- 8 bytes magic: `NTPS01\\r\\n`
- 4-byte big-endian JSON header length
- JSON `{ "method":"GET", "host":"example.ntp", "path":"/index.html" }`
- response uses the same magic + length + JSON metadata, followed by exactly `content_length` body bytes.

The protocol is intentionally small and versioned. The transport is not encrypted in this MVP; do not send secrets or treat it as production-secure. The production transport milestone must move this frame over authenticated QUIC (Iroh or Quinn + a reviewed identity handshake).

## Configuration

`ntps.toml` is created in the current directory. The identity file defaults to `.ntps/identity.key`; protect it and back it up securely. Losing it changes the node identity.

## CLI

- `ntps init`
- `ntps identity`
- `ntps register-node`
- `ntps register-domain example.ntp`
- `ntps resolve example.ntp`
- `ntps serve ./public --listen 127.0.0.1:7443`


## GitHub Actions

- `.github/workflows/ci.yml` checks formatting, runs Clippy and tests, and builds on Linux, Windows, and macOS.
- `.github/workflows/release.yml` builds binaries and attaches them to a GitHub Release when you push a `v*` tag.

Publish a release after CI passes:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The release matrix currently targets Linux x86_64, Windows x86_64, and macOS ARM64. Adjust the matrix if you want additional architectures.
