# Storage Layout

This document describes how the DepthDrip Contract stores data on-chain, explains the Soroban storage tier model, and outlines the TTL (time-to-live) management strategy required to keep registrations alive.

---

## Table of Contents

- [Soroban Storage Tiers](#soroban-storage-tiers)
- [Storage Keys and Values](#storage-keys-and-values)
- [TTL and Ledger Bump Strategy](#ttl-and-ledger-bump-strategy)
- [Storage Footprint and Fees](#storage-footprint-and-fees)
- [Off-chain Indexing](#off-chain-indexing)

---

## Soroban Storage Tiers

Soroban provides three storage tiers, each with different persistence and cost characteristics:

| Tier | Lifetime | Cost | Use case |
|---|---|---|---|
| `instance` | Lives as long as the contract instance | Cheapest per-read (always loaded) | Small, frequently accessed globals |
| `persistent` | Survives until TTL expires unless bumped | Pay to extend TTL | Per-record data that must survive across ledgers |
| `temporary` | Auto-deleted when TTL expires | Cheapest per-byte | Ephemeral caches; not used in this contract |

The full TTL specification is part of [CAP-0046](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0046-07.md).

---

## Storage Keys and Values

The contract uses a single typed enum for all keys:

```rust
#[contracttype]
enum DataKey {
    Admin,                    // → Address
    Count,                    // → u32
    Package(Symbol, String),  // → PackageRecord
}
```

### Instance Storage

| Key | Type | Description |
|---|---|---|
| `DataKey::Admin` | `Address` | The admin address set during `initialize`. Never changes. |
| `DataKey::Count` | `u32` | Monotonically increasing count of unique registrations. |

Instance storage is loaded in full on every contract invocation. Keep it small. Admin and Count are two small XDR values — well within acceptable instance footprint.

### Persistent Storage

| Key | Type | Description |
|---|---|---|
| `DataKey::Package(ecosystem, name)` | `PackageRecord` | One entry per registered package. |

Each `PackageRecord` serialises to approximately 200–400 bytes of XDR depending on package name length and address encoding. The key (`Package(Symbol, String)`) serialises to approximately 50–150 bytes.

Persistent entries are subject to Soroban's TTL mechanism. If a `PackageRecord` entry's TTL expires before it is bumped, **Soroban will delete it**. See the next section.

---

## TTL and Ledger Bump Strategy

### How TTL Works

Every persistent ledger entry has a `live_until_ledger` value. On every Soroban transaction that reads the entry, the host checks whether the entry's TTL has expired. If it has, the entry is treated as if it does not exist.

The current Soroban mainnet parameters (as of 2025):
- **Minimum TTL for persistent entries:** ~100,000 ledgers (~5.5 days at ~5s/ledger)
- **Maximum TTL for persistent entries:** ~3,110,400 ledgers (~180 days)

### Bump Strategy for DepthDrip

The contract itself does **not** proactively bump TTLs during reads. This keeps read costs low.

Instead, TTLs should be managed externally using one of these approaches:

**Option 1 — Bump on write (recommended for production)**

When `register` is called, the caller (or a post-transaction script) sends an `extendFootprintTtl` operation:

```bash
stellar contract extend \
  --id $CONTRACT_ID \
  --key DataKey::Package(ecosystem, name) \
  --ttl-ledger-count 3110400 \
  --source <PAYER> \
  --network testnet
```

**Option 2 — Maintenance bot**

Run a periodic process (e.g., daily cron) that:
1. Queries Horizon for `PkgReg` events to get all registered keys.
2. For each key, checks the current `live_until_ledger`.
3. Extends TTL for any entry within 30 days of expiry.

```bash
# Check live_until_ledger for a specific entry
stellar contract read \
  --id $CONTRACT_ID \
  --key <XDR_KEY> \
  --network mainnet
```

**Option 3 — Bump inside register (simplest, higher tx cost)**

Add `env.storage().persistent().extend_ttl(&key, min_ttl, max_ttl)` calls inside `register`, `verify`, and `remove`. This guarantees the entry stays alive after any write but increases transaction fees.

```rust
// Example addition to register():
env.storage().persistent().extend_ttl(&key, 100_000, 3_110_400);
```

### Instance Storage TTL

Instance storage TTL is managed by the contract's own instance footprint. Every invocation that touches instance storage automatically extends the instance TTL. No external bump is needed as long as the contract is actively used.

---

## Storage Footprint and Fees

Soroban fees depend on the read/write footprint of each transaction. Here are the approximate footprints:

| Function | Read footprint | Write footprint |
|---|---|---|
| `initialize` | instance (Admin, Count) | instance (Admin, Count) |
| `register` (new) | instance (Count) + Package key | instance (Count) + Package key |
| `register` (update) | instance + Package key | Package key |
| `get_address` | Package key | none |
| `get_batch(n)` | n Package keys | none |
| `remove` | instance (Admin) + Package key | Package key (delete) |
| `verify` | instance (Admin) + Package key | Package key |
| `get_stats` | instance (Count) | none |

Read-only functions (`get_address`, `get_batch`, `get_stats`, `get_record`) are cheap because Soroban's fee model is proportional to the ledger entry bytes read.

`get_batch` with a large input vector (e.g., 200 dependencies) will read up to 200 persistent entries in one invocation. Each entry read is bounded by the `PackageRecord` XDR size (~400 bytes), so 200 entries ≈ 80 KB of read data — well within Soroban's per-transaction limits as of the current network parameters.

---

## Off-chain Indexing

The contract does not support on-chain key iteration. To enumerate all registrations (e.g., to build a registry UI, run TTL maintenance, or seed a new contract on upgrade), use Horizon's event streaming:

```bash
# Stream all PkgReg events for the contract
curl "https://horizon-testnet.stellar.org/contracts/<CONTRACT_ID>/events?type=contract&cursor=0"
```

Or using the Stellar SDK:

```javascript
const server = new StellarSdk.Horizon.Server("https://horizon-testnet.stellar.org");
const events = await server.contracts().forContract(CONTRACT_ID).events();
```

Each `PkgReg` event data tuple contains `(ecosystem, package_name, stellar_address, timestamp)` — enough to reconstruct the full registry state from scratch.
