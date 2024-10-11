#!/bin/bash
if [ "$#" -ne 1 ]; then
    echo "Usage: ./10_init_sequencer.sh <NODE_COUNT>"
    exit 1
fi

SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

NODE_COUNT=$1

rm -rf $CURRENT_PATH/sequencers
mkdir -p $CURRENT_PATH/sequencers

# TODO: remove
SIGNING_KEY_LIST=("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" 
                  "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d" 
                  "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a" 
                  "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6" 
                  "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a" 
                  "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba" 
                  "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e" 
                  "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356" 
                  "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97" 
                  "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6")

for (( node_index=0; node_index<NODE_COUNT; node_index++ )) do
    echo "Initialize sequencer $node_index" 
    data_path=$CURRENT_PATH/sequencers/sequencer_$node_index
    
    $SEQUENCER_BIN_PATH init --path $data_path

    config_file_path=$data_path/config.toml
    
    sed -i.temp "s/sequencer_rpc_url = \"http:\/\/127.0.0.1:3000\"/sequencer_rpc_url = \"http:\/\/$HOST:300$node_index\"/g" $config_file_path
    sed -i.temp "s/internal_rpc_url = \"http:\/\/127.0.0.1:4000\"/internal_rpc_url = \"http:\/\/$HOST:400$node_index\"/g" $config_file_path
    # TODO: temp external_rpc_url
    sed -i.temp "s/cluster_rpc_url = \"http:\/\/127.0.0.1:3000\"/cluster_rpc_url = \"http:\/\/$HOST:300$node_index\"/g" $config_file_path

    sed -i.temp "s/seeder_rpc_url = \"http:\/\/127.0.0.1:6000\"/seeder_rpc_url = \"http:\/\/$SEEDER_HOST:6001\"/g" $config_file_path

    sed -i.temp "s/key_management_system_rpc_url = \"http:\/\/127.0.0.1:7100\"/key_management_system_rpc_url = \"http:\/\/$KEY_MANAGEMENT_SYSTEM_HOST:7100\"/g" $config_file_path
    

    # TODO: remove
    private_key_path=$data_path/signing_key
    signing_key=${SIGNING_KEY_LIST[$node_index]}
    sed -i.temp "s/0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80/$signing_key/g" $private_key_path

    rm $config_file_path.temp
    rm $private_key_path.temp
done  

