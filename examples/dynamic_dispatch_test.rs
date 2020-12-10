/*
    A basic test of how dynamic dispatch works in Rust.
*/

use std::fmt::Debug;

trait DynamicType: Debug {
    fn simple_func(&self) -> usize;
    fn noop(&self);
    // Note: dynamic typing is quite restrictive. None
    // of the following work.
    // fn get_val(&self) -> &Self {
    //     self
    // }
    // fn set_val(&mut self, val: &Self) {
    //     *self = val.clone();
    // }
}

impl DynamicType for String {
    fn simple_func(&self) -> usize {
        0
    }
    fn noop(&self) {}
}

impl DynamicType for usize {
    fn simple_func(&self) -> usize {
        1
    }
    fn noop(&self) {}
}

fn main() {
    let mut v1: Vec<Box<dyn DynamicType>> = Vec::new();
    let mut v2: Vec<Box<dyn DynamicType>> = Vec::new();
    v1.push(Box::new("Hello 1".to_owned()));
    v1.push(Box::new(100));
    v1.push(Box::new("World 1".to_owned()));
    v2.push(Box::new("Hello 2".to_owned()));
    v2.push(Box::new(200));
    v2.push(Box::new("World 2".to_owned()));
    // Printing
    println!("v1: {:?}", v1);
    println!("v2: {:?}", v2);
    // Calling functions
    let mut simple_results1 = Vec::new();
    for x1 in v1 {
        x1.noop();
        simple_results1.push(x1.simple_func());
    }
    let mut simple_results2 = Vec::new();
    for x2 in v2 {
        x2.noop();
        simple_results2.push(x2.simple_func());
    }
    println!("simple results 1: {:?}", simple_results1);
    println!("simple results 2: {:?}", simple_results2);
}
