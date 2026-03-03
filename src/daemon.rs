use crate::config::Config;
use nix::sys::signal::{Signal, kill};
use nix::unistd::{ForkResult, Pid, fork};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const DAEMON_SOCK: &str = "daemon.sock";
const DAEMON_PID: &str = "daemon.pid";
const LLAMA_LOG: &str = "llama-server.log";
const CURRENT_MODEL: &str = "current_model.txt";

fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home directory")
        .join(".local")
        .join("share")
        .join("spellbook")
}

fn ensure_data_dir() -> io::Result<PathBuf> {
    let dir = data_dir();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn read_pid_file() -> Option<Pid> {
    let pid_path = data_dir().join(DAEMON_PID);
    let contents = fs::read_to_string(pid_path).ok()?;
    let pid: i32 = contents.trim().parse().ok()?;
    Some(Pid::from_raw(pid))
}

fn write_pid_file(pid: Pid) -> io::Result<()> {
    let pid_path = ensure_data_dir()?.join(DAEMON_PID);
    fs::write(pid_path, pid.as_raw().to_string())
}

fn remove_pid_file() -> io::Result<()> {
    let pid_path = data_dir().join(DAEMON_PID);
    fs::remove_file(pid_path).ok();
    Ok(())
}

fn remove_socket() -> io::Result<()> {
    let sock_path = data_dir().join(DAEMON_SOCK);
    fs::remove_file(sock_path).ok();
    Ok(())
}

fn write_current_model(model: &str) -> io::Result<()> {
    let model_path = ensure_data_dir()?.join(CURRENT_MODEL);
    fs::write(model_path, model)
}

fn read_current_model() -> Option<String> {
    let model_path = data_dir().join(CURRENT_MODEL);
    fs::read_to_string(model_path).ok()
}

fn is_daemon_running() -> bool {
    if let Some(pid) = read_pid_file() {
        kill(pid, None).is_ok()
    } else {
        false
    }
}

fn expand_path(path: &str) -> PathBuf {
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

fn spawn_llama_server(model_name: &str, config: &Config) -> Option<std::process::Child> {
    let model = config.models.get(model_name)?;
    let models_dir = expand_path(&config.config.models_directory);
    let model_path = models_dir.join(&model.source.file);

    if !model_path.exists() {
        eprintln!("Model file not found: {:?}", model_path);
        return None;
    }

    let d = &config.model_defaults;
    let log_path = data_dir().join(LLAMA_LOG);

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok()?;

    let mut cmd = Command::new("llama-server");
    cmd.args([
        "-m",
        model_path.to_str()?,
        "--alias",
        model_name,
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
    ])
    .stdout(Stdio::from(log_file.try_clone().ok()?))
    .stderr(Stdio::from(log_file));

    if model.mlock {
        cmd.arg("--mlock");
    }
    if !d.mmap {
        cmd.arg("--no-mmap");
    }

    cmd.spawn().ok()
}

fn daemon_process_loop(initial_model: String, config: Config) {
    let sock_path = data_dir().join(DAEMON_SOCK);

    let listener = match UnixListener::bind(&sock_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind socket: {}", e);
            std::process::exit(1);
        }
    };

    let mut current_child = spawn_llama_server(&initial_model, &config);
    let mut _current_model = initial_model;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = [0u8; 256];
                match stream.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let cmd = String::from_utf8_lossy(&buf[..n]);
                        let cmd = cmd.trim();

                        if cmd == "stop" {
                            let _ = stream.write_all(b"Stopping daemon...\n");
                            if let Some(mut child) = current_child {
                                let _ = child.kill();
                            }
                            remove_pid_file().ok();
                            remove_socket().ok();
                            std::process::exit(0);
                        } else if let Some(new_model) = cmd.strip_prefix("switch ") {
                            if config.models.contains_key(new_model) {
                                let _ = stream.write_all(
                                    format!("Switching to {}, loading...\n", new_model).as_bytes(),
                                );

                                if let Some(mut child) = current_child {
                                    let _ = child.kill();
                                }

                                current_child = spawn_llama_server(new_model, &config);
                                _current_model = new_model.to_string();
                                let _ = write_current_model(new_model);
                            } else {
                                let _ = stream.write_all(
                                    format!("Error: model '{}' not found\n", new_model).as_bytes(),
                                );
                            }
                        } else {
                            let _ = stream.write_all(b"Unknown command\n");
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Socket error: {}", e);
            }
        }
    }
}

