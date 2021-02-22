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
    this is a convenience trait to summarize that.
*/
pub trait FnClone0<O>: Fn() -> O + Clone {}
impl<O, F: Fn() -> O + Clone> FnClone0<O> for F {}

pub trait FnClone1<I, O>: Fn(I) -> O + Clone {}
impl<I, O, F: Fn(I) -> O + Clone> FnClone1<I, O> for F {}

pub trait FnClone2<I1, I2, O>: Fn(I1, I2) -> O + Clone {}
impl<I1, I2, O, F: Fn(I1, I2) -> O + Clone> FnClone2<I1, I2, O> for F {}

/*
    QRE epsilon --
    Base construct which processes no data and immediately produces output
*/

pub struct Epsilon<I, D, O, F: FnClone1<I, O>> {
    action: F,
    ph_i: PhantomData<I>,
    ph_d: PhantomData<D>,
    ph_o: PhantomData<O>,
}
impl<I, D, O, F: FnClone1<I, O>> Epsilon<I, D, O, F> {
    fn new(action: F) -> Self {
        Self { action, ph_i: PhantomData, ph_d: PhantomData, ph_o: PhantomData }
    }
}
impl<I, D, O, F: FnClone1<I, O>> Clone for Epsilon<I, D, O, F> {
    fn clone(&self) -> Self {
        Self::new(self.action.clone())
    }
}
impl<I, D, O, F: FnClone1<I, O>> Transducer<I, D, O> for Epsilon<I, D, O, F> {
    fn init(&mut self, i: I) -> Ext<O> {
        Ext::One((self.action)(i))
    }
    fn update(&mut self, _item: D) -> Ext<O> {
        Ext::None
    }
    fn reset(&mut self) {}
    fn spawn_empty(&self) -> Self {
        self.clone()
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
pub fn epsilon<I, D, O, F: FnClone1<I, O>>(action: F) -> Epsilon<I, D, O, F> {
    Epsilon::new(action)
}

/*
    QRE atom --
    Base construct which processes a single data item and then
    produces output only if the data item satisfies a guard
*/

pub struct Atom<I, D, O, G: FnClone1<D, bool>, F: FnClone2<I, D, O>> {
    guard: G,
    action: F,
    istate: Ext<I>,
    ph_d: PhantomData<D>,
    ph_o: PhantomData<O>,
}
impl<I, D, O, G: FnClone1<D, bool>, F: FnClone2<I, D, O>> Atom<I, D, O, G, F> {
    fn new(guard: G, action: F) -> Self {
        Self {
            guard,
            action,
            istate: Ext::None,
            ph_d: PhantomData,
            ph_o: PhantomData,
        }
    }
}
impl<I: Clone, D, O, G: FnClone1<D, bool>, F: FnClone2<I, D, O>> Clone
    for Atom<I, D, O, G, F>
{
    fn clone(&self) -> Self {
        let mut new = self.spawn_empty();
        new.istate = self.istate.clone();
        new
    }
}
impl<I: Clone, D, O, G: FnClone1<D, bool>, F: FnClone2<I, D, O>>
    Transducer<I, D, O> for Atom<I, D, O, G, F>
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
    fn spawn_empty(&self) -> Self {
        Self::new(self.guard.clone(), self.action.clone())
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
pub fn atom<I, D, O, G: FnClone1<D, bool>, F: FnClone2<I, D, O>>(
    guard: G,
    action: F,
) -> Atom<I, D, O, G, F> {
    Atom::new(guard, action)
}

/*
    Unit Tests
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::RInput;

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
}
