# Test to-do list
- Concurrency
  - Sending ownership of a Pair between threads through channels
  - Sharing a Pair via &Pair<_> across multiple threads
  - Sending and sharing a Pair via Arc<Pair<_>> across multiple threads
  - Wrapping a Pair in Arc<Mutex/RwLock> and concurrently racily accessing from
    multiple threads
  - Verify pair is !Send when owner is !Send
  - Verify pair is !Send when dependent is !Send
  - Verify pair is !Sync when owner is !Sync
  - Verify pair is !Sync when dependent is !Sync
- Run with asan, threadsan, loom, and some kind of memory leak detector
