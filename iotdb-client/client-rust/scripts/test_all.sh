#!/bin/bash
# test_all.sh - 批量测试脚本

cd ~/iotdb/iotdb-client/rust

echo "╔════════════════════════════════════════════╗"
echo "║   IoTDB Rust Client - Test Suite           ║"
echo "╚════════════════════════════════════════════╝"
echo ""

# 检查 IoTDB
echo "🔍 Checking IoTDB..."
if nc -z localhost 6667 2>/dev/null; then
    echo "✅ IoTDB is running"
else
    echo "❌ IoTDB is NOT running"
    echo "   Start it with: cd ~/iotdb && ./sbin/start-standalone.sh"
    exit 1
fi
echo ""

# 构建
echo "📦 Building..."
cargo build --release --quiet
echo ""

# 运行测试
PASSED=0
FAILED=0

echo "🚀 Running tests:"
echo "─────────────────────────────────────────────"

# 使用列表方式（兼容旧版本 bash）
for test in config connect insert query batch_insert  insert_table_row metadata error_handling timeout; do
    printf "  %-15s " "$test"
    if cargo run --release --example "$test" > /tmp/test_$test.log 2>&1; then
        echo "✅ PASS"
        PASSED=$((PASSED + 1))
    else
        echo "❌ FAIL"
        echo "     Last 3 lines:"
        tail -3 /tmp/test_$test.log 2>/dev/null | sed 's/^/     /'
        FAILED=$((FAILED + 1))
    fi
done

echo "─────────────────────────────────────────────"
echo ""
echo "📊 Results: $PASSED passed, $FAILED failed"

if [ $FAILED -eq 0 ]; then
    echo "🎉 All tests passed!"
else
    echo "⚠️  Some tests failed."
    echo ""
    echo "To see full output of failed tests:"
    for test in config connect insert query batch_insert insert_table_row metadata error_handling timeout; do
        if [ ! -f /tmp/test_$test.log ] || ! grep -q "Example completed successfully" /tmp/test_$test.log 2>/dev/null; then
            echo "  cat /tmp/test_$test.log"
        fi
    done
fi