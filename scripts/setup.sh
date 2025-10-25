#!/bin/bash

# ChromaDB with Rust - Setup Script
# This script sets up the complete development environment

set -e  # Exit on any error

echo "🚀 Setting up ChromaDB with Rust development environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check if Docker is installed
print_step "Checking Docker installation..."
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker and try again."
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    print_error "Docker Compose is not installed. Please install Docker Compose and try again."
    exit 1
fi

print_status "Docker and Docker Compose are installed"

# Check if Rust is installed
print_step "Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed. Please install Rust from https://rustup.rs/ and try again."
    exit 1
fi

print_status "Rust is installed"

# Check if .env file exists, if not create from example
print_step "Setting up environment configuration..."
if [ ! -f .env ]; then
    if [ -f .env.example ]; then
        cp .env.example .env
        print_status "Created .env file from .env.example"
        print_warning "Please edit .env file and add your Google API key"
    else
        print_error ".env.example file not found"
        exit 1
    fi
else
    print_status ".env file already exists"
fi

# Start ChromaDB
print_step "Starting ChromaDB..."
docker-compose up -d

# Wait for ChromaDB to be ready
print_step "Waiting for ChromaDB to be ready..."
max_attempts=30
attempt=1

while [ $attempt -le $max_attempts ]; do
    if curl -f http://localhost:8000/api/v2/heartbeat &> /dev/null; then
        print_status "ChromaDB is ready!"
        break
    fi
    
    if [ $attempt -eq $max_attempts ]; then
        print_error "ChromaDB failed to start after $max_attempts attempts"
        print_error "Check logs with: docker-compose logs chromadb"
        exit 1
    fi
    
    print_warning "Attempt $attempt/$max_attempts - ChromaDB not ready yet, waiting..."
    sleep 2
    ((attempt++))
done

# Build the Rust project
print_step "Building Rust project..."
cargo build

# Run tests
print_step "Running tests..."
cargo test

print_status "Setup completed successfully! 🎉"
echo
echo "Next steps:"
echo "1. Edit .env file and add your Google API key"
echo "2. Run the application: cargo run --bin chroma_client"
echo "3. Check ChromaDB logs: docker-compose logs -f chromadb"
echo "4. Stop ChromaDB: docker-compose down"
echo
echo "ChromaDB is accessible at: http://localhost:8000"
echo "ChromaDB API documentation: http://localhost:8000/docs"