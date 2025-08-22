#!/bin/bash

# Multi-architecture Docker build script for tlq
# Builds for both AMD64 (x86_64) and ARM64 architectures
# Usage: ./build-multiarch.sh [version] [--local]

set -e

# Help function
show_help() {
    cat << EOF
Multi-architecture Docker build script for tlq

USAGE:
    ./build-multiarch.sh [version] [options]

OPTIONS:
    -h, --help      Show this help message and exit
    --local         Build for local use only (doesn't push to Docker Hub)

ARGUMENTS:
    version         Docker image version tag (optional)
                    If not provided, auto-detects from Cargo.toml or uses 'latest'

DESCRIPTION:
    This script builds Docker images for tlq with support for multiple architectures.
    
    By default, it:
    - Builds for both linux/amd64 and linux/arm64
    - Pushes to Docker Hub (nebojsa/tlq)
    - Tags the image with the specified version and 'latest'
    
    With --local flag:
    - Builds only for the current platform architecture
    - Loads the image locally (doesn't push to Docker Hub)
    - Uses local image name without Docker Hub prefix

EXAMPLES:
    # Build and push version 1.0.0 to Docker Hub
    ./build-multiarch.sh 1.0.0
    
    # Build version 2.0.0 locally for current platform only
    ./build-multiarch.sh 2.0.0 --local
    
    # Auto-detect version from Cargo.toml and push to Docker Hub
    ./build-multiarch.sh
    
    # Auto-detect version and build locally
    ./build-multiarch.sh --local

EOF
    exit 0
}

# Parse arguments
LOCAL_MODE=false
VERSION=""

for arg in "$@"; do
    case $arg in
        -h|--help)
            show_help
            ;;
        --local)
            LOCAL_MODE=true
            shift
            ;;
        *)
            if [ -z "$VERSION" ]; then
                VERSION="$arg"
            fi
            shift
            ;;
    esac
done

# Configuration
IMAGE_NAME="tlq"
DOCKER_HUB_USER="nebojsa"

# Auto-detect version from Cargo.toml if not provided
if [ -z "$VERSION" ]; then
    if [ -f "Cargo.toml" ]; then
        VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/' | head -1)
        if [ -n "$VERSION" ]; then
            echo "📌 Auto-detected version from Cargo.toml: $VERSION"
        else
            VERSION="latest"
            echo "⚠️  Could not parse version from Cargo.toml. Using: $VERSION"
        fi
    else
        VERSION="latest"
        echo "⚠️  Cargo.toml not found. Using: $VERSION"
    fi
else
    echo "📌 Using specified version: $VERSION"
fi

# Set image name and platform based on mode
if [ "$LOCAL_MODE" = true ]; then
    FULL_IMAGE_NAME="$IMAGE_NAME"
    MODE_TEXT="locally"
    BUILD_ARGS="--load"
    # For local builds, use current platform only (Docker load doesn't support multi-arch)
    PLATFORM="linux/$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/')"
    echo "🏗️  Local mode: building for current platform only ($PLATFORM)"
else
    FULL_IMAGE_NAME="$DOCKER_HUB_USER/$IMAGE_NAME"
    MODE_TEXT="and pushing to Docker Hub"
    BUILD_ARGS="--push"
    PLATFORM="linux/amd64,linux/arm64"
fi

echo "Building image $MODE_TEXT: $FULL_IMAGE_NAME:$VERSION"
echo "Platforms: $PLATFORM"

# Create and use a new builder instance for multi-arch
BUILDER_NAME="tlq-builder"
if [ "$LOCAL_MODE" = true ]; then
    BUILDER_NAME="tlq-local-builder"
fi

echo "Setting up buildx builder..."
docker buildx create --name "$BUILDER_NAME" --use --bootstrap 2>/dev/null || docker buildx use "$BUILDER_NAME"

# Build image
echo "Building image $MODE_TEXT..."
docker buildx build \
    --platform "$PLATFORM" \
    --tag "$FULL_IMAGE_NAME:$VERSION" \
    --tag "$FULL_IMAGE_NAME:latest" \
    $BUILD_ARGS \
    .

echo "✅ Build complete!"
echo "Image: $FULL_IMAGE_NAME:$VERSION"
echo "Platforms: $PLATFORM"

if [ "$LOCAL_MODE" = true ]; then
    echo ""
    echo "To run: docker run -p 1337:1337 $FULL_IMAGE_NAME:$VERSION"
    echo "To test: docker run --rm $FULL_IMAGE_NAME:$VERSION --help"
else
    echo ""
    echo "To run: docker run -p 1337:1337 $FULL_IMAGE_NAME:$VERSION"
    echo "To pull: docker pull $FULL_IMAGE_NAME:$VERSION"
fi

# Clean up builder
docker buildx rm "$BUILDER_NAME"
