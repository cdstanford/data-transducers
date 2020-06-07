/*
Module implementing the core state machine data structure
for data transducers, with core constructors and operations.
*/

use super::ext_value::Ext;

// A state is a data node which also contains the set of edges
// which target it: this is going to be given by a dictionary
// which takes a value and returns a closure which
// computes the new value of the state from some refererences
// to some other states.
struct State<T> {
    pub val: Ext<T>,
}
