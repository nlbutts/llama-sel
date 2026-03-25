use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub name: String,
    pub gguf_path: PathBuf,
    pub size: u64,
    pub mmproj_path: Option<PathBuf>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Params {
    pub ctx_size: Option<u32>,
    pub additional_args: Option<Vec<String>>,
    pub llama_server_path: Option<String>,
}

impl Params {
    pub fn default() -> Self {
        Self {
            ctx_size: None,
            additional_args: None,
            llama_server_path: None,
        }
    }
}
