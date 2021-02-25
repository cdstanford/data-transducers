/*
    Module implementing data transducers as explicit state machines.

    Generally the QRE constructs should be more convenient and high-level,
    but this can be used if you want to manually write
    the states and transitions yourself.

    For simplicity and safety of the implementation, states in
    the machine are limited to be of all the same type Q. In order to
    achieve multiple arbitrary types in the machine, Q can be set
    to an enum, or an unsafe Union:
    https://doc.rust-lang.org/reference/items/unions.html
    or even an unsafe pointer.
    I originally wanted to support multiple state types,
    but there is no easy way to deal with the complexity of types.
    Either the implementation would itself be inherently unsafe, or it
    would rely on a lot of dynamic manipulation of trait objects (something
    like Vec<Box<dyn Stateable>> for the states and Vec<Box<dyn Transition>> for
    the transitions, but then it is challenging because the Transitions need
    to also keep reference-counted pointers into the states to get/update
    their values). Overall, fixing Q is cleaner design.
*/

use super::ext_value::{self, Ext};
use super::interface::Transducer;
use std::fmt::{self, Debug};
use std::marker::PhantomData;

/*
    States are represented by an Id (index into the state vector of the
    data transducer). This opaque representation allows
    keeping states and transitions completely separate and thus
    avoids Rc<RefCell<T> shenanigans.

    Transitions are defined by a guard which says when they are active, and
    an action which says the function applied to the source states to
    give a new result for the target state.
    Transitions implement the Transition trait, providing an interface of
    their functionality, and will be stored in the data transducer as
    dynamic Box<dyn Transition> objects.
    This is because they are functions so do not share a common type.
*/

type StateId = usize;

struct Trans1<Q, D, G, F>
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

struct Trans2<Q, D, G, F>
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

pub trait Transition<D, Q> {
    fn source_ids(&self) -> Vec<StateId>;
    fn target_id(&self) -> StateId;
    fn is_active(&self, item: &D) -> bool;
    fn eval(&self, item: &D, states: &[Ext<Q>]) -> Ext<Q>;

    /* Derived functionality */
    fn eval_precond(&self, states: &[Ext<Q>]) -> bool {
        self.source_ids().iter().all(|&id| id < states.len())
    }
    fn all_ids(&self) -> Vec<StateId> {
        let mut result = self.source_ids();
        result.push(self.target_id());
        result
    }
    // Format string to be used for debugging
    // (lightweight Debug implementation)
    // This format string is rather incomplete, since function closures
    // do not implement Debug.
    fn fmt_as_ids(&self) -> String {
        format!("({:?}, {})", self.source_ids(), self.target_id())
    }
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
    fn is_active(&self, item: &D) -> bool {
        (self.guard)(item)
    }
    fn eval(&self, item: &D, states: &[Ext<Q>]) -> Ext<Q> {
        debug_assert!(self.eval_precond(states));
        ext_value::apply1(
            |q| (self.action)(item, q),
            states[self.source].as_ref(),
        )
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
    fn is_active(&self, item: &D) -> bool {
        (self.guard)(item)
    }
    fn eval(&self, item: &D, states: &[Ext<Q>]) -> Ext<Q> {
        debug_assert!(self.eval_precond(states));
        ext_value::apply2(
            |q1, q2| (self.action)(item, q1, q2),
            states[self.source1].as_ref(),
            states[self.source2].as_ref(),
        )
    }
}

/*
    The main DataTransducer state machine.
    Implements the Transducer interface.

    For now, DataTransducer does not implement Clone, due to the transitions
    being dynamic Trait objects.
*/

type TransId = usize;
struct TransitionList<'a, D, Q>(&'a Vec<Box<dyn Transition<D, Q>>>);

impl<Q, D> Debug for TransitionList<'_, D, Q>
where
    Q: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter().map(|tr| tr.fmt_as_ids())).finish()
    }
}

// Guard function for epsilon transitions -- should never be called, so panics
fn epsilon_guard<D>(_item: &D) -> bool {
    panic!("Called guard for epsilon transition!");
}

pub struct DataTransducer<Q, D>
where
    Q: Clone + 'static,
    D: 'static,
{
    // Initial state: states[0]
    // Final state: states[1]
    states: Vec<Ext<Q>>,
    // Transitions, divided into those executed on update from old to new states
    // and "epsilon transitions" which define a least fixed point on init and
    // after every update
    updates: Vec<Box<dyn Transition<D, Q>>>,
    epsilons: Vec<Box<dyn Transition<(), Q>>>,
    // Store for each state which epsilon-transitions go out from this state
    // (needed for the least fixed point calculation)
    eps_out: Vec<Vec<TransId>>,
    // Dummy marker for D
    ph_d: PhantomData<D>,
}

impl<Q, D> Default for DataTransducer<Q, D>
where
    Q: Clone + 'static,
    D: 'static,
{
    fn default() -> Self {
        let states = vec![Ext::None, Ext::None];
        let updates = vec![];
        let epsilons = vec![];
        let eps_out = vec![vec![], vec![]];
        let ph_d = PhantomData;
        let result = Self { states, updates, epsilons, eps_out, ph_d };
        debug_assert!(result.invariant());
        result
    }
}

