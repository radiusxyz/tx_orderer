#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
PROJECT_ROOT_PATH="$( cd $SCRIPT_PATH/../.. >/dev/null 2>&1 ; pwd -P )"
SEQUENCER_BIN_PATH="$PROJECT_ROOT_PATH/scripts/sequencer"

if [[ ! -f "$SEQUENCER_BIN_PATH" ]]; then
    echo "Error: Sequencer binary not found at $SEQUENCER_BIN_PATH"
    echo "Please run this command 'cp $PROJECT_ROOT_PATH/target/release/sequencer $PROJECT_ROOT_PATH/scripts"
    exit 1
fi

DATA_PATH=$PROJECT_ROOT_PATH/data
CONFIG_FILE_PATH=$DATA_PATH/Config.toml
PRIVATE_KEY_PATH=$DATA_PATH/signing_key

# Operating sequencer private key
SEQUENCER_PRIVATE_KEY="0x10732827c50f1e675a69927c1fdd1a4c0c3519090090fe34c4c98f7d86827aa6"

# Sequencer
SEQUENCER_INTERNAL_RPC_URL="127.0.0.1:4000"
SEQUENCER_CLUSTER_RPC_URL="127.0.0.1:5000"
SEQUENCER_EXTERNAL_RPC_URL="127.0.0.1:3000"

# DKG (for skde)
DISTRIBUTED_KEY_GENERATOR_RPC_URL="127.0.0.1:7100"

# Seeder
SEEDER_RPC_URL="127.0.0.1:6001"


