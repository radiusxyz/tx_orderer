#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

DATA_PATH=$CURRENT_PATH/seeder

rm -rf $DATA_PATH

$SEEDER_BIN_PATH init --path $DATA_PATH