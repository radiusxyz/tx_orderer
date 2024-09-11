#!/bin/bash
if [ "$#" -ne 1 ]; then
    echo "Usage: ./11_run_sequencer.sh <node_index>"
    exit 1
fi

SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

node_index=$1

DATA_PATH=$CURRENT_PATH/sequencers/sequencer_$node_index

$SEQUENCER_BIN_PATH start --path $DATA_PATH