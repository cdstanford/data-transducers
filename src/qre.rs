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
use std::marker::PhantomData;
use std::mem;

/*
    Functions in the transducer need to be clonable --
    we define convenience traits to summarize that.
*/
pub trait FnClone0<O>: Fn() -> O + Clone {}
impl<O, F: Fn() -> O + Clone> FnClone0<O> for F {}

pub trait FnClone1<I, O>: Fn(I) -> O + Clone {}
impl<I, O, F: Fn(I) -> O + Clone> FnClone1<I, O> for F {}

pub trait FnClone2<I1, I2, O>: Fn(I1, I2) -> O + Clone {}
impl<I1, I2, O, F: Fn(I1, I2) -> O + Clone> FnClone2<I1, I2, O> for F {}

/*
    QRE epsilon

    Base construct which processes no data and immediately produces output
*/

pub struct Epsilon<I, D, O, F>
where
    F: FnClone1<I, O>,
{
    action: F,
    ph_i: PhantomData<I>,
    ph_d: PhantomData<D>,
    ph_o: PhantomData<O>,
}
pub fn epsilon<I, D, O, F>(action: F) -> Epsilon<I, D, O, F>
where
    F: FnClone1<I, O>,
{
    Epsilon { action, ph_i: PhantomData, ph_d: PhantomData, ph_o: PhantomData }
}

