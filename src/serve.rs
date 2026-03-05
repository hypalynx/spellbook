use crate::checks::verify_dependencies;
use crate::config::Config;

fn expand_path(path: &str) -> std::path::PathBuf {
    let home = dirs::home_dir().or_else(|| std::env::var("HOME").ok().map(|h| h.into()));
    if let Some(home) = home {
        let stripped = if let Some(s) = path.strip_prefix("~") {
            s.trim_start_matches('/')
        } else if let Some(s) = path.strip_prefix("$HOME") {
            s.trim_start_matches('/')
        } else {
            return path.into();
        };
        return home.join(stripped);
    }
    path.into()
}

pub fn serve_model(model_name: &str, config: &Config) {
    verify_dependencies(config);
    let model = config.models.get(model_name).unwrap_or_else(|| {
        eprintln!("Model '{}' not found.", model_name);
        eprintln!(
            "Available: {}",
            config.models.keys().cloned().collect::<Vec<_>>().join(", ")
        );
        std::process::exit(1);
    });

    let models_dir = expand_path(&config.config.models_directory);
    let model_path = models_dir.join(&model.source.file);

    if !model_path.exists() {
        eprintln!(
            "Downloading {}/{} ...",
            model.source.repo, model.source.file
        );
        let status = std::process::Command::new("hf")
            .args([
                "download",
                &model.source.repo,
                &model.source.file,
                "--local-dir",
            ])
            .arg(&models_dir)
            .status()
            .expect("Failed to run hf; install with: pip install huggingface-hub");
        if !status.success() {
            std::process::exit(1);
        }
    }

    let d = &config.model_defaults;
    println!("Starting llama-server on {}:{} ...", model.host, model.port);

    let mut cmd = std::process::Command::new("llama-server");
    cmd.args([
        "-m",
        model_path.to_str().unwrap(),
        "--jinja",
        "--host",
        &model.host,
        "--port",
        &model.port.to_string(),
        "-c",
        &model.context.to_string(),
        "--metrics",
        "-ngl",
        &model.gpu_layers.to_string(),
        "-np",
        &d.n_parallel.to_string(),
        "-ctk",
        &d.ctx_token_key,
        "-ctv",
        &d.ctx_token_val,
        "--threads",
        &d.threads.to_string(),
        "--temp",
        &model.temp.to_string(),
        "--top-p",
        &model.top_p.to_string(),
        "--top-k",
        &model.top_k.to_string(),
        "--min-p",
        &model.min_p.to_string(),
        "--repeat-penalty",
        &model.repeat_penalty.to_string(),
    ]);
    if model.mlock {
        cmd.arg("--mlock");
    }
    if !d.mmap {
        cmd.arg("--no-mmap");
    }
    if model.flash_attn {
        cmd.args(["-fa", "on"]);
    }
    if let Some(kwargs) = &model.chat_template_kwargs {
        cmd.args(["--chat-template-kwargs", kwargs]);
    }

    cmd.status().expect("Failed to run llama-server");
}
