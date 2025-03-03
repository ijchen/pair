use std::convert::Infallible;

use pair::{HasDependent, Owner, Pair};

#[derive(Debug)]
struct BuffFallible(String);

impl<'owner> HasDependent<'owner> for BuffFallible {
    type Dependent = Vec<&'owner str>;
}

impl Owner for BuffFallible {
    type Context<'a> = ();
    type Error = String;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
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
    let pair = Pair::try_new(BuffFallible(String::from("This is a test of pair."))).unwrap();
    assert_eq!(pair.get_owner().0, "This is a test of pair.");

    let (buff, err) = Pair::try_new(BuffFallible(String::from("     "))).unwrap_err();
    assert_eq!(buff.0, "     ");
    assert_eq!(err, "Conversion failed");

    let pair = Pair::try_new_from_box(Box::new(BuffFallible(String::from(
        "This is a test of pair.",
    ))))
    .unwrap();
    assert_eq!(pair.get_owner().0, "This is a test of pair.");

    let (buff, err) =
        Pair::try_new_from_box(Box::new(BuffFallible(String::from("     ")))).unwrap_err();
    assert_eq!(buff.0, "     ");
    assert_eq!(err, "Conversion failed");
}

#[derive(Debug)]
struct BuffWithContext(String);

impl<'owner> HasDependent<'owner> for BuffWithContext {
    type Dependent = Vec<&'owner str>;
}

impl Owner for BuffWithContext {
    type Context<'a> = &'a str;
    type Error = Infallible;

    fn make_dependent(
        &self,
        context: Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(self.0.split(context).collect())
    }
}

#[test]
fn with_context() {
    let pair = Pair::new_with_context(BuffWithContext(String::from("foo, bar, bat, baz")), ", ");
    assert_eq!(pair.get_owner().0, "foo, bar, bat, baz");

    let pair = Pair::new_from_box_with_context(
        Box::new(BuffWithContext(String::from("foo, bar, bat, baz"))),
        ", ",
    );
    assert_eq!(pair.get_owner().0, "foo, bar, bat, baz");
}

#[derive(Debug)]
struct BuffFallibleWithContext(String);

impl<'owner> HasDependent<'owner> for BuffFallibleWithContext {
    type Dependent = Vec<&'owner str>;
}

impl Owner for BuffFallibleWithContext {
    type Context<'a> = &'a str;
    type Error = String;

    fn make_dependent(
        &self,
        context: Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
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
    let pair = Pair::try_new_with_context(
        BuffFallibleWithContext(String::from("foo, bar, bat, baz")),
        ", ",
    )
    .unwrap();
    assert_eq!(pair.get_owner().0, "foo, bar, bat, baz");

    let (buff, err) = Pair::try_new_with_context(
        BuffFallibleWithContext(String::from("This is a test of pair.")),
        ", ",
    )
    .unwrap_err();
    assert_eq!(buff.0, "This is a test of pair.");
    assert_eq!(
        err,
        "Conversion of string 'This is a test of pair.' with context ', ' failed."
    );

    let pair = Pair::try_new_from_box_with_context(
        Box::new(BuffFallibleWithContext(String::from("foo, bar, bat, baz"))),
        ", ",
    )
    .unwrap();
    assert_eq!(pair.get_owner().0, "foo, bar, bat, baz");

    let (buff, err) = Pair::try_new_from_box_with_context(
        Box::new(BuffFallibleWithContext(String::from(
            "This is a test of pair.",
        ))),
        ", ",
    )
    .unwrap_err();
    assert_eq!(buff.0, "This is a test of pair.");
    assert_eq!(
        err,
        "Conversion of string 'This is a test of pair.' with context ', ' failed."
    );
}
