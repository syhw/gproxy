#!/bin/bash

# gemini-proxy installation script (Rust version)

set -e

echo "╔═══════════════════════════════════════════════════════╗"
echo "║           Gemini Proxy Installation Script                      ║"
echo "╚═══════════════════════════════════════════════════════╝"
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ cargo is not installed. Please install Rust first."
    echo "   Visit: https://rustup.rs/"
    exit 1
fi

echo "✅ cargo found"
echo ""

# Install globally
echo "📦 Building and installing gemini-proxy..."
cargo install --path . --force
echo ""

echo "╔═══════════════════════════════════════════════════════╗"
echo "║  ✅ Installation Complete!                                  ║"
echo "╚═══════════════════════════════════════════════════════╝"
echo ""
echo "🚀 Next Steps:"
echo ""
echo "1. Authenticate:"
echo "   gemini-proxy login"
echo ""
echo "2. Start server:"
echo "   gemini-proxy start"
echo ""
echo "3. Use with OpenAI client:"
echo "   from openai import OpenAI"
echo "   client = OpenAI(base_url='http://localhost:3000/v1', api_key='any')"
echo ""
echo "📚 For more info, visit: https://github.com/your-repo/gemini-proxy"
echo ""
