#!/bin/bash
SEQUENCER_INTERNAL_RPC_URL="http://10.178.0.4:4000"

################################# Sequencing (liveness) Contract ####################
LIVENESS_PLATFORM="ethereum" # Option: [ethereum]
LIVENESS_SERVICE_PROVIDER="radius" # Option: [radius]
LIVENESS_RPC_URL="https://ethereum-holesky-rpc.publicnode.com"
LIVENESS_WS_URL="wss://ethereum-holesky-rpc.publicnode.com"
LIVENESS_CONTRACT_ADDRESS="0x3D1b85847D16D46a729e2471E8E15462Daf49244"
CLUSTER_ID="nodeinfra"
#####################################################################################


################################# Validation Contract ###############################
### For symbiotic
VALIDATION_PLATFORM="ethereum" # Option: [ethereum]
VALIDATION_SERVICE_PROVIDER="symbiotic" # Option:  [eigen_layer / symbiotic]
VALIDATION_RPC_URL="https://ethereum-holesky-rpc.publicnode.com"
VALIDATION_WS_URL="wss://ethereum-holesky-rpc.publicnode.com"
VALIDATION_SERVICE_MANAGER_CONTRACT_ADDRESS="0xB52B38186107C473779805C41a0e9B23df8f25Fb"

### For eigen_layer
# VALIDATION_PLATFORM="ethereum" # Option: [ethereum]
# VALIDATION_SERVICE_PROVIDER="eigen_layer" # Option:  [eigen_layer / symbiotic]
# VALIDATION_RPC_URL=""
# VALIDATION_WS_URL=""
# DELEGATION_MANAGER_CONTRACT_ADDRESS=""
# STAKE_REGISTRY_CONTRACT_ADDRESS=""
# AVS_DIRECTORY_CONTRACT_ADDRESS=""
# AVS_CONTRACT_ADDRESS=""
#####################################################################################