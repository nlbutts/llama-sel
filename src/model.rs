use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub name: String,
    pub gguf_path: PathBuf,
    pub size: u64,
    pub mmproj_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaServer {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub llama_server: Option<String>,
    pub ctx_size: Option<u32>,
    pub additional_args: Option<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            llama_server: None,
            ctx_size: None,
            additional_args: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub default_llama_server: String,
    pub llama_servers: Vec<LlamaServer>,
    pub model_defaults: ModelConfig,
    pub models: HashMap<String, ModelConfig>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_llama_server: "llama-server".to_string(),
            llama_servers: vec![LlamaServer {
                name: "llama-server".to_string(),
                path: "llama-server".to_string(),
            }],
            model_defaults: ModelConfig::default(),
            models: HashMap::new(),
        }
    }
}

impl GlobalConfig {
    pub fn get_model_config(&self, model_name: &str) -> ModelConfig {
        let mut config = self.model_defaults.clone();

        if let Some(model_config) = self.models.get(model_name) {
            if let Some(server) = &model_config.llama_server {
                config.llama_server = Some(server.clone());
            }
            if let Some(ctx) = model_config.ctx_size {
                config.ctx_size = Some(ctx);
            }
            if let Some(args) = &model_config.additional_args {
                config.additional_args = Some(args.clone());
            }
        }

        config
    }

    pub fn get_server_path(&self, server_name: &str) -> String {
        self.llama_servers
            .iter()
            .find(|s| s.name == server_name)
            .map(|s| s.path.clone())
            .unwrap_or_else(|| server_name.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub layers: Vec<Layer>,
    pub gguf_file: GgufFile,
    pub mmproj_file: Option<MmprojFile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Layer {
    pub digest: String,
    pub media_type: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GgufFile {
    pub rfilename: String,
    pub blob_id: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MmprojFile {
    pub rfilename: String,
    pub blob_id: String,
    pub size: u64,
}
