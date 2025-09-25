#!/bin/bash

# Linux Package Builder for Vortex
# This script demonstrates how to build Linux packages for Vortex

set -e

echo "🔧 Vortex Linux Package Builder"
echo "==============================="
echo
echo "This script shows how to build Vortex for Linux platforms."
echo "The GitHub Actions workflow is fully configured to build all platforms"
echo "automatically, but currently has permission issues that need to be resolved."
echo
echo "📦 Supported Platforms:"
echo "  - Linux x86_64 (.tar.gz binary)"
echo "  - Linux ARM64 (.tar.gz binary)" 
echo "  - Ubuntu/Debian packages (.deb for amd64, arm64)"
echo "  - RHEL/CentOS packages (.rpm for x86_64, aarch64)"
echo
echo "🛠️ To build manually:"
echo "1. Install Rust cross-compilation targets:"
echo "   rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu"
echo
echo "2. Install cross-compilation toolchain (Ubuntu/Debian):"
echo "   sudo apt-get install gcc-x86-64-linux-gnu gcc-aarch64-linux-gnu"
echo
echo "3. Build for Linux x86_64:"
echo "   cargo build --release --target x86_64-unknown-linux-gnu"
echo
echo "4. Build for Linux ARM64:"
echo "   CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \\"
echo "   cargo build --release --target aarch64-unknown-linux-gnu"
echo
echo "5. Create packages:"
echo "   # Binary packages"
echo "   tar -czf vortex-v0.3.0-x86_64-unknown-linux-gnu.tar.gz -C target/x86_64-unknown-linux-gnu/release vortex"
echo "   tar -czf vortex-v0.3.0-aarch64-unknown-linux-gnu.tar.gz -C target/aarch64-unknown-linux-gnu/release vortex"
echo
echo "   # .deb packages (requires dpkg-deb)"
echo "   # .rpm packages (requires rpmbuild)"
echo
echo "🚀 The GitHub Actions workflow at .github/workflows/release.yml"
echo "   automatically builds all these packages when properly configured."
echo
echo "📋 Current Status:"
echo "   ✅ CI/CD workflow complete"
echo "   ✅ Multi-platform build scripts ready"
echo "   ⚠️  GitHub Actions permissions need configuration"
echo "   ✅ macOS ARM64 package available now"
echo
echo "For the latest packages, check:"
echo "https://github.com/exec/vortex/releases/tag/v0.3.0"

# Create a simple demonstration package
mkdir -p demo-linux-package/bin
echo "#!/bin/bash
echo 'This is a demonstration package for Vortex Linux builds.'
echo 'The actual Linux binaries will be built by the CI/CD pipeline'
echo 'once GitHub Actions permissions are configured.'
echo 'For now, the macOS ARM64 package is available and fully functional.'
exit 0" > demo-linux-package/bin/vortex-demo

chmod +x demo-linux-package/bin/vortex-demo
tar -czf vortex-v0.3.0-linux-demo.tar.gz -C demo-linux-package .