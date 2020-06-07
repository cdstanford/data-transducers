/*
Top-level module and entrypoint for data transducers project.
*/

mod ext_value;

use ext_value::Ext;

// use std::cell::RefCell;
// use std::rc::{Rc, Weak};
// use std::fmt::Display;

struct State<T> {
    pub val: Ext<T>,
}

// struct Edge1<T1, T2> {
//     // An edge from int to int
//     pub fn 
// }
// 
// struct Edge2 {
// 
// }

fn main() {
    println!("=== Data Transducers Library ===");

    // let mut s1 = State::new(3);
    // let mut s2 = State::new(2);
    // let mut s2 = State::new(None);
}
