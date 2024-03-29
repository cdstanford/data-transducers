/*
    An implementation of the QRE language (Quantitative Regular Expressions)
    using data transducers.

    There is a design choice here: whether to separate out the logic of
    building up the states and transitions from the logic which
    updates the states and transitions in response to an input. Here, we choose
    to build the computation at the same time as the states and transitions,
    rather than as a separate evaluation algorithm, as it is more convenient
    to work with smaller data transducers as "black boxes" that way.
*/

use super::ext_value::{self, Ext};
use super::interface::Transducer;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;

/*
    QRE epsilon

    Base construct which processes no data and immediately produces output

    Derived constructs:

    - epsilon_iden
      Epsilon which is the identity function.
      This is the identity for QRE concatenation.

    - epsilon_const
      Epsilon which produces a constant output.
*/

pub struct Epsilon<I, D, O, F>
where
    F: Fn(I) -> O,
{
    action: F,
    ph_i: PhantomData<I>,
    ph_d: PhantomData<D>,
    ph_o: PhantomData<O>,
}
pub fn epsilon<I, D, O, F>(action: F) -> Epsilon<I, D, O, F>
where
    F: Fn(I) -> O,
{
    Epsilon { action, ph_i: PhantomData, ph_d: PhantomData, ph_o: PhantomData }
}
pub fn epsilon_iden<I, D>() -> impl Transducer<I, D, I> {
    epsilon(|i| i)
}
pub fn epsilon_const<I, D, O>(out: O) -> impl Transducer<I, D, O>
where
    O: Clone,
{
    epsilon(move |_i| out.clone())
}

impl<I, D, O, F> Clone for Epsilon<I, D, O, F>
where
    F: Fn(I) -> O + Clone,
{
    fn clone(&self) -> Self {
        epsilon(self.action.clone())
    }
}
impl<I, D, O, F> Transducer<I, D, O> for Epsilon<I, D, O, F>
where
    F: Fn(I) -> O,
{
    fn init(&mut self, i: Ext<I>) -> Ext<O> {
        ext_value::apply1(|x| (self.action)(x), i)
    }
    fn update(&mut self, _item: &D) -> Ext<O> {
        Ext::None
    }
    fn reset(&mut self) {}

    fn is_epsilon(&self) -> bool {
        true
    }
    fn is_restartable(&self) -> bool {
        true
    }
    fn n_states(&self) -> usize {
        // This would be 2 following the POPL definition, including 1 initial
        // and 1 final state. But we exclude the initial and final states here
        // from the implementation unless they need to be saved.
        0
    }
    fn n_transs(&self) -> usize {
        // We still record the fact that there is an epsilon transition, i.e.
        // the 'action' function.
        1
    }
}

/*
    QRE atom

    Base construct which processes a single data item and then
    produces output only if the data item satisfies a guard

    Derived constructs:

    - atom_univ
      Atom with no guard: applies some function to the input item

    - atom_guard
      Atom with no action: outputs () if item matches the guard

    - atom_iden
      Atom with no action or guard: just matches one item (any item) and
      outputs the initial input.

    - atom_item_iden
      Atom with no action or guard: just matches one item (any item) and
      outputs it.

    - atom_unit
      Atom with no action or guard: just matches one item (any item) and
      outputs ().
*/

pub struct Atom<I, D, O, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(I, &D) -> O,
{
    guard: G,
    action: F,
    istate: Ext<I>,
    ph_d: PhantomData<D>,
    ph_o: PhantomData<O>,
}
pub fn atom<I, D, O, G, F>(guard: G, action: F) -> Atom<I, D, O, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(I, &D) -> O,
{
    let istate = Ext::None;
    Atom { guard, action, istate, ph_d: PhantomData, ph_o: PhantomData }
}
pub fn atom_univ<I, D, O, F>(action: F) -> impl Transducer<I, D, O>
where
    F: Fn(I, &D) -> O,
{
    atom(|_d| true, action)
}
pub fn atom_guard<D, G>(guard: G) -> impl Transducer<(), D, ()>
where
    G: Fn(&D) -> bool,
{
    atom(guard, |(), _d| ())
}
pub fn atom_iden<I, D>() -> impl Transducer<I, D, I> {
    atom_univ(|i, _d| i)
}
pub fn atom_item_iden<D: Clone>() -> impl Transducer<(), D, D> {
    atom_univ(|(), d: &D| d.clone())
}
pub fn atom_unit<D>() -> impl Transducer<(), D, ()> {
    atom_univ(|(), _d| ())
}

