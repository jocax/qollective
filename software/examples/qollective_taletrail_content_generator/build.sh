#!/usr/bin/env bash
set -e

echo "=== Building TaleTrail Content Generator Workspace ==="
echo ""

# Build order: shared-types first, then all services
CRATES=(
    "shared-types"
    "orchestrator"
    "story-generator"
    "quality-control"
    "constraint-enforcer"
    "gateway"
)

for crate in "${CRATES[@]}"; do
    echo "Building $crate..."
    cargo build -p "$crate" --release
    if [ $? -eq 0 ]; then
        echo "✅ $crate built successfully"
    else
        echo "❌ $crate build failed"
        exit 1
    fi
    echo ""
done

echo "=== Build Complete ==="
echo ""
echo "All workspace members built successfully!"
