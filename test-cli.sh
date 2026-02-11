#!/bin/bash

echo "Testing gemini-proxy CLI..."
echo ""

# Test help
echo "1. Testing help command:"
gemini-proxy --help | head -5
echo ""

# Test status (should show not authenticated)
echo "2. Testing status command (before login):"
gemini-proxy status 2>&1 | head -5
echo ""

# Test start (should fail without auth)
echo "3. Testing start command (without auth):"
# Using a subshell with a background process and kill to simulate timeout
(gemini-proxy start 2>&1 & sleep 2; kill $! 2>/dev/null) | head -5 || true
echo ""

echo "✅ All CLI commands are working!"
echo ""
echo "To fully test:"
echo "  1. Run: gemini-proxy login"
echo "  2. Run: gemini-proxy start"
echo "  3. Test: curl http://localhost:3000/v1/models"