impl<I, D, O, G, F> Clone for Atom<I, D, O, G, F>
where
    I: Clone,
    G: Fn(&D) -> bool + Clone,
    F: Fn(I, &D) -> O + Clone,
{
    fn clone(&self) -> Self {
        let mut new = atom(self.guard.clone(), self.action.clone());
        new.istate = self.istate.clone();
        new
    }
}
impl<I, D, O, G, F> Transducer<I, D, O> for Atom<I, D, O, G, F>
where
    G: Fn(&D) -> bool,
    F: Fn(I, &D) -> O,
{
    fn init(&mut self, i: Ext<I>) -> Ext<O> {
        self.istate += i;
        Ext::None
    }
    fn update(&mut self, item: &D) -> Ext<O> {
        let mut istate = Ext::None;
        mem::swap(&mut self.istate, &mut istate);
        if (self.guard)(&item) {
            ext_value::apply1(move |x| (self.action)(x, &item), istate)
        } else {
            Ext::None
        }
    }
    fn reset(&mut self) {
        self.istate = Ext::None;
    }

    fn is_epsilon(&self) -> bool {
        false
    }
    fn is_restartable(&self) -> bool {
        true
    }
    fn n_states(&self) -> usize {
        1
    }
    fn n_transs(&self) -> usize {
        1
    }
}

/*
    QRE union

    Processes the input stream and produces the union (+ on Ext<T>)
    of the two results.
*/

pub struct Union<I, D, O, M1, M2>
where
    M1: Transducer<I, D, O>,
    M2: Transducer<I, D, O>,
{
    m1: M1,
    m2: M2,
    ph_i: PhantomData<I>,
    ph_d: PhantomData<D>,
    ph_o: PhantomData<O>,
}
pub fn union<I, D, O, M1, M2>(m1: M1, m2: M2) -> Union<I, D, O, M1, M2>
where
    M1: Transducer<I, D, O>,
    M2: Transducer<I, D, O>,
{
    Union { m1, m2, ph_i: PhantomData, ph_d: PhantomData, ph_o: PhantomData }
}

impl<I, D, O, M1, M2> Clone for Union<I, D, O, M1, M2>
where
    M1: Transducer<I, D, O> + Clone,
    M2: Transducer<I, D, O> + Clone,
{
    fn clone(&self) -> Self {
        union(self.m1.clone(), self.m2.clone())
    }
}
impl<I, D, O, M1, M2> Transducer<I, D, O> for Union<I, D, O, M1, M2>
where
    I: Clone,
    M1: Transducer<I, D, O>,
    M2: Transducer<I, D, O>,
{
    fn init(&mut self, i: Ext<I>) -> Ext<O> {
        let i2 = i.clone();
        self.m1.init(i) + self.m2.init(i2)
    }
    fn update(&mut self, item: &D) -> Ext<O> {
        self.m1.update(item) + self.m2.update(item)
    }
    fn reset(&mut self) {
        self.m1.reset();
        self.m2.reset();
    }

    fn is_epsilon(&self) -> bool {
        self.m1.is_epsilon() && self.m2.is_epsilon()
    }
    fn is_restartable(&self) -> bool {
        self.m1.is_restartable() && self.m2.is_restartable()
    }
    fn n_states(&self) -> usize {
        self.m1.n_states() + self.m2.n_states()
    }
    fn n_transs(&self) -> usize {
        self.m1.n_transs() + self.m2.n_transs()
    }
}

/*
    QRE Parallel Composition

    Processes the input stream and produces an ordered pair
    of the two results.
*/

pub struct ParComp<I, D, O1, O2, M1, M2>
where
    M1: Transducer<I, D, O1>,
    M2: Transducer<I, D, O2>,
{
    m1: M1,
    m2: M2,
    ph_i: PhantomData<I>,
    ph_d: PhantomData<D>,
    ph_o1: PhantomData<O1>,
    ph_o2: PhantomData<O2>,
}
pub fn parcomp<I, D, O1, O2, M1, M2>(
    m1: M1,
    m2: M2,
) -> ParComp<I, D, O1, O2, M1, M2>
where
    M1: Transducer<I, D, O1>,
    M2: Transducer<I, D, O2>,
{
    ParComp {
        m1,
        m2,
        ph_i: PhantomData,
        ph_d: PhantomData,
        ph_o1: PhantomData,
        ph_o2: PhantomData,
    }
}

impl<I, D, O1, O2, M1, M2> Clone for ParComp<I, D, O1, O2, M1, M2>
where
    M1: Transducer<I, D, O1> + Clone,
    M2: Transducer<I, D, O2> + Clone,
{
    fn clone(&self) -> Self {
        parcomp(self.m1.clone(), self.m2.clone())
    }
}
impl<I, D, O1, O2, M1, M2> Transducer<I, D, (O1, O2)>
    for ParComp<I, D, O1, O2, M1, M2>
