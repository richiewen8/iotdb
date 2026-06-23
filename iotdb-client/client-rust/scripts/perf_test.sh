#!/bin/bash
# perf_test.sh - 性能测试

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║         IoTDB Rust Client - Performance Test Suite         ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "📊 Performance Tests:"
echo ""

# 测试不同批次大小
for size in 100 500 1000 2000; do
    echo -e "${YELLOW}Testing batch size: $size${NC}"
    echo "─────────────────────────────────────────────────────────────"

    # 修改 batch_insert.rs 中的 BATCH_SIZE 并运行
    sed -i "s/let batch_size = [0-9]*;/let batch_size = $size;/" examples/batch_insert.rs

    cargo run --release --example batch_insert 2>&1 | grep -E "(records in|Average|Batch [0-9]+/[0-9]+)"
    echo ""
done

echo -e "${GREEN}✅ Performance tests completed${NC}"