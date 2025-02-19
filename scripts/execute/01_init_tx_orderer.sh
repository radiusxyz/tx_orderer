#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

rm -rf $DATA_PATH

echo "Initialize tx_orderer" 

$BIN_PATH init --path $DATA_PATH

TX_ORDERER_INTERNAL_RPC_URL=${TX_ORDERER_INTERNAL_RPC_URL:-$SEQUENCER_INTERNAL_RPC_URL}
TX_ORDERER_CLUSTER_RPC_URL=${TX_ORDERER_CLUSTER_RPC_URL:-$SEQUENCER_CLUSTER_RPC_URL}
TX_ORDERER_EXTERNAL_RPC_URL=${TX_ORDERER_EXTERNAL_RPC_URL:-$SEQUENCER_EXTERNAL_RPC_URL}

sed -i.temp "s|0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80|$TX_ORDERER_PRIVATE_KEY|g" $PRIVATE_KEY_PATH

sed -i.temp "s|internal_rpc_url = \"http://127.0.0.1:4000\"|internal_rpc_url = \"$TX_ORDERER_INTERNAL_RPC_URL\"|g" $CONFIG_FILE_PATH
sed -i.temp "s|cluster_rpc_url = \"http://127.0.0.1:5000\"|cluster_rpc_url = \"$TX_ORDERER_CLUSTER_RPC_URL\"|g" $CONFIG_FILE_PATH
sed -i.temp "s|external_rpc_url = \"http://127.0.0.1:3000\"|external_rpc_url = \"$TX_ORDERER_EXTERNAL_RPC_URL\"|g" $CONFIG_FILE_PATH

sed -i.temp "s|distributed_key_generation_rpc_url = \"http://127.0.0.1:7100\"|distributed_key_generation_rpc_url = \"$DISTRIBUTED_KEY_GENERATOR_RPC_URL\"|g" $CONFIG_FILE_PATH

sed -i.temp "s|seeder_rpc_url = \"http://127.0.0.1:6000\"|seeder_rpc_url = \"$SEEDER_RPC_URL\"|g" $CONFIG_FILE_PATH

sed -i.temp "s|reward_manager_rpc_url = \"http://127.0.0.1:6100\"|reward_manager_rpc_url = \"$REWARD_MANAGER_RPC_URL\"|g" $CONFIG_FILE_PATH

rm $CONFIG_FILE_PATH.temp
rm $PRIVATE_KEY_PATH.temp