#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

echo "add_sequencing_info"
response=$(curl --silent --location $SEEDER_RPC_URL \
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
}')
echo "add_sequencing_info done"
sleep 1

echo ""
echo "initialize_cluster"
response=$(curl --silent --location $SEEDER_RPC_URL \
--header 'Content-Type: application/json' \
--data '{
	"jsonrpc": "2.0",
	"method": "initialize_cluster",
	"params": {
    "platform": "'"$PLATFORM"'",
		"sequencing_function_type": "'"$SEQUENCING_FUNCTION_TYPE"'",
		"service_type": "'"$SERVICE_TYPE"'",
		"cluster_id": "'"$CLUSTER_ID"'"
	},
	"id": 1
}')
echo "initialize_cluster done"
sleep 1

echo ""
echo "register sequencer and rpc_url"
for ((i=0; i<$SEND_ENCRYPTED_TX_COUNT; i++))
do
  response=$(curl --silent --location $SEEDER_RPC_URL \
  --header 'Content-Type: application/json' \
  --data '{
    "jsonrpc": "2.0",
    "method": "register",
    "params": {
      "platform": "'"$PLATFORM"'",
      "sequencing_function_type": "'"$SEQUENCING_FUNCTION_TYPE"'",
      "service_type": "'"$SERVICE_TYPE"'",
      "cluster_id": "'"$CLUSTER_ID"'",

      "address": "'"${SEQUENCER_ADDRESSES[$i]}"'"
    },
    "id": 1
  }')
  echo "register done"
  sleep 1

  response=$(curl --silent --location $SEEDER_RPC_URL \
  --header 'Content-Type: application/json' \
  --data '{
    "jsonrpc": "2.0",
    "method": "register_rpc_url",
    "params": {
      "address": "'"${SEQUENCER_ADDRESSES[$i]}"'",
      "rpc_url": "'"${CLUSTER_RPC_URLS[$i]}"'"
    },
    "id": 1
  }')
  echo "register_rpc_url done - sequencer_address: ${SEQUENCER_ADDRESSES[$i]} / rpc_url: ${CLUSTER_RPC_URLS[$i]}"
  echo ""
  sleep 1
done






