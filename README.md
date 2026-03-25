# llama-sel

A TUI (Text User Interface) model selector for llama.cpp that scans your Hugging Face cache directory, displays available GGUF models, and launches `llama-server` with your selected model.

## Features

- **Interactive TUI**: Navigate through your models using a terminal-based interface
- **Model Discovery**: Automatically scans your Hugging Face cache directory for GGUF files
- **Model Details**: Shows model name, size, quantization type, and MMProj information
- **Smart Launch**: Automatically launches `llama-server` with the selected model
- **MMProj Support**: Handles multimodal projector files for vision models
- **Customizable**: Supports custom context size and additional command-line arguments

## Requirements

- `llama-server` binary must be installed and available in your PATH
- Models stored in GGUF format in the Hugging Face cache directory

## Installation

```bash
cargo install --path .
```

Or build locally:

```bash
cargo build --release
```

## Usage

Simply run:

```bash
llama-sel
```

The application will:
1. Scan your Hugging Face cache directory for GGUF models
2. Display an interactive list of available models
3. Launch `llama-server` with your selected model

### Keyboard Controls

- **↑/↓ or j/k**: Navigate up/down through the model list
- **Page Up/Page Down**: Jump 5 models up/down
- **Enter or o**: Select and launch the highlighted model
- **q or Esc**: Quit without selecting a model

## Configuration

### Cache Directory

By default, models are scanned from `~/.cache/huggingface/`. You can override this with:

```bash
export LLAMA_CACHE_DIR=/path/to/your/cache
```

### Server Parameters

Create a `llama_sel_params.yaml` file in your cache directory to customize server launch parameters:

```yaml
ctx_size: 4096
llama_server_path: "/path/to/llama-server"
additional_args:
  - "--host"
  - "0.0.0.0"
  - "--port"
  - "8080"
```

Available options:
- `ctx_size`: Context size for the model (e.g., 4096, 8192)
- `llama_server_path`: Custom path to the llama-server binary
- `additional_args`: Additional command-line arguments to pass to `llama-server`

## Supported Quantizations

The tool recognizes and displays common GGUF quantization formats:
- Q4_K_M, Q4_K_S, Q4_0
- Q5_K_M, Q5_K_S, Q5_0
- Q6_K, Q8_0
- I4_XX, I2_X
- F16, F32
- MXFP4, MXFP8
- Q2_K, Q3_K_L, Q3_K_M, Q3_K_S

## License

MIT
