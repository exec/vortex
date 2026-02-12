# Vortex Makefile
# Build and install Vortex locally without GitHub dependencies

PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
MANDIR = $(PREFIX)/share/man/man1

# Build configuration
CARGO_FLAGS = --release
TARGET_DIR = target/release
BINARY_NAME = vortex

.PHONY: all build test install uninstall clean help

# Default target
all: build

# Build the project
build:
	@echo "🔨 Building Vortex v0.5.0..."
	cargo build $(CARGO_FLAGS)
	@echo "✅ Build complete: $(TARGET_DIR)/$(BINARY_NAME)"

# Run comprehensive test suite
test:
	@echo "🧪 Running test suite..."
	./test_runner.sh

# Install locally from built binary
install: build
	@echo "📦 Installing Vortex to $(BINDIR)..."
	@mkdir -p $(BINDIR)
	@cp $(TARGET_DIR)/$(BINARY_NAME) $(BINDIR)/$(BINARY_NAME)
	@chmod +x $(BINDIR)/$(BINARY_NAME)
	@echo "✅ Vortex installed to $(BINDIR)/$(BINARY_NAME)"
	@echo "🚀 Run 'vortex --help' to get started"

# Uninstall
uninstall:
	@echo "🗑️  Removing Vortex from $(BINDIR)..."
	@rm -f $(BINDIR)/$(BINARY_NAME)
	@echo "✅ Vortex uninstalled"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean complete"

# Development helpers
dev: build
	@echo "🔥 Development build ready"
	@./$(TARGET_DIR)/$(BINARY_NAME) --version

# Format code
fmt:
	cargo fmt

# Lint code
lint:
	cargo clippy -- -D warnings

# Security audit
audit:
	cargo audit

# Check everything before commit
check: fmt lint test audit
	@echo "✅ All checks passed - ready for commit"

# Install development dependencies
dev-deps:
	cargo install cargo-audit cargo-watch cargo-edit

# Watch for changes during development
watch:
	cargo watch -x "build" -x "test"

# Help
help:
	@echo "Vortex Makefile - Available targets:"
	@echo ""
	@echo "  build     - Build release binary"
	@echo "  test      - Run comprehensive test suite"
	@echo "  install   - Install to $(BINDIR) (use PREFIX= to change)"
	@echo "  uninstall - Remove from $(BINDIR)"
	@echo "  clean     - Clean build artifacts"
	@echo ""
	@echo "  Development:"
	@echo "  dev       - Quick development build"
	@echo "  fmt       - Format code"
	@echo "  lint      - Run clippy lints"
	@echo "  audit     - Security audit"
	@echo "  check     - Run all quality checks"
	@echo "  watch     - Watch for changes"
	@echo ""
	@echo "  Examples:"
	@echo "  make install              # Install to /usr/local/bin"
	@echo "  sudo make install         # Install system-wide"
	@echo "  make install PREFIX=~/.local  # Install to user directory"
	@echo ""