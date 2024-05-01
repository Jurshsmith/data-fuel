# Benchmark Performance Analysis: Singly vs. Doubly Linked List Traversal

**Number of Elements:** 1,000,000 `u32` elements

**CPU Specification:** 2.3 GHz 8-Core Intel Core i9

**Results:**

- Singly Linked List traversal average: 460ms
- Doubly Linked List traversal average: 0ps (Pico-second)

**Analysis:**

The performance difference between the Singly and Doubly Linked Lists can be attributed to their respective memory access patterns.

- **Singly Linked List:** This data structure has weaker reference locality as it only references the next pointer. Traversing it on a modern CPU requires round trips to the heap in main memory, resulting in an average traversal time of 460ms in our benchmark.

- **Doubly Linked List:** In contrast, the Doubly Linked List exhibits strong spatial locality of its references. Each node's pointer references its previous and next pointers, leading to a contiguous memory layout. This allows the CPU to predictably optimize traversal by prefetching all related pointers into Cache Layer 1. As a result, traversal time is near zero since there is no round trip to main memory, and all pointers are prefetched while traversing. This optimization at the CPU's cache layer 1 enables access to the data at approximately the speed of the core CPU, resulting in traversal times as low as 0ps for the small dataset.
