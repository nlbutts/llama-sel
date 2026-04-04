use crate::model::{GlobalConfig, LlamaServer, Manifest, Model, ModelConfig};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_cache_dir() -> PathBuf {
    std::env::var("LLAMA_CACHE_DIR")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::cache_dir().map(|p| p.join("huggingface")))
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".cache").join("huggingface")
        })
}

pub fn scan_cache(cache_dir: &Path) -> Result<Vec<Model>> {
    let mut models = Vec::new();

    if !cache_dir.exists() {
        anyhow::bail!("Cache directory does not exist: {:?}", cache_dir);
    }

    let hub_dir = cache_dir.join("hub");
    if !hub_dir.exists() {
        return Ok(models);
    }

    for model_entry in fs::read_dir(&hub_dir)
        .context("Failed to read hub directory")?
        .filter_map(|entry| entry.ok())
    {
        let model_dir = model_entry.path();
        if !model_dir.is_dir() {
            continue;
        }

        let snapshots_dir = model_dir.join("snapshots");
        if !snapshots_dir.exists() {
            continue;
        }

        for snapshot_dir in fs::read_dir(&snapshots_dir)
            .context("Failed to read snapshots directory")?
            .filter_map(|entry| entry.ok())
        {
            let snapshot_path = snapshot_dir.path();
            if !snapshot_path.is_dir() {
                continue;
            }

            let gguf_files: Vec<PathBuf> = fs::read_dir(&snapshot_path)
                .context("Failed to read snapshot directory")?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    let path = entry.path();
                    if path.extension().map(|ext| ext != "gguf").unwrap_or(true) {
                        return false;
                    }
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    !filename.contains("mmproj") && !filename.contains("projector")
                })
                .map(|entry| entry.path())
                .collect();

            for gguf_path in gguf_files {
                let gguf_filename = gguf_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let manifest_path = snapshot_path.join(format!("manifest={}.json", gguf_filename));

                let (name, mmproj_path) = if manifest_path.exists() {
                    parse_manifest(&manifest_path, &gguf_path, cache_dir)?
                } else {
                    (gguf_filename.clone(), None)
                };

                let size = fs::metadata(&gguf_path).map(|m| m.len()).unwrap_or(0);

                models.push(Model {
                    name,
                    gguf_path,
                    size,
                    mmproj_path,
                });
            }
        }
    }

    Ok(models)
}

fn parse_manifest(
    manifest_path: &Path,
    _gguf_path: &Path,
    cache_dir: &Path,
) -> Result<(String, Option<PathBuf>)> {
    let content = fs::read_to_string(manifest_path).context("Failed to read manifest file")?;
    let manifest: Manifest =
        serde_json::from_str(&content).context("Failed to parse manifest JSON")?;

    let name = manifest
        .gguf_file
        .rfilename
        .split('/')
        .last()
        .unwrap_or(&manifest.gguf_file.rfilename)
        .to_string();

    let mmproj_path = manifest.mmproj_file.map(|mm| {
        let snapshot_dir = manifest_path.parent().unwrap_or(cache_dir);
        let mm_path = snapshot_dir.join(&mm.rfilename);
        if mm_path.exists() {
            mm_path
        } else {
            cache_dir.join(&mm.rfilename)
        }
    });

    Ok((name, mmproj_path))
}

pub fn load_config(cache_dir: &Path) -> Result<GlobalConfig> {
    let config_path = cache_dir.join("llama_sel_params.yaml");

    if !config_path.exists() {
        let default_config = GlobalConfig::default();
        save_config(&config_path, &default_config)?;
        return Ok(default_config);
    }

    let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
    let config: GlobalConfig =
        serde_yaml::from_str(&content).context("Failed to parse config YAML")?;

    Ok(config)
}

pub fn save_config(config_path: &Path, config: &GlobalConfig) -> Result<()> {
    let content = serde_yaml::to_string(config).context("Failed to serialize config")?;
    fs::write(config_path, content).context("Failed to write config file")?;
    Ok(())
}

pub fn add_model_config(config_path: &Path, model_name: &str, config: &ModelConfig) -> Result<()> {
    let current_content = fs::read_to_string(config_path).context("Failed to read config file")?;
    let mut global_config: GlobalConfig =
        serde_yaml::from_str(&current_content).context("Failed to parse config YAML")?;

    global_config
        .models
        .insert(model_name.to_string(), config.clone());

    save_config(config_path, &global_config)?;
    Ok(())
}

pub fn add_llama_server(config_path: &Path, server: &LlamaServer) -> Result<()> {
    let current_content = fs::read_to_string(config_path).context("Failed to read config file")?;
    let mut global_config: GlobalConfig =
        serde_yaml::from_str(&current_content).context("Failed to parse config YAML")?;

    if !global_config
        .llama_servers
        .iter()
        .any(|s| s.name == server.name)
    {
        global_config.llama_servers.push(server.clone());
        save_config(config_path, &global_config)?;
    }

    Ok(())
}

pub fn format_size(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn extract_quantization(name: &str) -> &str {
    let quant_patterns = [
        "Q4_K_M", "Q4_K_S", "Q5_K_M", "Q5_K_S", "Q6_K", "Q8_0", "Q4_0", "Q5_0", "I4_XX", "I2_X",
        "F16", "F32", "MXFP4", "MXFP8", "Q2_K", "Q3_K_L", "Q3_K_M", "Q3_K_S",
    ];

    for pattern in &quant_patterns {
        if name.contains(pattern) {
            return pattern;
        }
    }

    "Unknown"
}
