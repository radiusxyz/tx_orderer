#!/usr/bin/env bash

# Path to the radius.pem private key
PEM_FILE="radius.pem"

# The program to transfer
PROGRAM_FILE="/home/dev/radius/sequencer/target/release/sequencer"

# List of target hosts (space-separated)
TARGET_HOSTS=("34.64.94.33" "34.47.93.98" "34.64.32.56" "34.64.46.56" "34.47.120.77")

# The username for SSH connection
USER_NAME="dev"   # Example. Replace with the actual username.

echo "===== Starting program distribution (in parallel). ====="
echo "PEM file: $PEM_FILE"
echo "File to transfer: $PROGRAM_FILE"
echo "Target hosts: ${TARGET_HOSTS[@]}"
echo "========================================="

for HOST in "${TARGET_HOSTS[@]}"; do
  (
    echo "----- [${HOST}] Transferring file -----"
    echo "scp -i "$PEM_FILE" "$PROGRAM_FILE" "$USER_NAME@$HOST:~/radius/sequencer/target/release/sequencer""
    scp -i "$PEM_FILE" "$PROGRAM_FILE" "$USER_NAME@$HOST:~/radius/sequencer/target/release/sequencer"
    if [ $? -ne 0 ]; then
      echo "[${HOST}] scp failed."
    else
      echo "[${HOST}] Completed transfer."
    fi
  ) &
done

wait

echo "===== Distribution to all hosts is complete. ====="
