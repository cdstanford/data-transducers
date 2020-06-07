/*
Module implementing "extended values" Ext<T>,
that is, values which can be None, One (with a value),
or Many.

Ext<T> can be thought variant of Option<T>.
*/

extern crate derive_more;
use derive_more::{Display, From};
use std::{ops,i32};

#[derive(Debug, PartialEq, Display, From, Copy, Clone)]
pub enum Ext<T> {
    None,
    One(T),
    Many,
}

// Union operation
impl<T: Copy> ops::Add for Ext<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Ext::None => other,
            Ext::One(_x) => match other {
                Ext::None => self,
                Ext::One(_y) => Ext::Many,
                Ext::Many => Ext::Many,
            },
            Ext::Many => Ext::Many,
        }
    }
}

fn apply1<T1, T2>(op: fn(T1) -> T2,
                  v1: Ext<T1>)
                  -> Ext<T2> {
    match v1 {
        Ext::None => Ext::None,
        Ext::One(x) => Ext::One(op(x)),
        Ext::Many => Ext::Many,
    }
}

fn apply2<T1, T2, T3>(op: fn(T1, T2) -> T3,
                      v1: Ext<T1>,
                      v2:Ext<T2>)
                      -> Ext<T3> {
    match v1 {
        Ext::None => Ext::None,
        Ext::One(x) => match v2 {
            Ext::None => Ext::None,
            Ext::One(y) => Ext::One(op(x, y)),
            Ext::Many => Ext::Many,
        },
        Ext::Many => match v2 {
            Ext::None => Ext::None,
            Ext::One(y) => Ext::Many,
            Ext::Many => Ext::Many,
        }
    }
}

// ========== TESTS ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union() {
        let x0 : Ext<i32> = Ext::None;
        let x1 = Ext::One(3);
        let x2 : Ext<i32> = Ext::Many;
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
    fn test_eq() {
        assert_eq!(Ext::One(3), Ext::One(1 + 2));
        assert_ne!(Ext::One(-1), Ext::One(1));
        assert_ne!(Ext::One(0), Ext::None);
        assert_ne!(Ext::None, Ext::One(0));
        let mut x1 : Ext<i32> = Ext::None;
        let x2 : Ext<i32> = Ext::None;
        let x3 : Ext<i32> = Ext::Many;
        assert_ne!(x2, x3);
        assert_eq!(x1, x2);
        assert_eq!(x1, x1);
        // let x4 : Ext<i64> = Ext::None;
        // assert_ne!(x2, x4); // Note: Type error
    }

    #[test]
    fn test_apply() {
        let mut x0 : Ext<i32> = Ext::None;
        let x1 = Ext::One(3);
        let mut x2 = Ext::One(2);
        let x3 : Ext<i32> = Ext::Many;
        assert_eq!(apply1(i32::count_ones, x0), Ext::None);
        assert_eq!(apply1(i32::count_ones, x1), Ext::One(2));
        assert_eq!(apply2(ops::Add::add, x1, x2), Ext::One(5));
        assert_eq!(apply2(ops::Add::add, x2, x2), Ext::One(4));
        assert_eq!(apply2(ops::Mul::mul, x1, x3), Ext::Many);
        assert_eq!(apply2(ops::Mul::mul, x3, x0), Ext::None);
        let y0 : Ext<&str> = Ext::None;
        let y1 : Ext<&str> = Ext::from("hello");
        let y2 : Ext<String> = Ext::from(String::from("world"));
        assert_eq!(apply1(str::len, y0), Ext::None);
        assert_eq!(apply1(str::len, y1), Ext::One(5));
        assert_eq!(apply1(String::from, Ext::from("world")), y2);
        assert_eq!(
            apply1(Ext::from, Ext::One(5)),
            Ext::One(Ext::One(5))
        );
    }
}
