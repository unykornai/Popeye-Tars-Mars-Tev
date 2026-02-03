# Contributing to Unykorn L1

Thank you for your interest in contributing to Unykorn L1! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Process](#development-process)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Architecture Rules](#architecture-rules)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- Cargo

### Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/Popeye-Tars-Mars-Tev.git
   cd Popeye-Tars-Mars-Tev
   ```
3. Build the project:
   ```bash
   cargo build --workspace
   ```
4. Run tests:
   ```bash
   cargo test --workspace
   ```

## Development Process

### Branching Strategy

- `main` - Production-ready code
- `feature/*` - New features
- `fix/*` - Bug fixes
- `docs/*` - Documentation updates

### Creating a Feature Branch

```bash
git checkout -b feature/my-new-feature
```

## Pull Request Process

1. Ensure all tests pass: `cargo test --workspace`
2. Run clippy: `cargo clippy --workspace`
3. Format code: `cargo fmt --all`
4. Update documentation if needed
5. Create a pull request with a clear description

### PR Checklist

- [ ] Tests pass
- [ ] Clippy warnings addressed
- [ ] Code formatted
- [ ] Documentation updated
- [ ] Commit messages are clear

## Coding Standards

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `thiserror` for error types
- Document public APIs with doc comments
- Write unit tests for new functionality

### Documentation

- Use `///` for public item documentation
- Include examples in doc comments
- Keep comments up to date with code changes

### Error Handling

```rust
// Good - explicit error types
pub fn process_block(block: &Block) -> Result<(), RuntimeError> {
    // ...
}

// Bad - generic errors
pub fn process_block(block: &Block) -> Result<(), Box<dyn Error>> {
    // ...
}
```

## Architecture Rules

These rules **MUST** be followed for all contributions:

### 1. Crate Independence

Each crate must compile independently. No circular dependencies.

### 2. MARS Rules

- ❌ No networking code
- ❌ No disk IO
- ❌ No RPC handling
- ✅ Pure state transitions only

### 3. POPEYE Rules

- ❌ Never mutates blockchain state
- ❌ Never validates economics
- ✅ Message transport only

### 4. TEV Rules

- ❌ No state management
- ❌ No networking
- ❌ No persistence
- ✅ Stateless verification only

### 5. TAR Rules

- ❌ Never validates data
- ❌ Never executes logic
- ✅ Crash-safe persistence only

### Trust Boundaries

```
Network (untrusted) → TEV (verify) → MARS (validate) → TAR (persist)
```

Nothing crosses from POPEYE to MARS without passing TEV.

## Questions?

If you have questions, please open an issue or reach out to the maintainers.

Thank you for contributing! 🦄
