# Contributing to gg

Thank you for your interest in contributing to gg!

## Development Setup

### Prerequisites

- Rust 1.85 or later
- Git

### Building from Source

```bash
git clone https://github.com/noumi0k/gg.git
cd gg
cargo build
```

### Running Tests

```bash
cargo test
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run `cargo fmt` and `cargo clippy -- -D warnings`
5. Run tests with `cargo test`
6. Commit your changes with a descriptive message
7. Push to your branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Address all clippy warnings (`cargo clippy -- -D warnings`)
- Keep functions small and focused
- Add `#[cfg(test)]` unit tests in the same file
- Add integration tests in `tests/` for CLI behavior

## Commit Messages

- Use the imperative mood ("Add feature" not "Added feature")
- Keep the first line under 72 characters
- Reference issues when applicable

## Reporting Issues

- Use [GitHub Issues](https://github.com/noumi0k/gg/issues)
- Include your OS, Rust version, and gg version (`gg --version`)
- Include relevant config (redact sensitive patterns if needed)

## For Maintainers

### Releasing

We use [cargo-release](https://github.com/crate-ci/cargo-release) to automate version bumping, tagging, and changelog updates.

```bash
# Install cargo-release (once)
cargo install cargo-release

# Dry run (preview changes without executing)
cargo release patch

# Execute release
cargo release patch --execute   # 0.1.0 → 0.1.1
cargo release minor --execute   # 0.1.0 → 0.2.0
cargo release major --execute   # 0.1.0 → 1.0.0
```

This will:
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` (`[Unreleased]` → `[version] - date`)
3. Create commit: "chore: release {version}"
4. Create tag: `v{version}`
5. Push to remote

GitHub Actions will then automatically:
- Build binaries for Linux/macOS/Windows
- Create GitHub Release with binaries and release notes
- Publish to crates.io
- Update CHANGELOG.md with git-cliff

### Manual Release (without cargo-release)

1. Update version in `Cargo.toml`
2. Commit: `git commit -am "chore: release vX.Y.Z"`
3. Tag: `git tag vX.Y.Z`
4. Push: `git push && git push --tags`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
