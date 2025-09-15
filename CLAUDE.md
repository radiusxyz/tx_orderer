# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TX Orderer is a leader-based sequencing module for Radius Block Building Solution written in Rust. It processes encrypted transactions (PVDE/SKDE) with time-delayed decryption for MEV resistance and issues order commitments that guarantee transaction inclusion before decryption occurs.

### Core Architecture

The system operates in a **leader-follower cluster model**:
- **Leader**: Sequences encrypted transactions, issues order commitments, builds blocks of decrypted transactions
- **Followers**: Forward encrypted transactions to leader, validate block commitments

### Key Transaction Flow
1. User sends encrypted transaction (PVDE/SKDE) with specific `key_id` to TX Orderer Leader
2. Leader immediately assigns transaction order and issues Order Commitment (guarantees inclusion)
3. Leader adds encrypted transaction to decryptor and syncs with followers
4. Decryptor continuously polls DKG network (every 500ms) for decryption keys
5. When matching `key_id` decryption key arrives, transaction is decrypted and stored
6. Rollup requests transaction batches from Leader
7. Leader provides ordered batches of decrypted transactions to Rollup for block building

## Workspace Structure

This is a **Rust workspace** with 5 crates:

```
tx_orderer/
├── src/                     # Main binary entry point
├── primitives/              # Core types, error definitions, and traits
├── shared/                  # Shared utilities (logging, profiler)
├── node/                    # Core TX orderer functionality
│   ├── src/rpc/            # RPC servers (internal, cluster, external)
│   ├── src/services/       # Business logic services (NEW)
│   ├── src/tasks/          # Background workers (decryptor, batch processing)
│   ├── src/clients/        # External service clients
│   ├── src/types/          # Data models and structures
│   ├── src/utils/          # Utilities (merkle tree manager)
│   └── src/state.rs        # Application state management
└── cli/                     # Command-line interface
```

### Crate Responsibilities

- **`src/`**: Main binary entry point that delegates to CLI
- **`primitives/`**: Core types (Hash, Address, Platform), error definitions, and traits
- **`shared/`**: Shared utilities (logging, profiler)
- **`node/`**: Core TX orderer functionality with 3-layer architecture:
  - **RPC Layer**: External/cluster/internal RPC servers (controllers)
  - **Service Layer**: Business logic services (TransactionService, BatchService, ValidationService)
  - **Infrastructure Layer**: Background tasks, external clients, and application state
- **`cli/`**: Simple command-line interface (init, start commands)

## Development Commands

### Main Application
```bash
# Run TX Orderer CLI (main entry point)
cargo run -- --help                    # Show CLI help
cargo run -- init                      # Initialize node configuration
cargo run -- start                     # Start TX orderer node

# Or use the binary name directly
cargo run --bin tx_orderer -- start
```

### Build and Check
```bash
# Build entire workspace (5 crates)
cargo build

# Check compilation without building
cargo check

# Build specific crate (use actual crate names from Cargo.toml)
cargo build -p primitives
cargo build -p node
cargo build -p cli

# Check individual crates
cargo check -p shared
cargo check -p node
```

### Testing
```bash
# Run all tests in workspace
cargo test

# Test specific crate
cargo test -p node
cargo test -p cli

# Test with output
cargo test -- --nocapture
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

### Error Handling
Error types with From trait implementations for error conversion between crates.

### Configuration Management
Configuration managed through the CLI with structured types.

## Working with the Codebase

### Adding New Features
1. Determine appropriate crate based on responsibility
2. Use existing patterns from `primitives/` for data types
3. Maintain async interfaces for all I/O operations
4. Follow existing error handling patterns
5. Add tests in the relevant crate

### Common Development Tasks

**Adding New RPC Methods:**
1. Add request/response types to appropriate RPC module
2. Implement handler in RPC module (internal/cluster/external)
3. Register method in server initialization

**Adding New CLI Commands:**
1. Extend `Commands` enum in `cli/src/lib.rs` (currently: Init, Start)
2. Implement command handler function
3. Update match statement in `run()` function

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

- **Database Integration**: Uses Model trait from radius-sdk for persistence
- **RPC Architecture**: Three separate servers (internal, cluster, external)
- **Encryption Focus**: PVDE/SKDE transaction decryption with key-based timing
- **Cluster Coordination**: Leader-follower model for transaction sequencing

## Current Status

✅ **All 5 crates compile successfully**  
✅ **Service layer refactoring completed**  
✅ **Basic CLI with init/start commands**  
✅ **3-layer architecture implemented (RPC → Services → Infrastructure)**

The service layer refactoring is complete with TransactionService, BatchService, and ValidationService handling business logic.

## Refactoring Guidelines

**CRITICAL: When refactoring this codebase, you MUST NOT create new features that don't exist in the current code.** Only move and reorganize existing functionality. Every existing RPC endpoint and business logic must work exactly the same after refactoring.