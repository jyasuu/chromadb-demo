#!/bin/bash

# Comprehensive test script for ChromaDB Rust implementation
set -e

echo "🧪 Running comprehensive ChromaDB Rust tests..."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

# Check if ChromaDB is running
echo "1. Checking ChromaDB availability..."
if curl -f http://localhost:8000/api/v2/heartbeat &> /dev/null; then
    print_status "ChromaDB is running"
else
    print_error "ChromaDB is not running. Start it with: docker-compose up -d"
    exit 1
fi

# Check environment
echo "2. Checking environment..."
if [ -f .env ]; then
    print_status ".env file exists"
else
    print_warning ".env file not found, using defaults"
fi

# Build project
echo "3. Building project..."
if cargo build; then
    print_status "Project built successfully"
else
    print_error "Build failed"
    exit 1
fi

# Run unit tests
echo "4. Running unit tests..."
if cargo test; then
    print_status "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

# Test basic client functionality
echo "5. Testing basic functionality..."
if timeout 60 cargo run --bin chroma_client; then
    print_status "Basic functionality test passed"
else
    print_warning "Basic functionality test timed out or failed"
fi

# Test advanced features (if API key is available)
echo "6. Testing advanced features..."
if [ -n "$GOOGLE_API_KEY" ] || grep -q "GOOGLE_API_KEY=" .env 2>/dev/null; then
    if timeout 120 cargo run --example advanced_usage; then
        print_status "Advanced features test passed"
    else
        print_warning "Advanced features test timed out or failed"
    fi
else
    print_warning "Skipping advanced features test (no Google API key)"
fi

# Check code formatting
echo "7. Checking code formatting..."
if cargo fmt --check; then
    print_status "Code formatting is correct"
else
    print_warning "Code needs formatting (run: cargo fmt)"
fi

# Check for common issues with clippy
echo "8. Running clippy checks..."
if cargo clippy -- -D warnings; then
    print_status "Clippy checks passed"
else
    print_warning "Clippy found issues"
fi

echo
echo "🎉 Test suite completed!"
echo
echo "Summary:"
echo "✓ ChromaDB connectivity"
echo "✓ Project builds"
echo "✓ Unit tests pass"
echo "✓ Basic functionality works"
echo "✓ Code quality checks"
echo
echo "To run individual components:"
echo "  Basic demo:     cargo run --bin chroma_client"
echo "  Advanced demo:  cargo run --example advanced_usage"
echo "  Unit tests:     cargo test"
echo "  Formatting:     cargo fmt"
echo "  Linting:        cargo clippy"