// use std::cell::RefCell;
// use std::rc::{Rc, Weak};
// use std::fmt::Display;
// #[macro_use] extern crate custom_derive;
// #[macro_use] extern crate enum_derive;
extern crate derive_more;
// use derive_more::{Add, Display, From, Into};
use derive_more::{Display, From};
use std::ops::Add;

#[derive(Debug, PartialEq, Display, From)]
enum ExtValue<T> {
    None,
    One(T),
    Many,
}

impl<T> Add for ExtValue<T> {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        match self {
            ExtValue::None => other,
            ExtValue::One(_x) =>
                match other {
                    ExtValue::None => self,
                    ExtValue::One(_y) => ExtValue::Many,
                    ExtValue::Many => ExtValue::Many,
                },
            ExtValue::Many => ExtValue::Many,
        }
    }
}

// impl<T:Display> Display for ExtValue<T> {
//     fn fmt(&self, w: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
//         write!(w, "[")?;
//         let mut node = self.first.clone();
//         while let Some(n) = node {
//             write!(w, "{}", n.borrow().data)?;
//             node = n.borrow().next.clone();
//             if node.is_some() {
//                 write!(w, ", ")?;
//             }
//         }
//         write!(w, "]")
//     }
// }

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
