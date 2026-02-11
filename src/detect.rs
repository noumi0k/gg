use crate::config::{Config, Priority};
use crate::rules;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Git,
    Gh,
}

impl std::fmt::Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tool::Git => write!(f, "git"),
            Tool::Gh => write!(f, "gh"),
        }
    }
}

/// Detect whether args belong to git or gh.
/// 1. Check config rules for matches
/// 2. If both match, use priority setting
/// 3. If neither matches, fall back to known subcommand lists
pub fn detect(config: &Config, args: &[String]) -> Option<Tool> {
    let git_match = rules::has_any_match(&config.git.rules, args);
    let gh_match = rules::has_any_match(&config.gh.rules, args);

    match (git_match, gh_match) {
        (true, false) => Some(Tool::Git),
        (false, true) => Some(Tool::Gh),
        (true, true) => Some(match config.options.priority {
            Priority::Git => Tool::Git,
            Priority::Gh => Tool::Gh,
        }),
        (false, false) => detect_by_subcommand(args),
    }
}

fn detect_by_subcommand(args: &[String]) -> Option<Tool> {
    let sub = args.first().map(|s| s.as_str())?;

    const GIT_COMMANDS: &[&str] = &[
        "add",
        "bisect",
        "blame",
        "branch",
        "checkout",
        "cherry-pick",
        "clean",
        "clone",
        "commit",
        "config",
        "diff",
        "fetch",
        "init",
        "log",
        "merge",
        "mv",
        "pull",
        "push",
        "rebase",
        "reflog",
        "remote",
        "reset",
        "restore",
        "revert",
        "rm",
        "show",
        "stash",
        "submodule",
        "switch",
        "tag",
        "worktree",
    ];

    const GH_COMMANDS: &[&str] = &[
        "api",
        "auth",
        "cache",
        "codespace",
        "extension",
        "gist",
        "gpg-key",
        "issue",
        "label",
        "pr",
        "project",
        "release",
        "repo",
        "ruleset",
        "run",
        "search",
        "secret",
        "ssh-key",
        "variable",
    ];

    let is_git = GIT_COMMANDS.contains(&sub);
    let is_gh = GH_COMMANDS.contains(&sub);

    match (is_git, is_gh) {
        (true, false) => Some(Tool::Git),
        (false, true) => Some(Tool::Gh),
        _ => None, // "status" etc. — genuinely ambiguous
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Options, Priority, Rules, ToolConfig};

    fn empty_config() -> Config {
        Config::default()
    }

    fn config_with_rules() -> Config {
        Config {
            git: ToolConfig {
                rules: Rules {
                    allow: vec!["status".to_string(), "log*".to_string()],
                    confirm: vec!["push".to_string()],
                    deny: vec!["push --force*".to_string()],
                },
            },
            gh: ToolConfig {
                rules: Rules {
                    allow: vec!["pr list*".to_string(), "status".to_string()],
                    confirm: vec![],
                    deny: vec!["pr merge*".to_string()],
                },
            },
            options: Options {
                priority: Priority::Git,
                ..Options::default()
            },
        }
    }

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn test_detect_git_by_rules() {
        let config = config_with_rules();
        assert_eq!(detect(&config, &args("push origin main")), Some(Tool::Git));
    }

    #[test]
    fn test_detect_gh_by_rules() {
        let config = config_with_rules();
        assert_eq!(detect(&config, &args("pr list")), Some(Tool::Gh));
    }

    #[test]
    fn test_detect_ambiguous_uses_priority() {
        let config = config_with_rules();
        // "status" is in both git and gh rules
        assert_eq!(detect(&config, &args("status")), Some(Tool::Git));
    }

    #[test]
    fn test_detect_ambiguous_gh_priority() {
        let mut config = config_with_rules();
        config.options.priority = Priority::Gh;
        assert_eq!(detect(&config, &args("status")), Some(Tool::Gh));
    }

    #[test]
    fn test_detect_fallback_to_subcommand() {
        let config = empty_config();
        assert_eq!(detect(&config, &args("commit -m foo")), Some(Tool::Git));
        assert_eq!(detect(&config, &args("issue list")), Some(Tool::Gh));
    }

    #[test]
    fn test_detect_unknown_returns_none() {
        let config = empty_config();
        assert_eq!(detect(&config, &args("foobar")), None);
    }
}
