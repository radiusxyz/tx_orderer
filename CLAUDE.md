# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TX Orderer is a leader-based sequencing module for Radius Block Building Solution written in Rust. It processes encrypted transactions (PVDE/SKDE) with time-delayed decryption for MEV resistance and issues order commitments that guarantee transaction inclusion before decryption occurs.

### Core Architecture

The system operates in a **leader-follower cluster model**:
- **Leader**: Sequences encrypted transactions, issues order commitments, builds blocks of decrypted transactions
- **Followers**: Forward encrypted transactions to leader, validate block commitments

### Key Transaction Flow
1. Encrypted transactions (PVDE/SKDE) arrive with time-lock encryption
2. Leader issues order commitment before decryption time **t**
3. At time **t**, transactions are decrypted and sequenced
4. Leader builds blocks and submits block commitments to validation contract
5. Followers validate and respond with acceptance/rejection

## Workspace Structure

This is a **fully modularized Rust workspace** with 5 specialized crates:

```
tx_orderer/
├── src/main.rs              # Main binary entry point: logger + cli::run()
├── Cargo.toml               # Workspace root with binary configuration
├── primitives/              # Core types (Hash, Address, Platform) and traits
├── shared/                  # Shared utilities and abstractions
├── node/                    # TX orderer node components (RPC, tasks, clients, state)
└── cli/                     # Command-line interface
```

### Crate Responsibilities

- **`src/`**: Main binary entry point that delegates to CLI
- **`primitives/`**: Fundamental types, error definitions, and core traits
- **`shared/`**: Cross-cutting utilities (logging, storage, crypto, merkle trees)
- **`node/`**: Core business logic with 3-layer architecture:
  - **RPC Layer**: External/cluster/internal RPC servers (controllers)
  - **Service Layer**: Business logic services (TransactionService, BatchService, ValidationService)
  - **Infrastructure Layer**: Background tasks, external clients, and application state
- **`cli/`**: Command-line interface with configuration management

## Development Commands

### Main Application
```bash
# Run TX Orderer CLI (main entry point)
cargo run -- --help                    # Show CLI help
cargo run -- start --port 3000         # Start node on port 3000
cargo run -- init                      # Generate default configuration

# Or use the binary name directly
cargo run --bin tx_orderer -- start
```

### Build and Check
```bash
# Build entire workspace (5 crates)
cargo build

# Check compilation without building
cargo check

# Build specific crate
cargo build -p tx-orderer-primitives
cargo build -p tx-orderer-node
cargo build -p tx-orderer-cli

# Check individual crates
cargo check -p tx-orderer-shared
cargo check -p tx-orderer-node
```

### Testing
```bash
# Run all tests in workspace
cargo test

# Test specific crate
cargo test -p tx-orderer-node
cargo test -p tx-orderer-cli

# Test with output
cargo test --bin tx_orderer -- --nocapture
```

## Key Dependencies and Integration

**External Dependencies:**
- `radius-sdk`: Core blockchain SDK for storage, JSON-RPC, signatures
- `skde`: Time-delayed encryption library for MEV resistance
- `tokio`: Async runtime for all async operations
- `clap`: Command-line argument parsing
- `ethers-core`: Ethereum transaction handling
- `serde`: Serialization framework

**radius-sdk Integration:**
- `CachedKvStore` and `KvStore` for persistent storage
- JSON-RPC client/server infrastructure
- Address and signature handling
- Model trait for database operations

## Architecture Patterns

### Async-First Design
All storage, network, and processing operations use async/await with tokio runtime.

### Memory Management
- Strategic use of `Vec<u8>` for serialized data flexibility
- Reference-based parameters (`&[u8]`, `&str`) to avoid unnecessary copying
- Careful `.clone()` usage only when ownership transfer is required

### Error Handling
Comprehensive error types with proper From trait implementations for error conversion between crates.

### Configuration Management
TOML-based configuration files managed through the CLI with structured types.

## Working with the Codebase

### Adding New Features
1. Determine appropriate crate based on responsibility
2. Use existing patterns from `primitives/` for data types
3. Maintain async interfaces for all I/O operations
4. Follow existing error handling patterns
5. Add tests in the relevant crate

### Common Development Tasks

**Adding New RPC Methods:**
1. Add request/response types to `node/src/rpc/types.rs`
2. Implement handler in appropriate RPC module (internal/cluster/external)
3. Register method in server initialization

**Adding New CLI Commands:**
1. Extend command enums in `cli/src/lib.rs`
2. Implement command handler function
3. Add configuration if needed

**Adding New External Clients:**
1. Create new module in `node/src/clients/`
2. Implement client struct with HTTP methods
3. Add initialization in `node/src/lib.rs`

### Key Implementation Patterns

1. **Leader-Follower Architecture**: Single leader handles sequencing to reduce consensus overhead
2. **Time-Lock Security**: PVDE/SKDE encryption prevents transaction reordering before time **t**
3. **Merkle Proofs**: Transaction inclusion proofs for validation contract submissions
4. **Application State**: Centralized `AppState` in node crate for shared context
5. **Background Workers**: Async tasks for decryption, batch processing, and synchronization

## Important Notes

- **No Copy Trait Overuse**: Prefer references and strategic cloning
- **Database Integration**: Uses Model trait from radius-sdk for persistence
- **RPC Architecture**: Three separate servers (internal, cluster, external) with different purposes
- **Encryption Focus**: Core feature is time-delayed transaction decryption for MEV resistance
- **Cluster Coordination**: Leader election and follower validation are central to the design

## Compilation Status

✅ **All 5 crates compile successfully**  
✅ **CLI fully functional with all commands**  
✅ **Workspace dependencies properly configured**  
✅ **Clean separation of concerns across crates**

The refactoring from monolithic to modular architecture is **complete** and ready for production development.

## Refactoring Guidelines

**CRITICAL: When refactoring this codebase, you MUST NOT create new features that don't exist in the current code.** Only move and reorganize existing functionality. Every existing RPC endpoint and business logic must work exactly the same after refactoring.