where
    I: Clone,
    M1: Transducer<I, D, O1>,
    M2: Transducer<I, D, O2>,
{
    fn init(&mut self, i: Ext<I>) -> Ext<(O1, O2)> {
        let i2 = i.clone();
        self.m1.init(i) * self.m2.init(i2)
    }
    fn update(&mut self, item: &D) -> Ext<(O1, O2)> {
        self.m1.update(item) * self.m2.update(item)
    }
    fn reset(&mut self) {
        self.m1.reset();
        self.m2.reset();
    }

    fn is_epsilon(&self) -> bool {
        self.m1.is_epsilon() && self.m2.is_epsilon()
    }
    fn is_restartable(&self) -> bool {
        // TODO: Requires checking if the languages of the two transducers
        // agree. Need more infrastructure to encode and analyze regular
        // languages.
        unimplemented!()
    }
    fn n_states(&self) -> usize {
        self.m1.n_states() + self.m2.n_states()
    }
    fn n_transs(&self) -> usize {
        self.m1.n_transs() + self.m2.n_transs()
    }
}

/*
    QRE concat

    Processes the input stream w and splits it into
        w = uv
    where u matches the first transducer and v matches the second transducer.
    Feeds the output of the first as the input of the second.
    If multiple matches, produces Ext::Many.

    Here we're using X instead of I, Z instead of O as this makes
    the intermediate type Y clearer.

    This and iteration are the most interesting constructs, and the ones
    where restartability on at least one sub-transducer is a requirement
    for the construction.
*/

pub struct Concat<D, X, Y, Z, M1, M2>
where
    M1: Transducer<X, D, Y>,
    M2: Transducer<Y, D, Z>,
{
    m1: M1,
    m2: M2,
    ph_d: PhantomData<D>,
    ph_x: PhantomData<X>,
    ph_y: PhantomData<Y>,
    ph_z: PhantomData<Z>,
}
pub fn concat<D, X, Y, Z, M1, M2>(m1: M1, m2: M2) -> Concat<D, X, Y, Z, M1, M2>
where
    M1: Transducer<X, D, Y>,
    M2: Transducer<Y, D, Z>,
{
    // REQUIREMENT: m2 must be restartable OR m1 must be an epsilon
    assert!(m2.is_restartable() || m1.is_epsilon());
    Concat {
        m1,
        m2,
        ph_d: PhantomData,
        ph_x: PhantomData,
        ph_y: PhantomData,
        ph_z: PhantomData,
    }
}

impl<D, X, Y, Z, M1, M2> Clone for Concat<D, X, Y, Z, M1, M2>
where
    M1: Transducer<X, D, Y> + Clone,
    M2: Transducer<Y, D, Z> + Clone,
{
    fn clone(&self) -> Self {
        concat(self.m1.clone(), self.m2.clone())
    }
}
impl<D, X, Y, Z, M1, M2> Transducer<X, D, Z> for Concat<D, X, Y, Z, M1, M2>
where
    M1: Transducer<X, D, Y>,
    M2: Transducer<Y, D, Z>,
{
    fn init(&mut self, i: Ext<X>) -> Ext<Z> {
        self.m2.init(self.m1.init(i))
    }
    fn update(&mut self, item: &D) -> Ext<Z> {
        let y = self.m1.update(item);
        let z1 = self.m2.update(item);
        let z2 = self.m2.init(y);
        z1 + z2
    }
    fn reset(&mut self) {
        self.m1.reset();
        self.m2.reset();
    }

    fn is_epsilon(&self) -> bool {
        // Concatenation of two epsilons is an epsilon.
        // Note: to prove .update() is equivalent to .reset() for the concat,
        // note that y is Ext::None, so self.m2.init(y) has no effect
        // by the property of .init() that should hold for any transducer.
        self.m1.is_epsilon() && self.m2.is_epsilon()
    }
    fn is_restartable(&self) -> bool {
        // There are two cases here: m2 was restartable on construction,
        // or m1 was epsilon on construction. In the first case for
        // restartability, m1 must be restartable. In the second case,
        // m2 must be restartable. Either way, this is equivalent to
        // saying that both m1 and m2 are restartable, since .is_epsilon()
        // implies .is_restartable().
        self.m1.is_restartable() && self.m2.is_restartable()
    }
    fn n_states(&self) -> usize {
        self.m1.n_states() + self.m2.n_states()
    }
    fn n_transs(&self) -> usize {
        self.m1.n_transs() + self.m2.n_transs()
    }
}

/*
    QRE iteration

    Parse the input stream as a sequence of matches, and apply the
    sub-transducer to each match.

    Like concatenation this is a more interesting construction that requires
    restartability. Additionally, iteration is the only construct where the
    update logic is more complex because the evaluation involves a feedback
    loop (result of .update() feeds back in as .init()).
*/

