# Contributing to RepoNest

Thank you for your interest in contributing to RepoNest! This document provides guidelines for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)

## Code of Conduct

This project adheres to the Rust Code of Conduct. By participating, you are expected to uphold this code. Please be respectful and considerate in all interactions.

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue with:
- A clear, descriptive title
- Steps to reproduce the problem
- Expected behavior vs actual behavior
- Your environment (OS, Rust version, terminal emulator)
- Screenshots or terminal output if applicable

### Suggesting Features

Feature suggestions are welcome! Please create an issue describing:
- The problem you're trying to solve
- Your proposed solution
- Alternative solutions you've considered
- Any additional context or examples

### Contributing Code

1. **Fork the repository**
2. **Create a feature branch** from `main`
   ```bash
   git checkout -b feat/your-feature-name
   ```
3. **Make your changes** (see Development Workflow below)
4. **Commit your changes** with clear, descriptive messages
5. **Push to your fork**
6. **Submit a pull request**

## Development Setup

### Clone and Build

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/reponest.git
cd reponest

# Build the project
cargo build

# Run tests
cargo test

# Run in development mode
cargo run
```

### Install Development Tools

```bash
rustup component add rustfmt clippy
cargo install --locked cargo-deny typos-cli
```

### Setting Up Pre-commit Hooks

Pre-commit hooks automatically run code quality checks before each commit:

```bash
# Install pre-commit (included in Nix flake)
# Or install manually: pip install pre-commit

# Install the git hook scripts
pre-commit install

# Install pre-push hooks for heavier checks
pre-commit install --hook-type pre-push

# Run against all files manually
pre-commit run --all-files
```

The pre-commit hooks will automatically:
- Format code with `cargo fmt`
- Run linter with `cargo clippy`
- Check for typos with `typos`
- Check dependencies with `cargo deny` (on pre-push only)

## Development Workflow

### Project Structure

```
reponest/
├── src/
│   ├── main.rs          # Binary entry point
│   ├── lib.rs           # Library entry point
│   ├── cli/             # CLI argument parsing and execution
│   ├── config/          # Configuration management
│   ├── core/            # Core logic
│   └── tui/             # Terminal UI components
├── benches/             # Performance benchmarks
├── examples/            # Examples
├── scripts/             # Development and testing scripts
└── tests/               # Integration tests
```

### Making Changes

1. **Write tests first** (TDD approach recommended)
   ```bash
   # Add test to appropriate file
   # Run tests to verify they fail
   cargo test
   ```

2. **Implement your feature**
   - Follow the existing code structure
   - Add documentation comments for public APIs
   - Keep functions focused and modular

3. **Test your changes**
   ```bash
   # Run all tests
   cargo test
   
   # Run specific test
   cargo test test_name
   ```

4. **Run benchmarks** (if performance-related)
   ```bash
   cargo bench
   ```

5. **Test manually with TUI**
   ```bash
   # Create test environment
   ./scripts/setup-test-repos.sh
   
   # Run reponest
   cargo run -- test-repos
   
   # Clean up
   ./scripts/cleanup-test-repos.sh
   ```

## Code Style

### Formatting

```bash
# Format your code before committing
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

### Linting

```bash
# Run clippy to catch common mistakes
cargo clippy

# Fix clippy suggestions automatically (when possible)
cargo clippy --fix
```

### Spell Checking

```bash
# Check for typos
typos

# Fix typos automatically
typos --write-changes

# Check specific files
typos src/main.rs
```

### Dependency Checks

```bash
# Check dependencies for security vulnerabilities and license issues
cargo deny check

# Check only advisories
cargo deny check advisories

# Check only licenses
cargo deny check licenses

# Check for banned or duplicate dependencies
cargo deny check bans
```

### Code Guidelines

- **Follow Rust conventions**: Use `snake_case` for functions/variables, `PascalCase` for types
- **Document public APIs**: All public functions, structs, and modules should have doc comments
- **Handle errors properly**: Use `Result` and `anyhow` for error handling
- **Avoid unwrap**: Use `?` operator or proper error handling
- **Use meaningful names**: Variables and functions should be self-documenting
- **Add comments for complex logic**: Explain "why" not "what"

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test module
cargo test core::scanner
```

### Writing Tests

- **Unit tests**: Place in the same file as the code, in a `#[cfg(test)] mod tests` module
- **Integration tests**: Place in `tests/` directory
- **Benchmarks**: Place in `benches/` directory

## Submitting Changes

### Before Submitting

Checklist before creating a pull request:

- [ ] Code follows the style guidelines
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] No spelling errors (`typos`)
- [ ] Dependencies checked (`cargo deny check`)
- [ ] Pre-commit hooks pass (`pre-commit run --all-files`)
- [ ] All tests pass (`cargo test`)
- [ ] New tests added for new functionality
- [ ] Documentation updated if needed
- [ ] CHANGELOG.md updated with your changes
- [ ] Commit messages are clear and descriptive

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

Examples:
```
feat(scanner): add support for symlink following

Add configuration option to follow symbolic links during repository scanning.
This allows users to include repositories linked from other locations.

Closes #123
```

```
fix(tui): prevent crash on terminal resize

Handle edge case where terminal width becomes zero during rapid resizing.
Added bounds checking in the layout calculation.
```

**Note:** This project uses [git-cliff](https://git-cliff.org/) to automatically generate changelogs from conventional commits. Following this format ensures your contributions are properly documented in release notes.

### Pull Request Process

1. **Create a clear PR title** following commit message format
2. **Describe your changes** in the PR description:
   - What problem does it solve?
   - How does it solve it?
   - Are there any breaking changes?
   - Screenshots/GIFs for UI changes
3. **Link related issues** using "Fixes #123" or "Closes #123"
4. **Respond to review feedback** promptly
5. **Keep PR focused**: One feature/fix per PR when possible

### Review Process

- Maintainers will review your PR within a few days
- Address feedback by pushing new commits or updating existing ones
- Once approved, a maintainer will merge your PR
- Your contribution will be included in the next release!

## Questions?

- Open an issue for general questions
- Start a discussion for design proposals
- Join our community channels (not available yet)

## License

By contributing to RepoNest, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to RepoNest!
