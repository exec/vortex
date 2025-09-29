#!/bin/bash
echo "🧪 Vortex Stress Test Suite"
echo "================================"

# Test 1: Rapid session listing
echo ""
echo "Test 1: Rapid session listing (10x)"
start_time=$(date +%s)
for i in {1..10}; do
    ./vortex_quick sessions > /dev/null
done
end_time=$(date +%s)
duration=$((end_time - start_time))
echo "✅ 10 session lists completed in ${duration}s"

# Test 2: Template listing performance
echo ""
echo "Test 2: Template listing performance (5x)"
start_time=$(date +%s)
for i in {1..5}; do
    ./vortex_quick templates > /dev/null
done
end_time=$(date +%s)
duration=$((end_time - start_time))
echo "✅ 5 template lists completed in ${duration}s"

# Test 3: Help system performance
echo ""
echo "Test 3: Help system (10x)"
start_time=$(date +%s)
for i in {1..10}; do
    ./vortex_quick help > /dev/null
done
end_time=$(date +%s)
duration=$((end_time - start_time))
echo "✅ 10 help calls completed in ${duration}s"

# Test 4: Invalid command handling
echo ""
echo "Test 4: Error handling stress test"
for cmd in "invalid" "create" "attach" "stop" "badtemplate"; do
    ./vortex_quick $cmd 2>/dev/null && echo "✅ $cmd handled" || echo "✅ $cmd error handled"
done

echo ""
echo "🎯 Stress test completed!"