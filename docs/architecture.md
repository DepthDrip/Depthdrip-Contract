# Architecture

This document describes the system design of the DepthDrip Contract — the on-chain registry that powers developer payments on Stellar.

---

## Table of Contents

- [Overview](#overview)
- [System Context](#system-context)
- [Contract Internals](#contract-internals)
- [Data Flow](#data-flow)
- [Access Control](#access-control)
- [Events](#events)
- [Design Decisions](#design-decisions)
- [Known Limitations](#known-limitations)

---

## Overview

DepthDrip Contract is a **Soroban smart contract** deployed on the Stellar blockchain. Its single responsibility is maintaining an authoritative, publicly readable mapping of:

```
(ecosystem: Symbol, package_name: String) → stellar_address: Address
```

Ecosystems are short identifiers like `npm` or `cargo`. Package names are the canonical names used by each ecosystem's registry (e.g., `lodash`, `serde`).

The contract intentionally has **no payment logic**. It is a read-heavy registry. Payment routing, path payment construction, and dependency tree resolution all happen off-chain in `depthdrip-cli` and `depthdrip-app`, which query this contract as their source of truth.

---

## System Context

```
┌────────────────────────────────────────────────────────────────────┐
│                          Stellar Network                           │
│                                                                    │
│   ┌─────────────────────────────────────┐                         │
│   │       DepthDrip Contract (WASM)     │                         │
│   │                                     │                         │
│   │  register / remove / verify         │◄── Maintainer txn       │
│   │  get_address / get_batch            │◄── CLI / App read       │
│   └─────────────────────────────────────┘                         │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
         ▲                              ▲
         │                              │
┌────────────────┐            ┌──────────────────────┐
│ depthdrip-cli  │            │   depthdrip-app       │
│                │            │   (web UI)            │
│ 1. Parse       │            │                       │
│    package.json│            │ 1. Show registry      │
│ 2. Resolve     │            │ 2. Let maintainers    │
│    addresses   │            │    register / update  │
│ 3. Build path  │            └──────────────────────┘
│    payment txn │
│ 4. Submit      │
└────────────────┘
```

---

## Contract Internals

### Module Structure

The entire contract lives in `src/lib.rs`. It exports:

- **`DepthDripContract`** — the contract struct, decorated with `#[contract]`.
- **`PackageRecord`** — the value type stored for each registration.
- **`Stats`** — aggregate counters returned by `get_stats`.
- **`DataKey`** — the typed enum used as storage keys.

### Storage Keys

```rust
#[contracttype]
enum DataKey {
    Admin,                          // instance storage → Address
    Count,                          // instance storage → u32
    Package(Symbol, String),        // persistent storage → PackageRecord
}
```

Instance storage (`Admin`, `Count`) is cheap to access because Soroban loads the full instance footprint on every invocation. Persistent storage (`Package`) is loaded on demand — ideal for records that are not always needed.

See [docs/storage.md](storage.md) for the full TTL and bump strategy.

### Types

```rust
pub struct PackageRecord {
    pub ecosystem:       Symbol,   // "npm" | "cargo" | ...
    pub package_name:    String,   // canonical package name
    pub stellar_address: Address,  // current payment address
    pub registered_by:   Address,  // address that called register()
    pub registered_at:   u64,      // ledger timestamp of first registration
    pub verified:        bool,     // set true by admin via verify()
    pub verified_at:     u64,      // 0 if not verified
}

pub struct Stats {
    pub total_registered: u32,
    pub total_verified:   u32,
    pub npm_count:        u32,
    pub cargo_count:      u32,
}
```

---

## Data Flow

### Registration

```
Maintainer
  │
  ├─ signs txn with their Stellar key (G-address)
  │
  └─► invoke register(ecosystem, package_name, stellar_address)
            │
            ├─ stellar_address.require_auth()  ← enforces ownership
            │
            ├─ if new record: increment Count in instance storage
            │
            ├─ write PackageRecord to persistent storage
            │
            └─ emit PkgReg event
```

### Resolution (read path)

```
depthdrip-cli
  │
  ├─ for each dep in package.json:
  │     invoke get_address("npm", dep_name)
  │         └─► read PackageRecord from persistent storage
  │             return stellar_address (or None if not registered)
  │
  └─ or: invoke get_batch([(eco, name), ...])
             └─► vectorized lookup, returns Vec<Option<Address>>
```

### Removal

```
invoke remove(ecosystem, package_name)
  │
  ├─ load PackageRecord
  ├─ require_auth from registered_by OR admin
  ├─ delete from persistent storage
  └─ emit PkgRem event
```

### Verification

```
invoke verify(ecosystem, package_name)  ← admin only
  │
  ├─ load admin address from instance storage
  ├─ admin.require_auth()
  ├─ set record.verified = true, record.verified_at = now
  └─ emit PkgVer event
```

---

## Access Control

| Operation | Who can call |
|---|---|
| `initialize` | anyone — but only once; panics if admin is already set |
| `register` | must be signed by the address being registered |
| `get_address` | public |
| `get_record` | public |
| `get_batch` | public |
| `remove` | the original `registered_by` address **or** admin |
| `verify` | admin only |
| `get_stats` | public |

The admin address is set once at `initialize` and stored in instance storage. It is never rotatable in the current contract version. Use a multisig or governance contract address as the admin in production.

---

## Events

Events are emitted via `env.events().publish(topic, data)`.

| Symbol | When emitted | Data tuple |
|---|---|---|
| `PkgReg` | After a successful `register` | `(ecosystem, package_name, stellar_address, timestamp)` |
| `PkgRem` | After a successful `remove` | `(ecosystem, package_name, timestamp)` |
| `PkgVer` | After a successful `verify` | `(ecosystem, package_name, timestamp)` |

Off-chain indexers (Horizon event streaming, custom indexers) subscribe to these events to maintain read replicas without polling the contract directly.

---

## Design Decisions

**Why no payment logic in the contract?**
Keeping payment routing off-chain maximizes flexibility. The CLI can construct complex multi-hop path payments, apply weightings, respect per-ecosystem overrides, and evolve without requiring a contract upgrade.

**Why `Symbol` for ecosystem instead of a string enum?**
Soroban `Symbol` values are compact (up to 32 alphanumeric + underscore characters) and compare by value without heap allocation. They are idiomatic for short identifiers in Soroban.

**Why no on-chain iteration for `get_all_by_ecosystem`?**
Soroban's persistent storage does not expose a key-scan API. Full enumeration would require an on-chain index (e.g., a `Vec` of keys per ecosystem), which adds write cost to every `register` call. For DepthDrip's read-heavy workload, off-chain indexing via Horizon event streaming is the better trade-off.

**Why is `verified_at` a `u64` (0 = unverified) instead of `Option<u64>`?**
`Option<T>` in Soroban contracts serialises as an XDR union. For a simple "not yet set" sentinel, `0u64` is a smaller on-chain footprint and simplifies XDR encoding, at the cost of not being able to distinguish "verified at genesis" from "not verified". The ledger timestamp at genesis is well above 0 in practice.

---

## Known Limitations

- **No admin rotation.** If the admin key is lost or compromised, no recovery path exists in the current contract version.
- **No ecosystem enumeration.** The contract cannot return a list of all registered packages for a given ecosystem without an off-chain index.
- **No upgrade path.** The contract has no `upgrade` entrypoint. A new version requires redeployment and data migration. See [docs/deploying.md](deploying.md#upgrading).
- **TTL management is external.** Callers or a maintenance bot must bump TTLs for persistent storage entries to prevent them from expiring. See [docs/storage.md](storage.md).
