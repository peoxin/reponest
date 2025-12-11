# Development Workflow Guide

This guide explains how to use the various development tools configured in this project and how they work together to ensure code quality.

## Quick Reference

| Tool | Purpose | When to Use | Command |
|------|---------|-------------|---------|
| `cargo test` | Run tests | During development | `cargo test` |
| `cargo clippy` | Lint code | During development | `cargo clippy --all-features --workspace` |
| `cargo fmt` | Format code | Before committing | `cargo fmt` |
| `typos` | Check spelling | Before committing | `typos` |
| `cargo deny` | Check dependencies | Before pushing/releasing | `cargo deny check` |
| `git-cliff` | Generate changelog | Before releasing | `git cliff --unreleased` |
| `pre-commit` | Run checks automatically | Setup once | `pre-commit install` |

## Setup Steps

### Using Nix (recommended)

```bash
nix develop
```

Nix flake prepares all the development dependencies automatically.

### Install Pre-commit Hooks

```bash
# Install pre-commit (if not using Nix)
pip install pre-commit

# Install git hooks
pre-commit install
pre-commit install --hook-type pre-push
```

Now pre-commit hooks will run automatically on `git commit` and `git push`.

## Development Workflow

### Starting a New Feature

```bash
# 1. Create a feature branch
git checkout -b feat/your-feature-name

# 2. Make your changes
# Edit files...

# 3. Run tests frequently
cargo test

# 4. Check your code as you go
cargo clippy --all-features --workspace
```

### Before Committing

The pre-commit hooks will automatically run these checks, but you can run them manually:

```bash
# Format code
cargo fmt

# Check for linting issues
cargo clippy --all-features --workspace -- -D warnings

# Check spelling
typos

# Run tests
cargo test

# Or run all pre-commit checks
pre-commit run --all-files
```

### Making a Commit

```bash
# Stage your changes
git add .

# Commit (pre-commit hooks run automatically)
git commit -m "feat: add new feature"

# If hooks fail, fix issues and commit again
# Hooks will show what needs to be fixed
```

**Commit Message Format** (Conventional Commits):
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`

### Before Pushing

```bash
# Pre-push hooks will run automatically, but you can run manually:

# Check dependencies for security issues
cargo deny check

# Ensure all tests pass
cargo test

# Push (pre-push hooks run automatically)
git push origin feat/your-feature-name
```

## Release Workflow

When preparing a release:

```bash
# 1. Ensure you're on main with latest changes
git checkout main
git pull

# 2. Run all checks
cargo test --all-features
cargo clippy --all-features --workspace -- -D warnings
cargo deny check
typos

# 3. Update version in Cargo.toml
# Edit Cargo.toml: version = "0.2.0"

# 4. Generate changelog
git cliff --unreleased --tag v0.2.0 --prepend CHANGELOG.md

# 5. Review and edit CHANGELOG.md if needed

# 6. Commit version bump
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"

# 7. Create and push tag
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin main --tags

# 8. GitHub Actions automatically:
#    - Builds release binaries
#    - Creates GitHub release
#    - Publishes to crates.io
```

## Troubleshooting

### Pre-commit Hook Failures

**Problem**: Commit blocked by formatting check
```bash
# Solution: Format and try again
cargo fmt
git add .
git commit
```

**Problem**: Clippy warnings
```bash
# Solution: Fix warnings or suppress if needed
cargo clippy --fix
# Review changes
git add .
git commit
```

**Problem**: Typos detected
```bash
# Solution: Fix typos or add to whitelist
typos --write-changes  # Auto-fix
# Or add to typos.toml if false positive
```

### Skipping Hooks (Emergency Only)

```bash
# Skip pre-commit hooks (NOT recommended)
git commit --no-verify

# Skip pre-push hooks (NOT recommended)
git push --no-verify
```

**Warning**: Skipping hooks means CI might fail. Only use in emergencies.

### Dependency Issues

**Problem**: Cargo deny reports vulnerability
```bash
# Solution 1: Update dependencies
cargo update

# Solution 2: If no fix available, assess risk and ignore
# Add to deny.toml:
# ignore = [
#     { id = "RUSTSEC-XXXX-XXXX", reason = "explanation" }
# ]
```

**Problem**: License issue
```bash
# Solution: Add license to allow list in deny.toml
# [licenses]
# allow = [
#     "MIT",
#     "Apache-2.0",
#     "NewLicense",  # Add here
# ]
```
