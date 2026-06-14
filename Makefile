BUILD_DIR=target/wasm32v1-none/release
WASM=depthdrip_contract.wasm

.PHONY: build test fmt clippy wasm deploy-testnet deploy-mainnet invoke-register invoke-lookup invoke-batch

build:
	cargo build --release --target wasm32v1-none

wasm: build
	@echo "wasm build done"

test:
	cargo test

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy --all-targets -- -D warnings

deploy-testnet:
	./scripts/deploy.sh testnet $(WASM)

deploy-mainnet:
	./scripts/deploy.sh mainnet $(WASM)

invoke-register:
	# Example: make invoke-register ARGS="<env args>"
	echo "Use the soroban CLI to invoke register"

invoke-lookup:
	echo "Use the soroban CLI to invoke get_address"

invoke-batch:
	echo "Use the soroban CLI to invoke get_batch"
