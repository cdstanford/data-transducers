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

use super::ext_value::Ext;
use super::interface::Transducer;
use std::marker::PhantomData;

/*
    TODO: Update documentation here

    States are reference-counted refcells.
    This allows transitions to have direct shared pointers to their source
    and target states.
    Transitions are defined by a guard which says when they are active, and
    an action which says the function applied to the source states to
    give a new result for the target state.

    We still require one trait for Box<dyn Trait> objects, the Transition<Q>
    trait. This is because transitions are parameterized by function types.
*/

type StateId = usize;

pub struct Trans1<Q, D, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(&D, &Q) -> Q,
{
    source: StateId,
    target: StateId,
    guard: G,
    action: F,
    ph_q: PhantomData<Q>,
    ph_d: PhantomData<D>,
}
impl<Q, D, G, F> Trans1<Q, D, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(&D, &Q) -> Q,
{
    pub fn new(source: StateId, target: StateId, guard: G, action: F) -> Self {
        Self {
            source,
            target,
            guard,
            action,
            ph_q: PhantomData,
            ph_d: PhantomData,
        }
    }
}

pub struct Trans2<Q, D, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(&D, &Q, &Q) -> Q,
{
    source1: StateId,
    source2: StateId,
    target: StateId,
    guard: G,
    action: F,
    ph_q: PhantomData<Q>,
    ph_d: PhantomData<D>,
}
impl<Q, D, G, F> Trans2<Q, D, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(&D, &Q, &Q) -> Q,
{
    pub fn new(
        source1: StateId,
        source2: StateId,
        target: StateId,
        guard: G,
        action: F,
    ) -> Self {
        Self {
            source1,
            source2,
            target,
            guard,
            action,
            ph_q: PhantomData,
            ph_d: PhantomData,
        }
    }
}

pub trait Transition<D, Q> {
    fn source_ids(&self) -> Vec<StateId>;
    fn target_id(&self) -> StateId;
}
impl<Q, D, G, F> Transition<D, Q> for Trans1<Q, D, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(&D, &Q) -> Q,
{
    fn source_ids(&self) -> Vec<StateId> {
        vec![self.source]
    }
    fn target_id(&self) -> StateId {
        self.target
    }
}
impl<Q, D, G, F> Transition<D, Q> for Trans2<Q, D, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(&D, &Q, &Q) -> Q,
{
    fn source_ids(&self) -> Vec<StateId> {
        vec![self.source1, self.source2]
    }
    fn target_id(&self) -> StateId {
        self.target
    }
}

/*
    The main DataTransducer state machine.
    Implements the Transducer interface.

    For now, DataTransducer does not implement Clone, due to the transitions
    being dynamic Trait objects.
*/

pub struct DataTransducer<Q: Clone, D> {
    // Initial state: states[0]
    // Final state: states[1]
    states: Vec<Ext<Q>>,
    updates: Vec<Box<dyn Transition<D, Q>>>,
    epsilons: Vec<Box<dyn Transition<(), Q>>>,
    ph_d: PhantomData<D>,
}

impl<Q: Clone, D> Default for DataTransducer<Q, D> {
    fn default() -> Self {
        let states = vec![Ext::None, Ext::None];
        let updates = vec![];
        let epsilons = vec![];
        Self { states, updates, epsilons, ph_d: PhantomData }
    }
}

impl<Q: Clone, D> DataTransducer<Q, D> {
    /* Initialization and adding states */
    pub fn new() -> Self {
        Default::default()
    }
    pub fn add_state(&mut self) {
        debug_assert!(self.states.len() >= 2);
        self.states.push(Ext::None);
    }
    pub fn add_transition<Tr>(&mut self, tr: Tr)
    where
        Tr: Transition<D, Q> + 'static,
    {
        self.assert_trans_precondition(&tr);
        self.updates.push(Box::new(tr));
    }
    pub fn add_epsilon<Tr>(&mut self, tr: Tr)
    where
        Tr: Transition<(), Q> + 'static,
    {
        self.assert_trans_precondition(&tr);
        self.epsilons.push(Box::new(tr));
    }

    /* Utility */
    fn add_to_istate(&mut self, i: Ext<Q>) {
        self.states[0] += i
    }
    fn get_fstate(&self) -> Ext<Q> {
        self.states[1].clone()
    }

    /* Invariant checks and assertions */
    // TODO
    // fn assert_invariant(&self) {
    // }
    fn assert_trans_precondition<I, Tr>(&self, tr: &Tr)
    where
        Tr: Transition<I, Q>,
    {
        // PRECONDITION for add_transition() and add_epsilon():
        // transition sources and targets must
        // already have been added to the machine.
        debug_assert!(tr.target_id() <= self.states.len());
        for &id in &tr.source_ids() {
            debug_assert!(id <= self.states.len());
        }
    }

    /* Streaming Algorithm */
    fn eval_epsilons(&mut self) {
        // The main streaming algorithm for updating the data transducer
        // following least-fixed-point semantics, and implemented using
        // worklists.
        // TODO
        unimplemented!()
    }
    fn eval_updates(&mut self, _item: &D) {
        // The update logic prior to evaluating epsilons -- not as complex
        // as eval_epsilons() as here we assume updates only take old states
        // and return new states.
        let new_states = vec![Ext::None; self.states.len()];
        // TODO
        self.states = new_states;
    }
}

impl<Q: Clone, D> Transducer<Q, D, Q> for DataTransducer<Q, D> {
    fn init(&mut self, i: Ext<Q>) -> Ext<Q> {
        self.add_to_istate(i);
        self.eval_epsilons();
        self.get_fstate()
    }
    fn update(&mut self, item: &D) -> Ext<Q> {
        self.eval_updates(item);
        self.eval_epsilons();
        self.get_fstate()
    }
    fn reset(&mut self) {
        for state in self.states.iter_mut() {
            *state = Ext::None;
        }
    }

    fn is_epsilon(&self) -> bool {
        // The transducer is an epsilon if it only has epsilon transitions
        self.updates.is_empty()
    }
    fn is_restartable(&self) -> bool {
        // TODO: we could implement the decision procedure for this, but it is
        // rather complex (PSPACE-complete)
        unimplemented!()
    }
    fn n_states(&self) -> usize {
        debug_assert!(self.states.len() >= 2);
        self.states.len()
    }
    fn n_transs(&self) -> usize {
        self.updates.len() + self.epsilons.len()
    }
}
