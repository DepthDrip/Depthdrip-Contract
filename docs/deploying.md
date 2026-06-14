# Deploying

Step-by-step guide for building, deploying, invoking, and upgrading the DepthDrip Contract on Stellar Testnet and Mainnet.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Build](#build)
- [Deploy to Testnet](#deploy-to-testnet)
- [Deploy to Mainnet](#deploy-to-mainnet)
- [Initialize the Contract](#initialize-the-contract)
- [Invoking Functions](#invoking-functions)
- [Verifying Deployment](#verifying-deployment)
- [Upgrading](#upgrading)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

Install the following tools before deploying:

**Rust and the WASM target:**
```bash
curl https://sh.rustup.rs -sSf | sh
rustup target add wasm32v1-none
```

**Stellar CLI:**
```bash
# macOS / Linux via cargo
cargo install --locked stellar-cli

# Or follow the official guide:
# https://developers.stellar.org/docs/tools/developer-tools/cli/install-stellar-cli
```

Verify:
```bash
stellar --version   # should be 21.x or newer
rustc --version     # should be 1.84+
```

---

## Build

Compile the contract to a WASM binary:

```bash
make build
# Equivalent: cargo build --release --target wasm32v1-none
```

Output: `target/wasm32v1-none/release/depthdrip_contract.wasm`

The `opt-level = "z"`, `lto = true`, and `codegen-units = 1` profile settings in `Cargo.toml` produce the smallest possible binary, which directly reduces deployment fees.

---

## Deploy to Testnet

### 1. Generate and fund a deployer account

```bash
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet
```

This creates a new keypair and requests Friendbot to fund it with test XLM.

### 2. Deploy the WASM binary

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/depthdrip_contract.wasm \
  --source deployer \
  --network testnet
```

The CLI prints the **contract ID** (a C-address, e.g. `CABC...`). Save it — you will need it for all subsequent invocations.

You can also run this via the Makefile:

```bash
make deploy-testnet
```

### 3. Set the contract ID in your environment

```bash
export CONTRACT_ID=CABC...
```

---

## Deploy to Mainnet

The process is identical to testnet, with `--network mainnet` substituted throughout. Before deploying to mainnet:

- Use a funded mainnet account (not Friendbot).
- Confirm the WASM binary has been reviewed and tested thoroughly on testnet.
- Consider using a multisig account as the admin address.

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/depthdrip_contract.wasm \
  --source <MAINNET_KEY_NAME> \
  --network mainnet
```

Or:
```bash
make deploy-mainnet
```

---

## Initialize the Contract

After deployment, call `initialize` exactly once to set the admin address. This must be done before any registrations.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- initialize \
  --admin G...   # replace with your admin G-address
```

**Important:** Use a multisig or governance contract as the admin in production. There is no way to rotate the admin after initialization without deploying a new contract.

---

## Invoking Functions

### Register a package

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source <MAINTAINER_KEYPAIR> \
  --network testnet \
  -- register \
  --ecosystem npm \
  --package_name lodash \
  --stellar_address G...
```

The `--source` must be the keypair corresponding to the `--stellar_address` argument, because the contract enforces `stellar_address.require_auth()`.

### Look up an address

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_address \
  --ecosystem npm \
  --package_name lodash
```

### Verify a package (admin only)

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source <ADMIN_KEYPAIR> \
  --network testnet \
  -- verify \
  --ecosystem npm \
  --package_name lodash
```

### Remove a package

```bash
# As the registrant:
stellar contract invoke \
  --id $CONTRACT_ID \
  --source <MAINTAINER_KEYPAIR> \
  --network testnet \
  -- remove \
  --ecosystem npm \
  --package_name lodash
```

### Get stats

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_stats
```

---

## Verifying Deployment

After deploying and initializing, run a quick smoke test:

```bash
# 1. Register a test package
stellar contract invoke --id $CONTRACT_ID --source <KEY> --network testnet \
  -- register --ecosystem npm --package_name test-pkg --stellar_address G...

# 2. Retrieve it
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- get_address --ecosystem npm --package_name test-pkg
# Expect: Some(G...)

# 3. Check stats
stellar contract invoke --id $CONTRACT_ID --network testnet \
  -- get_stats
# Expect: total_registered = 1

# 4. Remove the test entry
stellar contract invoke --id $CONTRACT_ID --source <KEY> --network testnet \
  -- remove --ecosystem npm --package_name test-pkg
```

---

## Upgrading

The current contract has **no `upgrade` entrypoint**. Soroban supports in-place WASM replacement via the `stellar contract upload` + host function approach, but this contract does not expose that mechanism to keep the security surface minimal.

To upgrade to a new contract version:

### Option A — New contract, data migration script

1. Deploy the new contract version.
2. Initialize it with the same admin address.
3. Run a migration script that:
   - Reads all existing registrations from the old contract's event history (via Horizon `getEvents` for `PkgReg`).
   - Re-invokes `register` on the new contract for each entry.
4. Announce the new contract ID to `depthdrip-cli` and `depthdrip-app` users.
5. Deprecate the old contract (optionally call `remove` on its admin-controlled entries).

### Option B — Add upgrade support in a new version

If in-place upgrades are needed, the next contract version can add:

```rust
pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
    admin.require_auth();
    env.deployer().update_current_contract_wasm(new_wasm_hash);
}
```

This allows the admin to upload a new WASM and update the contract in place without changing the contract ID.

---

## Troubleshooting

**`error: failed to select a version for soroban-sdk`**
Your `Cargo.toml` has an invalid SDK version. Ensure `soroban-sdk = "26.1.0"` (not `0.34.0`).

**`'testutils' feature is not supported on 'wasm' target`**
The `testutils` feature must only be enabled for `[dev-dependencies]`. Your `[dependencies]` entry must not include `features = ["testutils"]`. See `Cargo.toml` in this repo for the correct split.

**`wasm32-unknown-unknown is unsupported`**
soroban-sdk 26.x requires `wasm32v1-none`. Run `rustup target add wasm32v1-none` and change your build command to `--target wasm32v1-none`.

**`Transaction simulation failed`**
Ensure the account funding the transaction has enough XLM to cover the upload fee (WASM size × fee rate) plus the contract deployment base fee. On testnet, run `stellar keys fund <name> --network testnet` to top up.

**`Contract not initialized`**
You must call `initialize(admin)` before any other function. It can only be called once.
