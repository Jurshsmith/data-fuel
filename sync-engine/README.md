# Sync Engine

## Problem Definition

The task is to sync with the network. It means fetch and valdiate all [`Block`]s for the range `0..10_000_000`, but:

- Before the requesting transactions, we need to call [`BlockHeader::verify`]
  and fetch transactions only if it returns `true`.
- Before combining [`BlockHeader`] and [`Vec<Transaction>`] into [`Block`], we need to
  iterate over each transaction and call [`Transaction::execute`]. If all results are [`Result::Ok`],
  then we can create a [`Block`].

The goal of the task is to request data as fast as possible(in parallel). Blocks can be executed
and verified independently. It means verification or execution of the block
at height `X` can be done without block at height `X - 1` (Order-agnostic)

Further constraints: As documented in [ServerAPI](https://github.com/Jurshsmith/data-fuel/blob/7c33e2fffa4c9739b1e0ba05b884b3d4f90ea1e2/sync-engine/src/lib.rs#L88), each service for fetching [`Block`] partial data has a rate limit of `100` requests/second, `10_000` range per request. Equally, [[`Transactions::execute`](https://github.com/Jurshsmith/data-fuel/blob/7c33e2fffa4c9739b1e0ba05b884b3d4f90ea1e2/sync-engine/src/lib.rs#L56)], is blocking and can take up to 1 second.

N/B: All `BlockHeader`, `Block` & `Transaction` related definitions can be found in entry [lib](https://github.com/Jurshsmith/data-fuel/blob/7c33e2fffa4c9739b1e0ba05b884b3d4f90ea1e2/sync-engine/src/lib.rs#L14).

### Assumptions

- This task is the primary workload on a system with sufficient resources, ensuring that green threads will not be exhausted.
- The system can create blocks in memory efficiently, allowing for one-block-at-a-time processing without the need for batching.
- The `ServerAPI` rate limits with an error margin to accommodate time monotonic errors. As long as we stay within the local rate limit, there should be no need for retries.
- The `ServerAPI` is considered reliable, eliminating the need for stateful retries.

## Architectural Analysis (High-level)

The defined constraints scream the need for a non-blocking I/O process to fetch `BlockHeader` and `Transactions` in parallel (order-agnostic), along with dedicated threads to handle the blocking `Transaction::Execute` task. Additionally, the `ServerAPI` can handle a constant flow of 1 million requests per second -- our I/O's maximum throughput. The blocking tasks will be processed in parallel across multiple CPU cores. For this task, `Tokio` is chosen as the main async runtime, and `Rayon` is used for processing the blocking tasks in parallel.

An ergonomic approach to building the constant flow async pipeline involves using `Streams`, leveraging their various APIs, particularly `BufferUnordered` for order-agnostic processing. For our use-case, we could use `buffer_unordered(100)` to batch request `10_000` blocks per second. However, due to their declarative nature, it can be cumbersome to solve the pipe friction situation where `1` batch slice has to wait for others before re-fetching the others. Additionally, streams are processed concurrently on the same thread by default (not parallel), and would need a `ThreadManaging` API like Tokio's `spawn` to fully leverage a Multi-threaded environment.

The `SyncEngine` uses a `Master-Worker` architecture. On startup, it creates a `WorkerPool` of `100` workers, with each worker tasked to fetch the `Block` partials and offload the blocking-composition operation to a `Rayon` dedicated thread. Each `Worker` is limited to `1` request per second and continously re-fetches without waiting for other `Worker`s, enabling non-blocking I/O requests and leveraging the full processing speed of the system's CPU.
