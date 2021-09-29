/*
    Module implementing "extended values" Ext<T> --
    None, One (with a value in T), or Many.

    Ext<T> can be thought variant of Option<T>, where Many
    represents a multiset of two or more values.
*/

use derive_more::{Display, From};
use std::iter::FromIterator;
use std::ops;

#[derive(Clone, Copy, Debug, Display, Eq, From, PartialEq)]
pub enum Ext<T> {
    None,
    One(T),
    Many,
}

/* Basic getters */

impl<T> Ext<T> {
    pub fn is_none(&self) -> bool {
        matches!(self, Ext::None)
    }
    pub fn is_one(&self) -> bool {
        matches!(self, Ext::One(_))
    }
    pub fn is_many(&self) -> bool {
        matches!(self, Ext::Many)
    }
    pub fn get_one(&self) -> Option<&T> {
        match self {
            Ext::One(x) => Some(x),
            _ => None,
        }
    }
    pub fn get_one_mut(&mut self) -> Option<&mut T> {
        match self {
            Ext::One(x) => Some(x),
            _ => None,
        }
    }
    pub fn into_inner(self) -> Option<T> {
        match self {
            Ext::One(x) => Some(x),
            _ => None,
        }
    }
    // Note: it seems natural to implement TryInto<T> for Ext<T>
    // instead of unwrap. Unfortunately this doesn't work due to
    // conflicting trait implementations, but I think it may be
    // a compiler edge case.
    pub fn unwrap(self) -> T {
        self.into_inner().expect("Conversion from Ext failed: not a One value")
    }
    pub fn split<T1, T2, F>(self, f: F) -> (Ext<T1>, Ext<T2>)
    where
        F: FnOnce(T) -> (T1, T2),
    {
        match self {
            Ext::None => (Ext::None, Ext::None),
            Ext::One(x) => {
                let (x1, x2) = f(x);
                (Ext::One(x1), Ext::One(x2))
            }
            Ext::Many => (Ext::Many, Ext::Many),
        }
    }
    pub fn to_unit(&self) -> Ext<()> {
        match self {
            Ext::None => Ext::None,
            Ext::One(_) => Ext::One(()),
            Ext::Many => Ext::Many,
        }
    }
    pub fn as_ref(&self) -> Ext<&T> {
        match self {
            Ext::None => Ext::None,
            Ext::One(x) => Ext::One(&x),
            Ext::Many => Ext::Many,
        }
    }
}

/* Default value and from/to relationships */

impl<T> Default for Ext<T> {
    fn default() -> Self {
        Ext::None
    }
}

impl<T> From<Option<T>> for Ext<T> {
    fn from(opt_t: Option<T>) -> Self {
        match opt_t {
            None => Ext::None,
            Some(t) => Ext::One(t),
        }
    }
}

impl<T> From<Ext<T>> for Option<T> {
    fn from(e: Ext<T>) -> Self {
        e.into_inner()
    }
}

// .collect() from an iterator
impl<T> FromIterator<T> for Ext<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        match iter.next() {
            None => Ext::None,
            Some(x) => match iter.next() {
                None => Ext::One(x),
                Some(_) => Ext::Many,
            },
        }
    }
}

/* Union operation */

impl<T> ops::Add for Ext<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Ext::None => other,
            Ext::One(_) => match other {
                Ext::None => self,
                _ => Ext::Many,
            },
            Ext::Many => Ext::Many,
        }
    }
}

impl<T> ops::AddAssign for Ext<T> {
    fn add_assign(&mut self, other: Self) {
        if self.is_none() {
            *self = other;
        } else if !other.is_none() {
            *self = Ext::Many;
        }
    }
}

/* Product (pair) operation */

impl<T1, T2> ops::Mul<Ext<T2>> for Ext<T1> {
    type Output = Ext<(T1, T2)>;

