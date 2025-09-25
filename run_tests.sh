#!/bin/bash

set -e

echo "🧪 Running Ephemeral VM Test Suite..."
echo "=================================="

# Basic functionality tests
echo "📦 Testing basic VM creation..."
./ephemeral-vm.sh run alpine -e "echo 'Basic test passed!'"

echo ""
echo "🐧 Testing Ubuntu VM..."
./ephemeral-vm.sh run ubuntu -e "cat /etc/os-release | head -3"

echo ""
echo "🏷️  Testing image aliases..."
./ephemeral-vm.sh run node -e "node --version"
./ephemeral-vm.sh run python -e "python3 --version"

echo ""
echo "⚡ Testing complex shell commands..."
./ephemeral-vm.sh run alpine -e "ps aux | wc -l && echo 'Complex commands work!'"

echo ""
echo "📝 Testing templates..."
./ephemeral-vm.sh template minimal --command "echo 'Template test passed!'"

echo ""
echo "📋 Testing list command..."
./ephemeral-vm.sh list

echo ""
echo "🧹 Testing cleanup..."
./ephemeral-vm.sh cleanup

echo ""
echo "✅ All tests passed! Ephemeral VM is working correctly."