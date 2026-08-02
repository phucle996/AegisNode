#!/usr/bin/env bash
# Script tự động đóng gói binary sản xuất AegisNode Release

set -euo pipefail

echo "======================================================"
echo "🚀 Bắt đầu quá trình Đóng gói Production Release AegisNode"
echo "======================================================"

# 1. Chạy toàn bộ kiểm thử tự động workspace
echo "🧪 Running full workspace unit & integration test suite..."
cargo test --workspace

# 2. Biên dịch binary phiên bản Release tối ưu
echo "⚙️ Building optimized release binary..."
cargo build --release

echo "======================================================"
echo "✅ Đã đóng gói thành công Release binary:"
echo "   Binary: target/release/aegisnode"
echo "======================================================"