    fn mul(self, rhs: Ext<T2>) -> Ext<(T1, T2)> {
        match self {
            Ext::One(x) => match rhs {
                Ext::One(y) => Ext::One((x, y)),
                Ext::None => Ext::None,
                Ext::Many => Ext::Many,
            },
            Ext::None => Ext::None,
            Ext::Many => match rhs {
                Ext::None => Ext::None,
                _ => Ext::Many,
            },
        }
    }
}

impl<T> ops::MulAssign<Ext<()>> for Ext<T> {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn mul_assign(&mut self, rhs: Ext<()>) {
        if rhs.is_none() {
            *self = Ext::None;
        } else if rhs.is_many() && self.is_one() {
            *self = Ext::Many;
        }
    }
}

/* Function application */

pub fn apply0<T1, F>(op: F) -> Ext<T1>
where
    F: FnOnce() -> T1,
{
    Ext::One(op())
}

pub fn apply1<T1, T2, F>(op: F, v1: Ext<T1>) -> Ext<T2>
where
    F: FnOnce(T1) -> T2,
{
    match v1 {
        Ext::None => Ext::None,
        Ext::One(x) => Ext::One(op(x)),
        Ext::Many => Ext::Many,
    }
}

pub fn apply2<T1, T2, T3, F>(op: F, v1: Ext<T1>, v2: Ext<T2>) -> Ext<T3>
where
    F: FnOnce(T1, T2) -> T3,
{
    apply1(|(x, y)| op(x, y), v1 * v2)
}

pub fn apply3<T1, T2, T3, T4, F>(
    op: F,
    v1: Ext<T1>,
    v2: Ext<T2>,
    v3: Ext<T3>,
) -> Ext<T4>
where
    F: FnOnce(T1, T2, T3) -> T4,
{
    apply1(|((x, y), z)| op(x, y, z), v1 * v2 * v3)
}

pub fn apply4<T1, T2, T3, T4, T5, F>(
    op: F,
    v1: Ext<T1>,
    v2: Ext<T2>,
    v3: Ext<T3>,
    v4: Ext<T4>,
) -> Ext<T5>
where
    F: FnOnce(T1, T2, T3, T4) -> T5,
{
    apply1(|(((x, y), z), t)| op(x, y, z, t), v1 * v2 * v3 * v4)
}

/* ========== TESTS ========== */

#[cfg(test)]
mod tests {
    use super::*;
    use std::i32;

    #[test]
    fn test_union() {
        let x0: Ext<i32> = Ext::None;
        let x1 = Ext::One(3);
        let x2: Ext<i32> = Ext::Many;
        assert_eq!(x0 + x0, x0);
        assert_eq!(x0 + x1, x1);
        assert_eq!(x1 + x0, x1);
        assert_eq!(x0 + x2, x2);
        assert_eq!(x2 + x0, x2);
        assert_eq!(x1 + x2, x2);
        assert_eq!(x2 + x1, x2);
        assert_eq!(x2 + x2, x2);
    }

    #[test]
    fn test_prod() {
        let x0: Ext<i32> = Ext::None;
        let x1 = Ext::One(3);
        let x2 = Ext::One(5);
        let x3: Ext<i32> = Ext::Many;
        assert_eq!(x0 * x0, Ext::None);
        assert_eq!(x0 * x1, Ext::None);
        assert_eq!(x0 * x3, Ext::None);
        assert_eq!(x2 * x0, Ext::None);
        assert_eq!(x3 * x0, Ext::None);
        assert_eq!(x1 * x1, Ext::One((3, 3)));
        assert_eq!(x1 * x2, Ext::One((3, 5)));
        assert_eq!(x2 * x1, Ext::One((5, 3)));
        assert_eq!(x2 * x2, Ext::One((5, 5)));
        assert_eq!(x1 * x3, Ext::Many);
        assert_eq!(x3 * x2, Ext::Many);
        assert_eq!(x3 * x3, Ext::Many);
    }

