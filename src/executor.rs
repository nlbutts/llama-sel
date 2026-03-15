use crate::model::{Model, Params};
use anyhow::{Context, Result};
use std::process::Command;

pub fn launch_server(model: &Model, params: &Params) -> Result<()> {
    let mut cmd = Command::new("llama-server");

    cmd.arg("-m").arg(&model.gguf_path);

    if let Some(ctx_size) = params.ctx_size {
        cmd.arg("-c").arg(ctx_size.to_string());
    }

    if let Some(mmproj_path) = &model.mmproj_path {
        if mmproj_path.exists() {
            cmd.arg("--mmproj").arg(mmproj_path);
        }
    }

    if let Some(additional_args) = &params.additional_args {
        cmd.args(additional_args);
    }

    // Print the full command line for debugging
    println!("Executing command: {:?}", cmd);

    // Wait for the llama-server process to complete before returning
    cmd.status()
        .map(|_| ())
        .context("Failed to start llama-server")
}
