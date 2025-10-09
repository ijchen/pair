#![allow(missing_docs, reason = "integration test")]

use pair::{Dependent, Owner, Pair};

#[derive(Debug)]
struct BuffFallible(String);

impl<'owner> Owner<'owner> for BuffFallible {
    type Dependent = Vec<&'owner str>;
}

impl BuffFallible {
    fn parse(&self) -> Result<Dependent<'_, Self>, String> {
        let parts: Vec<_> = self.0.split_whitespace().collect();

        if parts.is_empty() {
            Err(String::from("Conversion failed"))
        } else {
            Ok(parts)
        }
    }
}

#[test]
fn fallible() {
    let pair = Pair::try_new(
        BuffFallible(String::from("This is a test of pair.")),
        BuffFallible::parse,
    )
    .unwrap();
    assert_eq!(pair.owner().0, "This is a test of pair.");

    let (buff, err) =
        Pair::try_new(BuffFallible(String::from("     ")), BuffFallible::parse).unwrap_err();
    assert_eq!(buff.0, "     ");
    assert_eq!(err, "Conversion failed");

    let pair = Pair::try_new_from_box(
        Box::new(BuffFallible(String::from("This is a test of pair."))),
        BuffFallible::parse,
    )
    .unwrap();
    assert_eq!(pair.owner().0, "This is a test of pair.");

    let (buff, err) = Pair::try_new_from_box(
        Box::new(BuffFallible(String::from("     "))),
        BuffFallible::parse,
    )
    .unwrap_err();
    assert_eq!(buff.0, "     ");
    assert_eq!(err, "Conversion failed");
}

#[derive(Debug)]
struct BuffWithContext(String);

impl<'owner> Owner<'owner> for BuffWithContext {
    type Dependent = Vec<&'owner str>;
}

impl BuffWithContext {
    fn parse(&self, context: &str) -> Dependent<'_, Self> {
        self.0.split(context).collect()
    }
}

#[test]
fn with_context() {
    let pair = Pair::new(
        BuffWithContext(String::from("foo, bar, bat, baz")),
        |owner| BuffWithContext::parse(owner, ", "),
    );
    assert_eq!(pair.owner().0, "foo, bar, bat, baz");

    let pair = Pair::new_from_box(
        Box::new(BuffWithContext(String::from("foo, bar, bat, baz"))),
        |owner| BuffWithContext::parse(owner, ", "),
    );
    assert_eq!(pair.owner().0, "foo, bar, bat, baz");
}

#[derive(Debug)]
struct BuffFallibleWithContext(String);

impl<'owner> Owner<'owner> for BuffFallibleWithContext {
    type Dependent = Vec<&'owner str>;
}

impl BuffFallibleWithContext {
    fn parse(&self, context: &str) -> Result<Dependent<'_, Self>, String> {
        let parts: Vec<_> = self.0.split(context).collect();

        if parts.len() > 1 {
            Ok(parts)
        } else {
            Err(format!(
                "Conversion of string '{}' with context '{context}' failed.",
                self.0
            ))
        }
    }
}

#[test]
fn fallible_with_context() {
    let pair = Pair::try_new(
        BuffFallibleWithContext(String::from("foo, bar, bat, baz")),
        |owner| BuffFallibleWithContext::parse(owner, ", "),
    )
    .unwrap();
    assert_eq!(pair.owner().0, "foo, bar, bat, baz");

    let (buff, err) = Pair::try_new(
        BuffFallibleWithContext(String::from("This is a test of pair.")),
        |owner| BuffFallibleWithContext::parse(owner, ", "),
    )
    .unwrap_err();
    assert_eq!(buff.0, "This is a test of pair.");
    assert_eq!(
        err,
        "Conversion of string 'This is a test of pair.' with context ', ' failed."
    );

    let pair = Pair::try_new_from_box(
        Box::new(BuffFallibleWithContext(String::from("foo, bar, bat, baz"))),
        |owner| BuffFallibleWithContext::parse(owner, ", "),
    )
    .unwrap();
    assert_eq!(pair.owner().0, "foo, bar, bat, baz");

    let (buff, err) = Pair::try_new_from_box(
        Box::new(BuffFallibleWithContext(String::from(
            "This is a test of pair.",
        ))),
        |owner| BuffFallibleWithContext::parse(owner, ", "),
    )
    .unwrap_err();
    assert_eq!(buff.0, "This is a test of pair.");
    assert_eq!(
        err,
        "Conversion of string 'This is a test of pair.' with context ', ' failed."
    );
}
