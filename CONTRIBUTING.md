# Contributing to gg

Thank you for your interest in contributing to gg!

## Development Setup

```bash
git clone https://github.com/noumi0k/gg.git
cd gg
cargo build
cargo test
```

## Before Submitting a PR

1. **Run tests**: `cargo test`
2. **Run clippy**: `cargo clippy -- -D warnings`
3. **Run formatter**: `cargo fmt`
4. **Add tests** for new functionality

## Reporting Issues

- Use [GitHub Issues](https://github.com/noumi0k/gg/issues)
- Include your OS, Rust version, and gg version (`gg --version`)
- Include relevant config (redact sensitive patterns if needed)

## Code Style

- Follow standard Rust conventions
- Keep functions small and focused
- Add `#[cfg(test)]` unit tests in the same file
- Add integration tests in `tests/` for CLI behavior
