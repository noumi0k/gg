# Security Policy

## Threat Model

`gg` is a **policy enforcement proxy** for `git` and `gh` commands. It is designed to prevent AI coding agents (Claude Code, Cursor, Copilot, etc.) from executing dangerous operations.

### What gg protects against

- Accidental destructive commands (`git push --force`, `git reset --hard`, `gh repo delete`)
- Unreviewed operations by AI agents that run git/gh on your behalf
- Audit trail for all git/gh operations

### What gg does NOT protect against

- **Direct invocation**: If an agent calls `git` or `gh` directly (bypassing `gg`), no protection applies. You must configure your environment so that `gg` intercepts these commands (via PATH, aliases, or tool configuration).
- **Shell escapes**: Commands piped through `sh -c "git push --force"` bypass gg.
- **Config tampering**: If an agent can modify `gg.toml`, it can change the rules. Protect your config file with appropriate permissions.
- **Local config override**: By default, `./gg.toml` in the current directory takes highest priority. A malicious repository could include a permissive `gg.toml` to bypass your global policy. Set `GG_NO_LOCAL=1` to disable local and `$GG_CONFIG` config loading.
- **Binary replacement**: gg does not verify the integrity of the `git` or `gh` binaries it invokes.

## Recommended Setup

To maximize protection when using AI coding agents:

1. Place `gg` earlier in your `PATH` than `git`/`gh`, or use shell aliases
2. Set `GG_NO_LOCAL=1` to prevent untrusted repositories from overriding your policy via local `gg.toml`
3. Set config file permissions to read-only for the agent user
4. Enable audit logging (`log = true`) and monitor the log file
5. Use `deny_by_default = true` (the default) to block any unconfigured commands

## Reporting a Vulnerability

If you discover a security vulnerability, please report it via [GitHub Security Advisories](https://github.com/noumi0k/gg/security/advisories/new) rather than opening a public issue.
