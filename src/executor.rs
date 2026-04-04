use crate::model::{GlobalConfig, Model};
use anyhow::{Context, Result};
use std::process::Command;

pub fn launch_server(
    model: &Model,
    config: &GlobalConfig,
    selected_server: Option<&str>,
) -> Result<()> {
    let server_name = selected_server.unwrap_or(&config.default_llama_server);
    let server_cmd = config.get_server_path(server_name);

    let model_config = config.get_model_config(&model.name);

    let mut cmd = Command::new(&server_cmd);

    cmd.arg("-m").arg(&model.gguf_path);

    if let Some(ctx_size) = model_config.ctx_size {
        cmd.arg("-c").arg(ctx_size.to_string());
    }

    if let Some(mmproj_path) = &model.mmproj_path {
        if mmproj_path.exists() {
            cmd.arg("--mmproj").arg(mmproj_path);
        }
    }

    if let Some(additional_args) = &model_config.additional_args {
        for arg in additional_args.split_whitespace() {
            cmd.arg(arg);
        }
    }

    println!("Executing command: {:?}", cmd);

    cmd.status()
        .map(|_| ())
        .context("Failed to start llama-server")
}
