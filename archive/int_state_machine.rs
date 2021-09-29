/*
    Module implementing the core state machine data structure
    for data transducers, with core constructors and operations.

    First, I am implementing this where every state is
    an int, as that should be easier. Then I will try
    to adapt the code to handle arbitrary state types,
    probably using traits.
*/

#![allow(dead_code)]

use super::ext_value::{self, Ext};

type State = Ext<i32>;

struct Trans0<'a> {
    target: &'a mut State,
    eval: fn() -> i32,
}
struct Trans1<'a> {
    source1: &'a State,
    target: &'a mut State,
    eval: fn(i32) -> i32,
}
struct Trans2<'a> {
    source1: &'a State,
    source2: &'a State,
    target: &'a mut State,
    eval: fn(i32, i32) -> i32,
}
enum Trans<'a> {
    T0(Trans0<'a>),
    T1(Trans1<'a>),
    T2(Trans2<'a>),
}
// X: character type for input string to the transducer
struct Transition<'a, X> {
    t: Trans<'a>,
    is_enabled: fn(&X) -> bool,
}
fn sources<'a, X>(t: &Transition<'a, X>) -> Vec<&'a State> {
    let mut vec = Vec::new();
    match &t.t {
        Trans::T0(_) => (),
        Trans::T1(t1) => vec.push(t1.source1),
        Trans::T2(t2) => {
            vec.push(t2.source1);
            vec.push(t2.source2)
        }
    };
    vec
}
fn target<'a, X>(t: &'a mut Transition<'a, X>) -> &'a mut State {
    match &mut t.t {
        Trans::T0(t0) => t0.target,
        Trans::T1(t1) => t1.target,
        Trans::T2(t2) => t2.target,
    }
}
fn eval<X>(t: &Transition<X>) -> Ext<i32> {
    match &t.t {
        Trans::T0(t0) => ext_value::apply0(t0.eval),
        Trans::T1(t1) => ext_value::apply1(t1.eval, *t1.source1),
        Trans::T2(t2) => ext_value::apply2(t2.eval, *t2.source1, *t2.source2),
    }
}

struct StateMachine<'a, X> {
    n_states: usize,
    n_transitions: usize,
    states: Vec<State>,
    prev_states: Vec<State>,
    transitions: Vec<Transition<'a, X>>,
}

impl<'a, X> StateMachine<'a, X> {
    fn reset_cur(&mut self) {
        for x in &mut self.states {
            *x = Ext::None;
        }
    }
    fn set_prev(&mut self) {
        self.prev_states[..(self.n_states)]
            .clone_from_slice(&self.states[..(self.n_states)])
    }
    fn reset(&mut self) {
        self.reset_cur();
        self.set_prev();
    }
    fn update(&'a mut self, _event: &X) {
        /*
            Completely wrong implementation for now:
            Just evaluate all transitions, ignoring dependencies between them.
            This works as long as all transitions refer to only previous states
            as sources, never current states. So it doesn't allow e.g.
            epsilon transitions.
        */
        self.set_prev();
        self.reset_cur();
        // // Not working, TODO fix
        // for t in &self.transitions {
        //     if (t.is_enabled)(event) {
        //         let s: &'a mut State = target(t);
        //         *s = *s + eval(&t);
        //         // target(t) = target(t) + eval(t);
        //     }
        // }
    }
}
