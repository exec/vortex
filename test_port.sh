#!/bin/bash

set -e

echo "Testing port forwarding..."

# Start a simple HTTP server in the background
./ephemeral-vm.sh run alpine \
  -p "8080:80" \
  -e "echo 'HTTP/1.1 200 OK\n\nHello from VM!' | nc -l -p 80" &

VM_PID=$!
sleep 2

# Test the port forwarding
if curl -s http://localhost:8080 | grep -q "Hello from VM"; then
    echo "Port forwarding test passed!"
else
    echo "Port forwarding test failed!"
fi

# Clean up
kill $VM_PID 2>/dev/null || true

echo "Port forwarding test completed!"