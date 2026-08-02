# AegisNode Task Runner (Justfile)
# Tiện ích tự động hóa các tác vụ build, test, format và clippy cho dev team

# Default task: kiểm tra toàn bộ project
default: check test

# Build debug toàn bộ workspace
build:
    cargo build --workspace

# Build release binary
build-release:
    cargo build --workspace --release

# Chạy unit & integration tests
test:
    cargo test --workspace

# Kiểm tra định dạng code
fmt-check:
    cargo fmt --all --check

# Tự động định dạng code
fmt:
    cargo fmt --all

# Kiểm tra linter khắt khe
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Kiểm tra tổng thể (format, clippy, check)
check: fmt-check clippy
    cargo check --workspace --all-targets

# Chạy thử binary aegisnode local
run-local:
    cargo run --bin aegisnode -- local
