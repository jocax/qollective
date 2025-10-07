#!/usr/bin/env bash
#
# Regenerate Rust types from JSON Schema using Qollective Generator
#
# Usage: ./regenerate-types.sh
#
# This script automatically generates Rust types from the JSON Schema definition
# using the Qollective code generator. Changes to the schema will be reflected
# in the generated Rust code.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔄 Regenerating Rust types from JSON Schema..."
echo "📂 Schema: schemas/taletrail-content-generator.json"
echo "📦 Output: shared-types/src/generated/"
echo ""

# Check if schema exists
if [ ! -f "schemas/taletrail-content-generator.json" ]; then
    echo "❌ Error: Schema file not found: schemas/taletrail-content-generator.json"
    exit 1
fi

# Step 1: Validate JSON syntax
echo "1️⃣  Validating JSON schema syntax..."
if ! python3 -m json.tool schemas/taletrail-content-generator.json > /dev/null; then
    echo "❌ Error: Invalid JSON in schema file"
    exit 1
fi
echo "   ✅ Schema is valid JSON"
echo ""

# Step 2: Validate with Qollective generator
echo "2️⃣  Validating schema with Qollective generator..."
cd ../../generator
if ! cargo run --quiet -- validate ../examples/qollective_taletrail_content_generator/schemas/taletrail-content-generator.json --lint; then
    echo "❌ Error: Schema validation failed"
    exit 1
fi
echo ""

# Step 3: Generate Rust types
echo "3️⃣  Generating Rust types with Qollective generator..."
echo "   Using: RustCodeGenerator (handles \$defs properly)"

# Backup existing generated types
if [ -d "../examples/qollective_taletrail_content_generator/shared-types/src/generated" ]; then
    echo "   📦 Backing up existing types to generated.backup/"
    rm -rf ../examples/qollective_taletrail_content_generator/shared-types/src/generated.backup
    cp -r ../examples/qollective_taletrail_content_generator/shared-types/src/generated \
          ../examples/qollective_taletrail_content_generator/shared-types/src/generated.backup
fi

# Generate new types
cargo run --quiet -- generate \
    ../examples/qollective_taletrail_content_generator/schemas/taletrail-content-generator.json \
    --output ../examples/qollective_taletrail_content_generator/shared-types/src/generated \
    --language rust \
    --format single-file \
    --force

if [ $? -ne 0 ]; then
    echo "❌ Error: Code generation failed"
    exit 1
fi

echo "   ✅ Rust types generated successfully"
echo ""

# Step 4: Format generated code
echo "4️⃣  Formatting generated code..."
cd ../examples/qollective_taletrail_content_generator
rustfmt shared-types/src/generated/*.rs 2>/dev/null || true
echo "   ✅ Code formatted"
echo ""

# Step 5: Verify types compile
echo "5️⃣  Verifying generated types compile..."
if ! cargo build -p shared-types 2>&1 | grep -q "Finished"; then
    echo "❌ Error: Generated types do not compile"
    echo ""
    echo "Compilation errors:"
    cargo build -p shared-types
    exit 1
fi
echo "   ✅ Generated types compile successfully"
echo ""

# Step 6: Show statistics
echo "📊 Generation Statistics:"
GENERATED_FILE="shared-types/src/generated/taletrail_content_generator.rs"
if [ -f "$GENERATED_FILE" ]; then
    LINES=$(wc -l < "$GENERATED_FILE")
    TYPES=$(grep -c "^pub struct\|^pub enum" "$GENERATED_FILE")
    echo "   Lines of code: $LINES"
    echo "   Types generated: $TYPES"
fi
echo ""

echo "✅ Type regeneration complete!"
echo ""
echo "📋 Summary:"
echo "   - Schema: schemas/taletrail-content-generator.json"
echo "   - Generated: shared-types/src/generated/taletrail_content_generator.rs"
echo "   - Status: ✅ Validates and compiles"
echo ""
echo "🔧 Next steps:"
echo "   1. Review changes: git diff shared-types/src/generated/"
echo "   2. Run tests: cargo test -p shared-types"
echo "   3. Update lib.rs if module structure changed"
echo "   4. Commit changes: git add schemas/ shared-types/src/generated/"
