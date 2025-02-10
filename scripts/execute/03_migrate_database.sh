#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

DATABASE_MIGRATOR_BIN_PATH="$PROJECT_ROOT_PATH/target/release/database_migrator"

$DATABASE_MIGRATOR_BIN_PATH migrate -d $DATA_PATH