.PHONY: all build build-release release install install-user install-system uninstall uninstall-user uninstall-system clean test test-unit test-integration check fmt lint run

PREFIX    ?= /usr/local
USER_BIN  ?= $(HOME)/.local/bin
LEVEL     ?= minor

# Default target
all: check build test

# Build debug version
build:
	cargo build

# Build release version
build-release:
	cargo build --release

# Publish a new release (usage: make release or make release LEVEL=patch)
release:
	cargo release $(LEVEL) --execute --no-confirm

# Install via Cargo into ~/.cargo/bin
install:
	cargo install --path . --locked --force

# Copy the release binary to ~/.local/bin — no sudo
install-user: build-release
	mkdir -p $(USER_BIN)
	install -m 755 target/release/bd $(USER_BIN)/bd
	@echo "Installed $(USER_BIN)/bd"
	@echo "If \`bd\` is not found, add this to your shell: export PATH=\"$(USER_BIN):\$$PATH\""

# System-wide install — requires sudo
install-system: build-release
	sudo install -m 755 target/release/bd $(PREFIX)/bin/bd
	@echo "Installed $(PREFIX)/bin/bd"

uninstall:
	cargo uninstall byedroid 2>/dev/null || true

uninstall-user:
	rm -f $(USER_BIN)/bd

uninstall-system:
	sudo rm -f $(PREFIX)/bin/bd

# Clean build artifacts
clean:
	cargo clean

# Run all tests
test:
	cargo nextest run

# Run unit tests only
test-unit:
	cargo nextest run --lib --bins

# Run integration tests only
test-integration:
	cargo nextest run --tests

# cargo check + clippy
check:
	cargo check
	cargo clippy -- -D warnings

# Format code
fmt:
	cargo fmt

# Check formatting + clippy
lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings

# Run with arguments (usage: make run ARGS="--help")
run:
	cargo run -- $(ARGS)
