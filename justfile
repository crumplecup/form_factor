# Form Factor Development Justfile
#
# Common tasks for building, testing, and maintaining the Form Factor project.
# Run `just` or `just --list` to see all available commands.

# Load environment variables from .env file
set dotenv-load

# Default recipe to display help
default:
    @just --list

# Development Setup
# ================

# Install all development dependencies (Rust, cargo tools)
setup:
    @echo "📦 Installing development dependencies..."
    @just install-rust
    @just install-cargo-tools
    @echo "✅ Setup complete!"

# Install or update Rust toolchain
install-rust:
    @echo "🦀 Installing/updating Rust toolchain..."
    rustup update stable
    rustup default stable
    rustup component add clippy rustfmt

# Install required cargo plugins
install-cargo-tools:
    @echo "🔧 Installing cargo tools..."
    cargo install cargo-audit || true
    cargo install cargo-watch || true
    cargo install cargo-hack || true
    cargo install cargo-dist || true
    cargo install omnibor-cli || true
    cargo install cargo-nextest || true
    @echo "✅ Cargo tools installed"

# Update just itself
update-just:
    @echo "⚡ Updating just..."
    cargo install just || true

# Update all dependencies (Rust, cargo tools, just)
update-all: install-rust install-cargo-tools update-just
    @echo "✅ All tools updated!"

# Building and Checking
# ======================

# Build specific package or all workspace (default features)
build PACKAGE="":
    #!/usr/bin/env bash
    if [ -z "{{PACKAGE}}" ]; then
        cargo build --release
    else
        cargo build --release --package {{PACKAGE}}
    fi

# Build with dev features (all optional features except API-specific)
build-dev:
    cargo build --features dev

# Build with all features enabled
build-all:
    cargo build --all-features

# Build release with default features
build-release:
    cargo build --release

# Build release with dev features
build-release-dev:
    cargo build --release --features dev

# Build release with all features
build-release-all:
    cargo build --release --all-features

# Clean build artifacts
clean:
    cargo clean

# Clean and rebuild
rebuild: clean build

# Testing
# =======

# Run tests: just test [package] [test_name]
# Examples:
#   just test                     # Run all tests with default features
#   just test form_factor_core    # Run all tests for specific package
#   just test form_factor test1   # Run specific test
test PACKAGE="" TEST="":
    #!/usr/bin/env bash
    if [ -z "{{PACKAGE}}" ]; then
        # No package specified - run all tests with default features
        cargo test --workspace --lib --tests
    elif [ -z "{{TEST}}" ]; then
        # Package specified, no test - run all tests for package
        cargo test --package {{PACKAGE}} --lib --tests
    else
        # Package and test specified - run specific test
        cargo test --package {{PACKAGE}} --lib --tests {{TEST}} -- --nocapture
    fi

# Run tests with verbose output
test-verbose:
    cargo test --workspace --lib --tests -- --nocapture

# Run doctests (usually fast)
test-doc:
    cargo test --workspace --doc

# Run tests for a specific package, optionally filtering by test name
test-package package test_name="":
    #!/usr/bin/env bash
    echo "📦 Testing {{package}}"
    if [ -n "{{test_name}}" ]; then
        cargo test -p {{package}} --lib --tests {{test_name}} -- --nocapture
    else
        cargo test -p {{package}} --lib --tests
    fi

# Run the full test suite: tests + doc tests (legacy alias, use test-full)
test-full: test test-doc

# Run tests and show coverage (requires cargo-tarpaulin)
test-coverage:
    @command -v cargo-tarpaulin >/dev/null 2>&1 || (echo "Installing cargo-tarpaulin..." && cargo install cargo-tarpaulin)
    cargo tarpaulin --workspace --lib --tests --out Html --output-dir coverage

# Code Quality
# ============

# Check compilation (all features by default, or specific package)
check package="":
    #!/usr/bin/env bash
    if [ -z "{{package}}" ]; then
        echo "🔍 Checking all packages with all features..."
        cargo check --all-features
    else
        echo "🔍 Checking package: {{package}}"
        cargo check -p "{{package}}"
    fi

# Run clippy linter (no warnings allowed)
lint package='':
    #!/usr/bin/env bash
    if [ -z "{{package}}" ]; then
        echo "🔍 Linting entire workspace"
        cargo clippy --workspace --all-targets
    else
        echo "🔍 Linting {{package}}"
        cargo clippy -p {{package}} --all-targets
    fi

# Run clippy and fix issues automatically
lint-fix:
    cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Format all code
fmt:
    cargo fmt --all

