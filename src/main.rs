/*
Top-level module and entrypoint for data transducers project.
*/

mod ext_value;

use ext_value::ExtValue;

// use std::cell::RefCell;
// use std::rc::{Rc, Weak};
// use std::fmt::Display;



// 
// struct State {
//     pub val: ExtValue,    
// }

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

    // let mut three : i32 = 3;
    let ev0 : ExtValue<i32> = ExtValue::None;
    let ev1 = ExtValue::One(3);
    let ev2 : ExtValue<i32> = ExtValue::Many;

    println!("{}, {}, {}", ev0, ev1, ev2);

    // let mut ev3 : ExtValue<i32> = ExtValue::None;
    // let ev4 : ExtValue<i64> = ExtValue::None;
    // let ev5 : ExtValue<i32> = ExtValue::One(4);
    // 
    // 
    // assert!(ev1 != ev2);
    // assert!(ev0 == ev3);
    // 
    // println!("{}", ev1 + ev5);

    // assert!(ev3 == ev4);

    // let mut s1 = State::new(3);
    // let mut s2 = State::new(2);
    // let mut s2 = State::new(None);
}
