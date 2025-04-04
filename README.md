# Data Fuel

Houses a blockchain syncing engine & implementations of the LinkedList data-structure

## Sync Engine

A Tokio-powered syncing engine designed to parallel-fetch blockchain data from rate-limited distributed services with the most performance possible. To view its high-level architecture analysis, please see `README.md` in its directory [here](https://github.com/Jurshsmith/data-fuel/blob/main/sync-engine/README.md).

### Running

To run playgorund, run:

```sh
  cargo run -p sync-engine
```

N/B: Runs a long-running and computationally intensive process. To remove synchronous work simulation, comment out this line: [SyncEngine:L58](https://github.com/Jurshsmith/data-fuel/blob/d3f8fb736e63437cba8f2b5fc1b727b3ec278aff/sync-engine/src/lib.rs#L58)

For tests, run:

```sh
  cargo test -p sync-engine
```

## Linked Lists

Implementations of the `SinglyLinkedList` and `DoublyLinkedList` using `unsafe` Rust. `data-fuel-benches` contains benchmarks to compare both structures. Please view its directory's `README.md` [here](https://github.com/Jurshsmith/data-fuel/blob/main/linked-lists/README.md) for some analysis on their performances.

### Running

To run benchmarks, run:

```sh
  cargo bench -p data-fuel-benches
```

Equally, to run tests, run:

```sh
  cargo test -p linked-lists
```
