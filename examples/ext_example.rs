/*
    Basic example illustrating Ext<T>
*/

use data_transducers::ext_value::Ext;

fn main() {
    println!("=== Ext<T> Example ===");
    let x0: Ext<i32> = Ext::None;
    let x1 = Ext::One(3);
    let x2: Ext<i32> = Ext::Many;
    println!("Possible Ext Values: {}, {}, {}", x0, x1, x2);
}
