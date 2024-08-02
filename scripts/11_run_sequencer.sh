#!/bin/bash
if [ "$#" -ne 1 ]; then
    echo "Usage: ./11_run_sequencer.sh <sequencer_id>"
    exit 1
fi

SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

SEQUENCER_ID=$1

DATA_PATH=$CURRENT_PATH/sequencers/sequencer_$SEQUENCER_ID

$SEQUENCER_BIN_PATH start --path $DATA_PATH