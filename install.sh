#!/bin/bash

# Vortex Universal Install Script
# Automatically detects platform and installs the appropriate binary
# Also installs the systemd service for user-level daemon management

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print with colors
print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Vortex version to install (latest)
VERSION="v0.5.0"
REPO="exec/vortex"

print_status "🚀 Installing Vortex ${VERSION} - The Docker Killer"
echo

# Detect platform and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    armv7*) ARCH="armv7" ;;
    *) print_error "Unsupported architecture: $ARCH"; exit 1 ;;
esac

case $OS in
    linux)
        print_status "Detected Linux system"
        # Use simplified naming from GitHub Actions artifacts
        if [[ $ARCH == "x86_64" ]]; then
            PACKAGE="vortex-linux-amd64.tar.gz"
        else
            print_error "Unsupported Linux architecture: $ARCH (only x86_64/amd64 supported)"
            exit 1
        fi
        INSTALL_CMD="tar -xzf"
        ;;
    darwin)
        print_status "Detected macOS system"
        # Use simplified naming from GitHub Actions artifacts
        if [[ $ARCH == "aarch64" ]]; then
            PACKAGE="vortex-macos-arm64.tar.gz"
        elif [[ $ARCH == "x86_64" ]]; then
            PACKAGE="vortex-macos-amd64.tar.gz"
        else
            print_error "Unsupported macOS architecture: $ARCH"
            exit 1
        fi
        INSTALL_CMD="tar -xzf"
        ;;
    *)
        print_error "Unsupported operating system: $OS"
        exit 1
        ;;
esac

# Download URL
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${PACKAGE}"
print_status "Downloading: $PACKAGE"
print_status "From: $DOWNLOAD_URL"

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Download the package
if command -v curl &> /dev/null; then
    curl -fsSL -o "$PACKAGE" "$DOWNLOAD_URL"
elif command -v wget &> /dev/null; then
    wget -q -O "$PACKAGE" "$DOWNLOAD_URL"  
else
    print_error "Neither curl nor wget found. Please install one of them."
    exit 1
fi

print_success "Download completed!"

# Install based on package type
case $PACKAGE in
    *.deb)
        print_status "Installing Debian package..."
        $INSTALL_CMD "$PACKAGE"
        ;;
    *.rpm)
        print_status "Installing RPM package..."
        $INSTALL_CMD "$PACKAGE"
        ;;
    *.tar.gz)
        print_status "Extracting binary..."
        $INSTALL_CMD "$PACKAGE"

        # Find the vortex binary and copy it to /usr/local/bin
        BINARY_PATH=$(find . -name "vortex" -type f | head -1)
        if [[ -n "$BINARY_PATH" ]]; then
            print_status "Installing binary to /usr/local/bin/vortex"
            if [[ $OS == "darwin" ]]; then
                sudo cp "$BINARY_PATH" /usr/local/bin/vortex
            else
                sudo cp "$BINARY_PATH" /usr/local/bin/vortex
            fi
            sudo chmod +x /usr/local/bin/vortex
        else
            print_error "Could not find vortex binary in extracted package"
            exit 1
        fi

        # Install systemd service on Linux
        if [[ $OS == "linux" ]]; then
            print_status "Installing systemd service..."
            # Check if we're running from the source repo
            if [[ -f "$TEMP_DIR/systemd/vortexd.service" ]]; then
                SERVICE_PATH="$TEMP_DIR/systemd/vortexd.service"
                print_status "Found systemd service file"
            else
                # Try to find from installed location
                SERVICE_PATH="/usr/local/bin/../share/vortex/vortexd.service"
                if [[ ! -f "$SERVICE_PATH" ]]; then
                    print_warning "Systemd service file not found - manual installation may be needed"
                    SERVICE_PATH=""
                fi
            fi

            if [[ -n "$SERVICE_PATH" ]]; then
                print_status "Installing service to /etc/systemd/system/vortexd.service"
                sudo cp "$SERVICE_PATH" /etc/systemd/system/vortexd.service
                sudo systemctl daemon-reload
                sudo systemctl enable vortexd
                sudo systemctl start vortexd
                print_success "Systemd service installed and started!"
            fi
        fi
        ;;
esac

# Cleanup
cd /
rm -rf "$TEMP_DIR"

print_success "Vortex installation completed! 🎉"
echo
print_status "Systemd service installed automatically on Linux! 🚀"
echo
print_status "Get started:"
echo -e "  ${CYAN}vortex dev --list${NC}              # List available templates"
echo -e "  ${CYAN}vortex dev --init${NC}              # Create workspace from current directory"
echo -e "  ${CYAN}vortex session create${NC}          # Create a new persistent VM session"
echo -e "  ${CYAN}vortex --help${NC}                  # Show all available commands"
echo
print_status "Documentation: https://github.com/${REPO}"
print_success "Welcome to the future of development environments! 20x faster than Docker! 🚀"