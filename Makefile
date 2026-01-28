.PHONY: help build test clean run-server run-cli install dev check fmt clippy bench

# Default target
help:
	@echo "Sandwich Rust - Makefile Commands"
	@echo ""
	@echo "Build & Test:"
	@echo "  make build        - Build all crates in release mode"
	@echo "  make build-dev    - Build all crates in dev mode"
	@echo "  make test         - Run all tests"
	@echo "  make check        - Run cargo check on workspace"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt          - Format code with rustfmt"
	@echo "  make clippy       - Run clippy linter"
	@echo "  make bench        - Run benchmarks"
	@echo ""
	@echo "Run:"
	@echo "  make run-server   - Start the web server"
	@echo "  make run-cli      - Run CLI with example"
	@echo "  make dev          - Start server with auto-reload (requires cargo-watch)"
	@echo ""
	@echo "Install:"
	@echo "  make install      - Install binaries to cargo bin directory"

# Build targets
build:
	cargo build --workspace --release

build-dev:
	cargo build --workspace

# Test targets
test:
	cargo test --workspace

test-verbose:
	cargo test --workspace -- --nocapture

# Code quality
check:
	cargo check --workspace

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace -- -D warnings

bench:
	cargo bench

# Run targets
run-server:
	cargo run --release --bin sandwich-server

run-cli:
	cargo run --bin sandwich-cli -- examples

dev:
	cargo watch -x 'run --bin sandwich-server'

# Install
install:
	cargo install --path crates/sandwich-server
	cargo install --path crates/sandwich-cli

# Clean
clean:
	cargo clean

# Docker
docker-build:
	docker build -t sandwich-server:latest .

docker-run:
	docker run -p 3000:3000 --env-file .env sandwich-server:latest

# Examples
example-basic:
	cargo run --bin sandwich-cli -- compose --example basic --output examples/basic.jpg

example-full:
	cargo run --bin sandwich-cli -- compose --example full-outfit --output examples/full-outfit.jpg

example-patches:
	cargo run --bin sandwich-cli -- compose --example with-patches --output examples/with-patches.jpg

# All examples
examples: example-basic example-full example-patches
	@echo "All examples generated in examples/"
