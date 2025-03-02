# Test to-do list
- Interior mutability
  - Test with interior-mutable owner
  - Test with interior-mutable dependent
  - Test with interior-mutable for both owner and dependent
- Trybuild tests
  - Attempts to extract the dependent outside `with_dependent_mut`
  - Verify dependent cannot outlive the pair. This means writing code that
    attempts to extract and store a reference to the dependent that would
    outlive the Pair itself
  - Test that the compiler prevents use of borrowed data after into_owner()
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
