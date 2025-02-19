#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
PROJECT_ROOT_PATH="$( cd $SCRIPT_PATH/../.. >/dev/null 2>&1 ; pwd -P )"

BIN_FILE_NAME="tx_orderer"
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
    echo "Error: TxOrderer binary not found at $BIN_PATH"
    echo "Please run this command 'cp $PROJECT_ROOT_PATH/target/release/$BIN_FILE_NAME $PROJECT_ROOT_PATH/scripts"
    exit 1
fi

# Operating tx_orderer private key
TX_ORDERER_PRIVATE_KEY="0x52393245d4fa91df3686707f324f6700a536ebd2ff3bc06747cbd7c89ff40f5d" # Please change this.

# TxOrderer
TX_ORDERER_INTERNAL_RPC_URL="http://10.178.0.8:4000" # Internal IP - Please change this IP.
TX_ORDERER_CLUSTER_RPC_URL="http://34.47.113.152:5000"  # External IP - Please change this IP.
TX_ORDERER_EXTERNAL_RPC_URL="http://34.47.113.152:3000" # External IP - Please change this IP.

# DKG (for ENCRYPTED_TRANSACTION_TYPE=skde)
DISTRIBUTED_KEY_GENERATOR_RPC_URL="http://35.189.33.95:7100" # Please change this distribured key generator (external) rpc url.

# Seeder
SEEDER_RPC_URL="http://34.174.42.176:6000" # Please change this seeder (external) rpc url.

# Reward Manager
REWARD_MANAGER_RPC_URL="https://649a-59-10-110-198.ngrok-free.app/rewards" # Please change this reward manager (external) rpc url.
