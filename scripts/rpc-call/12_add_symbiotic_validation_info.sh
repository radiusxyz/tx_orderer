#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

echo "add_validation_info"

# For previous version
TX_ORDERER_INTERNAL_RPC_URL=${TX_ORDERER_INTERNAL_RPC_URL:-$SEQUENCER_INTERNAL_RPC_URL}

curl --location $TX_ORDERER_INTERNAL_RPC_URL \
--header 'Content-Type: application/json' \
--data '{
  "jsonrpc": "2.0",
  "method": "add_validation_info",
  "params": {
    "platform": "'"$VALIDATION_PLATFORM"'",
    "validation_service_provider": "'"$VALIDATION_SERVICE_PROVIDER"'",
    "validation_info": {
      "validation_rpc_url": "'"$VALIDATION_RPC_URL"'",
      "validation_websocket_url": "'"$VALIDATION_WS_URL"'",
      "validation_contract_address": "'"$VALIDATION_SERVICE_MANAGER_CONTRACT_ADDRESS"'"
    }
  },
  "id": 1
}'
echo ""
echo "add_validation_info done"