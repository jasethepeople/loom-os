#!/usr/bin/env bash
set -e

MODEL_DIR="./models"
mkdir -p "$MODEL_DIR"

echo "Downloading Mistral 7B Instruct (Q4_K_M quantized)..."
wget -c "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf"   -O "$MODEL_DIR/mistral-7b-instruct-v0.2.Q4_K_M.gguf" || true

echo ""
echo "Models directory:"
ls -lh "$MODEL_DIR/"

echo ""
echo "Update config/loom.toml with your model paths:"
echo "  model_path = "$MODEL_DIR/mistral-7b-instruct-v0.2.Q4_K_M.gguf""
