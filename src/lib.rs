mod convenience;
mod owner;
mod pair;
mod ref_owner;

pub use convenience::{AsRefPair, BorrowPair, DerefPair};
pub use owner::{HasDependent, Owner};
pub use pair::Pair;

// TODO: *extensive* testing, including:
// - Property-based testing
// - Fuzzing (possible? I kinda want "type fuzzing" which seems... hard)
// - Test against weird cases like contravariant types, "Oisann" types, weird
// Drop impls, impure Deref impls, etc.
// - https://docs.rs/trybuild test cases demonstrating that misuses of your API don't compile
// - All under MIRI

// impl<'any> ForLt<'any> for String {
//     type Dependent = &'any str;
// }
// impl Owner for String {
//     fn make_dependent(&self) -> <Self as ForLt<'_>>::Dependent {
//         self
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn my_test() {
//         let m1 = "<COMMENTED OUT>";
//         let thing = Pair::new_deref("Hi".to_string());
//         let d1 = thing.get_dependent();
//         let o1 = thing.get_owner_deref();
//         let d2 = thing.get_dependent();
//         let d3 = thing.get_dependent();
//         println!("{d3}{m1}{d2}{o1}{d1}");
//         let s: String = thing.into_owner_deref();
//         drop(s);

//         let thing = Pair::new_deref(vec![1, 2, 3, 4]);
//         println!("{:?}", thing.get_dependent());

//         // panic!();
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox() {
        let thing = DerefPair::new("Hi".to_string());
        let d1 = thing.get_dependent();
        let o1 = thing.get_owner();
        let d2 = thing.with_dependent(|dep| dep.to_string());
        let d3 = thing.get_dependent();
        println!("{d3}{d2}{o1}{d1}");
        let s: String = thing.into_owner();
        drop(s);

        // let thing = DerefPair::new(vec![1, 2, 3, 4]);
        // println!("{:?}", thing.get_dependent());

        // struct Foo;
        // struct Bar;
        // impl Drop for Foo {
        //     fn drop(&mut self) {
        //         println!("Dropping Foo");
        //     }
        // }
        // impl Drop for Bar {
        //     fn drop(&mut self) {
        //         println!("Dropping Bar");
        //         panic!();
        //     }
        // }

        // impl Owner for Foo {
        //     type Dependent<'a>
        //         = Bar
        //     where
        //         Self: 'a;

        //     fn make_dependent(&self) -> Self::Dependent<'_> {
        //         Bar
        //     }
        // }

        // let pair = Pair::new(Foo);
        // pair.into_owner();

        panic!();
    }
}
