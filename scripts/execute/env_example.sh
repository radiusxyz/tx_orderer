#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
PROJECT_ROOT_PATH="$( cd $SCRIPT_PATH/../.. >/dev/null 2>&1 ; pwd -P )"

BIN_FILE_NAME="sequencer"
BIN_PATH="$PROJECT_ROOT_PATH/scripts/$BIN_FILE_NAME"

DATA_PATH=$PROJECT_ROOT_PATH/data
CONFIG_FILE_PATH=$DATA_PATH/Config.toml
PRIVATE_KEY_PATH=$DATA_PATH/signing_key

# Copy the new version's binary to the scripts directory
if [[ -f "$PROJECT_ROOT_PATH/target/release/$BIN_FILE_NAME" ]]; then
  cp $PROJECT_ROOT_PATH/target/release/$BIN_FILE_NAME $PROJECT_ROOT_PATH/scripts
fi

# Check if the binary exists
if [[ ! -f "$BIN_PATH" ]]; then
    echo "Error: Sequencer binary not found at $BIN_PATH"
    echo "Please run this command 'cp $PROJECT_ROOT_PATH/target/release/$BIN_FILE_NAME $PROJECT_ROOT_PATH/scripts"
    exit 1
fi

# Operating sequencer private key
SEQUENCER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" # Please change this.

# Sequencer
SEQUENCER_INTERNAL_RPC_URL="http://127.0.0.1:4000" # Internal IP - Please change this IP.
SEQUENCER_CLUSTER_RPC_URL="http://127.0.0.1:5000"  # External IP - Please change this IP.
SEQUENCER_EXTERNAL_RPC_URL="http://127.0.0.1:3000" # External IP - Please change this IP.

# DKG (for ENCRYPTED_TRANSACTION_TYPE=skde)
DISTRIBUTED_KEY_GENERATOR_RPC_URL="http://127.0.0.1:7100" # Please change this distribured key generator (external) rpc url.

# Seeder
SEEDER_RPC_URL="http://127.0.0.1:6000" # Please change this seeder (external) rpc url.

# Reward Manager
REWARD_MANAGER_RPC_URL="http://127.0.0.1:6100" # Please change this reward manager (external) rpc url.

