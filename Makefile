# Vortex Makefile
# Build and install Vortex locally without GitHub dependencies

PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
MANDIR = $(PREFIX)/share/man/man1

# Build configuration
CARGO_FLAGS = --release
TARGET_DIR = target/release
BINARY_NAME = vortex

# Detect OS and architecture
OS ?= $(shell uname -s)
ARCH ?= $(shell uname -m)

# Check for package managers
HAVE_BREW = $(shell command -v brew 2>/dev/null && echo "yes" || echo "no")
HAVE_DNF = $(shell command -v dnf 2>/dev/null && echo "yes" || echo "no")
HAVE_APT = $(shell command -v apt-get 2>/dev/null && echo "yes" || echo "no")

.PHONY: all build test install uninstall clean help install-prereqs check-prereqs

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

# Check and install prereqs (libkrun, buildah, krunvm)
install-prereqs:
	@echo "🔍 Checking and installing prereqs for $(OS) $(ARCH)..."
	@sh -c ' \
	case "$(OS)" in \
	Darwin) \
		if command -v brew >/dev/null 2>&1; then \
			echo "Installing krunvm via Homebrew..."; \
			brew tap slp/krun 2>/dev/null || true; \
			brew install krunvm || { echo "❌ Failed to install krunvm via Homebrew"; exit 1; }; \
		else \
			echo "❌ Homebrew not found. Please install krunvm manually:"; \
			echo "  1. Install Homebrew: https://brew.sh"; \
			echo "  2. Then run: brew tap slp/krun && brew install krunvm"; \
			exit 1; \
		fi; \
		echo "✅ krunvm installed"; \
		;; \
	Linux) \
		if command -v apt-get >/dev/null 2>&1; then \
			echo "Installing buildah via APT..."; \
			sudo apt-get update -qq 2>/dev/null; \
			sudo apt-get install -y -qq buildah 2>/dev/null || { echo "⚠️  Could not install buildah via APT"; }; \
		fi; \
		if command -v dnf >/dev/null 2>&1; then \
			echo "Installing buildah via DNF..."; \
			sudo dnf install -y buildah 2>/dev/null || { echo "⚠️  Could not install buildah via DNF"; }; \
		fi; \
		echo "Building libkrun from source..."; \
		if [ ! -d /tmp/libkrun ]; then \
			git clone --depth 1 https://github.com/containers/libkrun.git /tmp/libkrun 2>/dev/null || { echo "❌ Failed to clone libkrun"; exit 1; }; \
		fi; \
		cd /tmp/libkrun && make -j$$(nproc) 2>/dev/null || { echo "❌ Failed to build libkrun"; exit 1; }; \
		sudo cp /tmp/libkrun/target/release/libkrun.so /usr/local/lib/ 2>/dev/null || sudo cp /tmp/libkrun/target/release/libkrun.so.1.17.3 /usr/local/lib/ 2>/dev/null || { echo "❌ Failed to install libkrun"; exit 1; }; \
		sudo ldconfig; \
		echo "Building krunvm from source..."; \
		if [ ! -d /tmp/krunvm ]; then \
			git clone --depth 1 --branch v0.2.6 https://github.com/containers/krunvm.git /tmp/krunvm 2>/dev/null || { echo "❌ Failed to clone krunvm"; exit 1; }; \
		fi; \
		cd /tmp/krunvm && make -j$$(nproc) 2>/dev/null || { echo "❌ Failed to build krunvm"; exit 1; }; \
		sudo cp /tmp/krunvm/target/release/krunvm /usr/local/bin/ 2>/dev/null || { echo "❌ Failed to install krunvm"; exit 1; }; \
		sudo chmod +x /usr/local/bin/krunvm; \
		echo "✅ Prereqs installed"; \
		;; \
	*) \
		echo "❌ Unsupported OS: $(OS)"; \
		echo "Please install krunvm and buildah manually:"; \
		echo "  - krunvm: https://github.com/containers/krunvm"; \
		echo "  - buildah: https://github.com/containers/buildah"; \
		exit 1; \
		;; \
	esac'

# Check if prereqs are installed (doesn't install)
check-prereqs:
	@echo "🔍 Checking prereqs..."
	@if ! command -v buildah >/dev/null 2>&1; then \
		echo "❌ buildah not found. Please install:"; \
		echo "  Linux (Debian/Ubuntu): sudo apt-get install buildah"; \
		echo "  Linux (Fedora): sudo dnf install buildah"; \
		echo "  macOS: brew install buildah"; \
		exit 1; \
	fi
	@if ! command -v krunvm >/dev/null 2>&1; then \
		echo "❌ krunvm not found. Please install:"; \
		echo "  Linux: See https://github.com/containers/krunvm"; \
		echo "  macOS: brew tap slp/krun && brew install krunvm"; \
		exit 1; \
	fi
	@echo "✅ All prereqs installed"

# Install locally from built binary
install: build check-prereqs
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

# Clean everything including prereqs (build artifacts)
clean-all: clean
	@echo "🗑️  Cleaning prereqs..."
	@rm -rf /tmp/libkrun /tmp/krunvm
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
	@echo "  build             - Build release binary"
	@echo "  test              - Run comprehensive test suite"
	@echo "  install           - Install to $(BINDIR) with prereq check"
	@echo "  install-prereqs   - Install libkrun, buildah, krunvm"
	@echo "  check-prereqs     - Check if prereqs are installed"
	@echo "  uninstall         - Remove from $(BINDIR)"
	@echo "  clean             - Clean build artifacts"
	@echo "  clean-all         - Clean artifacts and prereqs"
	@echo ""
	@echo "  Development:"
	@echo "  dev               - Quick development build"
	@echo "  fmt               - Format code"
	@echo "  lint              - Run clippy lints"
	@echo "  audit             - Security audit"
	@echo "  check             - Run all quality checks"
	@echo "  watch             - Watch for changes"
	@echo ""
	@echo "  Examples:"
	@echo "  make install                      # Install to /usr/local/bin"
	@echo "  sudo make install                 # Install system-wide"
	@echo "  make install-prereqs              # Install libkrun, buildah, krunvm"
	@echo "  make install PREFIX=~/.local      # Install to user directory"
	@echo ""