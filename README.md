# gg - Git & GitHub CLI Guard

[日本語](README.ja.md)

A safety proxy for `git` and `gh` that enforces command policies. Designed to prevent AI coding agents from executing dangerous operations.

```
$ gg push --force origin main
[gg] BLOCKED: `git push --force origin main` is denied by policy

$ gg status
On branch main
nothing to commit, working tree clean

$ gg pr list
Showing 3 of 3 pull requests in owner/repo
```

## Why

AI coding agents (Claude Code, Cursor, Copilot, etc.) run `git` and `gh` commands on your behalf. Most of the time this is fine, but some commands are destructive and hard to reverse:

- `git push --force` overwrites remote history
- `git reset --hard` discards uncommitted changes
- `gh repo delete` deletes an entire repository
- `gh pr merge` merges without human review

**gg** sits between the agent and git/gh, enforcing allow/confirm/deny rules before any command executes.

## Install

### From source

```bash
cargo install --git https://github.com/noumi0k/gg
```

### From crates.io

```bash
cargo install github-guard
```

### Binary releases

Download from [GitHub Releases](https://github.com/noumi0k/gg/releases).

## Quick Start

1. Create a config file at `~/.config/gg/config.toml`:

```toml
[options]
deny_by_default = true

[git.rules]
allow = ["status*", "log*", "diff*", "branch*"]
confirm = ["add*", "commit*", "push", "pull*"]
deny = ["push --force*", "push -f*", "reset --hard*"]

[gh.rules]
allow = ["pr list*", "pr view*", "issue list*"]
confirm = ["pr create*", "issue create*"]
deny = ["pr merge*", "repo delete*"]
```

2. Configure your AI agent to use `gg` instead of `git`/`gh`:

**Claude Code** (`~/.claude/settings.json`):
```json
{
  "permissions": {
    "allow": ["Bash(gg *)"],
    "deny": ["Bash(git *)", "Bash(gh *)"]
  }
}
```

**Shell aliases** (for manual use):
```bash
alias git='gg --git'
alias gh='gg --gh'
```

## How It Works

```
Agent runs: gg push origin main
                │
                ▼
        ┌──────────────┐
        │  Auto-detect  │  Is this git or gh?
        │  git vs gh    │  (rules → subcommand list)
        └──────┬───────┘
               │
               ▼
        ┌──────────────┐
        │  Evaluate     │  Check deny → confirm → allow
        │  rules        │  (first match wins)
        └──────┬───────┘
               │
       ┌───────┼────────┐
       ▼       ▼        ▼
    [ALLOW] [CONFIRM] [DENY]
       │       │        │
       │    prompt      block
       │    user        (exit 77)
       ▼       │
    Execute  ┌─┴─┐
    git/gh   y   n
             │   │
          Execute block
```

### Rule Evaluation Order

1. **deny** rules are checked first - if matched, command is blocked
2. **confirm** rules are checked next - if matched, user is prompted
3. **allow** rules are checked last - if matched, command runs
4. If no rule matches: `deny_by_default = true` blocks, `false` allows

### Pattern Matching

- Exact: `"status"` matches `gg status`
- Prefix: `"push"` matches `gg push origin main`
- Glob: `"push --force*"` matches `gg push --force origin main`

## Configuration

### Config Search Order

1. `./gg.toml` (project-local)
2. `$GG_CONFIG` (environment variable)
3. `~/.config/gg/config.toml` (XDG)
4. Platform config dir (`~/Library/Application Support/gg/config.toml` on macOS)
5. `~/.gg.toml` (home)

### Options

| Key | Default | Description |
|-----|---------|-------------|
| `deny_by_default` | `true` | Block commands with no matching rule |
| `log` | `true` | Write audit log |
| `log_file` | `~/.local/share/gg/audit.log` | Custom log file path |
| `priority` | `"git"` | Preferred tool when a command matches both git and gh rules |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `GG_CONFIG` | Path to config file |
| `GG_GIT_PATH` | Path to git binary (default: `git`) |
| `GG_GH_PATH` | Path to gh binary (default: `gh`) |
| `GG_VERBOSE` | Show config loading messages when set |
| `GG_NO_LOCAL` | Ignore local `./gg.toml` and `$GG_CONFIG` (use only global config) |

## CLI Reference

```
gg [--git|--gh] <command...>

Options:
  --git          Force command as git
  --gh           Force command as gh
  --dump-config  Show loaded configuration
  -h, --help     Show help
  -V, --version  Show version

Exit codes:
  0     Success
  77    Command blocked by policy
  78    Could not determine git/gh (use --git or --gh)
  other Passthrough from git/gh
```

## Security

See [SECURITY.md](SECURITY.md) for the threat model and limitations.

**Key point**: gg is a policy layer, not a sandbox. It only protects when commands go through it. See the security doc for recommended setup to maximize protection.

## License

[MIT](LICENSE)
