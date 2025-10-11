extern crate pair;

use pair::{Owner, Pair};

#[derive(Debug)]
struct Buff(String);

impl<'owner> Owner<'owner> for Buff {
    type Dependent = Vec<&'owner str>;
}

fn parse(buff: &Buff) -> Vec<&str> {
    buff.0.split_whitespace().collect()
}

fn main() {
    let pair = Pair::new(Buff(String::from("This is a test of pair.")), parse);
    let dep: &Vec<&str> = pair.with_dependent(|dep| dep);

    drop(pair);

    let _ = dep;
}
