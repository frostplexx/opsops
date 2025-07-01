# Show available recipes
default:
    @just --list

# Build the project using Nix if available, otherwise use Cargo
build:
    #!/usr/bin/env sh
    if command -v nix >/dev/null 2>&1; then
        echo "Building with Nix..."
        nix build
    else
        echo "Building with Cargo..."
        cargo build
    fi

# Build with release optimizations
release:
    #!/usr/bin/env sh
    if command -v nix >/dev/null 2>&1; then
        echo "Building release with Nix..."
        nix build --release
    else
        echo "Building release with Cargo..."
        cargo build --release
    fi

# Run the application
run *ARGS:
    @cargo run -- {{ARGS}}

# Run tests
test:
    @cargo test

# Run tests with output
test-verbose:
    @cargo test -- --nocapture

# Run specific test
test-one TEST:
    @cargo test {{TEST}}

# Run clippy lints
lint:
    @cargo clippy

# Format code
fmt:
    @cargo fmt

# Check formatting without modifying files
fmt-check:
    @cargo fmt -- --check

# Run clippy and formatting checks
check: fmt-check lint
    @echo "All checks passed!"

# Update dependencies
update:
    @cargo update

# Show dependency tree
deps:
    @cargo tree

# Clean build artifacts
clean:
    @cargo clean

# Generate documentation
docs:
    @cargo doc --no-deps --open

# Watch for changes and run tests
watch-test:
    @cargo watch -x test

# Watch for changes and run the app
watch-run:
    @cargo watch -x run

# Install development dependencies
setup-dev:
    #!/usr/bin/env sh
    echo "Installing development dependencies..."
    cargo install cargo-watch cargo-audit cargo-outdated cargo-edit
    if ! command -v just >/dev/null 2>&1; then
        cargo install just
    fi

# Audit for vulnerabilities
audit:
    @cargo audit

# Check for outdated dependencies
outdated:
    @cargo outdated

# Create a new release with bumped version
@release-version TYPE="patch":
    #!/usr/bin/env sh
    if ! command -v cargo-edit >/dev/null 2>&1; then
        echo "cargo-edit not found. Install it with: cargo install cargo-edit"
        exit 1
    fi
    cargo bump {{TYPE}}
    VERSION=$(grep -m 1 version Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
    git add Cargo.toml Cargo.lock
    git commit -m "Bump version to ${VERSION}"
    git tag -a "v${VERSION}" -m "Version ${VERSION}"
    echo "Created release v${VERSION}"
