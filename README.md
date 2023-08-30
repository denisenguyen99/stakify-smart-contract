# The smart contracts for stakify

[![CircleCI](https://dl.circleci.com/status-badge/img/gh/buzz-space/stakify-smart-contract/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/buzz-space/stakify-smart-contract/tree/main)
[![codecov](https://codecov.io/gh/buzz-space/stakify-smart-contract/graph/badge.svg?token=ZQMZKQWY0J)](https://codecov.io/gh/buzz-space/stakify-smart-contract)

The automated market-maker on [Aura network](https://aura.network/).

## Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install)
-   [Cosmos SDK](https://docs.cosmos.network/main)
-   [CosmWasm](https://cosmwasm.com/)

## Contracts

| Name                                                                                                            | Description                             |
| --------------------------------------------------------------------------------------------------------------- | --------------------------------------- |
| [`campaign_factory`](https://github.com/buzz-space/stakify-smart-contract/tree/main/contracts/campaign-factory) | Handle the information related to pairs |
| [`campaign`](https://github.com/buzz-space/stakify-smart-contract/tree/main/contracts/campaign)                 | Containing a pair of assets             |

## Running these contracts

You will need Rust 1.66.0+ with wasm32-unknown-unknown target installed.

### Build the contract

The contracts can be compiled using [cargo](https://doc.rust-lang.org/cargo/commands/cargo-build.html)

```
cargo build
```

with the optimizer is

```toml
optimizer_version = '0.12.11'
```

Build .wasm file stored in `target/wasm32-unknown-unknown/release/<CONTRACT_NAME>.wasm`
`--no-wasm-opt` is suitable for development, explained below

### Testing the contract

To run the tests for the contract, run the following command:

```bash
RUST_BACKTRACE=1 cargo unit-test
```

This will build the contract and run a series of tests to ensure that it functions correctly. The tests are defined in the ./tests directory.
