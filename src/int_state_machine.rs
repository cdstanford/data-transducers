/*
Module implementing the core state machine data structure
for data transducers, with core constructors and operations.

First, I am implementing this where every state is
an int, as that should be easier. Then I will try
to adapt the code to handle arbitrary state types,
probably using traits.
*/

use super::ext_value::Ext;
use super::ext_value;

use std::i32;
use std::vec::Vec;

struct State {
    curr: Ext<i32>,
    prev: Ext<i32>,
}

struct Trans0 {
    target: State,
    eval: fn() -> i32,
}
struct Trans1 {
    source1: State,
    target: State,
    eval: fn(i32) -> i32,
}
struct Trans2 {
    source1: State,
    source2: State,
    target: State,
    eval: fn(i32, i32) -> i32,
}

enum Transition {
    T0(Trans0),
    T1(Trans1),
    T2(Trans2),
}

fn sources(t: Transition) -> Vec<State> {
    let mut vec = Vec::new();
    match t {
        Transition::T0(_) => (),
        Transition::T1(t1) => vec.push(t1.source1),
        Transition::T2(t2) => {
            vec.push(t2.source1);
            vec.push(t2.source2)
        },
    };
    vec
}
fn target(t: Transition) -> State {
    match t {
        Transition::T0(t0) => t0.target,
        Transition::T1(t1) => t1.target,
        Transition::T2(t2) => t2.target,
    }
}
fn eval(t: Transition) -> Ext<i32> {
    match t {
        Transition::T0(t0) => ext_value::apply0(t0.eval),
        Transition::T1(t1) =>
            ext_value::apply1(t1.eval, t1.source1.curr),
        Transition::T2(t2) =>
            ext_value::apply2(t2.eval, t2.source1.curr, t2.source2.curr),
    }
}


// struct Transition {
//     sources: Vec<State>,
//     target: State,
//     update:
// }
//
// struct Trans2 {
//     source1: State,
//     source2: State,
//     target: State
// }
// impl Trans2{}
//
//
// impl IntState {
//     fn get(&self) -> Ext<i32> {
//         self.curr;
//     }
//     fn add(&mut self, v: Ext<i32>) -> () {
//         self.curr = self.curr + v;
//     }
//     fn reset(&mut self) -> () {
//         self.curr = Ext::None;
//         self.prev = Ext::None;
//     }
//     fn update(&mut self) -> () {
//         self.prev = self.curr
//         self.curr = Ext::None;
//     }
// }
//
//
// fn add(&self, _: &Ext<&Self::Val>) -> ();
// fn get(&self) -> Ext<&Self::Val>;
// fn reset(&self) -> ();
// fn update(&self) -> ();
//
// fn get_prev(&self) -> Ext<&Self::Val>;
// fn set_prev(&self) -> ();
//
//
