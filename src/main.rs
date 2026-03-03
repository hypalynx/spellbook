mod checks;
mod completions;
mod config;
mod daemon;
mod serve;

use clap::{Args, Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "spellbook",
    author,
    version,
    arg_required_else_help = true,
    about = "spellbook, Language Model eXperimentation CLI tool",
    long_about = "A CLI tool for managing and running LLM models via llama.cpp with YAML config.

 Examples:
    spellbook serve llama-3.2-1b  Run a model in foreground
    spellbook daemon llama-3.2-1b  Start daemon with model
    spellbook switch llama-3.2-3b  Switch daemon to new model
    spellbook status               Check daemon and current model
    spellbook logs                 View daemon logs
    spellbook list                 List available models
    spellbook config create        Create default config
    spellbook completions install  Install shell completions
    spellbook --help               See this help message again!
 "
)]
struct Cli {
    #[clap(flatten)]
    global: GlobalArgs,
    #[clap(subcommand)]
    cmd: SubCommands,
}

#[derive(Parser)]
struct GlobalArgs {
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum SubCommands {
    Serve(ServeArgs),
    List,
    Config(ConfigArgs),
    Daemon(DaemonArgs),
    Switch(SwitchArgs),
    Stop,
    Status,
    Logs,
    Completions(CompletionsArgs),
}

#[derive(Args)]
struct ServeArgs {
    model: String,
}

#[derive(Args)]
struct DaemonArgs {
    model: String,
}

#[derive(Args)]
struct SwitchArgs {
    model: String,
}

#[derive(Args)]
struct ConfigArgs {
    #[command(subcommand)]
    cmd: ConfigCmd,
}

#[derive(Subcommand)]
enum ConfigCmd {
    Create,
}

#[derive(Args)]
struct CompletionsArgs {
    shell: String,
}

fn default_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("spellbook")
        .join("spellbook.yaml")
}

fn main() {
    let cli = Cli::parse();

    let use_default = cli.global.config.is_none();
    let config_path = cli.global.config.unwrap_or_else(default_config_path);

    if !use_default && !config_path.exists() {
        eprintln!("Config file not found: {:?}", config_path);
        eprintln!("Create one with `spellbook config create` or use --config <path>");
        std::process::exit(1);
    } else if use_default && !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }
        let defaults = include_str!("default_config.yaml");
        fs::write(&config_path, defaults).expect("Failed to write default config");
        println!("Created default config at: {:?}", config_path);
    }

    let contents = fs::read_to_string(&config_path).expect("Failed to read config file");
    let cfg: config::Config = serde_yaml::from_str(&contents).expect("Failed to parse config");

    match cli.cmd {
        SubCommands::List => {
            for name in cfg.models.keys() {
                println!("{}", name);
            }
        }
        SubCommands::Serve(args) => serve::serve_model(&args.model, &cfg),
        SubCommands::Daemon(args) => daemon::start_daemon(&args.model, &cfg),
        SubCommands::Switch(args) => daemon::switch_model(&args.model, &cfg),
        SubCommands::Stop => daemon::stop_daemon(),
        SubCommands::Status => daemon::show_status(),
        SubCommands::Logs => daemon::show_logs(),
        SubCommands::Config(cfg_args) => match cfg_args.cmd {
            ConfigCmd::Create => println!("Config already at: {:?}", config_path),
        },
        SubCommands::Completions(args) => {
            if args.shell == "install" {
                if let Some(shell) = completions::auto_detect_shell() {
                    completions::install(&shell).expect("Failed to install completions");
                } else {
                    eprintln!("Could not auto-detect shell. Please specify: bash, zsh, or fish");
                    std::process::exit(1);
                }
            } else if let Some(shell) = completions::Shell::from_str(&args.shell) {
                print!("{}", completions::generate(&shell));
            } else {
                eprintln!("Unknown shell: {}", args.shell);
                eprintln!("Supported shells: bash, zsh, fish");
                eprintln!("Or use 'install' to auto-detect and install");
                std::process::exit(1);
            }
        }
    }
}
