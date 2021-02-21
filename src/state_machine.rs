/*
    Module implementing the core state machine data structure
    for data transducers.
*/

use super::ext_value::{self, Ext};

use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

/*
    To support arbitrary types inside the machine, we define both concrete
    State and Transition types as well as abstract State and
    Transition traits. These will be instantiated with states of a particular
    type and transitions of varying arities, which each have some number of
    input states and a single output state.
*/

type State<T> = Rc<RefCell<Ext<T>>>;

#[derive(Debug)]
pub struct Trans1<T1, T2, F1, F2> {
    pub source: State<T1>,
    pub target: State<T2>,
    pub active: F1,
    pub action: F2,
}
impl<T1, T2, F1, F2> Trans1<T1, T2, F1, F2>
where
    T1: Clone,
    T2: Clone + Debug,
    F1: Fn(char) -> bool, // whether the transition is active
    F2: Fn(char, T1) -> T2, // the effect of the transition
{
    pub fn update(&self, ch: char) {
        if (self.active)(ch) {
            *self.target.borrow_mut() = ext_value::apply1(
                |x| (self.action)(ch, x),
                self.source.borrow().clone(),
            );
            println!("New target: {:?}", self.target.borrow());
        }
    }
}

#[derive(Debug)]
pub struct Trans2<T1, T2, T3, F1, F2> {
    pub source1: State<T1>,
    pub source2: State<T2>,
    pub target: State<T3>,
    pub active: F1,
    pub action: F2,
}
impl<T1, T2, T3, F1, F2> Trans2<T1, T2, T3, F1, F2>
where
    T1: Clone,
    T2: Clone,
    T3: Clone + Debug,
    F1: Fn(char) -> bool, // whether the transition is active
    F2: Fn(char, T1, T2) -> T3, // the effect of the transition
{
    pub fn update(&self, ch: char) {
        if (self.active)(ch) {
            *self.target.borrow_mut() = ext_value::apply2(
                |x1, x2| (self.action)(ch, x1, x2),
                self.source1.borrow().clone(),
                self.source2.borrow().clone(),
            );
            println!("New target: {:?}", self.target.borrow());
        }
    }
}

pub trait AnyState: 'static + Debug {}
impl<T> AnyState for State<T> where T: 'static + Debug {}

pub trait AnyTrans: 'static + Debug {}
impl<T1, T2, F1, F2> AnyTrans for Trans1<T1, T2, F1, F2>
where
    T1: 'static + Debug,
    T2: 'static + Debug,
    F1: 'static + Debug,
    F2: 'static + Debug,
{
}
impl<T1, T2, T3, F1, F2> AnyTrans for Trans2<T1, T2, T3, F1, F2>
where
    T1: 'static + Debug,
    T2: 'static + Debug,
    T3: 'static + Debug,
    F1: 'static + Debug,
    F2: 'static + Debug,
{
}

/************************************/

#[derive(Debug)]
pub struct DataTransducer<I, F> {
    istate: State<I>,
    fstate: State<F>,
    states: Vec<Box<dyn AnyState>>,
    transs: Vec<Box<dyn AnyTrans>>,
}
impl<I, F> DataTransducer<I, F>
where
    I: Clone + Debug + Display,
    F: Clone + Debug + Display,
{
    pub fn new(istate: State<I>, fstate: State<F>) -> Self {
        let states = Vec::new();
        let transs = Vec::new();
        Self { istate, fstate, states, transs }
    }
    pub fn add_state<S: AnyState>(&mut self, s: S) {
        self.states.push(Box::new(s));
    }
    pub fn add_transition<T: AnyTrans>(&mut self, t: T) {
        // **Precondition:** sources and target of t
        // should be states already added to the DT
        // (or the initial/final states)
        self.transs.push(Box::new(t));
    }

    pub fn set_input(&mut self, i: I) -> Ext<I> {
        // returns the old value, if any;
        // maybe it should assert it's None instead
        self.istate.replace(Ext::One(i))
    }
    pub fn get_output(&self) -> Ext<F> {
        self.fstate.borrow().clone()
    }

    /* MAIN EVALUATION ALGORITHM */
    // Placeholder for now
    pub fn eval(&mut self, _ch: char) {
        // TODO
    }
}

/************************************/

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
