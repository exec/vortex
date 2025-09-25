#!/bin/bash

echo "=== Testing Vortex Interactive Shell ==="
echo ""
echo "This script will test the interactive shell functionality."
echo "You'll need to manually type commands and 'exit' to complete the test."
echo ""
echo "Press Enter to start the interactive shell test..."
read -r

echo "Starting interactive shell..."
./target/debug/vortex shell alpine

echo ""
echo "Shell test completed. The VM should have been automatically cleaned up."
echo ""
echo "Let's verify no VMs are running:"
./target/debug/vortex list