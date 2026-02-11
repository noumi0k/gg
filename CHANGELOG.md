# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-02-10

### Added
- Basic allow/confirm/deny rule engine
- git and gh command proxying with auto-detection
- TOML configuration with 5-location search order
- Glob pattern matching for rules (`push --force*`)
- `deny_by_default` option (default: `true`)
- `priority` option for ambiguous commands (`git` or `gh`)
- Audit logging with newline sanitization
- `--git` / `--gh` flags to force tool selection
- `--dump-config` flag
- `--help` / `-h` flag
- `--version` / `-V` flag
- `GG_NO_LOCAL` environment variable to disable local config loading (security hardening)
- `config.example.toml` with sensible defaults
- Integration tests and unit tests (42 total)
- CI/CD workflows (test with MSRV 1.85, clippy, fmt, cross-platform release)
- GitHub Actions pinned to commit SHA hashes for supply chain security
- `SECURITY.md`, `CONTRIBUTING.md`, `README.md`, `README.ja.md`
