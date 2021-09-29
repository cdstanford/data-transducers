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
use std::ops::{Deref, DerefMut, Index, IndexMut};

/*
    States are represented by an Id (index into the state vector of the
    data transducer). This opaque representation allows
    keeping states and transitions completely separate and thus
    avoids Rc<RefCell<T> shenanigans.

    We also enforce state IDs as a typing discipline:
    StateId is a newtype, and we write a StateList type
    for a vector indexed by StateId. By using Deref coercion, StateList<T> has
    all the functionality of Vec<T>, **except** that it can't be indexed by
    a usize, only by a StateId.
    Conversely, StateId can't be accidentally used to index some other Vec,
    only a StateList.
*/

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StateId(usize);

#[derive(Clone, Debug)]
struct StateList<T>(Vec<T>);
impl<T> Deref for StateList<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.0
    }
}
impl<T> DerefMut for StateList<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}
impl<T> Index<StateId> for StateList<T> {
    type Output = T;
    fn index(&self, id: StateId) -> &Self::Output {
        self.0.index(id.0)
    }
}
impl<T> IndexMut<StateId> for StateList<T> {
    fn index_mut(&mut self, id: StateId) -> &mut Self::Output {
        self.0.index_mut(id.0)
    }
}
impl<T> StateList<T> {
    // Additionally useful things that go together with indexing
    fn in_range(&self, id: StateId) -> bool {
        id.0 < self.len()
    }
    fn enumerate(&self) -> impl Iterator<Item = (StateId, &T)> {
        self.iter().enumerate().map(|(i, item)| (StateId(i), item))
    }
}

#[test]
fn test_stateid_index() {
    let v = StateList(vec![1, 2, 3]);
    assert_eq!(v[StateId(1)], 2);
    // The following does not compile:
    // assert_eq!(v[1], 2);
}

/*
    Transitions are defined by a guard which says when they are active, and
    an action which says the function applied to the source states to
    give a new result for the target state.
    Transitions implement the Transition trait, providing an interface of
    their functionality, and will be stored in the data transducer as
    dynamic Box<dyn Transition> objects.
    This is because they are functions so do not share a common type.
*/

struct Trans1<D, Q, G, F>
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

struct Trans2<D, Q, G, F>
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

trait Transition<D, Q> {
    fn source_ids(&self) -> Vec<StateId>;
    fn target_id(&self) -> StateId;
    fn is_active(&self, item: &D) -> bool;
    fn eval(&self, item: &D, states: &StateList<Ext<Q>>) -> Ext<Q>;

    /* Derived functionality */
    fn eval_precond(&self, states: &StateList<Ext<Q>>) -> bool {
        self.source_ids().iter().all(|&id| states.in_range(id))
    }
    fn all_ids(&self) -> Vec<StateId> {
        let mut result = self.source_ids();
        result.push(self.target_id());
        result
    }
}

// Lightweight Debug implementation
// This format string is rather incomplete, since function closures
// do not implement Debug.
// Note: the + '_ is important because otherwise trait objects default to
// 'static lifetime.
// https://stackoverflow.com/questions/63986183/format-requires-static-lifetime
impl<D, Q> Debug for dyn Transition<D, Q> + '_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for &id in &self.source_ids() {
            f.write_fmt(format_args!("{} ", id.0))?;
        }
        f.write_fmt(format_args!("-> {}]", self.target_id().0))
    }
}

impl<D, Q, G, F> Transition<D, Q> for Trans1<D, Q, G, F>
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
    fn eval(&self, item: &D, states: &StateList<Ext<Q>>) -> Ext<Q> {
        debug_assert!(self.eval_precond(states));
        ext_value::apply1(
            |q| (self.action)(item, q),
            states[self.source].as_ref(),
        )
    }
}
impl<D, Q, G, F> Transition<D, Q> for Trans2<D, Q, G, F>
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
    fn eval(&self, item: &D, states: &StateList<Ext<Q>>) -> Ext<Q> {
        debug_assert!(self.eval_precond(states));
        ext_value::apply2(
            |q1, q2| (self.action)(item, q1, q2),
            states[self.source1].as_ref(),
            states[self.source2].as_ref(),
        )
    }
}

