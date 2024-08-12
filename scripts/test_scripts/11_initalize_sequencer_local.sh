#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

for ((i=0; i<$SEND_ENCRYPTED_TX_COUNT; i++))
do
  echo "add_sequencing_info"
  curl --location ${INTERNAL_RPC_URLS[$i]} \
  --header 'Content-Type: application/json' \
  --data '{
    "jsonrpc": "2.0",
    "method": "add_sequencing_info",
    "params": {
      "platform": "'"$PLATFORM"'",
      "sequencing_function_type": "'"$SEQUENCING_FUNCTION_TYPE"'",
      "service_type": "'"$SERVICE_TYPE"'",

      "provider_rpc_url": "'"$PROVIDER_RPC_URL"'",
      "provider_websocket_url": "'"$PROVIDER_WEBSOCKET_URL"'",
      "contract_address": "'"$CONTRACT_ADDRESS"'"
    },
    "id": 1
  }'
  echo "add_sequencing_info done"
  sleep 1

  echo "add_cluster"
  curl --location ${INTERNAL_RPC_URLS[$i]} \
  --header 'Content-Type: application/json' \
  --data '{
    "jsonrpc": "2.0",
    "method": "add_cluster",
    "params": {
      "platform": "'"$PLATFORM"'",
      "sequencing_function_type": "'"$SEQUENCING_FUNCTION_TYPE"'",
      "service_type": "'"$SERVICE_TYPE"'",
      
      "cluster_id": "'"$CLUSTER_ID"'"
    },
    "id": 1
  }'
  echo "add_cluster done"
  sleep 1

  echo "add_rollup"
  curl --location ${INTERNAL_RPC_URLS[$i]} \
  --header 'Content-Type: application/json' \
  --data '{
      "jsonrpc": "2.0",
      "method": "add_rollup",
      "params": {
          "platform": "'"$PLATFORM"'",
          "sequencing_function_type": "'"$SEQUENCING_FUNCTION_TYPE"'",
          "service_type": "'"$SERVICE_TYPE"'",
          "cluster_id": "'"$CLUSTER_ID"'",
          "rollup_id": "'"$ROLLUP_ID"'",
          "rollup_type": "'"$ROLLUP_TYPE"'",
          "rollup_rpc_url": "'"$ROLLUP_RPC_URL"'",
          "rollup_websocket_url": "'"$ROLLUP_WEBSOCKET_URL"'", 
          "bundler_contract_address": "'"$BUNDLE_CONTRACT_ADDRESS"'"
      },
      "id": 1
  }'
  echo "add_rollup done"
done 

