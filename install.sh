#!/bin/bash

set -euo pipefail

echo "🔥 Installing Ephemeral VM..."

# Check if we're on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo "❌ This installer is for macOS only. For Linux, use Firecracker directly."
    exit 1
fi

# Check for Homebrew
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew is required. Install it from https://brew.sh/"
    exit 1
fi

# Install Rust if not present
if ! command -v rustc &> /dev/null; then
    echo "📦 Installing Rust..."
    brew install rust
fi

# Add krun tap and install dependencies
echo "📦 Installing krunvm and dependencies..."
brew tap slp/krun 2>/dev/null || true
brew install buildah libkrunfw libkrun krunvm

# Create case-sensitive volume for krunvm if it doesn't exist
if [[ ! -d "/Volumes/krunvm" ]]; then
    echo "💾 Creating case-sensitive APFS volume for krunvm..."
    diskutil apfs addVolume disk3 "Case-sensitive APFS" krunvm
fi

# Create container configuration
echo "🔧 Setting up container configuration..."
sudo mkdir -p /opt/homebrew/etc/containers/

sudo tee /opt/homebrew/etc/containers/registries.conf > /dev/null <<EOF
[registries.search]
registries = ['docker.io']

[registries.insecure]
registries = []

[registries.block]
registries = []
EOF

sudo tee /opt/homebrew/etc/containers/policy.json > /dev/null <<EOF
{
    "default": [
        {
            "type": "insecureAcceptAnything"
        }
    ],
    "transports":
        {
            "docker-daemon":
                {
                    "": [{"type":"insecureAcceptAnything"}]
                }
        }
}
EOF

# Build ephemeral-vm
echo "🔨 Building ephemeral-vm..."
cargo build --release

# Create wrapper script with proper library paths
echo "📝 Creating wrapper script..."
cat > ephemeral-vm <<EOF
#!/bin/bash
export DYLD_LIBRARY_PATH="/opt/homebrew/opt/libkrunfw/lib:/opt/homebrew/opt/libkrun/lib:\$DYLD_LIBRARY_PATH"
exec "\$(dirname "\$0")/target/release/ephemeral-vm" "\$@"
EOF

chmod +x ephemeral-vm

echo "✅ Installation complete!"
echo ""
echo "Usage:"
echo "  ./ephemeral-vm run alpine -e 'echo Hello World'"
echo "  ./ephemeral-vm run ubuntu -e 'uname -a && free -h'"
echo "  ./ephemeral-vm template dev --command 'bash'"
echo "  ./ephemeral-vm list"
echo "  ./ephemeral-vm cleanup"
echo ""
echo "To install system-wide, copy 'ephemeral-vm' to /usr/local/bin/"