impl<I, D, O, F> Clone for Epsilon<I, D, O, F>
where
    F: FnClone1<I, O>,
{
    fn clone(&self) -> Self {
        epsilon(self.action.clone())
    }
}
impl<I, D, O, F> Transducer<I, D, O> for Epsilon<I, D, O, F>
where
    F: FnClone1<I, O>,
{
    fn init(&mut self, i: I) -> Ext<O> {
        Ext::One((self.action)(i))
    }
    fn update(&mut self, _item: D) -> Ext<O> {
        Ext::None
    }
    fn reset(&mut self) {}

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
*/

pub struct Atom<I, D, O, G, F>
where
    G: FnClone1<D, bool>,
    F: FnClone2<I, D, O>,
{
    guard: G,
    action: F,
    istate: Ext<I>,
    ph_d: PhantomData<D>,
    ph_o: PhantomData<O>,
}
pub fn atom<I, D, O, G, F>(guard: G, action: F) -> Atom<I, D, O, G, F>
where
    G: FnClone1<D, bool>,
    F: FnClone2<I, D, O>,
{
    let istate = Ext::None;
    Atom { guard, action, istate, ph_d: PhantomData, ph_o: PhantomData }
}

impl<I, D, O, G, F> Clone for Atom<I, D, O, G, F>
where
    I: Clone,
    G: FnClone1<D, bool>,
    F: FnClone2<I, D, O>,
{
    fn clone(&self) -> Self {
        let mut new = atom(self.guard.clone(), self.action.clone());
        new.istate = self.istate.clone();
        new
    }
}
impl<I, D, O, G, F> Transducer<I, D, O> for Atom<I, D, O, G, F>
where
    I: Clone,
    G: FnClone1<D, bool>,
    F: FnClone2<I, D, O>,
{
    fn init(&mut self, i: I) -> Ext<O> {
        self.istate += Ext::One(i);
        Ext::None
    }
    fn update(&mut self, item: D) -> Ext<O> {
        let mut istate = Ext::None;
        mem::swap(&mut self.istate, &mut istate);
        ext_value::apply1(move |x| (self.action)(x, item), istate)
    }
    fn reset(&mut self) {
        self.istate = Ext::None;
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
    M1: Transducer<I, D, O>,
    M2: Transducer<I, D, O>,
{
    fn clone(&self) -> Self {
        union(self.m1.clone(), self.m2.clone())
    }
}
impl<I, D, O, M1, M2> Transducer<I, D, O> for Union<I, D, O, M1, M2>
where
    I: Clone,
    D: Clone,
    M1: Transducer<I, D, O>,
    M2: Transducer<I, D, O>,
{
    fn init(&mut self, i: I) -> Ext<O> {
        let i2 = i.clone();
        self.m1.init(i) + self.m2.init(i2)
    }
    fn update(&mut self, item: D) -> Ext<O> {
        let item2 = item.clone();
        self.m1.update(item) + self.m2.update(item2)
    }
    fn reset(&mut self) {
        self.m1.reset();
        self.m2.reset();
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
    QRE transducer top-level wrapper

    For now, all this does is saves the number of states, number of transitions,
    and restartability as this is more efficient than recomputing them all the
    time.
*/
pub struct TopWrapper<I, D, O, M>
where
    M: Transducer<I, D, O>,
{
    m: M,
    ph_i: PhantomData<I>,
    ph_d: PhantomData<D>,
    ph_o: PhantomData<O>,
    restartable: bool,
    n_states: usize,
    n_transs: usize,
}
pub fn top<I, D, O, M>(m: M) -> TopWrapper<I, D, O, M>
where
    M: Transducer<I, D, O>,
{
    let restartable = m.is_restartable();
    let n_states = m.n_states();
    let n_transs = m.n_transs();
    TopWrapper {
        m,
        ph_i: PhantomData,
        ph_d: PhantomData,
        ph_o: PhantomData,
        restartable,
        n_states,
        n_transs,
    }
}
impl<I, D, O, M> Clone for TopWrapper<I, D, O, M>
where
    M: Transducer<I, D, O>,
{
    fn clone(&self) -> Self {
        top(self.m.clone())
    }
}
impl<I, D, O, M> Transducer<I, D, O> for TopWrapper<I, D, O, M>
where
    M: Transducer<I, D, O>,
{
    fn init(&mut self, i: I) -> Ext<O> {
        self.m.init(i)
    }
    fn update(&mut self, item: D) -> Ext<O> {
        self.m.update(item)
    }
    fn reset(&mut self) {
        self.m.reset();
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
    use std::fmt::Debug;

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

    const EX_RSTRMS: &[&[RInput<i32, char>]] =
        &[EX_RSTRM_1, EX_RSTRM_2, EX_RSTRM_3, EX_RSTRM_4, EX_RSTRM_5];

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

    #[test]
    fn test_epsilon() {
        let mut m1 = epsilon(|i: i32| i + 1);
        assert_eq!(m1.init(1), Ext::One(2));
        assert_eq!(m1.init(-4), Ext::One(-3));
        assert_eq!(m1.update('a'), Ext::None);
        assert_eq!(m1.update('b'), Ext::None);
        let mut m2 = epsilon(|_i: i32| 0);
        assert_eq!(m2.update('a'), Ext::None);
        assert_eq!(m2.update('a'), Ext::None);
        assert_eq!(m2.init(3), Ext::One(0));
        let mut m3 = epsilon(|s: String| s + "ab");
        assert_eq!(m3.init("xyz".to_owned()), Ext::One("xyzab".to_owned()));
        assert_eq!(m3.update('a'), Ext::None);
        assert_eq!(m3.update('a'), Ext::None);
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
        for rstrm in EX_RSTRMS {
            m1.check_restartability_for(rstrm.iter().cloned());
        }
    }

    #[test]
    fn test_atom() {
        let mut m = atom(
            |ch: char| ch.is_ascii_digit(),
            |i, ch| format!("{}{}", i, ch),
        );
        assert_eq!(m.update('a'), Ext::None);
        assert_eq!(m.init("x".to_string()), Ext::None);
        assert_eq!(m.update('1'), Ext::One("x1".to_string()));
        assert_eq!(m.update('2'), Ext::None);
        assert_eq!(m.init("x".to_string()), Ext::None);
        assert_eq!(m.init("y".to_string()), Ext::None);
        assert_eq!(m.update('1'), Ext::Many);
        assert_eq!(m.update('2'), Ext::None);
        assert_eq!(m.update('3'), Ext::None);
        assert_eq!(m.init("".to_string()), Ext::None);
        assert_eq!(m.update('1'), Ext::One("1".to_string()));
    }

    #[test]
    fn test_atom_restartable() {
        let m1 = atom(|ch| ch == 'b', |i, _ch| i + 2);
        let m2 = atom(
            |ch| ch == 'b' || ch == 'c',
            |i, ch| {
                if ch == 'b' {
                    i + 2
                } else {
                    i + 1
                }
            },
        );
        let m3 = atom(|_ch| true, |i, _ch| i + 3);
        for rstrm in EX_RSTRMS {
            m1.check_restartability_for(rstrm.iter().cloned());
            m2.check_restartability_for(rstrm.iter().cloned());
            m3.check_restartability_for(rstrm.iter().cloned());
        }
    }

    #[test]
    fn test_union() {
        // TODO
    }

    #[test]
    fn test_union_restartable() {
        // TODO
    }

    #[test]
    fn test_top_wrapper() {
        let m1 = epsilon(|i: i32| i + 2);
        let m2 = atom(|ch: char| ch == 'a', |i, _ch| i + 3);
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
