#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

echo "add_sequencing_info (related to liveness)"

TX_ORDERER_INTERNAL_RPC_URL=${TX_ORDERER_INTERNAL_RPC_URL:-$SEQUENCER_INTERNAL_RPC_URL}

curl --location $TX_ORDERER_INTERNAL_RPC_URL \
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