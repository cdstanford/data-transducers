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

use super::ext_value::Ext;
use super::interface::Transducer;
use std::marker::PhantomData;

/*
    Functions in the transducer need to be clonable --
    this is a convenience trait to summarize that.
*/
pub trait FnClone<I, O>: Fn(I) -> O + Clone {}
impl<I, O, F: Fn(I) -> O + Clone> FnClone<I, O> for F {}

/*
    Atomic base constructs
*/

pub struct Epsilon<I, D, O, F: FnClone<I, O>> {
    ph_i: PhantomData<I>,
    ph_d: PhantomData<D>,
    ph_o: PhantomData<O>,
    f: F,
}
impl<I, D, O, F: FnClone<I, O>> Epsilon<I, D, O, F> {
    fn new(f: F) -> Self {
        Self { f, ph_i: PhantomData, ph_d: PhantomData, ph_o: PhantomData }
    }
}
impl<I, D, O, F: FnClone<I, O>> Clone for Epsilon<I, D, O, F> {
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            ph_d: PhantomData,
            ph_i: PhantomData,
            ph_o: PhantomData,
        }
    }
}
impl<I, D, O, F: FnClone<I, O>> Transducer<I, D, O> for Epsilon<I, D, O, F> {
    fn init(&mut self, i: I) -> Ext<O> {
        Ext::One((self.f)(i))
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
        2
    }
    fn n_transs(&self) -> usize {
        1
    }
}
pub fn epsilon<I, D, O, F: FnClone<I, O>>(f: F) -> Epsilon<I, D, O, F> {
    Epsilon::new(f)
}

/*
    Unit Tests
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::RInput;

    const EX_RSTRM_1: &[RInput<i32, char>] = &[
        RInput::Item('b'),
        RInput::Restart(3),
        RInput::Item('c'),
        RInput::Restart(4),
        RInput::Restart(6),
        RInput::Item('d'),
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
        RInput::Item('d'),
    ];

    const EX_RSTRMS: &[&[RInput<i32, char>]] =
        &[EX_RSTRM_1, EX_RSTRM_2, EX_RSTRM_3, EX_RSTRM_4, EX_RSTRM_5];

    #[test]
    fn test_epsilon() {
        let mut m1 = epsilon(|i: i32| i + 1);
        let mut m2 = epsilon(|_i: i32| 0);
        assert_eq!(m1.init(1), Ext::One(2));
        assert_eq!(m1.init(-4), Ext::One(-3));
        assert_eq!(m1.update('a'), Ext::None);
        assert_eq!(m1.update('b'), Ext::None);
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
            m1.check_restartability_for(rstrm.iter().cloned())
        }
    }
}
