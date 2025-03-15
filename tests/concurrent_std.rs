#![allow(missing_docs, reason = "integration test")]

use std::sync::{Arc, Mutex, RwLock, mpsc};

use pair::{Dependent, HasDependent, Owner, Pair};

// A simple owner type for testing thread safety
#[derive(Debug)]
struct Buff(String);

// A dependent type that borrows from Buff
#[derive(Debug)]
struct Parsed<'a> {
    tokens: Vec<&'a str>,
}

fn parse(buffer: &Buff) -> Parsed<'_> {
    let tokens = buffer.0.split_whitespace().collect();
    Parsed { tokens }
}

impl<'owner> HasDependent<'owner> for Buff {
    type Dependent = Parsed<'owner>;
}

impl Owner for Buff {
    type Context<'a> = ();
    type Error = std::convert::Infallible;

    fn make_dependent(&self, (): Self::Context<'_>) -> Result<Dependent<'_, Self>, Self::Error> {
        Ok(parse(self))
    }
}

// Test sending ownership of a Pair between threads through channels
#[test]
fn pair_ownership_transfer() {
    let (tx, rx) = mpsc::channel();

    let pair = Pair::new(Buff(String::from("this is a test")));

    let t1 = std::thread::spawn(move || {
        assert_eq!(pair.owner().0, "this is a test");
        assert_eq!(
            pair.with_dependent(|parsed| parsed).tokens,
            ["this", "is", "a", "test"]
        );
        tx.send(pair).unwrap();
    });

    let t2 = std::thread::spawn(move || {
        let received_pair = rx.recv().unwrap();

        assert_eq!(received_pair.owner().0, "this is a test");
        assert_eq!(
            received_pair.with_dependent(|parsed| parsed).tokens,
            ["this", "is", "a", "test"]
        );

        received_pair
    });

    t1.join().unwrap();
    let received_pair = t2.join().unwrap();

    assert_eq!(received_pair.owner().0, "this is a test");
    assert_eq!(
        received_pair.with_dependent(|parsed| parsed).tokens,
        ["this", "is", "a", "test"]
    );
}

// Test sending and sharing a Pair via Arc<Pair<_>> across multiple threads
#[test]
fn pair_arc_sharing() {
    let pair = Arc::new(Pair::new(Buff(String::from("arc sharing test"))));

    let pair1 = Arc::clone(&pair);
    let t1 = std::thread::spawn(move || {
        assert_eq!(pair1.owner().0, "arc sharing test");
        assert_eq!(
            pair1.with_dependent(|parsed| parsed).tokens,
            ["arc", "sharing", "test"]
        );
    });

    let pair2 = Arc::clone(&pair);
    let t2 = std::thread::spawn(move || {
        assert_eq!(pair2.owner().0, "arc sharing test");
        assert_eq!(
            pair2.with_dependent(|parsed| parsed).tokens,
            ["arc", "sharing", "test"]
        );
    });

    t1.join().unwrap();
    t2.join().unwrap();
}

// Test concurrently accessing an Arc<Mutex<Pair>> from multiple threads
#[test]
fn pair_mutex_concurrent_access() {
    let pair = Arc::new(Mutex::new(Pair::new(Buff(String::from(
        "mutex concurrent test",
    )))));

    let join_handles: Vec<_> = (0..4)
        .map(|_| {
            let pair_clone = Arc::clone(&pair);
            std::thread::spawn(move || {
                let mut pair = pair_clone.lock().unwrap();
                assert_eq!(
                    pair.with_dependent(|parsed| parsed).tokens[..3],
                    ["mutex", "concurrent", "test"]
                );
                pair.with_dependent_mut(|parsed| parsed.tokens.push("modified"));
            })
        })
        .collect();

    for join_handle in join_handles {
        join_handle.join().unwrap();
    }

    let pair = Arc::into_inner(pair).unwrap().into_inner().unwrap();
    pair.with_dependent(|parsed| {
        assert_eq!(
            parsed.tokens,
            [
                "mutex",
                "concurrent",
                "test",
                "modified",
                "modified",
                "modified",
                "modified",
            ]
        );
    });
}

// Test concurrently accessing an Arc<RwLock<Pair>> from multiple threads
#[test]
fn pair_rwlock_concurrent_access() {
    let pair = Arc::new(RwLock::new(Pair::new(Buff(String::from(
        "rwlock concurrent test",
    )))));

    let mut join_handles: Vec<_> = Vec::new();

    // Reader threads
    for _ in 0..2 {
        let pair_clone = Arc::clone(&pair);
        let join_handle = std::thread::spawn(move || {
            let pair = pair_clone.read().unwrap();
            assert_eq!(
                pair.with_dependent(|parsed| parsed).tokens[..3],
                ["rwlock", "concurrent", "test"]
            );
        });

        join_handles.push(join_handle);
    }

    // Writer threads
    for _ in 0..2 {
        let pair_clone = Arc::clone(&pair);
        let join_handle = std::thread::spawn(move || {
            let pair = pair_clone.read().unwrap();
            assert_eq!(
                pair.with_dependent(|parsed| parsed).tokens[..3],
                ["rwlock", "concurrent", "test"]
            );
            drop(pair);

            let mut pair = pair_clone.write().unwrap();
            pair.with_dependent_mut(|parsed| parsed.tokens.push("modified"));
        });

        join_handles.push(join_handle);
    }

    for join_handle in join_handles {
        join_handle.join().unwrap();
    }

    let pair = Arc::into_inner(pair).unwrap().into_inner().unwrap();
    pair.with_dependent(|parsed| {
        assert_eq!(
            parsed.tokens,
            ["rwlock", "concurrent", "test", "modified", "modified"]
        );
    });
}
