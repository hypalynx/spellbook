use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub config: UserConfig,
    #[serde(rename = "model-defaults", default)]
    pub model_defaults: ModelDefaults,
    pub models: HashMap<String, ModelConfig>,
}

#[derive(Deserialize, Default, Clone)]
pub struct UserConfig {
    #[serde(rename = "models-directory", default = "default_models_dir")]
    pub models_directory: String,
    #[serde(default)]
    pub tools: Tools,
}

fn default_models_dir() -> String {
    "~models".into()
}

#[derive(Deserialize, Default, Clone)]
pub struct Tools {
    #[serde(rename = "llama.cpp", default)]
    pub llama_cpp: ToolVersion,
}

#[derive(Deserialize, Default, Clone)]
pub struct ToolVersion {
    #[serde(default)]
    pub version: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct ModelDefaults {
    #[serde(rename = "n-parallel", default = "default_n_parallel")]
    pub n_parallel: u32,
    #[serde(default = "default_mmap")]
    pub mmap: bool,
    #[serde(rename = "ctx-token-key", default = "default_ctx_key")]
    pub ctx_token_key: String,
    #[serde(rename = "ctx-token-val", default = "default_ctx_key")]
    pub ctx_token_val: String,
    #[serde(default = "default_threads")]
    pub threads: u32,
}

#[derive(Deserialize, Clone)]
pub struct ModelConfig {
    pub source: ModelSource,
    #[serde(default = "default_context")]
    pub context: u32,
    #[serde(rename = "gpu-layers", default)]
    pub gpu_layers: u32,
    #[serde(default = "default_temp")]
    pub temp: f32,
    #[serde(rename = "top-p", default = "default_top_p")]
    pub top_p: f32,
    #[serde(rename = "top-k", default = "default_top_k")]
    pub top_k: u32,
    #[serde(rename = "repeat-penalty", default = "default_repeat_penalty")]
    pub repeat_penalty: f32,
    #[serde(rename = "min-p", default)]
    pub min_p: f32,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default)]
    pub mlock: bool,
}

#[derive(Deserialize, Clone)]
pub struct ModelSource {
    pub repo: String,
    pub file: String,
}

fn default_n_parallel() -> u32 {
    1
}
fn default_mmap() -> bool {
    true
}
fn default_ctx_key() -> String {
    "q4_0".into()
}
fn default_threads() -> u32 {
    10
}
fn default_context() -> u32 {
    32768
}
fn default_temp() -> f32 {
    0.7
}
fn default_top_p() -> f32 {
    0.8
}
fn default_top_k() -> u32 {
    20
}
fn default_repeat_penalty() -> f32 {
    1.05
}
fn default_port() -> u16 {
    7777
}
fn default_host() -> String {
    "127.0.0.1".into()
}