pub fn start_daemon(model: &str, config: &Config) {
    if is_daemon_running() {
        if let Some(pid) = read_pid_file() {
            eprintln!("Daemon already running (PID: {})", pid);
            eprintln!("Use 'spellbook switch <model>' to change models");
        }
        std::process::exit(1);
    }

    if !config.models.contains_key(model) {
        eprintln!("Model '{}' not found in config.", model);
        eprintln!(
            "Available: {}",
            config.models.keys().cloned().collect::<Vec<_>>().join(", ")
        );
        std::process::exit(1);
    }

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            println!("Daemon started with PID: {}", child);
            println!("Use 'spellbook switch <model>' to change models");
            println!("Use 'spellbook logs' to view llama-server output");
            std::process::exit(0);
        }
        Ok(ForkResult::Child) => {
            nix::unistd::setsid().ok();

            match unsafe { fork() } {
                Ok(ForkResult::Parent { .. }) => {
                    std::process::exit(0);
                }
                Ok(ForkResult::Child) => {
                    if let Err(e) = write_pid_file(nix::unistd::getpid()) {
                        eprintln!("Failed to write PID file: {}", e);
                        std::process::exit(1);
                    }
                    if let Err(e) = write_current_model(model) {
                        eprintln!("Failed to write current model: {}", e);
                        std::process::exit(1);
                    }
                    daemon_process_loop(model.to_string(), config.clone());
                }
                Err(e) => {
                    eprintln!("Fork failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Fork failed: {}", e);
            std::process::exit(1);
        }
    }
}

pub fn switch_model(model: &str, config: &Config) {
    if !config.models.contains_key(model) {
        eprintln!("Model '{}' not found in config.", model);
        eprintln!(
            "Available: {}",
            config.models.keys().cloned().collect::<Vec<_>>().join(", ")
        );
        std::process::exit(1);
    }

    let sock_path = data_dir().join(DAEMON_SOCK);

    match UnixStream::connect(&sock_path) {
        Ok(mut stream) => {
            let cmd = format!("switch {}", model);
            if let Err(e) = stream.write_all(cmd.as_bytes()) {
                eprintln!("Failed to send command: {}", e);
                std::process::exit(1);
            }

            let mut response = String::new();
            if let Err(e) = stream.read_to_string(&mut response) {
                eprintln!("Failed to read response: {}", e);
                std::process::exit(1);
            }

            print!("{}", response);
        }
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            eprintln!("Is the daemon running? Start it with 'spellbook daemon <model>'");
            std::process::exit(1);
        }
    }
}

pub fn stop_daemon() {
    let sock_path = data_dir().join(DAEMON_SOCK);

    match UnixStream::connect(&sock_path) {
        Ok(mut stream) => {
            if let Err(e) = stream.write_all(b"stop") {
                eprintln!("Failed to send stop command: {}", e);
                std::process::exit(1);
            }

            let mut response = String::new();
            if stream.read_to_string(&mut response).is_ok() {
                print!("{}", response);
            }

            for _ in 0..10 {
                if !is_daemon_running() {
                    println!("Daemon stopped.");
                    return;
                }
                thread::sleep(Duration::from_millis(100));
            }

            eprintln!("Daemon did not stop gracefully, killing...");
            if let Some(pid) = read_pid_file() {
                let _ = kill(pid, Signal::SIGTERM);
            }
            remove_pid_file().ok();
            remove_socket().ok();
        }
        Err(_) => {
            if let Some(pid) = read_pid_file() {
                eprintln!(
                    "Daemon socket not accessible, sending SIGTERM to PID {}...",
                    pid
                );
                let _ = kill(pid, Signal::SIGTERM);
                remove_pid_file().ok();
                remove_socket().ok();
            } else {
                eprintln!("Daemon not running.");
            }
        }
    }
}

pub fn show_logs(follow: bool) {
    let log_path = data_dir().join(LLAMA_LOG);

    if !log_path.exists() {
        println!("No log file found. Is the daemon running?");
        return;
    }

    // Read and print all existing lines
    let mut file = match File::open(&log_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open log file: {}", e);
            return;
        }
    };

    let reader = io::BufReader::new(&mut file);
    for line in reader.lines() {
        match line {
            Ok(l) => println!("{}", l),
            Err(_) => break,
        }
    }

    // Track position for follow mode
    let mut pos = match file.stream_position() {
        Ok(p) => p,
        Err(_) => return,
    };

    // Follow new lines if requested
    if follow {
        loop {
            thread::sleep(Duration::from_millis(200));
            let mut file = match File::open(&log_path) {
                Ok(f) => f,
                Err(_) => break,
            };
            if file.seek(SeekFrom::Start(pos)).is_err() {
                break;
            }
            let mut buf = String::new();
            if file.read_to_string(&mut buf).is_err() {
                break;
            }
            if !buf.is_empty() {
                print!("{}", buf);
                pos += buf.len() as u64;
            }
        }
    }
}

pub fn show_status() {
    if let Some(pid) = read_pid_file() {
        if kill(pid, None).is_ok() {
            println!("Daemon Status: running");
            println!("PID: {}", pid);
            if let Some(model) = read_current_model() {
                println!("Current Model: {}", model.trim());
            } else {
                println!("Current Model: unknown");
            }
        } else {
            println!("Daemon Status: not running");
        }
    } else {
        println!("Daemon Status: not running");
    }
}
