#!/bin/bash

SEQUENCER_INTERNAL_RPC_URL="127.0.0.1:4000"

LIVENESS_PLATFORM="ethereum" # Option: [ethereum]
LIVENESS_SERVICE_PROVIDER="radius" # Option: [radius]

LIVENESS_RPC_URL=""
LIVENESS_WS_URL=""
LIVENESS_CONTRACT_ADDRESS=""

echo "add_sequencing_info (related to liveness)"

curl --location $SEQUENCER_INTERNAL_RPC_URL \
--header 'Content-Type: application/json' \
--data '{
  "jsonrpc": "2.0",
  "method": "add_sequencing_info",
  "params": {
    "platform": "'"$LIVENESS_PLATFORM"'",
    "service_provider": "'"$LIVENESS_SERVICE_PROVIDER"'",

    "payload": {
      "liveness_rpc_url": "'"$LIVENESS_RPC_URL"'",
      "liveness_websocket_url": "'"$LIVENESS_WS_URL"'",
      "contract_address": "'"$LIVENESS_CONTRACT_ADDRESS"'"
    }
  },
  "id": 1
}'
echo ""
echo "add_sequencing_info done"