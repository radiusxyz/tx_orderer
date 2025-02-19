#!/bin/bash
TX_ORDERER_INTERNAL_RPC_URL="http://127.0.0.1:4000"

################################# Sequencing (liveness) Contract ####################
LIVENESS_PLATFORM="ethereum" # Option: [ethereum]
LIVENESS_SERVICE_PROVIDER="radius" # Option: [radius]
LIVENESS_RPC_URL=""
LIVENESS_WS_URL=""
LIVENESS_CONTRACT_ADDRESS=""
CLUSTER_ID=""
#####################################################################################


################################# Validation Contract ###############################
### For symbiotic
VALIDATION_PLATFORM="ethereum" # Option: [ethereum]
VALIDATION_SERVICE_PROVIDER="symbiotic" # Option:  [eigen_layer / symbiotic]
VALIDATION_RPC_URL=""
VALIDATION_WS_URL=""
VALIDATION_SERVICE_MANAGER_CONTRACT_ADDRESS=""

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