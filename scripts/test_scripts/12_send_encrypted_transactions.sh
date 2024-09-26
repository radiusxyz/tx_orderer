#!/bin/bash
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source $SCRIPT_PATH/env.sh

loop 1 $SEND_ENCRYPTED_TX_COUNT

echo "send_encrypted_transactions"
for ((i=0; i<$SEND_ENCRYPTED_TX_COUNT; i++))
do
  response=$(curl --silent --location $SEQUENCER_EXTERNAL_RPC_URL \
  --header 'Content-Type: application/json' \
  --data '{
      "jsonrpc": "2.0",
      "method": "send_encrypted_transaction",
      "params": {
          "rollup_id": "'"$ROLLUP_ID"'",
          "encrypted_transaction": {
              "Eth": {
                  "open_data": {
                      "raw_tx_hash": "0x83b002caeea5a70ec6b94fd2cf71de5321fd3b94e7ce4535aea3028e31f3b10d",
                      "from": "0x0000000000000000000000000000000000000000",
                      "nonce": "0x2d4e1f",
                      "gas_price": "0x67e30ea2",
                      "gas_limit": "0x100590",
                      "signature": {
                          "r": "0x21f06184dd61bd7b87f8d4fc56bc209dbb1a4e0ce3d5cb1e6a56c55f9cf60620",
                          "s": "0x1e1ae25306ebb1419a41c67db3e511d610fc9a7d83f332b7d7e3c8df951c4a5d",
                          "v": 38
                      },
                      "block_hash": null,
                      "block_number": null,
                      "transaction_index": null,
                      "transaction_type": null,
                      "access_list": null,
                      "max_priority_fee_per_gas": null,
                      "max_fee_per_gas": null,
                      "chain_id": "0x1",
                      "other": {}
                  },
                  "encrypted_data": "16525341917024737325106484584322256261727777345172584850240990032472453366384,18488538614982294004435794039081061793350935819653198597435900081569039526077,6230648815369184305007482001088576712654738956149182620393845874603215881273,12401233369897484656358566955886484060734057798739037588129611310041623957115,21175029189302991762795590042455601872266681927727863287009485964694927814973,13821240530790539884852771460044449860040179161171802339242363981012597584540,6260241981239265167567320542523275272327604346343883556153166995075905315900,21462681774056542128505594737226307761395184979782399495385046509460076677449,18520849493129238815139461670727774242450518348518274719598694020770679495800,3509093391818943401296032238951422537722595892671004595967516868839725696816,4857335857986297547659622287291192668127310255940623242216228209336511490714,5093540281459397010638729846867768839561755688234861817584052684138856305482",
                  "pvde_zkp": null
              }
          },
          "time_lock_puzzle": {
              "o": "24800632767858592699630021229619350340750515295123235515109382632361457959973462631494836542911461609615588445035146262900249613105057866944661823107276982940060681447663919671884062263565619849400305691748347334267117732649085728718857158770735476210710388838267460636764816992049770532127243050113527049841221368366375403619003551212954943237844718315030656359476553325704244754495205727026102328434495054390704815228730971524957467212888158618806421488481437734212884350072085334476673513602143257678445579792357566713658204719868253280227139656685411027713540888689251231536589989270059261228356031016983597592749",
              "t": 2048,
              "n": "25195908475657893494027183240048398571429282126204032027777137836043662020707595556264018525880784406918290641249515082189298559149176184502808489120072844992687392807287776735971418347270261896375014971824691165077613379859095700097330459748808428401797429100642458691817195118746121515172654632282216869987549182422433637259085141865462043576798423387184774447920739934236584823824281198163815010674810451660377306056201619676256133844143603833904414952634432190114657544454178424020924616515723350778707749817125772467962926386356373289912154831438167899885040445364023527381951378636564391212010397122822120720357"
          }
      },
      "id": 1
  }')

  block_height=$(echo "$response" | jq '.result.data.block_height')
  transaction_order=$(echo "$response" | jq '.result.data.transaction_order')
  
  echo "send_encrypted_transaction done - block_height: $block_height / transaction_order: $transaction_order"
  sleep 1
done
echo "send_encrypted_transactions done"

echo ""
echo "finalize_block - block_height: $block_height"
response=$(curl --silent --location $SEQUENCER_EXTERNAL_RPC_URL \
--header 'Content-Type: application/json' \
--data '{
    "jsonrpc": "2.0",
    "method": "finalize_block",
    "params": {
        "rollup_id": "'"$ROLLUP_ID"'",
        "cluster_block_height": 0,
        "rollup_block_height": '$block_height'
    },
    "id": 1
}')
echo "finalize_block done"

sleep 2

echo ""
echo "get_block - block_height: $block_height"
response=$(curl --silent --location $SEQUENCER_EXTERNAL_RPC_URL \
--header 'Content-Type: application/json' \
--data '{
    "jsonrpc": "2.0",
    "method": "get_block",
    "params": {
        "rollup_id": "'"$ROLLUP_ID"'",
        "rollup_block_height": '$block_height'
    },
    "id": 1
}')

block_height=$(echo "$response" | jq '.result.block.block_height')
transaction_count=$(echo "$response" | jq '.result.block.encrypted_transaction_list | length')
proposer_address=$(echo "$response" | jq -r '.result.block.proposer_address')

echo "get_block done - block_height: $block_height / transaction_count: $transaction_count / proposer_address: $proposer_address"