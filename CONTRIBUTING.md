# Contributing to DepthDrip Contract

Thank you for your interest in contributing! DepthDrip is an open-source project and we welcome contributions of all kinds — bug fixes, new features, documentation improvements, and test coverage.

Please read this document before opening a pull request. It will save you and the reviewers time.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Ways to Contribute](#ways-to-contribute)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Security Vulnerabilities](#security-vulnerabilities)

---

## Code of Conduct

This project follows the [Contributor Covenant](https://www.contributor-covenant.org/version/2/1/code_of_conduct/) Code of Conduct. By participating, you agree to uphold a welcoming and respectful environment for everyone. Harassment, discrimination, or exclusionary behaviour will not be tolerated.

---

## Ways to Contribute

- **Bug reports** — open a GitHub Issue with steps to reproduce, expected vs. actual behaviour, and your environment (Rust version, OS, Stellar CLI version).
- **Feature requests** — open a GitHub Issue describing the use case, not just the solution. Include context on how it fits the DepthDrip payment protocol.
- **Bug fixes** — fork, fix, add a test that reproduces the bug, open a PR.
- **New features** — for anything beyond a small change, open an Issue first to discuss design before writing code.
- **Documentation** — typos, clarifications, new examples, and additional doc pages are always welcome.
- **Tests** — additional integration test coverage is very welcome, especially for edge cases in auth and storage.

---

## Getting Started

### 1. Fork and clone

```bash
git clone https://github.com/<your-handle>/Depthdrip-Contract.git
cd Depthdrip-Contract
```

### 2. Install prerequisites

```bash
# Rust
curl https://sh.rustup.rs -sSf | sh

# WASM target
rustup target add wasm32v1-none

# Stellar CLI (optional, needed for manual testing)
cargo install --locked stellar-cli
```

### 3. Verify everything works

```bash
make build   # should produce target/wasm32v1-none/release/depthdrip_contract.wasm
make test    # all 7 integration tests should pass
make clippy  # no warnings
make fmt     # no diff
```

---

## Development Workflow

We use a standard fork-and-branch model.

```bash
# Create a feature branch off main
git checkout -b fix/my-bug-description

# Make your changes
# ...

# Run the full check suite
make fmt
make clippy
make test
make build

# Commit and push
git add src/ tests/ docs/   # stage only relevant files
git commit -m "fix: short description of what changed"
git push -u origin fix/my-bug-description
```

Then open a pull request on GitHub against the `main` branch.

---

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format:

```
<type>(<scope>): <short summary>

[optional body]

[optional footer]
```

Common types:

| Type | When to use |
|---|---|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation only |
| `test` | Adding or fixing tests |
| `refactor` | Code change that doesn't fix a bug or add a feature |
| `chore` | Build process, dependency updates, CI changes |

**Examples:**
```
feat(contract): add get_batch function for bulk address resolution
fix(storage): prevent count decrement on remove
docs(abi): document verified_at sentinel value convention
test(integration): add test for duplicate register idempotency
```

---

## Pull Request Process

1. **Fill in the PR template** — describe what changed, why, and how it was tested.
2. **Keep PRs focused** — one concern per PR. A PR that fixes a bug and refactors unrelated code will be asked to split.
3. **Pass CI** — all CI checks (build, test, clippy, fmt) must be green before review.
4. **Add tests** — any change to contract behaviour needs at least one new or updated test in `tests/integration_tests.rs`.
5. **Update docs** — if you change a function signature, storage key, or event, update the relevant file in `docs/`.
6. **Changelog** — add an entry under `## [Unreleased]` in `CHANGELOG.md`.
7. **Two approvals** — PRs require two maintainer approvals before merging (one is sufficient for documentation-only PRs).

---

## Testing Guidelines

All tests live in `tests/integration_tests.rs` and use Soroban's `testutils` harness.

**Run all tests:**
```bash
make test
# or: cargo test
```

**Run a single test:**
```bash
cargo test register_and_lookup
```

**Test naming:** use `snake_case` and describe the behaviour, not the implementation:
- Good: `register_and_lookup`, `duplicate_register_does_not_increment_count`
- Avoid: `test1`, `test_register`

**What to test:**

- The happy path for each function.
- Auth failures (e.g., calling `verify` as non-admin should panic).
- Edge cases: double `initialize`, duplicate `register`, `remove` of non-existent key.
- Counter invariants: count never double-increments, count is not decremented on removal.

**Auth in tests:** use `env.mock_all_auths()` in the `setup()` helper to bypass signature verification. For tests that specifically check auth enforcement, use `env.mock_auth_for(...)` or remove `mock_all_auths` and sign the transaction explicitly.

---

## Documentation

The `docs/` directory contains the authoritative reference for this contract. When contributing changes that affect the contract's public interface, storage layout, or event schema, update the relevant document:

| Changed area | Doc to update |
|---|---|
| Function signature / behaviour | [`docs/abi.md`](docs/abi.md) |
| Storage keys or types | [`docs/storage.md`](docs/storage.md) |
| System design or data flow | [`docs/architecture.md`](docs/architecture.md) |
| Deployment steps or tooling | [`docs/deploying.md`](docs/deploying.md) |

Documentation PRs (typo fixes, added examples) are merged with a single maintainer review.

---

## Security Vulnerabilities

**Please do not open a public GitHub Issue for security vulnerabilities.**

If you discover a security issue in the contract (e.g., an auth bypass, a storage manipulation vector, or a panic that can be triggered by an adversary), report it privately:

1. Email `security@depthdrip.io` with a description and steps to reproduce.
2. We will acknowledge your report within 48 hours and work with you on a coordinated disclosure timeline.

We take security seriously. Critical vulnerabilities may be eligible for a bug bounty at maintainer discretion.
