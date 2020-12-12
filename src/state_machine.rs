/*
    Module implementing the core state machine data structure
    for data transducers.
*/

use super::ext_value::{apply1, Ext};

use std::cell::RefCell;
use std::fmt::{Debug, Display};

/*
    To support arbitrary types inside the machine, we define abstract State and
    Transition traits. These will be instantiated with states of a particular
    type and transitions of varying arities, which each have some number of
    input states and a single output state.
*/

// trait State: Clone + Debug {
//
// }

pub struct Transition<T1, T2, F> {
    pub source: RefCell<Ext<T1>>,
    pub target: RefCell<Ext<T2>>,
    pub f: F,
}
impl<T1, T2, F> Transition<T1, T2, F>
where
    T1: Clone + Debug + Display,
    T2: Clone + Debug + Display,
    F: Fn(char, T1) -> T2,
{
    pub fn update(&self, ch: char) {
        *self.target.borrow_mut() =
            apply1(|x| (self.f)(ch, x), self.source.borrow().clone());
        // if ch == 'a' {
        //     *self.target.borrow_mut() = self.source.borrow().clone();
        // } else if ch == 'b' {
        //     *self.target.borrow_mut() = self.source.borrow().clone();
        // }
        println!("New target: {:?}", self.target.borrow());
    }
}

/*
    OLD
    States and Transitions are implemented as traits.
    Both states and transitions are objects which have
    an Ext value and can be initialized, read, updated,
    and reset.

    add: add a new current value
    get: get the current value
    reset: reset the current value
    update: update the current value

    get_prev: get the previous value
    set_prev: update the previous value to the current one
*/
// pub trait Statelike {
//     type Val;
//
//     fn add(&self, _: &Ext<&Self::Val>);
//     fn get(&self) -> Ext<&Self::Val>;
//     fn reset(&self);
//     fn update(&self);
//
//     fn get_prev(&self) -> Ext<&Self::Val>;
//     fn set_prev(&self);
// }

/*
    A state machine (data transducer) is then constructed
    from States and Transitions, whih both implement the
    Statelike trait.
    - UnionState: the value of the state is the union of
      TODO
    - TODO
*/

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
