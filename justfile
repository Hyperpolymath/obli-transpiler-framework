# SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
# obli-transpiler-framework - Development Tasks
set shell := ["bash", "-uc"]
set dotenv-load := true

project := "obli-transpiler-framework"

# Show all recipes
default:
    @just --list --unsorted

# Build the transpiler
build:
    cargo build --release

# Run all tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Run all checks (fmt + lint + test)
check: fmt-check lint test

# Transpile a .mobli file to Rust
transpile file:
    cargo run --release -- transpile -i {{file}}

# Run an expression directly
run expr:
    cargo run --release -- run -e "{{expr}}"

# Check a .mobli file for errors
check-file file:
    cargo run --release -- check -i {{file}}

# Transpile all examples
examples:
    @for f in examples/*.mobli; do \
        echo "=== $f ==="; \
        cargo run --quiet --release -- transpile -i "$f"; \
        echo ""; \
    done

# Demo: show the key obliviousness transformation
demo:
    @echo "=== Public conditional (stays as if) ==="
    @cargo run --quiet --release -- run -e "let x = 1 if x > 0 then 1 else 0"
    @echo ""
    @echo "=== Secret conditional (becomes ct_select) ==="
    @cargo run --quiet --release -- run -e "let x = secret(1) if x > 0 then secret(1) else secret(0)"

# Install the CLI locally
install:
    cargo install --path .

# Generate documentation
docs:
    cargo doc --no-deps --open
