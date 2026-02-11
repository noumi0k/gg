use crate::detect::Tool;
use crate::rules::Decision;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub fn log_command(tool: Tool, args: &[String], decision: &Decision, log_file: Option<&str>) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let command = sanitize_for_log(&args.join(" "));
    let line = format!("[{}] {} | {} {}\n", timestamp, decision, tool, command);

    let path = log_file.map(PathBuf::from).or_else(default_log_path);

    let Some(path) = path else {
        eprintln!("[gg] could not determine log path");
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut file) => {
            let _ = file.write_all(line.as_bytes());
        }
        Err(e) => {
            eprintln!("[gg] log write error: {}", e);
        }
    }
}

fn sanitize_for_log(s: &str) -> String {
    s.replace('\n', "\\n").replace('\r', "\\r")
}

fn default_log_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".local").join("share").join("gg").join("audit.log"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_log_creates_file_and_writes() {
        let dir = std::env::temp_dir().join("gg_test_log");
        let _ = fs::remove_dir_all(&dir);
        let log_file = dir.join("test.log");

        let args = vec!["push".to_string(), "origin".to_string()];
        log_command(
            Tool::Git,
            &args,
            &Decision::Allow,
            Some(log_file.to_str().unwrap()),
        );

        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("ALLOW"));
        assert!(content.contains("git push origin"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_log_appends() {
        let dir = std::env::temp_dir().join("gg_test_log_append");
        let _ = fs::remove_dir_all(&dir);
        let log_file = dir.join("test.log");

        let args1 = vec!["status".to_string()];
        let args2 = vec!["push".to_string()];
        log_command(
            Tool::Git,
            &args1,
            &Decision::Allow,
            Some(log_file.to_str().unwrap()),
        );
        log_command(
            Tool::Git,
            &args2,
            &Decision::Deny,
            Some(log_file.to_str().unwrap()),
        );

        let content = fs::read_to_string(&log_file).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("ALLOW"));
        assert!(lines[1].contains("DENY"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_log_sanitizes_newlines() {
        let dir = std::env::temp_dir().join("gg_test_log_sanitize");
        let _ = fs::remove_dir_all(&dir);
        let log_file = dir.join("test.log");

        let args = vec!["push\nfake-entry".to_string()];
        log_command(
            Tool::Git,
            &args,
            &Decision::Allow,
            Some(log_file.to_str().unwrap()),
        );

        let content = fs::read_to_string(&log_file).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // Should be a single line — newline in arg was escaped
        assert_eq!(lines.len(), 1);
        assert!(content.contains("push\\nfake-entry"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_default_log_path_exists() {
        let path = default_log_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_str().unwrap().contains("gg"));
        assert!(path.to_str().unwrap().ends_with("audit.log"));
    }
}
