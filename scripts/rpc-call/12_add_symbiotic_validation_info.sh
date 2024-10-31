#!/bin/bash

SEQUENCER_INTERNAL_RPC_URL="127.0.0.1:4000"

VALIDATION_PLATFORM="ethereum" # Option: [ethereum]
VALIDATION_SERVICE_PROVIDER="symbiotic" # Option:  [eigen_layer / symbiotic]

VALIDATION_RPC_URL=""
VALIDATION_WS_URL=""

VALIDATION_SERVICE_MANAGER_CONTRACT_ADDRESS=""

echo "add_validation_info"

curl --location $SEQUENCER_INTERNAL_RPC_URL \
--header 'Content-Type: application/json' \
--data '{
  "jsonrpc": "2.0",
  "method": "add_validation_info",
  "params": {
    "platform": "'"$VALIDATION_PLATFORM"'",
    "validation_service_provider": "'"$VALIDATION_SERVICE_PROVIDER"'",
    "payload": {
      "validation_rpc_url": "'"$VALIDATION_RPC_URL"'",
      "validation_websocket_url": "'"$VALIDATION_WS_URL"'",
      "validation_contract_address": "'"$VALIDATION_SERVICE_MANAGER_CONTRACT_ADDRESS"'"
    }
  },
  "id": 1
}'
echo ""
echo "add_validation_info done"