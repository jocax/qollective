#!/usr/bin/env bash
#
# Regenerate Rust types from JSON Schema using Qollective Generator
#
# Usage: ./regenerate-types.sh
#
# This script automatically generates Rust types from the JSON Schema definition
# using the Qollective code generator with split-by-type format.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔄 Regenerating Rust types from JSON Schema..."
echo "📂 Schema: ../schemas/taletrail-content-generator.json"
echo "📦 Output: ./src/generated.rs (single file)"
echo ""

# Check if schema exists
if [ ! -f "../schemas/taletrail-content-generator.json" ]; then
    echo "❌ Error: Schema file not found: ../schemas/taletrail-content-generator.json"
    exit 1
fi

# Step 1: Validate JSON syntax
echo "1️⃣  Validating JSON schema syntax..."
if ! python3 -m json.tool ../schemas/taletrail-content-generator.json > /dev/null; then
    echo "❌ Error: Invalid JSON in schema file"
    exit 1
fi
echo "   ✅ Schema is valid JSON"
echo ""

# Step 2: Validate with Qollective generator
echo "2️⃣  Validating schema with Qollective generator..."
cd ../../../generator
if ! cargo run --quiet -- validate ../examples/qollective_taletrail_content_generator/schemas/taletrail-content-generator.json --lint; then
    echo "❌ Error: Schema validation failed"
    exit 1
fi
echo ""

# Step 3: Generate Rust types as single file
echo "3️⃣  Generating Rust types with Qollective generator..."
echo "   Using: --format crate (outputs to taletrail_content_generator/src/lib.rs)"

# Generate to temporary location
TEMP_OUT="../examples/qollective_taletrail_content_generator/shared-types-generated/temp_gen"
cargo run --quiet -- generate \
    ../examples/qollective_taletrail_content_generator/schemas/taletrail-content-generator.json \
    --output "$TEMP_OUT" \
    --language rust \
    --format crate \
    --force

# Move the generated lib.rs to our src/generated.rs
if [ -f "$TEMP_OUT/taletrail_content_generator/src/lib.rs" ]; then
    mv "$TEMP_OUT/taletrail_content_generator/src/lib.rs" ../examples/qollective_taletrail_content_generator/shared-types-generated/src/generated.rs
    rm -rf "$TEMP_OUT"
else
    echo "❌ Error: Generated file not found"
    exit 1
fi

if [ $? -ne 0 ]; then
    echo "❌ Error: Code generation failed"
    exit 1
fi

echo "   ✅ Rust types generated successfully"
echo ""

# Step 4: Format generated code
echo "4️⃣  Formatting generated code..."
cd ../examples/qollective_taletrail_content_generator/shared-types-generated
rustfmt src/*.rs 2>/dev/null || true
echo "   ✅ Code formatted"
echo ""

# Step 5: Verify types compile
echo "5️⃣  Verifying generated types compile..."
if ! cargo build 2>&1 | grep -q "Finished"; then
    echo "❌ Error: Generated types do not compile"
    echo ""
    echo "Compilation errors:"
    cargo build
    exit 1
fi
echo "   ✅ Generated types compile successfully"
echo ""

# Step 6: Show statistics
echo "📊 Generation Statistics:"
TOTAL_LINES=$(wc -l < "src/generated.rs" 2>/dev/null || echo "0")
ENUM_COUNT=$(grep -c "^pub enum" src/generated.rs 2>/dev/null || echo "0")
STRUCT_COUNT=$(grep -c "^pub struct" src/generated.rs 2>/dev/null || echo "0")

echo "   Lines of code: $TOTAL_LINES"
echo "   - Enums: $ENUM_COUNT"
echo "   - Structs: $STRUCT_COUNT"
echo ""

echo "✅ Type regeneration complete!"
echo ""
echo "📋 Summary:"
echo "   - Schema: ../schemas/taletrail-content-generator.json"
echo "   - Generated: ./src/generated.rs"
echo "   - Status: ✅ Validates and compiles"
echo ""
echo "🔧 Next steps:"
echo "   1. Review changes: git diff src/"
echo "   2. Verify dependent crates compile: cargo build -p shared-types"
echo "   3. Run tests: cargo test"
