#!/bin/bash

# Vortex Universal Install Script
# Automatically detects platform and installs the appropriate binary

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
VERSION="v0.3.0"
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
        # Check for package manager and install accordingly
        if command -v apt-get &> /dev/null; then
            print_status "Detected Ubuntu/Debian system"
            PACKAGE="vortex_${VERSION#v}_${ARCH}.deb"
            INSTALL_CMD="sudo dpkg -i"
        elif command -v yum &> /dev/null || command -v dnf &> /dev/null; then
            print_status "Detected RHEL/CentOS/Fedora system"  
            PACKAGE="vortex-${VERSION#v}-1.${ARCH}.rpm"
            if command -v dnf &> /dev/null; then
                INSTALL_CMD="sudo dnf install -y"
            else
                INSTALL_CMD="sudo yum install -y"
            fi
        else
            print_status "Generic Linux system - using binary install"
            PACKAGE="vortex-${VERSION}-${ARCH}-unknown-linux-gnu.tar.gz"
            INSTALL_CMD="tar -xzf"
        fi
        ;;
    darwin)
        print_status "Detected macOS system"
        if [[ $ARCH == "aarch64" ]]; then
            ARCH="arm64"
        fi
        PACKAGE="vortex-${VERSION}-${ARCH}-apple-darwin.tar.gz"
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
        ;;
esac

# Cleanup
cd /
rm -rf "$TEMP_DIR"

print_success "Vortex installation completed! 🎉"
echo
print_status "Verify installation:"
echo -e "  ${CYAN}vortex --version${NC}"
echo
print_status "Get started:"
echo -e "  ${CYAN}vortex shell alpine${NC}    # Start interactive Alpine VM"
echo -e "  ${CYAN}vortex run ubuntu -e 'uname -a'${NC}    # Run command in Ubuntu"
echo -e "  ${CYAN}vortex --help${NC}         # Show all available commands"
echo
print_status "Documentation: https://github.com/${REPO}"
print_success "Welcome to the future of containerization! Docker who? 😎"