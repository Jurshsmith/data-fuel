# Data Fuel

Houses a blockchain syncing engine & implementations of the LinkedList data-structure

## Sync Engine

A Tokio-powered syncing engine designed to parallel-fetch blockchain data from rate-limited remote services with the most performance possible. To learn more about its high-level architecture, please read `README.md` in its context.

### Running

To run playgorund, run:

```sh
  cargo run -p sync-engine
```

Equally, to run tests:

```sh
  cargo tests -p sync-engine
```

## Linked Lists

An implementation of the `SinglyLinkedList` and `DoublyLinkedList` using `unsafe` Rust. `data-fuel-benches` contains benchmarks to compare both structures. Please read its context's `README.md` to view some performance analysis on their comparisons.

### Running

To run benchmarks, run:

```sh
  cargo bench -p data-fuel-benches
```

Equally, to run tests, run:

```sh
  cargo run -p linked-lists
```
