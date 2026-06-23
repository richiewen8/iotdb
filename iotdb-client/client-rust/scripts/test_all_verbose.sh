#!/bin/bash
# test_all_verbose.sh - 带详细输出的批量测试

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║      IoTDB Rust Client - Verbose Batch Test Suite          ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 检查 IoTDB 是否运行
echo -n "🔍 Checking IoTDB connection... "
if nc -z localhost 6667 2>/dev/null; then
    echo -e "${GREEN}✓ IoTDB is running${NC}"
else
    echo -e "${YELLOW}⚠ IoTDB not running (some tests may fail)${NC}"
fi
echo ""

# 构建
echo "📦 Building project..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error|warning)" | tail -10
echo ""

# 运行测试函数
run_example() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}▶ Running: $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    if cargo run --release --example $1; then
        echo -e "${GREEN}✓ $1 completed successfully${NC}"
        return 0
    else
        echo -e "${RED}✗ $1 failed${NC}"
        return 1
    fi
}

# 运行所有示例
run_example "config"
run_example "connect"
run_example "insert"
run_example "insert_table_row"
run_example "query"
run_example "batch_insert"
run_example "metadata"
run_example "error_handling"
run_example "timeout"

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All examples completed${NC}"