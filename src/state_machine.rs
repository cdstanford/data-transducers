/*
Module implementing the core state machine data structure
for data transducers, with core constructors and operations.
*/

use super::ext_value::Ext;

/*
    Trait for states: objects which have an Ext value and
    can be initialized, read, updated, and reset.

    add: add a new current value
    get: get the current value
    reset: reset the current value
    update: update the current value

    get_prev: get the previous value
    set_prev: update the previous value to the current one
*/
pub trait State {
    type Val;

    // fn add(&self, &Self::Val, i32) -> ();
    fn add(&self, _: &Ext<&Self::Val>) -> ();
    fn get(&self) -> Ext<&Self::Val>;
    fn reset(&self) -> ();
    fn update(&self) -> ();

    fn get_prev(&self) -> Ext<&Self::Val>;
    fn set_prev(&self) -> ();
}

// States and transitions are objects which have a value

// A state is a data node which also contains the set of edges
// which target it: this is going to be given by a dictionary
// which takes a value and returns a closure which
// computes the new value of the state from some refererences
// to some other states.
// struct State<T> {
//     pub val: Ext<T>,
//     pub sources:
// }
