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

pub struct DataTransducer<I, Q, O, D, FI, FO>
where
    FI: Fn(I) -> Q,
    FO: Fn(&Q) -> O,
{
    istate: State<I>,
    fstate: State<O>,
    init_fun: FI,
    fin_fun: FO,
    states: Vec<State<Q>>,
    transitions: Vec<Box<dyn Transition<Q>>>,
    ph_d: PhantomData<D>,
}

impl<I, Q, O, D, FI, FO> DataTransducer<I, Q, O, D, FI, FO>
where
    FI: Fn(I) -> Q,
    FO: Fn(&Q) -> O,
{
    fn new(init_fun: FI, fin_fun: FO) -> Self {
        Self {
            istate: Rc::new(RefCell::new(Ext::None)),
            fstate: Rc::new(RefCell::new(Ext::None)),
            init_fun,
            fin_fun,
            states: Vec::new(),
            transitions: Vec::new(),
            ph_d: PhantomData,
        }
    }
    fn add_state(&mut self, s: State<Q>) {
        self.states.push(s);
    }
    fn add_transition<Tr>(&mut self, tr: Tr)
    where
        Tr: Transition<Q> + 'static,
    {
        // PRECONDITION: Transition<Q> sources and targets must
        // already have been added to the machine.
        // TODO: can we check this using debug_assert!() ?
        self.transitions.push(Box::new(tr));
    }
}

impl<I, Q, O, D, FI, FO> Clone for DataTransducer<I, Q, O, D, FI, FO>
where
    I: Clone,
    Q: Clone,
    O: Clone,
    FI: Fn(I) -> Q + Clone,
    FO: Fn(&Q) -> O + Clone,
{
    fn clone(&self) -> Self {
        // TODO: There is a problem here.
        // How do we clone transitions? When they have source references,
        // it is not clear how to do so.
        unimplemented!()
    }
}

impl<I, Q, O, D, FI, FO> Transducer<I, D, O>
    for DataTransducer<I, Q, O, D, FI, FO>
where
    FI: Fn(I) -> Q,
    FO: Fn(&Q) -> O,
{
    fn init(&mut self, i: Ext<I>) -> Ext<O> {
        // TODO
        Ext::None
    }
    fn update(&mut self, item: &D) -> Ext<O> {
        // TODO
        Ext::None
    }
    fn reset(&mut self) {
        self.istate.replace(Ext::None);
        self.fstate.replace(Ext::None);
        for state in &self.states {
            state.replace(Ext::None);
        }
    }

    fn is_epsilon(&self) -> bool {
        // TODO
        unimplemented!()
    }
    fn is_restartable(&self) -> bool {
        // TODO: write the (complex) decision procedure which determines
        // this? Unfortunately PSPACE-complete.
        unimplemented!()
    }
    fn n_states(&self) -> usize {
        self.states.len() + 2
    }
    fn n_transs(&self) -> usize {
        self.transitions.len() + 2
    }
}
