# ABI Reference

Complete reference for all types, functions, and events exposed by the DepthDrip Contract.

---

## Table of Contents

- [Types](#types)
  - [PackageRecord](#packagerecord)
  - [Stats](#stats)
- [Functions](#functions)
  - [initialize](#initialize)
  - [register](#register)
  - [get_address](#get_address)
  - [get_record](#get_record)
  - [get_batch](#get_batch)
  - [remove](#remove)
  - [verify](#verify)
  - [get_stats](#get_stats)
- [Events](#events)
- [Error Panics](#error-panics)
- [CLI Examples](#cli-examples)

---

## Types

### PackageRecord

Stored for every registered package. Retrieved via `get_record`.

| Field | Type | Description |
|---|---|---|
| `ecosystem` | `Symbol` | Package ecosystem identifier, e.g. `npm`, `cargo` |
| `package_name` | `String` | Canonical package name within the ecosystem |
| `stellar_address` | `Address` | Current registered Stellar G-address for payments |
| `registered_by` | `Address` | Address that called `register` (same as `stellar_address` at registration time) |
| `registered_at` | `u64` | Ledger timestamp when the record was first created |
| `verified` | `bool` | `true` if the admin has called `verify` on this record |
| `verified_at` | `u64` | Ledger timestamp of verification; `0` if not yet verified |

### Stats

Returned by `get_stats`. Contains aggregate counters from instance storage.

| Field | Type | Description |
|---|---|---|
| `total_registered` | `u32` | Total number of unique `(ecosystem, package_name)` registrations |
| `total_verified` | `u32` | Total number of verified records |
| `npm_count` | `u32` | Registrations in the `npm` ecosystem |
| `cargo_count` | `u32` | Registrations in the `cargo` ecosystem |

---

## Functions

### initialize

```
initialize(admin: Address)
```

One-time contract setup. Sets the admin address that controls `verify` and admin-level `remove`.

**Auth:** None required at the contract level. Must be called before any other function.

**Panics:** `"already_initialized"` if called more than once.

**Side effects:** Stores `admin` and initialises the registration counter to 0 in instance storage.

---

### register

```
register(
    ecosystem:       Symbol,
    package_name:    String,
    stellar_address: Address,
) -> ()
```

Register or update the payment address for a package. If the `(ecosystem, package_name)` key already exists, the record is overwritten and the counter is **not** incremented.

**Auth:** `stellar_address.require_auth()` — the transaction must be signed by the address being registered. This prevents registering on behalf of someone else.

**Side effects:**
- Creates or overwrites the `PackageRecord` in persistent storage.
- Increments `Count` in instance storage (only on first registration).
- Emits a `PkgReg` event.

**Example invocation (Stellar CLI):**
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <MAINTAINER_SECRET> \
  --network testnet \
  -- register \
  --ecosystem npm \
  --package_name lodash \
  --stellar_address G...
```

---

### get_address

```
get_address(
    ecosystem:    Symbol,
    package_name: String,
) -> Option<Address>
```

Returns the registered Stellar address for a package, or `None` if not found.

**Auth:** None.

**Notes:** This is the primary read function used by `depthdrip-cli` for payment resolution. For bulk lookups, prefer `get_batch`.

---

### get_record

```
get_record(
    ecosystem:    Symbol,
    package_name: String,
) -> Option<PackageRecord>
```

Returns the full `PackageRecord` for a package, or `None` if not found.

**Auth:** None.

**Notes:** Use this when you need metadata beyond the address (e.g., verification status, registration timestamp).

---

### get_batch

```
get_batch(
    packages: Vec<(Symbol, String)>,
) -> Vec<Option<Address>>
```

Batch version of `get_address`. Accepts a vector of `(ecosystem, package_name)` tuples and returns a vector of `Option<Address>` in the **same order**. Missing entries resolve to `None`.

**Auth:** None.

**Notes:** Prefer this over calling `get_address` in a loop — it reads all entries in a single contract invocation, which is significantly cheaper in terms of Stellar operations.

**Example:**
```bash
# Input: [("npm", "lodash"), ("cargo", "serde"), ("npm", "unknown")]
# Output: [Some(G...), Some(G...), None]
```

---

### remove

```
remove(
    ecosystem:    Symbol,
    package_name: String,
) -> ()
```

Deletes a registration. The caller must be either the original registrant (`registered_by`) or the admin.

**Auth:** `registered_by.require_auth()` or `admin.require_auth()`.

**Panics:** `"not_found"` if no record exists for the given key.

**Side effects:**
- Deletes the `PackageRecord` from persistent storage.
- Emits a `PkgRem` event.

**Note:** The registration counter (`Count`) is **not** decremented on removal to keep the counter monotonic and consistent with historical totals.

---

### verify

```
verify(
    ecosystem:    Symbol,
    package_name: String,
) -> ()
```

Marks a registration as admin-verified. Downstream tooling can use this as an additional trust signal when resolving payment addresses.

**Auth:** `admin.require_auth()`.

**Panics:**
- `"not_initialized"` if `initialize` has not been called.
- `"not_found"` if no record exists for the given key.

**Side effects:**
- Sets `record.verified = true` and `record.verified_at = current_timestamp`.
- Emits a `PkgVer` event.

---

### get_stats

```
get_stats() -> Stats
```

Returns aggregate registry statistics from instance storage.

**Auth:** None.

**Notes:** Counters are maintained incrementally on `register` / `verify`. The `npm_count` and `cargo_count` fields are tracked separately for convenience; the `total_registered` counter is the canonical total.

---

## Events

Events are emitted via `env.events().publish(topics, data)`. You can stream them via [Horizon](https://developers.stellar.org/api/horizon) or the Stellar RPC `getEvents` endpoint.

### PkgReg — Package Registered

Emitted on every successful `register` call (new registration or update).

| Field | Value |
|---|---|
| Topic[0] | `Symbol("PkgReg")` |
| Data | `(ecosystem: Symbol, package_name: String, stellar_address: Address, timestamp: u64)` |

### PkgRem — Package Removed

Emitted on every successful `remove` call.

| Field | Value |
|---|---|
| Topic[0] | `Symbol("PkgRem")` |
| Data | `(ecosystem: Symbol, package_name: String, timestamp: u64)` |

### PkgVer — Package Verified

Emitted on every successful `verify` call.

| Field | Value |
|---|---|
| Topic[0] | `Symbol("PkgVer")` |
| Data | `(ecosystem: Symbol, package_name: String, timestamp: u64)` |

---

## Error Panics

Soroban contracts communicate errors via `panic!`. The following panic messages are produced by this contract:

| Message | Function | Cause |
|---|---|---|
| `"already_initialized"` | `initialize` | Called more than once |
| `"not_found"` | `remove`, `verify` | Record does not exist |
| `"not_initialized"` | `verify` | `initialize` was never called |
| `"count overflow"` | `register` | Registration counter wrapped `u32::MAX` (theoretical) |

---

## CLI Examples

All examples use the `stellar` CLI. Replace `<CONTRACT_ID>`, `<MAINTAINER_SECRET>`, and `<ADMIN_SECRET>` with real values.

```bash
# Initialize the contract
stellar contract invoke \
  --id <CONTRACT_ID> --source-account <ADMIN_SECRET> --network testnet \
  -- initialize --admin G...

# Register a package
stellar contract invoke \
  --id <CONTRACT_ID> --source-account <MAINTAINER_SECRET> --network testnet \
  -- register --ecosystem npm --package_name lodash --stellar_address G...

# Look up a single address
stellar contract invoke \
  --id <CONTRACT_ID> --network testnet \
  -- get_address --ecosystem npm --package_name lodash

# Get a full record
stellar contract invoke \
  --id <CONTRACT_ID> --network testnet \
  -- get_record --ecosystem cargo --package_name serde

# Verify a package (admin only)
stellar contract invoke \
  --id <CONTRACT_ID> --source-account <ADMIN_SECRET> --network testnet \
  -- verify --ecosystem npm --package_name lodash

# Remove a package (registrant or admin)
stellar contract invoke \
  --id <CONTRACT_ID> --source-account <MAINTAINER_SECRET> --network testnet \
  -- remove --ecosystem npm --package_name lodash

# Get stats
stellar contract invoke \
  --id <CONTRACT_ID> --network testnet \
  -- get_stats
```