# Check markdown files for issues
lint-md:
    @command -v markdownlint-cli2 >/dev/null 2>&1 || (echo "❌ markdownlint-cli2 not installed. Install with: npm install -g markdownlint-cli2" && exit 1)
    markdownlint-cli2 "**/*.md" "#target" "#node_modules"

# Test various feature gate combinations (requires cargo-hack)
check-features:
    #!/usr/bin/env bash
    set -e
    command -v cargo-hack >/dev/null 2>&1 || (echo "❌ cargo-hack not installed. Run: cargo install cargo-hack" && exit 1)
    
    LOG_FILE="/tmp/form_factor-check-features.log"
    rm -f "$LOG_FILE"
    
    echo "🔍 Checking workspace libraries with no-default-features..."
    for crate in form_factor_core form_factor_drawing form_factor_cv form_factor_ocr form_factor_backends form_factor_plugins; do
        echo "  📦 Checking $crate --no-default-features..."
        if ! cargo check -p "$crate" --no-default-features 2>&1 | tee -a "$LOG_FILE"; then
            echo "❌ No-default-features check failed for $crate. See: $LOG_FILE"
            exit 1
        fi
    done
    
    echo "🔍 Checking all-features..."
    if ! cargo check --all-features 2>&1 | tee -a "$LOG_FILE"; then
        echo "❌ All-features check failed. See: $LOG_FILE"
        exit 1
    fi
    
    echo "🔍 Checking feature powerset (excluding form_factor binary)..."
    # Check workspace libraries with feature powerset
    # Exclude form_factor binary since it requires at least one backend
    if ! cargo hack check --feature-powerset --no-dev-deps --workspace --exclude form_factor 2>&1 | tee -a "$LOG_FILE"; then
        echo "❌ Feature powerset check failed. See: $LOG_FILE"
        exit 1
    fi
    
    echo "🔍 Checking form_factor with default features (includes backend)..."
    if ! cargo check -p form_factor 2>&1 | tee -a "$LOG_FILE"; then
        echo "❌ form_factor check failed. See: $LOG_FILE"
        exit 1
    fi
    
    # Check for any errors/warnings in the log
    if [ -s "$LOG_FILE" ] && grep -qE "^(warning:|error:|\s+\^|error\[)" "$LOG_FILE"; then
        echo "⚠️  Feature gate checks completed with warnings/errors. See: $LOG_FILE"
        exit 1
    else
        echo "✅ All feature gate checks passed!"
        rm -f "$LOG_FILE"
    fi

# Run all checks (lint, format check, tests)
test-all package='':
    #!/usr/bin/env bash
    set -uo pipefail  # Removed -e so we can capture exit codes
    LOG_FILE="/tmp/form_factor_check_all.log"
    rm -f "$LOG_FILE"
    EXIT_CODE=0

    if [ -z "{{package}}" ]; then
        echo "🔍 Running all checks on entire workspace..."

        # Run fmt (errors only)
        cargo fmt --all

        # Run lint (show output and log warnings/errors)
        echo "🔍 Linting entire workspace"
        if ! cargo clippy --workspace --all-targets 2>&1 | tee -a "$LOG_FILE"; then
            EXIT_CODE=1
        fi

        # Run tests (show output and log failures)
        if ! cargo test --workspace --features dev --lib --tests 2>&1 | tee -a "$LOG_FILE"; then
            EXIT_CODE=1
        fi

        # Report results
        if [ $EXIT_CODE -ne 0 ]; then
            echo ""
            echo "⚠️  Checks completed with warnings/errors. Full log saved to: $LOG_FILE"
            exit 1
        else
            echo ""
            echo "✅ All checks passed!"
            rm -f "$LOG_FILE"
        fi
    else
        echo "🔍 Running all checks on {{package}}..."
        just fmt
        just lint "{{package}}"
        just test-package "{{package}}"
        # Run doc tests for the package
        cargo test -p "{{package}}" --doc
    fi
    echo "✅ All checks passed!"

# Fix all auto-fixable issues
fix-all: fmt lint-fix
    @echo "✅ Auto-fixes applied!"

# Security
# ========

# Check for security vulnerabilities in dependencies
audit:
    cargo audit

# Update dependencies and check for vulnerabilities
audit-fix:
    cargo update
    cargo audit

# Development
# ===========

# Watch for changes and run tests
watch:
    @command -v cargo-watch >/dev/null 2>&1 || (echo "Installing cargo-watch..." && cargo install cargo-watch)
    cargo watch -x 'test --workspace --lib --tests'

# Watch and run specific command on changes
watch-cmd cmd:
    @command -v cargo-watch >/dev/null 2>&1 || (echo "Installing cargo-watch..." && cargo install cargo-watch)
    cargo watch -x '{{cmd}}'

