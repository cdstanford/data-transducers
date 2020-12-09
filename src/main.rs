/*
    Top-level module and entrypoint for data transducers project.
*/

#![allow(dead_code)]

mod ext_value;
mod int_state_machine;
mod state_machine;

use ext_value::Ext;

fn main() {
    println!("=== Data Transducers Library ===");
    let x0: Ext<i32> = Ext::None;
    let x1 = Ext::One(3);
    let x2: Ext<i32> = Ext::Many;
    println!("Ext values: {}, {}, {}", x0, x1, x2);
}
