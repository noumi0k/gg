use serde::Deserialize;
use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub git: ToolConfig,
    #[serde(default)]
    pub gh: ToolConfig,
    #[serde(default)]
    pub options: Options,
}

#[derive(Debug, Default, Deserialize)]
pub struct ToolConfig {
    #[serde(default)]
    pub rules: Rules,
}

#[derive(Debug, Default, Deserialize)]
pub struct Rules {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub confirm: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    #[default]
    Git,
    Gh,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Git => write!(f, "git"),
            Priority::Gh => write!(f, "gh"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Options {
    #[serde(default = "default_true")]
    pub log: bool,
    #[serde(default = "default_true")]
    pub deny_by_default: bool,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub log_file: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            log: true,
            deny_by_default: true,
            priority: Priority::default(),
            log_file: None,
        }
    }
}

fn default_true() -> bool {
    true
}

impl Config {
    #[cfg(test)]
    pub fn from_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn load() -> Self {
        Self::load_inner(std::env::var("GG_VERBOSE").is_ok())
    }

    fn load_inner(verbose: bool) -> Self {
        let paths = config_search_paths();
        for path in &paths {
            if let Ok(content) = fs::read_to_string(path) {
                match toml::from_str::<Config>(&content) {
                    Ok(config) => {
                        if verbose {
                            eprintln!("[gg] config loaded from {}", path.display());
                        }
                        return config;
                    }
                    Err(e) => {
                        eprintln!("[gg] config parse error in {}: {}", path.display(), e);
                    }
                }
            }
        }
        if verbose {
            eprintln!("[gg] no config found, using defaults (deny all)");
        }
        Config::default()
    }
}

pub fn config_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let no_local = std::env::var("GG_NO_LOCAL").is_ok();

    // 1. Current directory (skipped if GG_NO_LOCAL is set)
    if !no_local {
        paths.push(PathBuf::from("gg.toml"));
    }

    // 2. $GG_CONFIG env (skipped if GG_NO_LOCAL is set)
    if !no_local {
        if let Ok(p) = std::env::var("GG_CONFIG") {
            paths.push(PathBuf::from(p));
        }
    }

    if let Some(home) = dirs::home_dir() {
        // 3. ~/.config/gg/config.toml (XDG-style)
        paths.push(home.join(".config").join("gg").join("config.toml"));

        // 4. Platform config dir (~/Library/Application Support on macOS)
        if let Some(config_dir) = dirs::config_dir() {
            let platform_path = config_dir.join("gg").join("config.toml");
            if !paths.contains(&platform_path) {
                paths.push(platform_path);
            }
        }

        // 5. ~/.gg.toml
        paths.push(home.join(".gg.toml"));
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.options.deny_by_default);
        assert!(config.options.log);
        assert_eq!(config.options.priority, Priority::Git);
        assert!(config.options.log_file.is_none());
        assert!(config.git.rules.allow.is_empty());
        assert!(config.gh.rules.deny.is_empty());
    }

    #[test]
    fn test_parse_empty_toml() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.options.deny_by_default);
        assert!(config.options.log);
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
[options]
deny_by_default = false
log = false
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(!config.options.deny_by_default);
        assert!(!config.options.log);
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
[options]
deny_by_default = true
priority = "gh"
log_file = "/tmp/gg.log"

[git.rules]
allow = ["status*", "log*"]
confirm = ["push"]
deny = ["push --force*"]

[gh.rules]
allow = ["pr list*"]
deny = ["repo delete*"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.options.priority, Priority::Gh);
        assert_eq!(config.options.log_file.as_deref(), Some("/tmp/gg.log"));
        assert_eq!(config.git.rules.allow.len(), 2);
        assert_eq!(config.git.rules.confirm.len(), 1);
        assert_eq!(config.git.rules.deny.len(), 1);
        assert_eq!(config.gh.rules.allow.len(), 1);
        assert_eq!(config.gh.rules.deny.len(), 1);
    }

    #[test]
    fn test_parse_invalid_toml() {
        let result = Config::from_str("invalid = [[[");
        assert!(result.is_err());
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", Priority::Git), "git");
        assert_eq!(format!("{}", Priority::Gh), "gh");
    }

    #[test]
    fn test_config_search_paths_includes_local() {
        let paths = config_search_paths();
        assert!(paths.iter().any(|p| p.ends_with("gg.toml")));
    }
}
