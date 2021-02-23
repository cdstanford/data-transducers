/*
    Module implementing data transducers as explicit state machines.

    Generally the QRE constructs should be more convenient and high-level,
    but this can be used if you want to manually write
    the states and transitions yourself.

    For simplicity and safety of the implementation, internal states in
    the machine are limited to be of all the same type Q. In order to
    achieve multiple arbitrary types in the machine, Q can be set
    to an enum, or an unsafe Union:
    https://doc.rust-lang.org/reference/items/unions.html
    or even an unsafe pointer.
    I originally wanted to support multiple state types in the implementation
    itself, but there is no easy way to deal with the complexity of types.
    Either the implementation would itself be inherently unsafe, or it
    would rely on a lot of dynamic manipulation of trait objects (something
    like Vec<Box<dyn Stateable>> for the states Vec<Box<dyn Transition>> for
    the transitions, but then it is challenging because the Transitions need
    to also keep reference-counted pointers into the states to get/update
    their values). Overall, fixing Q is cleaner design.
*/

use super::ext_value::{self, Ext};
use super::interface::Transducer;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::rc::Rc;

/*
    States are reference-counted refcells.
    This allows transitions to have direct shared pointers to their source
    and target states.
    Transitions are defined by a guard which says when they are active, and
    an action which says the function applied to the source states to
    give a new result for the target state.

    We still require one trait for Box<dyn Trait> objects, the Transition<Q>
    trait. This is because transitions are parameterized by function types.
*/

type State<Q> = Rc<RefCell<Ext<Q>>>;

pub struct Trans1<Q, D, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(&D, &Q) -> Q,
{
    source: State<Q>,
    target: State<Q>,
    guard: G,
    action: F,
    ph_d: PhantomData<D>,
}

pub struct Trans2<Q, D, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(&D, &Q, &Q) -> Q,
{
    source1: State<Q>,
    source2: State<Q>,
    target: State<Q>,
    guard: G,
    action: F,
    ph_d: PhantomData<D>,
}

trait Transition<Q> {}
impl<Q, D, G, F> Transition<Q> for Trans1<Q, D, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(&D, &Q) -> Q,
{
}
impl<Q, D, G, F> Transition<Q> for Trans2<Q, D, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(&D, &Q, &Q) -> Q,
{
}

/*
    The main DataTransducer state machine.
    Implements the Transducer interface.
*/

pub struct DataTransducer<I, Q, O, D> {
    istate: State<I>,
    fstate: State<O>,
    states: Vec<State<Q>>,
    transitions: Vec<Box<dyn Transition<Q>>>,
    ph_d: PhantomData<D>,
}
