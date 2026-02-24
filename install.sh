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

# Vortex version to install - automatically fetch latest from GitHub
# Override with VORTEX_VERSION environment variable for testing
REPO="exec/vortex"
if [ -n "$VORTEX_VERSION" ]; then
    VERSION="$VORTEX_VERSION"
else
    VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | cut -d'"' -f4)
    if [ -z "$VERSION" ]; then
        # Fallback if API fails
        VERSION="v0.5.0"
    fi
fi

print_status "🚀 Installing Vortex ${VERSION} - The Docker Killer"
echo

# Detect platform and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    armv7*) ARCH="armv7" ;;
    *) print_error "Unsupported architecture: $ARCH"; exit 1 ;;
esac

case $OS in
    linux)
        print_status "Detected Linux system"
        # Check if we're on Arch Linux
        if [[ -f /etc/arch-release ]]; then
            ARCH_PACKAGE_TYPE="arch"
        elif [[ -f /etc/redhat-release ]] || command -v dnf &> /dev/null || command -v yum &> /dev/null; then
            ARCH_PACKAGE_TYPE="rpm"
        else
            ARCH_PACKAGE_TYPE="generic"
        fi

        # Use simplified naming from GitHub Actions artifacts
        if [[ $ARCH == "amd64" ]]; then
            if [[ $ARCH_PACKAGE_TYPE == "arch" ]]; then
                PACKAGE="vortex-${VERSION}-x86_64-unknown-linux-gnu.pkg.tar.gz"
            elif [[ $ARCH_PACKAGE_TYPE == "rpm" ]]; then
                # RPM packages are built as vortex-VERSION-1.x86_64.rpm
                PACKAGE="vortex-${VERSION}-1.x86_64.rpm"
            else
                PACKAGE="vortex-${VERSION}-linux-amd64.tar.gz"
            fi
        elif [[ $ARCH == "arm64" ]]; then
            if [[ $ARCH_PACKAGE_TYPE == "arch" ]]; then
                print_error "Arch Linux ARM64 packages not yet available"
                exit 1
            elif [[ $ARCH_PACKAGE_TYPE == "rpm" ]]; then
                print_error "RPM ARM64 packages not yet available"
                exit 1
            else
                PACKAGE="vortex-${VERSION}-linux-arm64.tar.gz"
            fi
        else
            print_error "Unsupported Linux architecture: $ARCH (only amd64 and arm64 supported)"
            exit 1
        fi
        INSTALL_CMD="tar -xzf"
        ;;
    darwin)
        print_status "Detected macOS system"
        # Use simplified naming from GitHub Actions artifacts
        if [[ $ARCH == "arm64" ]]; then
            PACKAGE="vortex-${VERSION}-macos-arm64.tar.gz"
        elif [[ $ARCH == "amd64" ]]; then
            PACKAGE="vortex-${VERSION}-macos-amd64.tar.gz"
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
        sudo dpkg -i "$PACKAGE"
        ;;
    *.rpm)
        print_status "Installing RPM package..."
        if command -v dnf &> /dev/null; then
            sudo dnf install -y "$PACKAGE"
        elif command -v yum &> /dev/null; then
            sudo yum install -y "$PACKAGE"
        else
            # Fallback to rpm command if dnf/yum not available
            sudo rpm -ivh "$PACKAGE"
        fi
        ;;
    *.pkg.tar.gz)
        print_status "Installing Arch package..."
        $INSTALL_CMD "$PACKAGE"

        # Find the vortex binary and copy it to /usr/bin
        BINARY_PATH=$(find . -name "vortex" -type f | head -1)
        if [[ -n "$BINARY_PATH" ]]; then
            print_status "Installing binary to /usr/bin/vortex"
            sudo cp "$BINARY_PATH" /usr/bin/vortex
            sudo chmod +x /usr/bin/vortex
        else
            print_error "Could not find vortex binary in extracted package"
            exit 1
        fi

        # Install systemd service on Linux
        print_status "Installing systemd service..."
        SERVICE_PATH="$TEMP_DIR/usr/share/vortex/vortexd.service"
        if [[ -f "$SERVICE_PATH" ]]; then
            print_status "Found systemd service file"
            print_status "Installing service to /etc/systemd/system/vortexd.service"
            sudo cp "$SERVICE_PATH" /etc/systemd/system/vortexd.service
            sudo systemctl daemon-reload
            sudo systemctl enable vortexd
            sudo systemctl start vortexd
            print_success "Systemd service installed and started!"
        else
            print_warning "Systemd service file not found - manual installation may be needed"
        fi
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