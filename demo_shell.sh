#!/bin/bash

echo "🎬 Interactive Shell Demo"
echo "========================="

echo ""
echo "1️⃣ Basic Alpine shell:"
echo "   $ ./ephemeral-vm shell alpine"
echo ""

echo "2️⃣ Ubuntu with bash:"
echo "   $ ./ephemeral-vm shell ubuntu --shell bash"
echo ""  

echo "3️⃣ Node.js development environment:"
echo "   $ ./ephemeral-vm shell node --memory 1024 --shell bash"
echo ""

echo "4️⃣ Python with volume mount:"
echo "   $ ./ephemeral-vm shell python -v \"\$(pwd):/workspace\" --shell bash"
echo ""

echo "5️⃣ Multi-port web development:"
echo "   $ ./ephemeral-vm shell node -p 3000:3000 -p 8080:8080 --memory 2048"
echo ""

echo "6️⃣ Quick demo (auto-exit):"
echo ""

echo "Running: echo 'uname -a; free -h; node --version; exit' | ./ephemeral-vm shell node --quiet"
echo ""

echo 'uname -a; free -h; node --version; exit' | ./ephemeral-vm.sh shell node --quiet

echo ""
echo "✨ Interactive shell mode is ready for development!"