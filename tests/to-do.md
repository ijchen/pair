# Test to-do list
- Concurrency
  - Sending ownership of a Pair between threads through channels
  - Sharing a Pair via &Pair<_> across multiple threads
  - Sending and sharing a Pair via Arc<Pair<_>> across multiple threads
  - Wrapping a Pair in Arc<Mutex/RwLock> and concurrently racily accessing from
    multiple threads
- Run with asan, threadsan, loom, and some kind of memory leak detector
