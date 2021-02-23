/*
    Some convenience traits for clonable functions
*/

pub trait FnClone0<O>: Fn() -> O + Clone {}
impl<O, F: Fn() -> O + Clone> FnClone0<O> for F {}

pub trait FnClone1<I, O>: Fn(I) -> O + Clone {}
impl<I, O, F: Fn(I) -> O + Clone> FnClone1<I, O> for F {}

pub trait FnClone2<I1, I2, O>: Fn(I1, I2) -> O + Clone {}
impl<I1, I2, O, F: Fn(I1, I2) -> O + Clone> FnClone2<I1, I2, O> for F {}

// More for when the argument is a reference

pub trait FnClone1Ref<I, O>: Fn(&I) -> O + Clone {}
impl<I, O, F: Fn(&I) -> O + Clone> FnClone1Ref<I, O> for F {}

pub trait FnClone2LRef<I1, I2, O>: Fn(&I1, I2) -> O + Clone {}
impl<I1, I2, O, F: Fn(&I1, I2) -> O + Clone> FnClone2LRef<I1, I2, O> for F {}

pub trait FnClone2RRef<I1, I2, O>: Fn(I1, &I2) -> O + Clone {}
impl<I1, I2, O, F: Fn(I1, &I2) -> O + Clone> FnClone2RRef<I1, I2, O> for F {}

pub trait FnClone2LRRef<I1, I2, O>: Fn(&I1, &I2) -> O + Clone {}
impl<I1, I2, O, F: Fn(&I1, &I2) -> O + Clone> FnClone2LRRef<I1, I2, O> for F {}
