use std::fs;
use std::io;
use std::path::PathBuf;

pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            _ => None,
        }
    }

    fn completion_dir(&self) -> PathBuf {
        let home = dirs::home_dir().expect("Failed to get home directory");
        match self {
            Shell::Bash => home
                .join(".local")
                .join("share")
                .join("bash-completion")
                .join("completions"),
            Shell::Zsh => home.join(".zsh").join("completions"),
            Shell::Fish => home.join(".config").join("fish").join("completions"),
        }
    }

    fn filename(&self) -> &'static str {
        match self {
            Shell::Bash => "spellbook",
            Shell::Zsh => "_spellbook",
            Shell::Fish => "spellbook.fish",
        }
    }
}

fn generate_bash_completion() -> String {
    r#"_spellbook_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    local commands="serve list config daemon switch logs completions"
    local config_commands="create"
    local shells="bash zsh fish"
    
    case "${COMP_CWORD}" in
        1)
            COMPREPLY=( $(compgen -W "${commands}" -- "${cur}") )
            ;;
        2)
            case "${prev}" in
                config)
                    COMPREPLY=( $(compgen -W "${config_commands}" -- "${cur}") )
                    ;;
                completions)
                    COMPREPLY=( $(compgen -W "${shells} install" -- "${cur}") )
                    ;;
                serve|daemon|switch)
                    local models=$(spellbook list 2>/dev/null)
                    COMPREPLY=( $(compgen -W "${models}" -- "${cur}") )
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            ;;
        *)
            COMPREPLY=()
            ;;
    esac
}

complete -F _spellbook_completions spellbook
"#
    .to_string()
}

fn generate_zsh_completion() -> String {
    r#"#compdef spellbook

_spellbook() {
    local line
    local -a cmd subcmds

    _arguments -C \
        '1: :->command' \
        '2: :->subcommand' \
        '*:: :->args' && return 0

    case "$line[1]" in
        command)
            local commands=(
                'serve:Run a model in foreground'
                'list:List available models'
                'config:Config management'
                'daemon:Start daemon with model'
                'switch:Switch daemon to new model'
                'logs:View daemon logs'
                'completions:Shell completion scripts'
            )
            _describe -t commands 'spellbook command' commands
            ;;
        subcommand)
            case "$line[1]" in
                config)
                    local config_cmds=(
                        'create:Create default config'
                    )
                    _describe -t config_cmds 'config command' config_cmds
                    ;;
                completions)
                    local shells=(bash zsh fish install)
                    _describe -t shells 'shell or install' shells
                    ;;
                serve|daemon|switch)
                    local models=(${(f)"$(spellbook list 2>/dev/null)"})
                    _describe -t models 'available model' models
                    ;;
            esac
            ;;
    esac
}

_spellbook "$@"
"#
    .to_string()
}

fn generate_fish_completion() -> String {
    r#"complete -c spellbook -n "__fish_use_subcommand" -a "serve" -d "Run a model in foreground"
complete -c spellbook -n "__fish_use_subcommand" -a "list" -d "List available models"
complete -c spellbook -n "__fish_use_subcommand" -a "config" -d "Config management"
complete -c spellbook -n "__fish_use_subcommand" -a "daemon" -d "Start daemon with model"
complete -c spellbook -n "__fish_use_subcommand" -a "switch" -d "Switch daemon to new model"
complete -c spellbook -n "__fish_use_subcommand" -a "logs" -d "View daemon logs"
complete -c spellbook -n "__fish_use_subcommand" -a "completions" -d "Shell completion scripts"

complete -c spellbook -n "__fish_seen_subcommand_from config" -a "create" -d "Create default config"
complete -c spellbook -n "__fish_seen_subcommand_from completions" -a "bash zsh fish install"

complete -c spellbook -n "__fish_seen_subcommand_from serve" -a "(spellbook list)" -d "Model name"
complete -c spellbook -n "__fish_seen_subcommand_from daemon" -a "(spellbook list)" -d "Model name"
complete -c spellbook -n "__fish_seen_subcommand_from switch" -a "(spellbook list)" -d "Model name"
"#
    .to_string()
}

pub fn generate(shell: &Shell) -> String {
    match shell {
        Shell::Bash => generate_bash_completion(),
        Shell::Zsh => generate_zsh_completion(),
        Shell::Fish => generate_fish_completion(),
    }
}

pub fn install(shell: &Shell) -> io::Result<()> {
    let completion_dir = shell.completion_dir();
    fs::create_dir_all(&completion_dir)?;

    let completion_content = generate(shell);
    let completion_path = completion_dir.join(shell.filename());

    fs::write(&completion_path, completion_content)?;

    println!(
        "Installed {} completion to {:?}",
        match shell {
            Shell::Bash => "Bash",
            Shell::Zsh => "Zsh",
            Shell::Fish => "Fish",
        },
        completion_path
    );

    match shell {
        Shell::Bash => {
            println!(
                "Ensure ~/.local/share/bash-completion/completions is in your $PATH or source it manually"
            );
        }
        Shell::Zsh => {
            println!("Add to your ~/.zshrc:");
            println!("  fpath+=~/.zsh/completions");
            println!("  autoload -U compinit && compinit");
        }
        Shell::Fish => {
            println!(
                "Fish will automatically pick up completions from ~/.config/fish/completions/"
            );
        }
    }

    Ok(())
}

pub fn auto_detect_shell() -> Option<Shell> {
    if let Ok(shell) = std::env::var("SHELL") {
        let shell_lower = shell.to_lowercase();
        if shell_lower.contains("bash") {
            Some(Shell::Bash)
        } else if shell_lower.contains("zsh") {
            Some(Shell::Zsh)
        } else if shell_lower.contains("fish") {
            Some(Shell::Fish)
        } else {
            None
        }
    } else {
        None
    }
}
