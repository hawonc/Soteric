use anyhow::{Result, anyhow};
use std::collections::HashSet;
use std::process::Command as ProcessCommand;

const AGENT_PROCESS_KEYWORDS: &[&str] = &[
    "codex",
    "claude",
    "opencode",
    "claude-code",
    "claude_code",
    "openhands",
    "cursor",
    "copilot",
    "windsurf",
    "antigravity",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedProcess {
    pub pid: u32,
    pub name: String,
    pub command: String,
}

pub fn scan_agent_processes() -> Result<Vec<DetectedProcess>> {
    #[cfg(not(target_os = "windows"))]
    {
        let output = ProcessCommand::new("ps")
            .args(["-eo", "pid=,command="])
            .output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to run `ps` while scanning processes"));
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        Ok(parse_unix_process_listing(&raw))
    }

    #[cfg(target_os = "windows")]
    {
        let output = ProcessCommand::new("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to run `tasklist` while scanning processes"));
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        Ok(parse_windows_tasklist(&raw))
    }
}

fn parse_unix_process_listing(raw: &str) -> Vec<DetectedProcess> {
    let mut seen = HashSet::<(u32, String)>::new();
    let mut processes = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let pid_part = parts.next().unwrap_or("");
        let command = parts.next().unwrap_or("").trim();

        let pid = match pid_part.parse::<u32>() {
            Ok(value) => value,
            Err(_) => continue,
        };

        if !contains_agent_keyword(command) {
            continue;
        }

        let normalized = command.to_lowercase();
        if !seen.insert((pid, normalized)) {
            continue;
        }

        processes.push(DetectedProcess {
            pid,
            name: display_process_name(command).to_string(),
            command: command.to_string(),
        });
    }

    sort_processes(&mut processes);
    processes
}

#[cfg(target_os = "windows")]
fn parse_windows_tasklist(raw: &str) -> Vec<DetectedProcess> {
    let mut seen = HashSet::<(u32, String)>::new();
    let mut processes = Vec::new();

    for line in raw.lines() {
        let fields = split_csv_row(line);
        if fields.len() < 2 {
            continue;
        }

        let name = fields[0].to_lowercase();
        if !contains_agent_keyword(&name) {
            continue;
        }

        let pid = match fields[1].parse::<u32>() {
            Ok(value) => value,
            Err(_) => continue,
        };

        if !seen.insert((pid, name)) {
            continue;
        }

        processes.push(DetectedProcess {
            pid,
            name: fields[0].clone(),
            command: fields[0].clone(),
        });
    }

    sort_processes(&mut processes);
    processes
}

fn contains_agent_keyword(command: &str) -> bool {
    command
        .to_lowercase()
        .split_whitespace()
        .filter_map(normalize_binary_name)
        .any(|name| AGENT_PROCESS_KEYWORDS.contains(&name))
}

fn normalize_binary_name(token: &str) -> Option<&str> {
    let clean = token.trim_matches(&['"', '\''] as &[char]).trim();
    if clean.is_empty() {
        return None;
    }

    let binary = clean
        .rsplit('/')
        .next()
        .unwrap_or(clean)
        .rsplit('\\')
        .next()
        .unwrap_or(clean);
    Some(binary.strip_suffix(".exe").unwrap_or(binary))
}

fn sort_processes(processes: &mut [DetectedProcess]) {
    processes
        .sort_unstable_by(|left, right| left.name.cmp(&right.name).then(left.pid.cmp(&right.pid)));
}

#[cfg(target_os = "windows")]
fn split_csv_row(row: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in row.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current.trim().to_string());
    fields
        .into_iter()
        .map(|value| value.trim_matches('"').to_string())
        .collect()
}

fn first_token(value: &str) -> &str {
    value.split_whitespace().next().unwrap_or(value)
}

fn display_process_name(command: &str) -> &str {
    let token = first_token(command);
    normalize_binary_name(token).unwrap_or(token)
}

#[cfg(test)]
mod tests {
    use super::{
        contains_agent_keyword, display_process_name, normalize_binary_name,
        parse_unix_process_listing,
    };

    #[test]
    fn detects_supported_agent_binaries() {
        assert!(contains_agent_keyword("/opt/homebrew/bin/codex"));
        assert!(contains_agent_keyword("node /opt/homebrew/bin/codex"));
        assert!(contains_agent_keyword("\"C:\\Tools\\cursor.exe\""));
        assert!(!contains_agent_keyword("/usr/bin/vim"));
    }

    #[test]
    fn normalizes_binary_names_from_paths() {
        assert_eq!(
            normalize_binary_name("/opt/homebrew/bin/codex"),
            Some("codex")
        );
        assert_eq!(
            normalize_binary_name("C:\\Tools\\claude-code"),
            Some("claude-code")
        );
        assert_eq!(normalize_binary_name(""), None);
    }

    #[test]
    fn parses_unix_process_output_and_deduplicates() {
        let parsed = parse_unix_process_listing(
            "\
123 /opt/homebrew/bin/codex
456 /usr/bin/vim
123 /opt/homebrew/bin/codex
789 node /opt/homebrew/bin/codex
",
        );

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].pid, 123);
        assert_eq!(parsed[0].name, "codex");
        assert_eq!(parsed[1].pid, 789);
        assert_eq!(parsed[1].name, "node");
    }

    #[test]
    fn display_name_prefers_binary_basename() {
        assert_eq!(display_process_name("/opt/homebrew/bin/codex"), "codex");
        assert_eq!(display_process_name("node /opt/homebrew/bin/codex"), "node");
    }
}
