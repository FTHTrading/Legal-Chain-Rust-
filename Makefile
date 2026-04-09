# Makefile — Common commands for LEGAL-CHAIN development

.PHONY: build build-release check test fmt clippy dev local docker purge clean

# Build debug
build:
	cargo build

# Build release (including WASM runtime)
build-release:
	cargo build --release

# Check compilation without producing binaries
check:
	cargo check --all-targets

# Run all tests
test:
	cargo test --all

# Format all code
fmt:
	cargo fmt --all

# Run clippy lints
clippy:
	cargo clippy --all-targets -- -D warnings

# Start single-node dev chain (Alice)
dev: build-release
	./target/release/legal-chain-node \
		--dev \
		--tmp \
		--rpc-cors all \
		--rpc-methods unsafe \
		--rpc-port 9944

# Start local testnet (Alice only, use docker-compose for multi-node)
local: build-release
	./target/release/legal-chain-node \
		--chain local \
		--alice \
		--validator \
		--tmp \
		--rpc-cors all \
		--rpc-methods unsafe \
		--rpc-port 9944

# Build Docker image
docker:
	docker build -t legal-chain-node .

# Purge dev chain data
purge:
	./target/release/legal-chain-node purge-chain --dev -y

# Clean all build artifacts
clean:
	cargo clean
