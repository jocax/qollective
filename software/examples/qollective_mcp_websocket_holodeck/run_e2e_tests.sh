#!/bin/bash

# ABOUTME: Test runner script for holodeck end-to-end integration tests
# ABOUTME: Ensures proper single-threaded execution and comprehensive validation of all 7 use cases

set -e

echo "🎭 HOLODECK END-TO-END INTEGRATION TEST RUNNER"
echo "=============================================="
echo "Testing all 7 holodeck use cases with real MCP servers"
echo ""

# Set environment for testing
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Ensure we're in the correct directory
cd "$(dirname "$0")"

echo "📋 Test Configuration:"
echo "  - Single-threaded execution (--test-threads=1)"
echo "  - Full logging enabled"
echo "  - Comprehensive validation of all use cases"
echo ""

echo "🚀 Starting holodeck end-to-end integration tests..."
echo ""

# Run the main end-to-end integration test
echo "🎯 Running complete user journey test (all 7 use cases)..."
cargo test --test holodeck_integration_e2e_test test_holodeck_complete_user_journey_all_7_use_cases -- --test-threads=1 --nocapture

echo ""
echo "⚡ Running performance benchmark tests..."
cargo test --test holodeck_integration_e2e_test test_holodeck_performance_benchmarks -- --test-threads=1 --nocapture

echo ""
echo "🔀 Running concurrent operations tests..."
cargo test --test holodeck_integration_e2e_test test_holodeck_concurrent_operations -- --test-threads=1 --nocapture

echo ""
echo "✅ ALL END-TO-END INTEGRATION TESTS COMPLETED!"
echo ""
echo "📊 Test Summary:"
echo "  ✅ USE CASE 1: App Start - System initialization and health validation"
echo "  ✅ USE CASE 2: Enter (Welcome Screen) - System status and user onboarding"
echo "  ✅ USE CASE 3: Prepare Story (Configuration) - Story generation and template creation"
echo "  ✅ USE CASE 4: Scene Definition - Validate story structure and scene connectivity"
echo "  ✅ USE CASE 5: User Plays Scenes - Interactive character and environment testing"
echo "  ✅ USE CASE 6: Story History - Session management and data persistence"
echo "  ✅ USE CASE 7: Live Information - Real-time system monitoring and performance tracking"
echo ""
echo "🚀 Holodeck system validated and ready for production demonstration!"
echo "   All critical MCP server integrations working correctly"
echo "   Performance SLAs met for story generation (<3s) and character interactions (<2s)"
echo "   Error handling and recovery mechanisms validated"
echo "   Real-time monitoring and health checks functional"