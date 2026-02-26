# Spellbook

_spellbook is a CLI tool you can use to configure and run LLMs
locally using opencode + llama.cpp._

This tool was created so I could test out different language
models locally and keep opencode/llama.cpp up to date (or pinned
since they both update very frequently).

## Getting Started

- `spellbook config create` will create a minimal config for you.
- `spellbook config edit` (optional) if you want to setup your own
  models/tweak settings, it will open the config file with
  whatever is set to `EDITOR` in your shell configuration.
- `spellbook serve llama-3.2-3b` will run the model using the
  configuration you've defined.
- Then you can use `opencode` to connect to your running LLM
  server (via llama.cpp) and start prompting!

## Example Configuration

`spellbook` will read a YAML file that it uses for configuration,
here's what I use:

```yaml
tools:
  auto-install: true
  opencode: # An agentic interface to interact with models.
    version: "1.2.3"
  llama.cpp: # used to run the LLM models you download.
    version: "05fa625e"

model-defaults:
  n-parallel: 1
  mmap: true
  ctx-token-key: q4_0
  ctx-token-val: q4_0
  threads: 10

models:
  qwen3-coder-30b-a3b:
    source:
      repo: "unsloth/Qwen3-Coder-30B-A3B-Instruct-GGUF"
      file: "Qwen3-Coder-30B-A3B-Instruct-Q4_K_M.gguf"
    context: 65536
    gpu-layers: 45 # 60 works but testing lower
    temp: 0.7
    top-p: 0.8
    top-k: 20
    min-p: 0.0
    repeat-penalty: 1.05
  qwen3-30b-a3b-instruct:
    source:
      repo: "unsloth/Qwen3-30B-A3B-Instruct-2507-GGUF"
      file: "Qwen3-30B-A3B-Instruct-2507-Q4_K_M.gguf"
    context: 32768
    gpu-layers: 45
    temp: 0.7
    top-p: 0.8
    top-k: 20
    repeat-penalty: 1.15
  qwen3-4b-instruct-2507:
    source:
      repo: "unsloth/Qwen3-4B-Instruct-2507-GGUF"
      file: "Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
    context: 65536
    gpu-layers: 99
    temp: 0.7
    top-p: 0.8
    top-k: 20
    repeat-penalty: 1.15
  llama-3.2-3b:
    source:
      repo: "bartowski/Llama-3.2-3B-Instruct-GGUF"
      file: "Llama-3.2-3B-Instruct-Q4_K_M.gguf"
    context: 32768
    gpu-layers: 99
    temp: 0.6
    top-p: 0.85
    top-k: 30
    repeat-penalty: 1.15
```

## Releasing

The `release.sh` script builds for Linux amd64 and Apple Silicon.

You'll need cross-compilation targets i.e:
`rustup target add aarch64-apple-darwin`
