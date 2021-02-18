/*
    A basic test of the state_machine module.
*/

use data_transducers::ext_value::Ext;
use data_transducers::state_machine::Transition;

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let state1 = Rc::new(RefCell::new(Ext::One(2)));
    let state2 = Rc::new(RefCell::new(Ext::One("init".to_owned())));
    let t = Transition {
        source: state1.clone(),
        target: state2.clone(),
        f: |ch, _x| {
            if ch == 'a' {
                "a".to_owned()
            } else {
                "-".to_owned()
            }
        },
    };
    t.update('c');
    t.update('b');
    t.update('a');
    t.update('b');

    // This is working! Just forgot to use Rc before
    assert_eq!(*state1.borrow(), Ext::One(2));
    assert_eq!(*state2.borrow(), Ext::One("-".to_owned()));
    println!("New value of states: {:?} {:?}", state1, state2);

    let mut states: Vec<Box<dyn Any>> = Vec::new();
    states.push(Box::new(state1));
    states.push(Box::new(state2));
    println!("States: {:?}", states);
}
