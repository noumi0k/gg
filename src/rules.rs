use crate::config::Rules;
use glob_match::glob_match;

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Allow,
    Confirm,
    Deny,
    DefaultDeny,
}

impl std::fmt::Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decision::Allow => write!(f, "ALLOW"),
            Decision::Confirm => write!(f, "CONFIRM"),
            Decision::Deny => write!(f, "DENY"),
            Decision::DefaultDeny => write!(f, "DEFAULT_DENY"),
        }
    }
}

/// Check if any rule in a RuleSet matches the given args
pub fn has_any_match(rules: &Rules, args: &[String]) -> bool {
    let command = args.join(" ");
    rules
        .allow
        .iter()
        .chain(rules.confirm.iter())
        .chain(rules.deny.iter())
        .any(|p| matches_pattern(p, &command))
}

/// Evaluate args against a specific tool's rules
pub fn evaluate(rules: &Rules, args: &[String], deny_by_default: bool) -> Decision {
    let command = args.join(" ");

    for pattern in &rules.deny {
        if matches_pattern(pattern, &command) {
            return Decision::Deny;
        }
    }

    for pattern in &rules.confirm {
        if matches_pattern(pattern, &command) {
            return Decision::Confirm;
        }
    }

    for pattern in &rules.allow {
        if matches_pattern(pattern, &command) {
            return Decision::Allow;
        }
    }

    if deny_by_default {
        Decision::DefaultDeny
    } else {
        Decision::Allow
    }
}

fn matches_pattern(pattern: &str, command: &str) -> bool {
    if pattern == command {
        return true;
    }

    let glob_pattern = pattern.replace("*", "**");
    if glob_match(&glob_pattern, command) {
        return true;
    }

    if !pattern.contains('*')
        && command.starts_with(pattern)
        && command.as_bytes().get(pattern.len()) == Some(&b' ')
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rules(allow: Vec<&str>, confirm: Vec<&str>, deny: Vec<&str>) -> Rules {
        Rules {
            allow: allow.into_iter().map(String::from).collect(),
            confirm: confirm.into_iter().map(String::from).collect(),
            deny: deny.into_iter().map(String::from).collect(),
        }
    }

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn test_exact_allow() {
        let rules = make_rules(vec!["pr list"], vec![], vec![]);
        assert_eq!(evaluate(&rules, &args("pr list"), true), Decision::Allow);
    }

    #[test]
    fn test_prefix_allow() {
        let rules = make_rules(vec!["pr list"], vec![], vec![]);
        assert_eq!(
            evaluate(&rules, &args("pr list --json url"), true),
            Decision::Allow
        );
    }

    #[test]
    fn test_glob_allow() {
        let rules = make_rules(vec!["api GET *"], vec![], vec![]);
        assert_eq!(
            evaluate(&rules, &args("api GET /repos/foo/bar"), true),
            Decision::Allow
        );
    }

    #[test]
    fn test_deny_priority_over_allow() {
        let rules = make_rules(vec!["pr *"], vec![], vec!["pr merge"]);
        assert_eq!(evaluate(&rules, &args("pr merge"), true), Decision::Deny);
    }

    #[test]
    fn test_confirm() {
        let rules = make_rules(vec![], vec!["pr create"], vec![]);
        assert_eq!(
            evaluate(&rules, &args("pr create"), true),
            Decision::Confirm
        );
    }

    #[test]
    fn test_default_deny() {
        let rules = make_rules(vec!["pr list"], vec![], vec![]);
        assert_eq!(
            evaluate(&rules, &args("repo delete foo"), true),
            Decision::DefaultDeny
        );
    }

    #[test]
    fn test_default_allow() {
        let rules = make_rules(vec![], vec![], vec![]);
        assert_eq!(evaluate(&rules, &args("anything"), false), Decision::Allow);
    }

    #[test]
    fn test_deny_over_confirm() {
        let rules = make_rules(vec![], vec!["pr *"], vec!["pr merge"]);
        assert_eq!(evaluate(&rules, &args("pr merge"), true), Decision::Deny);
        assert_eq!(
            evaluate(&rules, &args("pr create"), true),
            Decision::Confirm
        );
    }

    #[test]
    fn test_has_any_match() {
        let rules = make_rules(vec!["push"], vec![], vec!["push --force*"]);
        assert!(has_any_match(&rules, &args("push origin main")));
        assert!(has_any_match(&rules, &args("push --force")));
        assert!(!has_any_match(&rules, &args("pull")));
    }

    #[test]
    fn test_git_push_force_deny() {
        let rules = make_rules(vec!["push"], vec![], vec!["push --force*", "push -f*"]);
        assert_eq!(
            evaluate(&rules, &args("push --force origin main"), true),
            Decision::Deny
        );
        assert_eq!(
            evaluate(&rules, &args("push origin main"), true),
            Decision::Allow
        );
    }
}
