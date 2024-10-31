#!/bin/bash

SEQUENCER_INTERNAL_RPC_URL="127.0.0.1:4000"

LIVENESS_PLATFORM="ethereum" # Option: [ethereum]
LIVENESS_SERVICE_PROVIDER="radius" # Option: [radius]

CLUSTER_ID=""

echo "add_cluster"

curl --location $SEQUENCER_INTERNAL_RPC_URL \
--header 'Content-Type: application/json' \
--data '{
  "jsonrpc": "2.0",
  "method": "add_cluster",
  "params": {
    "platform": "'"$LIVENESS_PLATFORM"'",
    "service_provider": "'"$LIVENESS_SERVICE_PROVIDER"'",
    
    "cluster_id": "'"$CLUSTER_ID"'"
  },
  "id": 1
}'
echo ""
echo "add_cluster done"