pub struct Iterate<X, D, M>
where
    M: Transducer<X, D, X>,
{
    m: M,
    // Tracks the accumulation of values we have .init() into m
    istate: Ext<()>,
    // True if m produced output in response to an .init() (degenerate case),
    // false if it does not produce such output, None if this is not known
    // yet.
    // Once we determine true or false, self.loopy never changes;
    // this is sound because the behavior of .init() is independent of the
    // context, which is not true in general but holds due to the requirement
    // that M is restartable.
    loopy: Option<bool>,
    ph_x: PhantomData<X>,
    ph_d: PhantomData<D>,
}
pub fn iterate<X, D, M>(m: M) -> Iterate<X, D, M>
where
    M: Transducer<X, D, X>,
{
    // REQUIREMENT: m must be restartable
    assert!(m.is_restartable());
    let istate = Ext::None;
    let loopy = None;
    Iterate { m, istate, loopy, ph_x: PhantomData, ph_d: PhantomData }
}

impl<X, D, M> Clone for Iterate<X, D, M>
where
    M: Transducer<X, D, X> + Clone,
{
    fn clone(&self) -> Self {
        let m = self.m.clone();
        let istate = self.istate;
        let loopy = self.loopy;
        Iterate { m, istate, loopy, ph_x: PhantomData, ph_d: PhantomData }
    }
}
impl<X, D, M> Transducer<X, D, X> for Iterate<X, D, M>
where
    X: Clone + Debug + Eq,
    M: Transducer<X, D, X>,
{
    fn init(&mut self, i: Ext<X>) -> Ext<X> {
        if i.is_none() {
            return Ext::None;
        }
        match self.loopy {
            Some(true) => {
                if cfg!(debug_assertions) {
                    self.istate = Ext::Many;
                    assert_eq!(self.m.init(Ext::Many), Ext::Many);
                } else if !self.istate.is_many() {
                    self.istate = Ext::Many;
                    self.m.init(Ext::Many);
                }
                Ext::Many
            }
            Some(false) => {
                if cfg!(debug_assertions) {
                    self.istate += i.to_unit();
                    assert_eq!(self.m.init(i.clone()), Ext::None);
                } else if !self.istate.is_many() {
                    self.istate += i.to_unit();
                    self.m.init(i.clone());
                }
                // Return the input (epsilon/identity case)
                i
            }
            None => {
                // This is where we find out if m is loopy
                debug_assert!(self.istate.is_none());
                self.istate = i.to_unit();
                let out = self.m.init(i.clone());
                if out.is_none() {
                    // Not loopy
                    self.loopy = Some(false);
                    // Return the input (epsilon/identity case)
                    i
                } else {
                    // Loopy; set this knowledge and rerun function
                    // with new output
                    self.loopy = Some(true);
                    self.init(out)
                }
            }
        }
    }
    fn update(&mut self, item: &D) -> Ext<X> {
        self.istate = Ext::None;
        let sub_out = self.m.update(item);
        self.init(sub_out)
    }
    fn reset(&mut self) {
        self.m.reset();
        self.istate = Ext::None;
        // Don't need to reset self.loopy; this information will remain valid
    }

    fn is_epsilon(&self) -> bool {
        self.m.is_epsilon()
    }
    fn is_restartable(&self) -> bool {
        // m was restartable on construction, so this should always be true.
        debug_assert!(self.m.is_restartable());
        true
    }
    fn n_states(&self) -> usize {
        self.m.n_states() + 1
    }
    fn n_transs(&self) -> usize {
        self.m.n_transs()
    }
}

/*
    QRE aggregate (aka "prefix sum")

    Each time the sub-transducer matches, apply a 'sum' function
    to get the new aggregate and produce it as output. Sum can be any abstract
    sequential function Z x Y -> Z; it doesn't need to be commutative or
    associative.

    This transducer is not restartable because it is not in general possible
    to store all aggregates of several computations simultaneously using
    a finite number of state variables.
    So, .init() should be called once at the start of computation; if it is
    not called or called multiple times (or called with Ext::Many), the
    behavior of the implementation may not be what is desired or even
    anything reasonable.
    It is possible to be restartable in some special cases, in particular
    if the sub-transducer matches on all input streams, but we do not
    currently detect these cases.
*/

pub struct Aggregate<D, X, Y, Z, M, F>
where
    M: Transducer<X, D, Y>,
    F: Fn(Z, Y) -> Z,
{
    m: M,
    agg_fun: F,
    // The most recently produced aggregate
    agg: Ext<Z>,
    ph_d: PhantomData<D>,
    ph_x: PhantomData<X>,
    ph_y: PhantomData<Y>,
}
pub fn aggregate<D, X, Y, Z, M, F>(
    m: M,
    agg_fun: F,
) -> Aggregate<D, X, Y, Z, M, F>
where
    M: Transducer<X, D, Y>,
    F: Fn(Z, Y) -> Z,
{
    Aggregate {
        m,
        agg_fun,
        agg: Ext::None,
        ph_d: PhantomData,
        ph_x: PhantomData,
        ph_y: PhantomData,
    }
}

