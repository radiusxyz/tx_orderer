# Tx_orderer

:warning: Under Construction
> This crate is actively being developed. Breaking changes will occur until mainnet when we will start [Semantic Versioning](https://semver.org/).

Sequencing module for [Radius Block Building Solution](https://github.com/radiusxyz/radius-docs-bbs/blob/main/docs/radius_block_building_solution.md) written in Rust programming language.

Tx_orderer plays a core role in our block-building solution. Working in cluster with leader-based approach brings the following benefits over consensus-based approach:

- Simplicity: With a single leader responsible for sequencing, the system simplifies the decision-making process. This centralized approach reduces the complexity and overhead associated with achieving consensus among multiple nodes.

- Efficiency: Leader-based systems can implement more efficient ordering and syncing related decisions  since the leader node acts as the authoritative source for sequencing. This streamlines the process of agreeing on the state of the system, as there's no need for multiple nodes to negotiate each sequence.

- Reduced Latency: By centralizing the sequencing tasks, leader-based systems can often reduce communication latency. Messages do not need to traverse multiple nodes to reach a consensus, as the leader directly sequences and processes requests. However, note that the leader manages all processing, meaning its performance directly influences the overall network's functionality.

- Optimized Throughput: The leader can optimize sequencing and resource allocation based on the current system load and priorities, potentially improving the overall throughput of the system.

The follower tx_orderer forwards the encrypted transaction to the leader and validates the block commitment made by the leader. The leader tx_orderer issues an order commitment for the encrypted transaction which guarantees that the user transaction will be included in a block and is responsible for registering a block commitment to be validated by followers.

## Encrypted Transaction and Order Commitment
Tx_orderer processes two types of encrypted transactions:
- [PVDE](https://ethresear.ch/t/mev-resistant-zk-rollups-with-practical-vde-pvde/12677) encrypted transaction
- [SKDE](https://ethresear.ch/t/radius-skde-enhancing-rollup-composability-with-trustless-sequencing/19185) encrypted transaction

If a user receives the order-commitment before a specified time ***t*** has elapsed (prior to decryption in the tx_orderer), it confirms that the proposer has sequenced the transaction without decrypting it. This is due to the encryption mechanism that makes it impossible to decrypt the transaction before time ***t***. In case the proposer attempts to reorder transactions after providing the user with this order commitment, the user has a basis to challenge such actions. The order commitment includes critical details such as the exact promised order of the transaction within the block, the rollup block number, and the proposer's signature. These elements serve as evidence of the original commitment made by the tx_orderer.

## Block Building and Validation
As rollup executors requests for a block. The leader tx_orderer builds a block made of decrypted transactions. In order to prove that transactions are properly ordered, the leader submits a block commitment on Validation Contract and followers, upon receiving the submission event, respond to the same contract with boolean response whether the block made by the leader is valid.

## Contributing
We appreciate your contributions to our project. Visit [issues](https://github.com/radiusxyz/tx_orderer/issues) page to start with or refer to the [Contributing guide](https://github.com/radiusxyz/radius-docs-bbs/blob/main/docs/contributing_guide.md).

## Getting Help
Our developers are willing to answer your questions. If you are first and bewildered, refer to the [Getting Help](https://github.com/radiusxyz/radius-docs-bbs/blob/main/docs/getting_help.md) page.
