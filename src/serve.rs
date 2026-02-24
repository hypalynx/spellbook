use crate::config::Config;

pub fn serve_model(model_name: &str, config: &Config) {
    let model = config.models.get(model_name).unwrap_or_else(|| {
        eprintln!("Model '{}' not found.", model_name);
        eprintln!(
            "Available: {}",
            config
                .models
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
        std::process::exit(1);
    });

    let model_path = dirs::home_dir()
        .expect("Could not find home dir")
        .join("models")
        .join(&model.source.file);

    if !model_path.exists() {
        eprintln!(
            "Downloading {}/{} ...",
            model.source.repo, model.source.file
        );
        let status = std::process::Command::new("hf")
            .args(["download", &model.source.repo, &model.source.file, "--local-dir"])
            .arg(dirs::home_dir().unwrap().join("models"))
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

    cmd.status().expect("Failed to run llama-server");
}