impl<D, X, Y, Z, M, F> Aggregate<D, X, Y, Z, M, F>
where
    Z: Clone,
    M: Transducer<X, D, Y>,
    F: Fn(Z, Y) -> Z,
{
    // Auxiliary function used by both .init and .update
    // Update the aggregate and return the new result (if any)
    fn update_agg(&mut self, y: Ext<Y>) -> Ext<Z> {
        if y.is_none() {
            Ext::None
        } else {
            let mut tmp = Ext::None;
            mem::swap(&mut tmp, &mut self.agg);
            self.agg = ext_value::apply2(&self.agg_fun, tmp, y);
            self.agg.clone()
        }
    }
}
impl<D, X, Y, Z, M, F> Clone for Aggregate<D, X, Y, Z, M, F>
where
    Z: Clone,
    M: Transducer<X, D, Y> + Clone,
    F: Fn(Z, Y) -> Z + Clone,
{
    fn clone(&self) -> Self {
        let mut result = aggregate(self.m.clone(), self.agg_fun.clone());
        result.agg = self.agg.clone();
        result
    }
}
impl<D, X, Y, Z, M, F> Transducer<(X, Z), D, Z> for Aggregate<D, X, Y, Z, M, F>
where
    Z: Clone,
    M: Transducer<X, D, Y>,
    F: Fn(Z, Y) -> Z,
{
    fn init(&mut self, i: Ext<(X, Z)>) -> Ext<Z> {
        let (x, z) = i.split(|(x, z)| (x, z));
        let y = self.m.init(x);
        self.agg += z;
        self.update_agg(y)
    }
    fn update(&mut self, item: &D) -> Ext<Z> {
        let y = self.m.update(item);
        self.update_agg(y)
    }
    fn reset(&mut self) {
        self.m.reset();
        self.agg = Ext::None;
    }

    fn is_epsilon(&self) -> bool {
        self.m.is_epsilon()
    }
    fn is_restartable(&self) -> bool {
        false
    }
    fn n_states(&self) -> usize {
        self.m.n_states() + 1
    }
    fn n_transs(&self) -> usize {
        self.m.n_transs() + 1
    }
}

/*
    QRE additional derived constructs

    - stream_iden.
      Match the entire input stream (any input stream) and apply the
      identity function. Analagous to atom_iden and epsilon_iden.

    - repeat
      Repeat a constant item initially and on every update
      (In case multiple .inits() or .init(Ext::Many), obeys restartability
      semantics)

    - map
      Apply a function to every item in the input stream

    - apply_op
      Apply a function to the outputs of two transducers.
      (This is parcomp followed by an epsilon.)
      (More versions of this could be written for ops of differing arities.)
*/

pub fn stream_iden<I, D>() -> impl Transducer<I, D, I>
where
    I: Clone + Debug + Eq,
{
    iterate(atom_iden())
}

pub fn repeat<D, O>(out: O) -> impl Transducer<(), D, O>
where
    O: Clone,
{
    concat(stream_iden(), epsilon_const(out))
}

pub fn map<D, E, F>(map_fun: F) -> impl Transducer<(), D, E>
where
    F: Fn(&D) -> E,
{
    concat(stream_iden(), atom_univ(move |(), d| map_fun(d)))
}

pub fn apply_op<I, D, O1, O2, O, M1, M2, F>(
    m1: M1,
    m2: M2,
    op: F,
) -> impl Transducer<I, D, O>
where
    I: Clone,
    M1: Transducer<I, D, O1>,
    M2: Transducer<I, D, O2>,
    F: Fn(O1, O2) -> O,
{
    concat(parcomp(m1, m2), epsilon(move |(o1, o2)| op(o1, o2)))
}

/*
    QRE transducer top-level wrapper

    For now, all this does is save the number of states, number of transitions,
    epsilon-ness, and restartability as this is more efficient than recomputing
    them all the time.
*/

pub struct TopWrapper<I, D, O, M>
where
    M: Transducer<I, D, O>,
{
    m: M,
    ph_i: PhantomData<I>,
    ph_d: PhantomData<D>,
    ph_o: PhantomData<O>,
    epsilon: bool,
    restartable: bool,
    n_states: usize,
    n_transs: usize,
}
pub fn top<I, D, O, M>(m: M) -> TopWrapper<I, D, O, M>
where
    M: Transducer<I, D, O>,
{
    let epsilon = m.is_epsilon();
    let restartable = m.is_restartable();
    let n_states = m.n_states();
    let n_transs = m.n_transs();
    TopWrapper {
        m,
        ph_i: PhantomData,
        ph_d: PhantomData,
        ph_o: PhantomData,
        epsilon,
        restartable,
        n_states,
        n_transs,
    }
}

