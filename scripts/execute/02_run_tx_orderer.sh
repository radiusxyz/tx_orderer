#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

source $SCRIPT_PATH/util.sh
update_env_file $SCRIPT_PATH/env.sh

source $SCRIPT_PATH/env.sh

$BIN_PATH start --path $DATA_PATH