/*
    More transition functionality.

    Exactly the same as StateId and StateList, TransId and TransList are type
    wrappers over usize and Vec<T> where the latter can be indexed by the
    former. The most important thing is that TransList can't be indexed by
    StateId and StateList can't be indexed by TransId. In fact, I already caught
    a bug due to such a mistake as I was introducing this discipline.
*/

#[derive(Copy, Clone, Debug)]
struct TransId(usize);

#[derive(Clone, Debug)]
struct TransList<T>(Vec<T>);
impl<T> Deref for TransList<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.0
    }
}
impl<T> DerefMut for TransList<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}
impl<T> Index<TransId> for TransList<T> {
    type Output = T;
    fn index(&self, id: TransId) -> &Self::Output {
        self.0.index(id.0)
    }
}
impl<T> IndexMut<TransId> for TransList<T> {
    fn index_mut(&mut self, id: TransId) -> &mut Self::Output {
        self.0.index_mut(id.0)
    }
}

// Guard function for epsilon transitions -- should never be called, so panics
fn epsilon_guard<D>(_item: &D) -> bool {
    panic!("Called guard for epsilon transition!");
}

/*
    The main DataTransducer state machine.
    Implements the Transducer interface.

    For now, DataTransducer does not implement Clone, due to the transitions
    being dynamic Trait objects.
*/

const ISTATE_ID: StateId = StateId(0);
const FSTATE_ID: StateId = StateId(1);

pub struct DataTransducer<'a, D, Q>
where
    Q: 'a + Clone,
    D: 'a,
{
    // Initial state: states[0]
    // Final state: states[1]
    states: StateList<Ext<Q>>,
    // Transitions, divided into those executed on update from old to new states
    // and "epsilon transitions" which define a least fixed point on init and
    // after every update
    updates: TransList<Box<dyn Transition<D, Q> + 'a>>,
    epsilons: TransList<Box<dyn Transition<(), Q> + 'a>>,
    // Store for each state which epsilon-transitions go out from this state
    // (needed for the least fixed point calculation)
    eps_out: StateList<Vec<TransId>>,
    // Dummy marker for D
    ph_d: PhantomData<D>,
}

impl<D, Q> Default for DataTransducer<'_, D, Q>
where
    Q: Clone,
{
    fn default() -> Self {
        let states = StateList(vec![Ext::None, Ext::None]);
        let updates = TransList(vec![]);
        let epsilons = TransList(vec![]);
        let eps_out = StateList(vec![vec![], vec![]]);
        let ph_d = PhantomData;
        let result = Self { states, updates, epsilons, eps_out, ph_d };
        debug_assert!(result.invariant());
        result
    }
}

impl<D, Q> Debug for DataTransducer<'_, D, Q>
where
    Q: Clone + Debug,
    D: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataTransducer")
            .field("states", &self.states)
            .field("updates", &self.updates)
            .field("epsilons", &self.epsilons)
            .field("eps_out", &self.eps_out)
            .finish()
    }
}

