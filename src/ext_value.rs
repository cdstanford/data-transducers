/*
Module implementing "extended values" Ext<T>,
that is, values which can be None, One (with a value),
or Many.

Ext<T> can be thought variant of Option<T>.
*/

extern crate derive_more;
use derive_more::{Display, From};
use std::ops;

#[derive(Debug, PartialEq, Display, From, Copy, Clone)]
pub enum Ext<T> {
    None,
    One(T),
    Many,
}

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
        Ext::Many => Ext::Many,
    }
}
