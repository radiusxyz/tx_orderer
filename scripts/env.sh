#!/bin/bash
CURRENT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

SEEDER_BIN_PATH=$CURRENT_PATH/../target/release/seeder
SEQUENCER_BIN_PATH=$CURRENT_PATH/../target/release/sequencer