impl<Q, D> Debug for DataTransducer<Q, D>
where
    Q: Clone + Debug + 'static,
    D: Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataTransducer")
            .field("states", &self.states)
            .field("updates", &TransitionList(&self.updates))
            .field("epsilons", &TransitionList(&self.epsilons))
            .field("eps_out", &self.eps_out)
            .finish()
    }
}

impl<Q, D> DataTransducer<Q, D>
where
    Q: Clone + 'static,
    D: 'static,
{
    /* Initialization (forming the states and transitions) */
    pub fn new() -> Self {
        Default::default()
    }
    pub fn add_state(&mut self) {
        debug_assert!(self.states.len() >= 2);
        self.states.push(Ext::None);
        self.eps_out.push(Vec::new());
        debug_assert!(self.invariant());
    }
    // Set the number of states directly
    // (instead of repeatedly calling .add_state())
    pub fn set_nstates(&mut self, n: usize) {
        assert!(self.states.len() <= n);
        while self.states.len() < n {
            self.add_state();
        }
    }
    // Add an update transition with one source state
    pub fn add_transition1<G, F>(
        &mut self,
        source: StateId,
        target: StateId,
        guard: G,
        action: F,
    ) where
        G: Fn(&D) -> bool + 'static,
        F: Fn(&D, &Q) -> Q + 'static,
    {
        let ph_d = PhantomData;
        let ph_q = PhantomData;
        let t = Trans1 { source, target, guard, action, ph_d, ph_q };
        self.add_transition_core(t);
    }
    // Add an update transition with two source states
    pub fn add_transition2<G, F>(
        &mut self,
        source1: StateId,
        source2: StateId,
        target: StateId,
        guard: G,
        action: F,
    ) where
        G: Fn(&D) -> bool + 'static,
        F: Fn(&D, &Q, &Q) -> Q + 'static,
    {
        let ph_d = PhantomData;
        let ph_q = PhantomData;
        let t = Trans2 { source1, source2, target, guard, action, ph_d, ph_q };
        self.add_transition_core(t);
    }
    // Add an "identity transition" which preserves a particular state from one
    // timestep to the next. (This is common enough that it's worth exposing
    // specifically in the API.)
    pub fn add_iden(&mut self, source: StateId, target: StateId) {
        self.add_transition1(source, target, |_| true, |_, q| q.clone())
    }
    // Add an epsilon transition with one source state
    pub fn add_epsilon1<F>(
        &mut self,
        source: StateId,
        target: StateId,
        action: F,
    ) where
        F: Fn(&Q) -> Q + 'static,
    {
        self.add_epsilon_core(Trans1 {
            source,
            target,
            guard: epsilon_guard,
            action: move |_, q| action(q),
            ph_d: PhantomData,
            ph_q: PhantomData,
        });
    }
    // Add an update transition with two source states
    pub fn add_epsilon2<F>(
        &mut self,
        source1: StateId,
        source2: StateId,
        target: StateId,
        action: F,
    ) where
        F: Fn(&Q, &Q) -> Q + 'static,
    {
        self.add_epsilon_core(Trans2 {
            source1,
            source2,
            target,
            guard: epsilon_guard,
            action: move |_, q1, q2| action(q1, q2),
            ph_d: PhantomData,
            ph_q: PhantomData,
        });
    }

    /* Utility / conveniences and hidden functionality */
    fn add_to_istate(&mut self, i: Ext<Q>) {
        self.states[0] += i
    }
    fn get_fstate(&self) -> Ext<Q> {
        self.states[1].clone()
    }
    fn eval_epsilon(&self, tid: TransId) -> Ext<Q> {
        self.epsilons[tid].eval(&(), &self.states)
    }
    fn add_transition_core<Tr>(&mut self, tr: Tr)
    where
        Tr: Transition<D, Q> + 'static,
    {
        debug_assert!(self.trans_precond(&tr));
        self.updates.push(Box::new(tr));
        debug_assert!(self.invariant());
    }
    fn add_epsilon_core<Tr>(&mut self, tr: Tr)
    where
        Tr: Transition<(), Q> + 'static,
    {
        debug_assert!(self.trans_precond(&tr));
        for source_id in tr.source_ids() {
            self.eps_out[source_id].push(tr.target_id());
        }
        self.epsilons.push(Box::new(tr));
        debug_assert!(self.invariant());
    }

    /* Invariant checks and preconditions */
    fn invariant(&self) -> bool {
        // Returns true for convenience of debug_assert!(self.invariant())
        debug_assert!(self.states.len() >= 2);
        debug_assert_eq!(self.states.len(), self.eps_out.len());
        debug_assert_eq!(
            self.eps_out.iter().map(|ids| ids.len()).sum::<usize>(),
            self.epsilons.iter().map(|eps| eps.source_ids().len()).sum(),
        );
        for (state_id, ids) in self.eps_out.iter().enumerate() {
            for &id in ids {
                debug_assert!(self.epsilons[id]
                    .source_ids()
                    .iter()
                    .any(|&s| { s == state_id }));
            }
        }
        true
    }
    fn trans_precond<I, Tr>(&self, tr: &Tr) -> bool
    where
        Tr: Transition<I, Q>,
    {
        // PRECONDITION for add_transition() and add_epsilon():
        // transition sources and targets must
        // already have been added to the machine.
        tr.all_ids().iter().all(|&id| id < self.states.len())
    }

    /* Streaming Algorithm */
    fn eval_epsilons(&mut self) {
        // The main streaming algorithm for updating the data transducer
        // following least-fixed-point semantics, and implemented using
        // a transition worklist.
        // Note on efficiency: it is slightly more efficient to also
        // keep a count of how many input states are Ext::None for each
        // transition, and only add a transition to the worklist when this
        // number increases. But this only really matters for transitions with
        // more than one or two source states.
        let n_epsilons = self.epsilons.len();
        let mut trans_wklist: Vec<TransId> = (0..n_epsilons).collect();
        let mut trans_vals: Vec<Ext<()>> = vec![Ext::None; n_epsilons];
        while let Some(tr_id) = trans_wklist.pop() {
            let cur = trans_vals[tr_id];
            let tgt_id = self.epsilons[tr_id].target_id();
            // Only evaluate the transition if its value may cause a change
            if cur.is_many() || self.states[tgt_id].is_many() {
                continue;
            }
            let new = self.eval_epsilon(tr_id);
            if new.is_none() || new.is_one() && cur.is_one() {
                continue;
            }
            // Here we know: the value of the transition has increased
            // (from None to One(x), None to Many, or One(x) to Many)
            // AND the target state is either None or One(x), so should
            // be increased by One(x), Many, or Many respectively
            trans_vals[tr_id] = new.to_unit();
            self.states[tgt_id] += new;
            for &eps_id in &self.eps_out[tgt_id] {
                trans_wklist.push(eps_id);
            }
        }
    }
    fn eval_updates(&mut self, item: &D) {
        // The update logic prior to evaluating epsilons -- not as complex
        // as eval_epsilons() as here we assume updates only take old states
        // and return new states.
        let mut new_states = vec![Ext::None; self.states.len()];
        for tr in &self.updates {
            if tr.is_active(item) {
                new_states[tr.target_id()] += tr.eval(item, &self.states);
            }
        }
        self.states = new_states;
    }
}

