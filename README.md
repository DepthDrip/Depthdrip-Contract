# DepthDrip Contract

> On-chain registry mapping open-source package names to Stellar maintainer addresses — powering trust-minimized developer payments on Stellar.

[![CI](https://github.com/your-org/Depthdrip-Contract/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/Depthdrip-Contract/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![soroban-sdk 26.1.0](https://img.shields.io/badge/soroban--sdk-26.1.0-blueviolet)](https://crates.io/crates/soroban-sdk)

---

## What is DepthDrip?

DepthDrip is a developer-payment protocol built on Stellar. Its core idea: when a developer ships software that depends on open-source packages, those package maintainers should be easy to pay — without asking them to set up anything special.

This repository contains the **Soroban smart contract** that acts as the on-chain source of truth: a registry that maps `(ecosystem, package_name)` pairs (e.g. `npm/lodash`, `cargo/serde`) to the Stellar G-address of each maintainer.

`depthdrip-cli` and `depthdrip-app` query this contract to resolve addresses, then use Stellar Path Payments to distribute funds across all registered dependencies in a single transaction.

---

## Table of Contents

- [How It Works](#how-it-works)
- [Repository Structure](#repository-structure)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Contract Functions at a Glance](#contract-functions-at-a-glance)
- [Events](#events)
- [Security Model](#security-model)
- [Contributing](#contributing)
- [License](#license)

---

## How It Works

```
Maintainer registers:
  register("npm", "lodash", G-address)
      └─► stored on-chain keyed by (ecosystem, package_name)

Developer pays:
  depthdrip-cli resolve package.json
      └─► reads registry for each dep
      └─► builds Stellar Path Payment
      └─► sends XLM/USDC to all maintainers atomically
```

1. A maintainer calls `register` on this contract, proving ownership by signing the transaction with the Stellar key they register.
2. An optional admin can `verify` a registration, providing an extra trust signal to downstream tooling.
3. Clients call `get_address` or `get_batch` to resolve package names to payment addresses.
4. Removals require the original registrant or the contract admin.

---

## Repository Structure

```
Depthdrip-Contract/
├── src/
│   └── lib.rs              # Contract source — all functions, types, storage keys
├── tests/
│   └── integration_tests.rs # End-to-end tests using soroban testutils
├── docs/
│   ├── architecture.md     # System design and data-flow diagrams
│   ├── abi.md              # Full function/type/event reference
│   ├── deploying.md        # How to deploy and upgrade the contract
│   └── storage.md          # Persistent storage layout and TTL strategy
├── scripts/
│   └── deploy.sh           # Deployment helper for testnet / mainnet
├── .github/
│   └── workflows/
│       └── ci.yml          # GitHub Actions CI pipeline
├── Cargo.toml
├── Makefile
├── CONTRIBUTING.md         # How to contribute
├── CHANGELOG.md            # Version history
└── LICENSE                 # MIT OR Apache-2.0
```

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | stable (≥ 1.84 recommended) | `curl https://sh.rustup.rs -sSf \| sh` |
| `wasm32v1-none` target | — | `rustup target add wasm32v1-none` |
| Stellar CLI (`stellar`) | latest | [Install guide](https://developers.stellar.org/docs/tools/developer-tools/cli/install-stellar-cli) |

> **Note:** soroban-sdk 26.x requires Rust 1.82+ and the `wasm32v1-none` target. The older `wasm32-unknown-unknown` target is no longer supported for WASM builds.

---

## Quick Start

### Build

```bash
rustup target add wasm32v1-none
make build
# Output: target/wasm32v1-none/release/depthdrip_contract.wasm
```

### Test

```bash
make test
```

### Format & Lint

```bash
make fmt
make clippy
```

### Deploy to Testnet

```bash
# First, configure and fund an account with the Stellar CLI:
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet

make deploy-testnet
```

See [docs/deploying.md](docs/deploying.md) for full deployment and upgrade instructions.

---

## Documentation

| Document | Description |
|----------|-------------|
| [docs/architecture.md](docs/architecture.md) | System design, data-flow, access control model |
| [docs/abi.md](docs/abi.md) | Complete function, type, and event reference |
| [docs/deploying.md](docs/deploying.md) | Step-by-step deploy, invoke, and upgrade guide |
| [docs/storage.md](docs/storage.md) | On-chain storage layout and TTL strategy |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to file issues and submit pull requests |
| [CHANGELOG.md](CHANGELOG.md) | Version history and migration notes |

---

## Contract Functions at a Glance

| Function | Auth required | Description |
|----------|--------------|-------------|
| `initialize(admin)` | — | One-time setup; sets the contract admin |
| `register(ecosystem, name, address)` | `address` must sign | Register or update a package → address mapping |
| `get_address(ecosystem, name)` | none | Look up the Stellar address for a package |
| `get_record(ecosystem, name)` | none | Full `PackageRecord` including metadata |
| `get_batch(packages)` | none | Batch address lookup |
| `remove(ecosystem, name)` | registrant or admin | Remove a registration |
| `verify(ecosystem, name)` | admin | Mark a registration as verified |
| `get_stats()` | none | Return aggregate registry statistics |

Full argument types and return values in [docs/abi.md](docs/abi.md).

---

## Events

The contract emits the following events (topic, data):

| Topic symbol | Fired when | Data |
|---|---|---|
| `PkgReg` | Package registered or updated | `(ecosystem, name, address, timestamp)` |
| `PkgRem` | Package removed | `(ecosystem, name, timestamp)` |
| `PkgVer` | Package verified by admin | `(ecosystem, name, timestamp)` |

---

## Security Model

- **Ownership proof**: `register` requires the transaction to be signed by the Stellar key being registered (`stellar_address.require_auth()`). You cannot register someone else's address.
- **Admin key**: stored in instance storage at init time. Admin can `verify` and `remove` any entry. The admin address should be a multisig or governance contract in production.
- **No upgradability**: the contract has no `upgrade` function. Deploying a new version requires migrating registrations. See [docs/deploying.md](docs/deploying.md#upgrading).
- **Persistent storage**: package records use `storage().persistent()`. Instance counters use `storage().instance()`. Both are subject to Soroban's TTL / ledger bump requirements — see [docs/storage.md](docs/storage.md).

---

## Contributing

We welcome contributions of all sizes. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

Quick checklist:
- `make fmt` and `make clippy` must pass
- `make test` must pass
- New behavior needs a test in `tests/integration_tests.rs`

---

## License

Licensed under either of:

- [MIT](LICENSE) license
- [Apache License, Version 2.0](LICENSE)

at your option.
