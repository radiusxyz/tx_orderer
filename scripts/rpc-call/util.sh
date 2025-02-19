update_env_file() {
  local env_file="$1"

  if [[ ! -f "$env_file" ]]; then
    echo "Error: Environment file not found at $env_file"
    return 1
  fi

  sed -i.bak \
      -e 's/Sequencer/Tx_orderer/' \
      -e 's/sequencer/tx_orderer/' \
      -e 's/^SEQUENCER_INTERNAL_RPC_URL=/TX_ORDERER_INTERNAL_RPC_URL=/' \
      "$env_file"

  echo "✅ Environment file updated successfully: $env_file"
}