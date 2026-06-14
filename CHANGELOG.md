# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

_Changes staged for the next release go here._

---

## [0.1.0] — 2025-06-14

Initial public release of the DepthDrip Soroban smart contract.

### Added

- **`initialize(admin)`** — one-time contract setup; sets the admin `Address` in instance storage.
- **`register(ecosystem, package_name, stellar_address)`** — register or update the payment address for a package. Requires the transaction to be signed by the address being registered (`require_auth`).
- **`get_address(ecosystem, package_name) → Option<Address>`** — single-package address lookup.
- **`get_record(ecosystem, package_name) → Option<PackageRecord>`** — full record lookup including verification metadata.
- **`get_batch(packages) → Vec<Option<Address>>`** — vectorised address lookup for bulk dependency resolution.
- **`remove(ecosystem, package_name)`** — delete a registration; callable by the registrant or the admin.
- **`verify(ecosystem, package_name)`** — admin-only; marks a registration as verified and records the timestamp.
- **`get_stats() → Stats`** — returns `total_registered`, `total_verified`, `npm_count`, `cargo_count` from instance storage.
- **`PackageRecord`** type with fields: `ecosystem`, `package_name`, `stellar_address`, `registered_by`, `registered_at`, `verified`, `verified_at`.
- **`Stats`** type with aggregate registry counters.
- **`DataKey`** typed enum for type-safe storage keys.
- Events: `PkgReg`, `PkgRem`, `PkgVer` emitted on register, remove, and verify respectively.
- 7 integration tests covering registration, lookup, batch lookup, duplicate registration idempotency, removal, verification, and double-initialize guard.
- Full documentation: `README.md`, `docs/architecture.md`, `docs/abi.md`, `docs/deploying.md`, `docs/storage.md`, `CONTRIBUTING.md`.
- `Makefile` with `build`, `test`, `fmt`, `clippy`, `deploy-testnet`, `deploy-mainnet` targets.
- GitHub Actions CI pipeline (`.github/workflows/ci.yml`).

### Technical notes

- Uses `soroban-sdk 26.1.0` targeting `wasm32v1-none` (Rust 1.82+ requirement).
- `testutils` feature is declared only in `[dev-dependencies]` to prevent incompatibility with the wasm build target.
- Instance storage holds admin and count; persistent storage holds per-package records.
- No upgrade entrypoint in this version — see [docs/deploying.md](docs/deploying.md#upgrading) for migration strategy.

---

## Version History Reference

| Version | Date | Notes |
|---|---|---|
| 0.1.0 | 2025-06-14 | Initial release |

[Unreleased]: https://github.com/your-org/Depthdrip-Contract/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-org/Depthdrip-Contract/releases/tag/v0.1.0
