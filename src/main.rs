mod checks;
mod config;
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
    spellbook serve llama-3.2-1b  Run a model
    spellbook config create       Create default config
    spellbook list                List available models
    spellbook --help              See this help message again!
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
}

#[derive(Args)]
struct ServeArgs {
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

fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".")) // more graceful handling?
        .join("spellbook")
        .join("spellbook.yaml")
}

fn main() {
    let cli = Cli::parse();

    let use_default = cli.global.config.is_none();
    let config_path = cli.global.config.unwrap_or_else(default_config_path);

    if !use_default && !config_path.exists() {
        eprintln!("Config file not found: {:?}", config_path);
        eprintln!("Create one with `xlm config create` or use --config <path>");
        std::process::exit(1);
    } else if use_default && !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }
        let defaults = include_str!("default_config.yaml");
        fs::write(&config_path, defaults).expect("Failed to write default config");
        println!("Created default config at: {:?}", config_path);
    }

    println!("Reading config from {:?}", config_path);

    let contents = fs::read_to_string(&config_path).expect("Failed to read config file");

    let cfg: config::Config = serde_yaml::from_str(&contents).expect("Failed to parse config");

    match cli.cmd {
        SubCommands::List => {
            for name in cfg.models.keys() {
                println!("{}", name);
            }
        }
        SubCommands::Serve(args) => serve::serve_model(&args.model, &cfg),
        SubCommands::Config(cfg_args) => match cfg_args.cmd {
            ConfigCmd::Create => println!("Config already at: {:?}", config_path),
        },
    }
}
