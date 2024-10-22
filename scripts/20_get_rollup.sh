#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

response=$(curl --silent --location $SEQUENCER_EXTERNAL_RPC_URL \
--header 'Content-Type: application/json' \
--data '{
    "jsonrpc": "2.0",
    "method": "get_rollup",
    "params": {
        "platform": "'"$PLATFORM"'",
        "service_provider": "'"$SERVICE_PROVIDER"'",
        "cluster_id": "'"$CLUSTER_ID"'",
        "rollup_id": "'"$ROLLUP_ID"'"
    },
    "id": 1
}')

echo $response
