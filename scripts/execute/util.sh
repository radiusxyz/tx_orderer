update_env_file() {
  local env_file="$1"

  if [[ ! -f "$env_file" ]]; then
    echo "Error: Environment file not found at $env_file"
    return 1
  fi

  sed -i \
      -e 's/Sequencer/Tx_orderer/' \
      -e 's/sequencer/tx_orderer/' \
      -e 's/^SEQUENCER_PRIVATE_KEY=/TX_ORDERER_PRIVATE_KEY=/' \
      -e 's/^SEQUENCER_INTERNAL_RPC_URL=/TX_ORDERER_INTERNAL_RPC_URL=/' \
      -e 's/^SEQUENCER_CLUSTER_RPC_URL=/TX_ORDERER_CLUSTER_RPC_URL=/' \
      -e 's/^SEQUENCER_EXTERNAL_RPC_URL=/TX_ORDERER_EXTERNAL_RPC_URL=/' \
      "$env_file"

  if ! grep -q '^REWARD_MANAGER_RPC_URL=' "$env_file"; then
    echo -e "\n\n# Reward Manager" >> "$env_file"
    echo 'REWARD_MANAGER_RPC_URL="http://127.0.0.1:6100" # Please change this reward manager (external) rpc url.' >> "$env_file"
  fi

  source "$env_file"
  if ! grep -q '^reward_manager_rpc_url =' "$CONFIG_FILE_PATH"; then
    echo -e "\nreward_manager_rpc_url = \"http://127.0.0.1:6100\"" >> "$CONFIG_FILE_PATH"
  fi
}