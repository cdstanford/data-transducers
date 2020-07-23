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

type State = Ext<i32>;

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
enum Trans {
    T0(Trans0),
    T1(Trans1),
    T2(Trans2),
}
struct Transition<X> {
    t: Trans,
    is_enabled: fn(&X) -> bool,
}

fn sources<X>(t: Transition<X>) -> Vec<State> {
    let mut vec = Vec::new();
    match t.t {
        Trans::T0(_) => (),
        Trans::T1(t1) => vec.push(t1.source1),
        Trans::T2(t2) => {
            vec.push(t2.source1);
            vec.push(t2.source2)
        },
    };
    vec
}
fn target<X>(t: Transition<X>) -> State {
    match t.t {
        Trans::T0(t0) => t0.target,
        Trans::T1(t1) => t1.target,
        Trans::T2(t2) => t2.target,
    }
}
fn eval<X>(t: Transition<X>) -> Ext<i32> {
    match t.t {
        Trans::T0(t0) => ext_value::apply0(t0.eval),
        Trans::T1(t1) => ext_value::apply1(t1.eval, t1.source1),
        Trans::T2(t2) => ext_value::apply2(t2.eval, t2.source1, t2.source2),
    }
}

struct StateMachine<X> {
    n_states: usize,
    n_transitions: usize,
    states: Vec<State>,
    prev_states: Vec<State>,
    transitions: Vec<Transition<X>>,
}

impl<X> StateMachine<X> {
    fn reset_cur(&mut self) -> () {
        for x in &mut self.states {
            *x = Ext::None;
        };
    }
    fn update_prev(&mut self) -> () {
        for i in 0..(self.n_states) {
            self.prev_states[i] = self.states[i];
        }
    }
    fn reset(&mut self) -> () {
        self.reset_cur();
        self.update_prev();
    }
    fn update(&mut self, event: &X) -> () {
        /*
            Completely wrong implementation for now:
            Just evaluate all transitions, ignoring dependencies between them.
            This works as long as all transitions refer to only previous states
            as sources, never current states. So it doesn't allow e.g.
            epsilon transitions.
        */
        self.update_prev();
        self.reset_cur();
        // Not working, TODO fix
        // for t in &mut self.transitions {
        //     if (t.is_enabled)(event) {
        //         let s: State = target(t);
        //         s = s + eval(t);
        //         // target(t) = target(t) + eval(t);
        //     }
        // }
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