    #[test]
    fn test_union_string() {
        let x0: Ext<String> = Ext::None;
        let x1: Ext<String> = Ext::One("Hello".to_owned());
        let x2: Ext<String> = Ext::One("World".to_owned());
        let x3: Ext<String> = Ext::Many;
        assert_eq!(x0.clone() + x0.clone(), x0);
        assert_eq!(x0.clone() + x1.clone(), x1);
        assert_eq!(x1.clone() + x0.clone(), x1);
        assert_eq!(x1.clone() + x2.clone(), x3);
        assert_eq!(x0.clone() + x2.clone(), x2);
        assert_eq!(x0.clone() + x3.clone(), x3);
        assert_eq!(x3.clone() + x0.clone(), x3);
        assert_eq!(x1.clone() + x3.clone(), x3);
        assert_eq!(x3.clone() + x2.clone(), x3);
        assert_eq!(x3.clone() + x3.clone(), x3);
        drop(x0);
        drop(x1);
        drop(x2);
        drop(x3);
    }

    #[test]
    #[allow(clippy::eq_op)]
    fn test_eq() {
        assert_eq!(Ext::One(3), Ext::One(1 + 2));
        assert_ne!(Ext::One(-1), Ext::One(1));
        assert_ne!(Ext::One(0), Ext::None);
        assert_ne!(Ext::None, Ext::One(0));
        let x1: Ext<i32> = Ext::None;
        let x2: Ext<i32> = Ext::None;
        let x3: Ext<i32> = Ext::Many;
        assert_ne!(x2, x3);
        assert_eq!(x1, x2);
        assert_eq!(x1, x1);
    }

    #[test]
    fn test_apply() {
        let x0: Ext<i32> = Ext::None;
        let x1 = Ext::One(3);
        assert_eq!(apply1(i32::count_ones, x0), Ext::None);
        assert_eq!(apply1(i32::count_ones, x1), Ext::One(2));
        let y0: Ext<&str> = Ext::None;
        let y1: Ext<&str> = Ext::from("hello");
        let y2: Ext<String> = Ext::from(String::from("world"));
        assert_eq!(apply1(str::len, y0), Ext::None);
        assert_eq!(apply1(str::len, y1), Ext::One(5));
        assert_eq!(apply1(String::from, Ext::from("world")), y2);
        assert_eq!(apply1(Ext::from, Ext::One(5)), Ext::One(Ext::One(5)));
    }

    #[test]
    fn test_apply2() {
        let x0: Ext<i32> = Ext::None;
        let x1 = Ext::One(3);
        let x2 = Ext::One(2);
        let x3: Ext<i32> = Ext::Many;
        assert_eq!(apply2(ops::Add::add, x1, x2), Ext::One(5));
        assert_eq!(apply2(ops::Add::add, x2, x2), Ext::One(4));
        assert_eq!(apply2(ops::Mul::mul, x1, x3), Ext::Many);
        assert_eq!(apply2(ops::Mul::mul, x3, x0), Ext::None);
    }

    #[test]
    fn test_apply3() {
        let x0: Ext<i32> = Ext::None;
        let x1 = Ext::One(3);
        let x2 = Ext::One(2);
        let x3: Ext<i32> = Ext::Many;
        let vec_3 = |x1: i32, x2: i32, x3: i32| vec![x1, x2, x3];
        assert_eq!(apply3(vec_3, x1, x2, x1), Ext::One(vec![3, 2, 3]));
        assert_eq!(apply3(vec_3, x1, x0, x3), Ext::None);
        assert_eq!(apply3(vec_3, x1, x3, x1), Ext::Many);
    }

    #[test]
    fn test_apply4() {
        let x0: Ext<i32> = Ext::None;
        let x1 = Ext::One(3);
        let x2 = Ext::One(2);
        let x3: Ext<i32> = Ext::Many;
        let vec_4 = |x1: i32, x2: i32, x3: i32, x4: i32| vec![x1, x2, x3, x4];
        assert_eq!(apply4(vec_4, x1, x2, x1, x1), Ext::One(vec![3, 2, 3, 3]));
        assert_eq!(apply4(vec_4, x1, x0, x3, x1), Ext::None);
        assert_eq!(apply4(vec_4, x1, x3, x1, x1), Ext::Many);
    }
}
