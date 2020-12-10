/*
    Module implementing the core state machine data structure
    for data transducers.
*/

// use super::ext_value::Ext;

/*
    To support arbitrary types inside the machine, we define a Transition
    type that has a (fixed) number of source states and a target state.
*/

// TODO

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