impl<I, D, O, M> Clone for TopWrapper<I, D, O, M>
where
    M: Transducer<I, D, O> + Clone,
{
    fn clone(&self) -> Self {
        top(self.m.clone())
    }
}
impl<I, D, O, M> Transducer<I, D, O> for TopWrapper<I, D, O, M>
where
    M: Transducer<I, D, O>,
{
    fn init(&mut self, i: Ext<I>) -> Ext<O> {
        self.m.init(i)
    }
    fn update(&mut self, item: &D) -> Ext<O> {
        self.m.update(item)
    }
    fn reset(&mut self) {
        self.m.reset();
    }

    fn is_epsilon(&self) -> bool {
        self.epsilon
    }
    fn is_restartable(&self) -> bool {
        self.restartable
    }
    fn n_states(&self) -> usize {
        self.n_states
    }
    fn n_transs(&self) -> usize {
        self.n_transs
    }
}

/*
    Unit Tests
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::RInput;

    // Constants (examples)

    const EX_RSTRM_1: &[RInput<i32, char>] = &[
        RInput::Item('a'),
        RInput::Restart(3),
        RInput::Item('b'),
        RInput::Restart(4),
        RInput::Restart(6),
        RInput::Item('c'),
    ];

    const EX_RSTRM_2: &[RInput<i32, char>] = &[RInput::Item('b')];

    const EX_RSTRM_3: &[RInput<i32, char>] = &[RInput::Restart(3)];

    const EX_RSTRM_4: &[RInput<i32, char>] = &[];

    const EX_RSTRM_5: &[RInput<i32, char>] = &[
        RInput::Restart(3),
        RInput::Item('a'),
        RInput::Item('b'),
        RInput::Restart(4),
        RInput::Item('c'),
        RInput::Restart(5),
        RInput::Restart(6),
        RInput::Item('b'),
    ];

    const EX_RSTRM_6: &[RInput<i32, char>] = &[
        RInput::Restart(1),
        RInput::Item('2'),
        RInput::Restart(3),
        RInput::Item('4'),
    ];

    const EX_RSTRMS: &[&[RInput<i32, char>]] = &[
        EX_RSTRM_1, EX_RSTRM_2, EX_RSTRM_3, EX_RSTRM_4, EX_RSTRM_5, EX_RSTRM_6,
    ];

    // Test helpers

    fn test_equiv<O, M1, M2>(mut m1: M1, mut m2: M2)
    where
        M1: Transducer<i32, char, O>,
        M2: Transducer<i32, char, O>,
        O: Debug + PartialEq,
    {
        // Try to test if two transducers are the same
        assert_eq!(m1.is_restartable(), m2.is_restartable());
        assert_eq!(m1.n_states(), m2.n_states());
        assert_eq!(m1.n_transs(), m2.n_transs());
        for rstrm in EX_RSTRMS {
            let rstrm1 = rstrm.iter().cloned();
            let rstrm2 = rstrm.iter().cloned();
            assert_eq!(
                m1.process_rstream_single(rstrm1).collect::<Vec<Ext<O>>>(),
                m2.process_rstream_single(rstrm2).collect::<Vec<Ext<O>>>(),
            );
        }
    }

    fn test_restartable<O, M>(m: &M)
    where
        M: Transducer<i32, char, O> + Clone,
        O: Debug + Eq,
    {
        // TODO: uncomment this line when restartability variable
        // is implemented for parcomp
        // assert!(m.is_restartable());
        for rstrm in EX_RSTRMS {
            assert!(m.restartability_holds_for(rstrm.iter().cloned()));
        }
    }

    fn test_not_restartable<O, M>(m: &M)
    where
        M: Transducer<i32, char, O> + Clone,
        O: Debug + Eq,
    {
        // TODO: uncomment this line when restartability variable
        // is implemented for parcomp
        // assert!(!m.is_restartable());
        for rstrm in EX_RSTRMS {
            if !(m.restartability_holds_for(rstrm.iter().cloned())) {
                return;
            }
        }
        panic!("Not-restartable test failed: no counterexample stream found");
    }

    // The tests

    #[test]
    fn test_epsilon() {
        let mut m1 = epsilon(|i: i32| i + 1);
        assert_eq!(m1.init_one(1), Ext::One(2));
        assert_eq!(m1.init_one(-4), Ext::One(-3));
        assert_eq!(m1.update_val('a'), Ext::None);
        assert_eq!(m1.update_val('b'), Ext::None);
        let mut m2 = epsilon(|_i: i32| 0);
        assert_eq!(m2.update_val('a'), Ext::None);
        assert_eq!(m2.update_val('a'), Ext::None);
        assert_eq!(m2.init_one(3), Ext::One(0));
        let mut m3 = epsilon(|s: String| s + "ab");
        assert_eq!(m3.init_one("xyz".to_owned()), Ext::One("xyzab".to_owned()));
        assert_eq!(m3.update_val('a'), Ext::None);
        assert_eq!(m3.update_val('a'), Ext::None);
    }
    #[test]
    fn test_epsilon_process() {
        // We probably do not need to write separate tests using .process_stream
        // for all constructs, but useful to have at least one test using it
        let mut m1 = epsilon(|i: i32| i + 2);
        let strm1 = vec!['a', 'b'].into_iter();
        let strm2 = vec![].into_iter();
        assert_eq!(
            m1.process_stream(2, strm1).collect::<Vec<Ext<i32>>>(),
            vec![Ext::One(4), Ext::None, Ext::None],
        );
        assert_eq!(
            m1.process_stream(3, strm2).collect::<Vec<Ext<i32>>>(),
            vec![Ext::One(5)],
        );
    }
    #[test]
    fn test_epsilon_restartable() {
        let m1 = epsilon(|i: i32| i * 2);
        test_restartable(&m1);
    }

    #[test]
    fn test_atom() {
        let mut m = atom(
            |ch: &char| ch.is_ascii_digit(),
            |i, ch| format!("{}{}", i, ch),
        );
        assert_eq!(m.update_val('a'), Ext::None);
        assert_eq!(m.init_one("x".to_string()), Ext::None);
        assert_eq!(m.update_val('a'), Ext::None);
        assert_eq!(m.init_one("x".to_string()), Ext::None);
        assert_eq!(m.update_val('1'), Ext::One("x1".to_string()));
        assert_eq!(m.update_val('2'), Ext::None);
        assert_eq!(m.init_one("x".to_string()), Ext::None);
        assert_eq!(m.init_one("y".to_string()), Ext::None);
        assert_eq!(m.update_val('1'), Ext::Many);
        assert_eq!(m.update_val('2'), Ext::None);
        assert_eq!(m.update_val('3'), Ext::None);
        assert_eq!(m.init_one("".to_string()), Ext::None);
        assert_eq!(m.update_val('1'), Ext::One("1".to_string()));
    }
    #[test]
    fn test_atom_restartable() {
        let m1 = atom(|&ch| ch == 'b', |i, _ch| i + 2);
        let m2 = atom(
            |&ch| ch == 'b' || ch == 'c',
            |i, &ch| {
                if ch == 'b' {
                    i + 2
                } else {
                    i + 1
                }
            },
        );
        let m3 = atom(|_ch| true, |i, _ch| i + 3);
        test_restartable(&m1);
        test_restartable(&m2);
        test_restartable(&m3);
    }

    #[test]
    fn test_union() {
        let m1 = atom(
            |ch: &char| ch.is_ascii_digit(),
            |i, ch| i + (ch.to_digit(10).unwrap() as i32),
        );
        let m2 = epsilon(|i: i32| i + 1);
        let mut m = union(m1, m2);

        assert_eq!(m.update_val('1'), Ext::None);
        assert_eq!(m.init_one(3), Ext::One(4));
        assert_eq!(m.update_val('7'), Ext::One(10));
        assert_eq!(m.update_val('2'), Ext::None);
        assert_eq!(m.init_one(0), Ext::One(1));
        assert_eq!(m.update_val('a'), Ext::None);

        test_restartable(&m);
    }

    #[test]
    fn test_parcomp() {
        let m1 = atom(
            |ch: &char| ch.is_ascii_digit(),
            |i, ch| i + (ch.to_digit(10).unwrap() as i32),
        );
        let m2 = atom(|ch: &char| ch == &'5', |i, _ch| i + 1);
        let mut m = parcomp(m1, m2);

        assert_eq!(m.init_one(10), Ext::None);
        assert_eq!(m.update_val('5'), Ext::One((15, 11)));
        assert_eq!(m.update_val('5'), Ext::None);
        assert_eq!(m.init_one(10), Ext::None);
        assert_eq!(m.update_val('6'), Ext::None);
        assert_eq!(m.init_one(100), Ext::None);
        assert_eq!(m.update_val('5'), Ext::One((105, 101)));

        // this m is restartable
        test_restartable(&m);
    }

    #[test]
    fn test_parcomp_not_restarable() {
        // Non-restartable example
        let m1 = atom(
            |ch: &char| ch.is_ascii_digit(),
            |i, ch| i + (ch.to_digit(10).unwrap() as i32),
        );
        let m2 = concat(m1.clone(), m1.clone());
        let m = parcomp(m1, m2);

        // this m is not restartable
        test_not_restartable(&m);
    }

    #[test]
    fn test_concat() {
        let m1 = atom(|ch: &char| ch.is_ascii_digit(), |i, _ch| i + 1);
        let m2 = atom(|ch: &char| *ch == '1' || *ch == 'a', |i, _ch| i + 1);
        let mut m = concat(m1, m2);

        assert_eq!(m.update_val('1'), Ext::None);
        assert_eq!(m.update_val('1'), Ext::None);
        assert_eq!(m.init_one(0), Ext::None);
        assert_eq!(m.update_val('5'), Ext::None);
        assert_eq!(m.update_val('1'), Ext::One(2));
        assert_eq!(m.update_val('1'), Ext::None);

        assert_eq!(m.init_one(1), Ext::None);
        assert_eq!(m.update_val('1'), Ext::None);
        assert_eq!(m.init_one(2), Ext::None);
        assert_eq!(m.update_val('2'), Ext::None);
        assert_eq!(m.init_one(3), Ext::None);
        assert_eq!(m.update_val('1'), Ext::One(4));
        assert_eq!(m.init_one(4), Ext::None);
        assert_eq!(m.update_val('1'), Ext::One(5));
        assert_eq!(m.init_one(5), Ext::None);
        assert_eq!(m.update_val('a'), Ext::One(6));
        assert_eq!(m.update_val('1'), Ext::None);
        assert_eq!(m.update_val('1'), Ext::None);

        test_restartable(&m);
    }

    #[test]
    fn test_iterate() {
        let m1 = atom(|ch: &char| ch.is_ascii_digit(), |i, _ch| i + 1);
        let m2 = atom(|_ch: &char| true, |i, _ch| i + 1);
        let m3 = concat(m1, m2);
        let mut m = iterate(m3);

        assert_eq!(m.update_val('1'), Ext::None);
        assert_eq!(m.init_one(100), Ext::One(100));
        assert_eq!(m.update_val('0'), Ext::None);
        assert_eq!(m.update_val('0'), Ext::One(102));
        assert_eq!(m.update_val('0'), Ext::None);
        assert_eq!(m.update_val('0'), Ext::One(104));
        assert_eq!(m.update_val('0'), Ext::None);
        assert_eq!(m.init_one(200), Ext::One(200));
        assert_eq!(m.update_val('0'), Ext::One(106));
        assert_eq!(m.update_val('0'), Ext::One(202));
        assert_eq!(m.update_val('0'), Ext::One(108));
        assert_eq!(m.update_val('0'), Ext::One(204));
        assert_eq!(m.init_one(300), Ext::One(300));
        assert_eq!(m.update_val('0'), Ext::One(110));
        assert_eq!(m.update_val('0'), Ext::Many);
        assert_eq!(m.update_val('0'), Ext::One(112));
        assert_eq!(m.update_val('0'), Ext::Many);
        assert_eq!(m.update_val('a'), Ext::One(114));
        assert_eq!(m.update_val('0'), Ext::None);
        assert_eq!(m.update_val('0'), Ext::One(116));
        assert_eq!(m.update_val('a'), Ext::None);
        assert_eq!(m.update_val('0'), Ext::None);

        test_restartable(&m);
    }

    #[test]
    fn test_aggregate() {
        let m1 = atom(|ch: &char| ch.is_ascii_digit(), |i, _ch| i + 1);
        let m2 = iterate(m1);
        let mut m = aggregate(m2, |x1, x2| x1 + x2);

        // Aggregating 1, 2, 3, 4, ... gives the triangular numbers
        assert_eq!(m.init_one((1, 100)), Ext::One(101));
        assert_eq!(m.update_val('0'), Ext::One(103));
        assert_eq!(m.update_val('0'), Ext::One(106));
        assert_eq!(m.update_val('0'), Ext::One(110));
        assert_eq!(m.update_val('0'), Ext::One(115));

        // If the sub-transducer stops producing output, the aggregate
        // becomes None
        assert_eq!(m.update_val('a'), Ext::None);
        assert_eq!(m.update_val('0'), Ext::None);
        assert_eq!(m.update_val('0'), Ext::None);

        // Aggregate is not restartable
        // (convert input first)
        let m = concat(epsilon(|x| (x, x)), m);
        test_not_restartable(&m);
    }

    #[test]
    fn test_top_wrapper() {
        let m1 = epsilon(|i: i32| i + 2);
        let m2 = atom(|&ch: &char| ch == 'a', |i, _ch| i + 3);
        let m3 = union(m1.clone(), m2.clone());
        let m4 = union(top(m1.clone()), top(top(m2.clone())));
        let t1 = top(m1.clone());
        let t2 = top(m2.clone());
        let t3 = top(m3.clone());
        let t4 = top(m4.clone());
        test_equiv(m1, t1);
        test_equiv(m2, t2);
        test_equiv(m3, t3);
        test_equiv(m4, t4);
    }
}
