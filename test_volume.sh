#!/bin/bash

set -e

echo "Testing volume mounting..."

# Create test file
mkdir -p test_data
echo "Hello from host!" > test_data/hello.txt

# Run VM with volume mount
./ephemeral-vm.sh run alpine \
  -v "$(pwd)/test_data:/data" \
  -e "cat /data/hello.txt && echo 'Volume test passed!'"

# Clean up
rm -rf test_data

echo "Volume mounting test completed!"