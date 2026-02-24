use crate::config::Config;
use std::process::Command;

pub fn verify_dependencies(config: &Config) {
    check_hf_cli();
    check_llama_server();
    check_tool_version("opencode", &config.tools.opencode.version);
    check_tool_version("llama-server", &config.tools.llama_cpp.version);
}

fn check_hf_cli() {
    if !command_exists("hf") {
        eprintln!("Error: huggingface-hub CLI not found");
        eprintln!("Install with: pip install huggingface-hub");
        std::process::exit(1);
    }
}

fn check_llama_server() {
    if !command_exists("llama-server") {
        eprintln!("Error: llama-server not found");
        eprintln!("Install from: https://github.com/ggerganov/llama.cpp/releases");
        std::process::exit(1);
    }
}

fn check_tool_version(tool: &str, expected_version: &str) {
    if expected_version.is_empty() {
        return;
    }

    let version = get_tool_version(tool);
    if let Some(actual) = version {
        if actual != expected_version {
            eprintln!(
                "Warning: {} version mismatch. Expected: {}, Found: {}",
                tool, expected_version, actual
            );
        }
    } else {
        eprintln!("Warning: Could not determine {} version", tool);
    }
}

fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

fn get_tool_version(tool: &str) -> Option<String> {
    let output = match tool {
        "opencode" => Command::new("opencode").arg("--version").output().ok()?,
        "llama-server" => {
            // llama-server outputs version to stderr
            Command::new("llama-server")
                .arg("--version")
                .output()
                .ok()?
        }
        _ => return None,
    };

    let version_output = if !output.stdout.is_empty() {
        String::from_utf8(output.stdout).ok()
    } else {
        String::from_utf8(output.stderr).ok()
    };

    version_output.and_then(|s| {
        // Look for version line with git SHA in parentheses: "version: 113 (05fa625e)"
        s.lines()
            .find_map(|line| {
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.find(')') {
                        let sha = &line[start + 1..end];
                        if sha.len() == 8 && sha.chars().all(|c| c.is_ascii_hexdigit()) {
                            return Some(sha.to_string());
                        }
                    }
                }
                None
            })
    })
}