impl<Q, D> Transducer<Q, D, Q> for DataTransducer<Q, D>
where
    Q: Clone + 'static,
    D: 'static,
{
    fn init(&mut self, i: Ext<Q>) -> Ext<Q> {
        self.add_to_istate(i);
        self.eval_epsilons();
        debug_assert!(self.invariant());
        self.get_fstate()
    }
    fn update(&mut self, item: &D) -> Ext<Q> {
        self.eval_updates(item);
        self.eval_epsilons();
        debug_assert!(self.invariant());
        self.get_fstate()
    }
    fn reset(&mut self) {
        for state in self.states.iter_mut() {
            *state = Ext::None;
        }
        debug_assert!(self.invariant());
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

#[cfg(test)]
mod tests {
    use super::*;

    type ExQ = usize;
    type ExD = (char, usize);

    #[test]
    fn test_popl19_ex1() {
        // Initialize
        let mut m = DataTransducer::<ExQ, ExD>::new();
        m.set_nstates(4);
        m.add_iden(0, 0);
        m.add_transition1(0, 3, |&d| d.0 == 'a', |&d, _| d.1);
        m.add_transition1(3, 2, |&d| d.0 == 'a', |&d, &q| d.1 + q);
        m.add_transition1(2, 1, |&d| d.0 == 'a', |&d, &q| d.1 + q);
        m.add_transition1(2, 2, |&d| d.0 == 'b', |_, &q| q);
        m.add_transition1(3, 3, |&d| d.0 == 'b', |_, &q| q);
        // Test
        println!("{:?}", m);
        assert_eq!(m.init_one(0), Ext::None);
        assert_eq!(m.update_val(('a', 6)), Ext::None);
        assert_eq!(m.update_val(('b', 2)), Ext::None);
        assert_eq!(m.update_val(('a', 5)), Ext::None);
        assert_eq!(m.update_val(('a', 7)), Ext::One(18));
        assert_eq!(m.update_val(('b', 2)), Ext::None);
        assert_eq!(m.update_val(('a', 8)), Ext::One(20));
        println!("{:?}", m);
        assert_eq!(m.update_val(('#', 0)), Ext::None);
        assert_eq!(m.update_val(('b', 2)), Ext::None);
        assert_eq!(m.update_val(('a', 2)), Ext::None);
        assert_eq!(m.update_val(('a', 2)), Ext::None);
        assert_eq!(m.update_val(('a', 2)), Ext::One(6));
        assert_eq!(m.update_val(('a', 0)), Ext::One(4));
        println!("{:?}", m);
    }
}
