mod config;
mod detect;
mod logger;
mod rules;

use config::Config;
use detect::Tool;
use rules::Decision;
use std::io::{self, IsTerminal, Write};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    if raw_args.is_empty() {
        print_usage();
        return ExitCode::SUCCESS;
    }

    match raw_args.first().map(|s| s.as_str()) {
        Some("--help" | "-h") => {
            print_usage();
            return ExitCode::SUCCESS;
        }
        Some("--version" | "-V") => {
            eprintln!("gg {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Some("--dump-config") => {
            let config = Config::load();
            eprintln!("{:#?}", config);
            return ExitCode::SUCCESS;
        }
        _ => {}
    }

    let (forced_tool, args) = parse_tool_flag(&raw_args);

    if args.is_empty() {
        eprintln!("[gg] no command given");
        return ExitCode::FAILURE;
    }

    let config = Config::load();

    let tool = match forced_tool {
        Some(t) => t,
        None => match detect::detect(&config, &args) {
            Some(t) => t,
            None => {
                eprintln!(
                    "[gg] BLOCKED: cannot determine if `{}` is git or gh",
                    args.join(" ")
                );
                eprintln!(
                    "[gg] hint: use `gg --git {}` or `gg --gh {}`",
                    args.join(" "),
                    args.join(" ")
                );
                return ExitCode::from(78);
            }
        },
    };

    let tool_rules = match tool {
        Tool::Git => &config.git.rules,
        Tool::Gh => &config.gh.rules,
    };

    let decision = rules::evaluate(tool_rules, &args, config.options.deny_by_default);

    if config.options.log {
        logger::log_command(tool, &args, &decision, config.options.log_file.as_deref());
    }

    match decision {
        Decision::Allow => exec(tool, &args),
        Decision::Confirm => {
            if confirm_with_user(tool, &args) {
                exec(tool, &args)
            } else {
                eprintln!("[gg] cancelled by user");
                ExitCode::FAILURE
            }
        }
        Decision::Deny => {
            eprintln!(
                "[gg] BLOCKED: `{} {}` is denied by policy",
                tool,
                args.join(" ")
            );
            ExitCode::from(77)
        }
        Decision::DefaultDeny => {
            eprintln!(
                "[gg] BLOCKED: `{} {}` has no matching rule (deny_by_default=true)",
                tool,
                args.join(" ")
            );
            ExitCode::from(77)
        }
    }
}

fn parse_tool_flag(args: &[String]) -> (Option<Tool>, Vec<String>) {
    match args.first().map(|s| s.as_str()) {
        Some("--git") => (Some(Tool::Git), args[1..].to_vec()),
        Some("--gh") => (Some(Tool::Gh), args[1..].to_vec()),
        _ => (None, args.to_vec()),
    }
}

fn exec(tool: Tool, args: &[String]) -> ExitCode {
    let bin = match tool {
        Tool::Git => std::env::var("GG_GIT_PATH").unwrap_or_else(|_| "git".to_string()),
        Tool::Gh => std::env::var("GG_GH_PATH").unwrap_or_else(|_| "gh".to_string()),
    };

    match Command::new(&bin).args(args).status() {
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            ExitCode::from(code.clamp(0, 255) as u8)
        }
        Err(e) => {
            eprintln!("[gg] failed to execute {}: {}", bin, e);
            ExitCode::FAILURE
        }
    }
}

fn confirm_with_user(tool: Tool, args: &[String]) -> bool {
    if !io::stdin().is_terminal() {
        eprintln!("[gg] confirmation required but stdin is not a terminal, denying");
        return false;
    }

    eprint!(
        "[gg] confirm: `{} {}` — proceed? [y/N] ",
        tool,
        args.join(" ")
    );
    io::stderr().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    } else {
        false
    }
}

fn print_usage() {
    eprintln!(
        "gg - Git & GitHub CLI Guard v{}

Usage: gg [--git|--gh] <command...>

A safety proxy for git and gh that enforces command policies.
Auto-detects whether a command is git or gh.

Options:
  --git          Force command as git
  --gh           Force command as gh
  --dump-config  Show loaded configuration and exit
  -h, --help     Show this help message
  -V, --version  Show version

Examples:
  gg push origin main          # auto-detect → git push
  gg pr list                   # auto-detect → gh pr list
  gg --git status              # force → git status
  gg --gh status               # force → gh status
  gg push --force origin main  # denied if configured

Config search order:
  1. ./gg.toml
  2. $GG_CONFIG
  3. ~/.config/gg/config.toml
  4. Platform config dir (~/Library/Application Support/gg/config.toml on macOS)
  5. ~/.gg.toml

Exit codes:
  0     Success
  77    Command blocked by policy
  78    Could not determine git/gh (use --git or --gh)
  other Passthrough from git/gh",
        env!("CARGO_PKG_VERSION")
    );
}
