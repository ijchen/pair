extern crate pair;

use pair::{Owner, Pair};

struct MyOwner<'a>(&'a str);

impl Owner<'_> for MyOwner<'_> {
    type Dependent = ();
}

fn main() {
    let my_string: String = String::from("hello");
    let _pair: Pair<MyOwner<'_>> = Pair::new(MyOwner(&my_string), |_| ());
    drop(my_string);
}
