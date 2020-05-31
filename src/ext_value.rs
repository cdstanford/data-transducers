/*
Module implementing "extended values" ExtValue<T>,
that is, values which can be None, One (with a value),
or Many.

ExtValue<T> can be thought variant of Option<T>.
*/

extern crate derive_more;
use derive_more::{Display, From};
use std::ops::Add;

#[derive(Debug, PartialEq, Display, From, Copy, Clone)]
pub enum ExtValue<T> {
    None,
    One(T),
    Many,
}

impl<T: Copy> Add for ExtValue<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            ExtValue::None => other,
            ExtValue::One(_x) =>
                match other {
                    ExtValue::None => self,
                    ExtValue::One(_y) => ExtValue::Many,
                    ExtValue::Many => ExtValue::Many,
                },
            ExtValue::Many => ExtValue::Many,
        }
    }
}
