use std::{borrow::Cow, convert::Infallible, fmt::Debug};

use pair::{HasDependent, Owner};

mod real {
    pub use pair::Pair;
}

mod fake {
    use std::fmt::Debug;

    use pair::{HasDependent, Owner};

    #[derive(Debug)]
    pub struct Pair<'a, O: Owner>
    where
        <O as HasDependent<'a>>::Dependent: Debug,
    {
        #[expect(dead_code, reason = "we actually care about the derived Debug")]
        pub owner: O,
        #[expect(dead_code, reason = "we actually care about the derived Debug")]
        pub dependent: <O as HasDependent<'a>>::Dependent,
    }
}

fn debugs_match<O: for<'any> Owner<Context<'any> = (), Err = Infallible> + Clone + Debug>(owner: O)
where
    for<'any> <O as HasDependent<'any>>::Dependent: Debug,
{
    let Ok(dependent) = owner.make_dependent(());
    let pair_real = real::Pair::new(owner.clone());
    let pair_fake = fake::Pair {
        owner: owner.clone(),
        dependent,
    };

    // Normal debug (and pretty-print)
    assert_eq!(format!("{pair_real:?}"), format!("{pair_fake:?}"));
    assert_eq!(format!("{pair_real:#?}"), format!("{pair_fake:#?}"));

    // Hex integers (lowercase and uppercase)
    // See: https://doc.rust-lang.org/std/fmt/index.html#formatting-traits
    assert_eq!(format!("{pair_real:x?}"), format!("{pair_fake:x?}"));
    assert_eq!(format!("{pair_real:X?}"), format!("{pair_fake:X?}"));
    assert_eq!(format!("{pair_real:#x?}"), format!("{pair_fake:#x?}"));
    assert_eq!(format!("{pair_real:#X?}"), format!("{pair_fake:#X?}"));

    // Getting crazy with it (I'm not gonna test every combination, but I'm down
    // to just throw a bunch of random stuff at it and make sure that works out)
    //
    // The "🦀^+#12.5?" means: ferris fill, center aligned, with sign, pretty
    // printed, no "0" option integer formatting (would override fill/align),
    // width 12, 5 digits of precision, debug formatted.
    //
    // See: https://doc.rust-lang.org/std/fmt/index.html#formatting-parameters
    assert_eq!(
        format!("{pair_real:🦀^+#12.5?}"),
        format!("{pair_fake:🦀^+#12.5?}")
    );
    assert_eq!(
        format!("{pair_real:🦀^+#12.5?}"),
        format!("{pair_fake:🦀^+#12.5?}")
    );
}

macro_rules! debug_tests {
    (
        $struct_or_enum:ident $name:ident $decl_body:tt $(; $semicolon_exists:vis)?
        & $lt:lifetime $self_kw:ident => $dep_ty:ty : $dep_expr:expr ;

        $($owner:expr),+$(,)?
    ) => {
        #[derive(Debug, Clone)]
        $struct_or_enum $name $decl_body $(; $semicolon_exists)?

        impl<$lt> HasDependent<$lt> for $name {
            type Dependent = $dep_ty;
        }
        impl Owner for $name {
            type Context<'a> = ();
            type Err = Infallible;
            fn make_dependent(
                &$self_kw,
                (): Self::Context<'_>,
            ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Err> {
                Ok($dep_expr)
            }
        }
        $(
            debugs_match($owner);
        )+
    };
}

#[test]
fn debug_impls_match_derive_nomiri() {
    debug_tests! {
        struct O1(String);
        &'owner self => &'owner str: &self.0;

        O1(String::new()),
        O1(String::from("Hello, world!")),
    }

    debug_tests! {
        struct O2(i32);
        &'owner self => &'owner i32: &self.0;

        O2(0),
        O2(i32::MAX),
    }

    debug_tests! {
        struct O3(char);
        &'owner self => Option<Vec<()>>:
            Some(std::iter::repeat(()).take(self.0 as usize % 20).collect());

        O3('🦀'), O3(' '), O3('!'), O3('a'), O3('A'), O3('*'), O3('-'), O3('~'),
        O3('\\'), O3('"'), O3('\x00'), O3('\n'), O3('\t'), O3('\''), O3('&'),
    }

    debug_tests! {
        struct O4(f64);
        &'owner self => u8:
            self.0
                .to_be_bytes()
                .iter()
                .copied()
                .reduce(|acc, e| u8::wrapping_add(acc, e))
                .unwrap();

        O4(0.0), O4(-0.0), O4(1.0), O4(3.14), O4(f64::NAN), O4(f64::INFINITY),
        O4(f64::NEG_INFINITY), O4(f64::EPSILON),
    }

    debug_tests! {
        struct O5([isize; 42]);
        &'owner self => &'owner [isize]: &self.0[5..24];

        O5([0; 42]), O5([69; 42]),
    }

    debug_tests! {
        struct O6(Vec<u8>, [char; 6], Option<()>);
        &'owner self => (&'owner char, Cow<'owner, str>, Option<&'owner ()>):
            (&self.1[1], String::from_utf8_lossy(&self.0), self.2.as_ref());

        O6(b"Foo bar bat baz".to_vec(), ['f', 'o', 'o', 'b', 'a', 'r'], Some(())),
        O6(b"My friend, I wish that I could say that I agree".to_vec(), ['g', 't', 'w', 'w', 'o', 'a'], None),
    }

    debug_tests! {
        enum O7 {
            Foo,
            Bar(u8),
            Bat(String),
            Baz {
                name: String,
                age: u8,
            },
        }
        &'owner self => Option<Cow<'owner, str>>: match self {
            O7::Foo => None,
            O7::Bar(n) => Some(Cow::Owned(format!("{n}"))),
            O7::Bat(s) => Some(Cow::Borrowed(s)),
            O7::Baz { name, age } => Some(Cow::Owned(format!("{name} is {age}"))),
        };

        O7::Foo, O7::Bar(0), O7::Bar(1), O7::Bar(42), O7::Bar(69), O7::Bar(u8::MAX),
        O7::Bat(String::from("testing")), O7::Baz { name: String::from("Hermes"), age: u8::MAX },
    }
}