# Run the binary in development mode
run *args:
    cargo run -p form_factor -- {{args}}

# Run with dev features
run-dev *args:
    cargo run -p form_factor --features dev -- {{args}}

# Run with all features
run-all *args:
    cargo run -p form_factor --all-features -- {{args}}

# Full Workflow (CI/CD)
# ====================

# Run the complete CI pipeline locally
ci: fmt-check lint check-features test-all audit
    @echo "✅ CI pipeline completed successfully!"

# Prepare for commit (format, lint, tests, feature checks)
pre-commit: fix-all check-features test-all
    @echo "✅ Ready to commit!"

# Prepare for merge (all checks)
pre-merge: pre-commit
    @echo "✅ Ready to merge!"

# Prepare for release (all checks + release build)
pre-release: ci build-release
    @echo "✅ Ready for release!"

# Git helpers
# ===========

# Stage all changes and show status
stage:
    git add -A
    git status --short

# Quick commit with message
commit msg: pre-commit stage
    git commit -m "{{msg}}"

# Quick commit and push to current branch
push msg:
    @just commit "{{msg}}"
    git push origin $(git branch --show-current)

# Documentation
# =============

# Generate and open Rust documentation
docs:
    cargo doc --workspace --no-deps --open

# Check documentation for issues
docs-check:
    cargo doc --workspace --no-deps

# Build and view documentation for a specific crate
docs-crate crate:
    cargo doc --package {{crate}} --no-deps --open

# Information
# ===========

# Show project statistics
stats:
    @echo "📊 Project Statistics"
    @echo "===================="
    @echo ""
    @echo "Workspace crates:"
    @ls -1d crates/*/ | wc -l
    @echo ""
    @echo "Lines of Rust code (all crates):"
    @find crates -name '*.rs' -not -path '*/target/*' -exec wc -l {} + 2>/dev/null | tail -1 || echo "  0"
    @echo ""
    @echo "Lines of test code:"
    @find crates/*/tests tests -name '*.rs' 2>/dev/null -exec wc -l {} + 2>/dev/null | tail -1 || echo "  0"
    @echo ""
    @echo "Number of dependencies:"
    @grep -c "^name =" Cargo.lock 2>/dev/null || echo "  0"

# Show environment information
env:
    #!/usr/bin/env bash
    set +u
    echo "🔧 Environment Information"
    echo "========================="
    echo ""
    echo "Rust version:"
    rustc --version
    echo ""
    echo "Cargo version:"
    cargo --version
    echo ""
    echo "Just version:"
    just --version

# Show available features
features:
    @echo "🎛️  Available Features"
    @echo "===================="
    @echo ""
    @echo "Main crate features:"
    @grep '^\[features\]' -A 20 crates/form_factor/Cargo.toml | grep -v '^\[' | grep '='

# Utility
# =======

# Remove generated files and caches
clean-all: clean
    @echo "🧹 Deep cleaning..."
    rm -rf target/
    rm -rf coverage/
    rm -f Cargo.lock
    @echo "✅ All build artifacts removed"

# Check for outdated dependencies
outdated:
    @command -v cargo-outdated >/dev/null 2>&1 || (echo "Installing cargo-outdated..." && cargo install cargo-outdated)
    cargo outdated

# Update dependencies to latest compatible versions
update-deps:
    cargo update
    @echo "✅ Dependencies updated. Run 'just test' to verify."

# Generate OmniBOR artifact tree for supply chain transparency
omnibor:
    @command -v omnibor >/dev/null 2>&1 || (echo "Installing omnibor-cli..." && cargo install omnibor-cli)
    omnibor --help > /dev/null && echo "✅ OmniBOR installed" || echo "❌ OmniBOR not found - install with: cargo install omnibor"

# Run all security checks
security: audit omnibor
    @echo "✅ Security checks completed!"

# Release Management
# ==================

# Build distribution artifacts for current platform
dist-build:
    dist build

# Build and check distribution artifacts (doesn't upload)
dist-check:
    dist build --check

# Generate release configuration
dist-init:
    dist init

# Plan a release (preview changes)
dist-plan:
    dist plan

# Generate CI workflow files
dist-generate:
    dist generate

# Benchmarking
# ============

# Run benchmarks (requires bench tests)
bench:
    cargo bench

# Aliases for common tasks
# ========================

alias b := build
alias t := test
alias l := lint
alias f := fmt
alias c := test-all
alias r := run
alias d := docs

# Run tests with timing information using nextest
test-timings:
    cargo nextest run --workspace

# Install nextest if not present
install-nextest:
    cargo install cargo-nextest --locked