impl<'a, D, Q> DataTransducer<'a, D, Q>
where
    Q: Clone,
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
        source: usize,
        target: usize,
        guard: G,
        action: F,
    ) where
        G: 'a + Fn(&D) -> bool,
        F: 'a + Fn(&D, &Q) -> Q,
    {
        self.add_transition_core(Trans1 {
            source: StateId(source),
            target: StateId(target),
            guard,
            action,
            ph_d: PhantomData,
            ph_q: PhantomData,
        });
    }
    // Add an update transition with two source states
    pub fn add_transition2<G, F>(
        &mut self,
        source1: usize,
        source2: usize,
        target: usize,
        guard: G,
        action: F,
    ) where
        G: 'a + Fn(&D) -> bool,
        F: 'a + Fn(&D, &Q, &Q) -> Q,
    {
        self.add_transition_core(Trans2 {
            source1: StateId(source1),
            source2: StateId(source2),
            target: StateId(target),
            guard,
            action,
            ph_d: PhantomData,
            ph_q: PhantomData,
        });
    }
    // Add an "identity transition" which preserves a particular state from one
    // timestep to the next. (This is common enough that it's worth exposing
    // specifically in the API.)
    pub fn add_iden<G>(&mut self, source: usize, target: usize, guard: G)
    where
        G: 'a + Fn(&D) -> bool,
    {
        self.add_transition1(source, target, guard, |_, q| q.clone())
    }
    // Add an epsilon transition with one source state
    pub fn add_epsilon1<F>(&mut self, source: usize, target: usize, action: F)
    where
        F: 'a + Fn(&Q) -> Q,
    {
        self.add_epsilon_core(Trans1 {
            source: StateId(source),
            target: StateId(target),
            guard: epsilon_guard,
            action: move |_, q| action(q),
            ph_d: PhantomData,
            ph_q: PhantomData,
        });
    }
    // Add an update transition with two source states
    pub fn add_epsilon2<F>(
        &mut self,
        source1: usize,
        source2: usize,
        target: usize,
        action: F,
    ) where
        F: 'a + Fn(&Q, &Q) -> Q,
    {
        self.add_epsilon_core(Trans2 {
            source1: StateId(source1),
            source2: StateId(source2),
            target: StateId(target),
            guard: epsilon_guard,
            action: move |_, q1, q2| action(q1, q2),
            ph_d: PhantomData,
            ph_q: PhantomData,
        });
    }

    /* Utility / conveniences */
    fn add_to_istate(&mut self, i: Ext<Q>) {
        self.states[ISTATE_ID] += i
    }
    fn get_fstate(&self) -> Ext<Q> {
        self.states[FSTATE_ID].clone()
    }
    fn eval_epsilon(&self, tid: TransId) -> Ext<Q> {
        self.epsilons[tid].eval(&(), &self.states)
    }
    fn add_transition_core<Tr>(&mut self, tr: Tr)
    where
        Tr: 'a + Transition<D, Q>,
    {
        assert!(self.trans_precond(&tr));
        self.updates.push(Box::new(tr));
        debug_assert!(self.invariant());
    }
    fn add_epsilon_core<Tr>(&mut self, tr: Tr)
    where
        Tr: 'a + Transition<(), Q>,
    {
        debug_assert!(self.trans_precond(&tr));
        let new_tr_id = TransId(self.epsilons.len());
        for source_id in tr.source_ids() {
            self.eps_out[source_id].push(new_tr_id);
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
        for (state_id, eps_ids) in self.eps_out.enumerate() {
            for &id in eps_ids {
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
        tr.all_ids().iter().all(|&id| self.states.in_range(id))
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
        let mut trans_wklist: Vec<TransId> =
            (0..n_epsilons).map(TransId).collect();
        let mut trans_vals: TransList<Ext<()>> =
            TransList(vec![Ext::None; n_epsilons]);
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
        let mut new_states = StateList(vec![Ext::None; self.states.len()]);
        for tr in self.updates.iter() {
            if tr.is_active(item) {
                new_states[tr.target_id()] += tr.eval(item, &self.states);
            }
        }
        self.states = new_states;
    }
}

impl<D, Q> Transducer<Q, D, Q> for DataTransducer<'_, D, Q>
where
    Q: Clone,
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
        // rather complex (PSPACE-complete).
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

    type ExD = (char, isize);
    type ExQ = isize;

    /* Additional methods for testing */
    impl<D, Q> DataTransducer<'_, D, Q>
    where
        D: Debug,
        Q: Clone + Debug + Eq,
    {
        fn init_expect(&mut self, i: Q, o: Ext<Q>) {
            println!("State: {:?}", self);
            println!("===== init: {:?} =====", i);
            println!("Expected output: {:?}", o);
            assert_eq!(self.init_one(i), o);
            println!("Output is correct");
        }
        fn update_expect(&mut self, d: D, o: Ext<Q>) {
            println!("State: {:?}", self);
            println!("===== update: {:?} =====", d);
            println!("Expected output: {:?}", o);
            assert_eq!(self.update_val(d), o);
            println!("Output is correct");
        }
    }

    #[test]
    fn test_popl19_ex1() {
        // Initialize
        let mut m = DataTransducer::<ExD, ExQ>::new();
        m.set_nstates(4);
        assert_eq!(m.n_states(), 4);
        assert_eq!(m.n_transs(), 0);
        assert!(m.is_epsilon());
        // 0: Initial, always set to Ext::One
        // 1: Final, sum of last three 'a' events
        // 2: Sum of last two 'a' events
        // 3: Sum of last 'a' event
        m.add_iden(0, 0, |_d| true);
        m.add_iden(2, 2, |&d| d.0 == 'b');
        m.add_iden(3, 3, |&d| d.0 == 'b');
        m.add_transition1(0, 3, |&d| d.0 == 'a', |&d, _q| d.1);
        m.add_transition1(3, 2, |&d| d.0 == 'a', |&d, &q| q + d.1);
        m.add_transition1(2, 1, |&d| d.0 == 'a', |&d, &q| q + d.1);
        assert_eq!(m.n_states(), 4);
        assert_eq!(m.n_transs(), 6);
        assert!(!m.is_epsilon());
        // Test
        m.init_expect(0, Ext::None);
        m.update_expect(('a', 6), Ext::None);
        m.update_expect(('b', 2), Ext::None);
        m.update_expect(('a', 5), Ext::None);
        m.update_expect(('a', 7), Ext::One(18));
        m.update_expect(('b', 2), Ext::None);
        m.update_expect(('a', 8), Ext::One(20));
        m.update_expect(('#', 0), Ext::None);
        m.update_expect(('b', 2), Ext::None);
        m.update_expect(('a', 2), Ext::None);
        m.update_expect(('a', 2), Ext::None);
        m.update_expect(('a', 2), Ext::One(6));
        m.update_expect(('a', 0), Ext::One(4));
    }

    #[test]
    fn test_popl19_ex2() {
        // Initialize
        let mut m = DataTransducer::<ExD, ExQ>::new();
        m.set_nstates(4);
        // 0: Initial, previous window average
        // 1: Final, end of window average
        // 2: Sum of 'a' events in current window
        // 3: Count of 'a' events in current window
        m.add_iden(0, 0, |&d| d.0 == 'b');
        m.add_iden(2, 2, |&d| d.0 == 'b');
        m.add_iden(3, 3, |&d| d.0 == 'b');
        m.add_transition1(0, 2, |&d| d.0 == 'a', |&d, _q| d.1);
        m.add_transition1(0, 3, |&d| d.0 == 'a', |_d, _q| 1);
        m.add_transition1(2, 2, |&d| d.0 == 'a', |&d, &q| q + d.1);
        m.add_transition1(3, 3, |&d| d.0 == 'a', |_d, &q| q + 1);
        m.add_transition2(2, 3, 1, |&d| d.0 == '#', |_d, &q2, &q3| q2 / q3);
        m.add_iden(0, 1, |&d| d.0 == '#');
        m.add_epsilon1(1, 0, |&q| q);
        assert_eq!(m.n_states(), 4);
        assert_eq!(m.n_transs(), 10);
        // Test
        m.init_expect(0, Ext::None);
        m.update_expect(('b', 2), Ext::None);
        m.update_expect(('a', 6), Ext::None);
        m.update_expect(('b', 2), Ext::None);
        m.update_expect(('a', 8), Ext::None);
        m.update_expect(('a', 9), Ext::None);
        // (6 + 8 + 9) / 3 = 7
        m.update_expect(('#', 0), Ext::One(7));
        m.update_expect(('b', 2), Ext::None);
        m.update_expect(('#', 2), Ext::One(7));
        m.update_expect(('a', 2), Ext::None);
        m.update_expect(('#', 0), Ext::One(2));
    }

    #[test]
    fn test_popl19_ex3() {
        // Initialize
        let mut m = DataTransducer::<ExD, ExQ>::new();
        m.set_nstates(7);
        // 0: Initial, Ext::One initially and after each '#'
        // 1: Final, stores the final answer after each '#'
        // 2: Ext::One if we haven't yet seen an 'a'
        // 3: Ext::One if we haven't yet seen a 'b'
        // 4: Max 'a' if we have seen at least one
        // 5: Max 'b' if we have seen at least one
        // 6: Ext::One always (used to set state 0 on '#')
        m.add_epsilon1(0, 2, |_q| 0);
        m.add_epsilon1(0, 3, |_q| 0);
        m.add_epsilon1(0, 6, |_q| 0);
        assert!(m.is_epsilon());
        m.add_iden(2, 2, |&d| d.0 == 'b');
        m.add_iden(4, 4, |&d| d.0 == 'b');
        m.add_iden(3, 3, |&d| d.0 == 'a');
        m.add_iden(5, 5, |&d| d.0 == 'a');
        m.add_iden(6, 6, |&d| d.0 != '#');
        m.add_transition1(2, 4, |&d| d.0 == 'a', |&d, _q| d.1);
        m.add_transition1(4, 4, |&d| d.0 == 'a', |&d, &q| q.max(d.1));
        m.add_transition1(3, 5, |&d| d.0 == 'b', |&d, _q| d.1);
        m.add_transition1(5, 5, |&d| d.0 == 'b', |&d, &q| q.max(d.1));
        m.add_transition2(4, 5, 1, |&d| d.0 == '#', |_d, &q4, &q5| q4 - q5);
        m.add_transition1(6, 0, |&d| d.0 == '#', |_d, _q| 0);
        assert_eq!(m.n_transs(), 14);
        // Test
        m.init_expect(0, Ext::None);
        m.update_expect(('b', 2), Ext::None);
        m.update_expect(('a', 6), Ext::None);
        m.update_expect(('b', 3), Ext::None);
        m.update_expect(('b', 1), Ext::None);
        m.update_expect(('a', 8), Ext::None);
        m.update_expect(('#', 0), Ext::One(5));
        m.update_expect(('b', 2), Ext::None);
        m.update_expect(('#', 2), Ext::None);
        m.update_expect(('a', 1), Ext::None);
        m.update_expect(('a', 3), Ext::None);
        m.update_expect(('#', 2), Ext::None);
        m.update_expect(('a', 7), Ext::None);
        m.update_expect(('b', 1), Ext::None);
        m.update_expect(('#', 0), Ext::One(6));
    }

    #[test]
    #[should_panic]
    fn test_set_nstates_bad() {
        let mut m = DataTransducer::<ExD, ExQ>::new();
        m.set_nstates(5);
        m.set_nstates(4);
    }

    #[test]
    #[should_panic]
    fn test_nonexistent_target() {
        let mut m = DataTransducer::<ExD, ExQ>::new();
        m.set_nstates(4);
        m.add_iden(3, 4, |_| false);
    }

    #[test]
    #[should_panic]
    fn test_nonexistent_source() {
        let mut m = DataTransducer::<ExD, ExQ>::new();
        m.set_nstates(4);
        m.add_transition2(1, 4, 3, |_| false, |_, _, _| 0);
    }

    #[test]
    fn test_loop_1() {
        let mut m = DataTransducer::<ExD, ExQ>::new();
        m.set_nstates(3);
        m.add_epsilon1(0, 1, |_| 0);
        m.add_epsilon1(1, 2, |_| 0);
        m.add_epsilon1(2, 0, |_| 0);
        m.update_expect(('a', 0), Ext::None);
        m.init_expect(0, Ext::Many);
        m.init_expect(0, Ext::Many);
        m.update_expect(('a', 0), Ext::None);
        m.init_expect(0, Ext::Many);
        m.reset();
        m.update_expect(('a', 0), Ext::None);
        m.init_expect(0, Ext::Many);
    }

    #[test]
    fn test_loop_2() {
        let mut m = DataTransducer::<ExD, ExQ>::new();
        m.set_nstates(4);
        m.add_epsilon1(0, 1, |_| 0);
        m.add_epsilon2(0, 1, 2, |_, _| 0);
        m.add_epsilon2(2, 3, 1, |_, _| 0);
        m.add_epsilon1(3, 0, |_| 0);
        m.add_iden(2, 3, |_| true);
        m.update_expect(('a', 0), Ext::None);
        m.init_expect(3, Ext::One(0));
        m.update_expect(('a', 0), Ext::Many);
        m.update_expect(('a', 2), Ext::Many);
        m.init_expect(0, Ext::Many);
    }

    #[test]
    fn test_reset() {
        let mut m = DataTransducer::<ExD, ExQ>::new();
        m.add_epsilon1(0, 1, |&q| q);
        m.add_iden(1, 1, |_d| true);
        m.update_expect(('a', 0), Ext::None);
        m.init_expect(1, Ext::One(1));
        m.update_expect(('a', 6), Ext::One(1));
        m.update_expect(('a', 5), Ext::One(1));
        m.init_expect(0, Ext::Many);
        m.update_expect(('a', 5), Ext::Many);
        m.init_expect(1, Ext::Many);
        m.reset();
        m.update_expect(('a', 0), Ext::None);
        m.init_expect(2, Ext::One(2));
    }
}
