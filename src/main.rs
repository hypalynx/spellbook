use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "xlm", author, version, about, long_about = None)]
struct Cli {
    // Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>
}

fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".")) // more graceful handling?
        .join("xlm")
        .join("xlm.yaml")
}

fn main() {
    let cli = Cli::parse();

    let use_default = cli.config.is_none();
    let config_path = cli.config.unwrap_or_else(default_config_path);

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

    let contents = fs::read_to_string(&config_path)
        .expect("Failed to read config file");

    let value: serde_yaml::Value = serde_yaml::from_str(&contents)
        .expect("Failed to parse YAML");

    println!("{:#?}", value);
}
