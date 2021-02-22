/*
    Interface for data transducer implementations.

    TYPES THROUGHOUT THE FILE:
    - I: The type of initial input to a transducer
    - D: The type of the input data stream (updates to the transducer)
    - O: The type of output data for the transducer produced after each update
    Also:
    - RInput<I, D>: An input item which could also be a "restart event"
    - Strm: an iterator over D items or RInput<I, D> items
*/

use super::ext_value::Ext;
use std::fmt::Debug;
use std::iter;

/*
    Input to the transducer is given as an initial value,
    followed by a stream of items.

    Alternatively, it can be given as a single augmented stream of items and
    "restart" events with an initial value: which we call RInput items.
    If the only restart event is at the beginning, an RInput stream is
    equivalent to the previous paragraph. However having multiple restarts
    is relevant in the context of "restartable" transducers which are more
    composable.
*/
#[derive(Copy, Clone, Debug)]
pub enum RInput<I, D> {
    Restart(I),
    Item(D),
}

pub trait Transducer<I, D, O>: Clone {
    /* FUNCTIONALITY TO IMPLEMENT */

    // Computation
    // init: record an initial value for the computation (or a restart)
    // update: process an input data item
    // reset: restore the transducer to its original state
    fn init(&mut self, i: I) -> Ext<O>;
    fn update(&mut self, item: D) -> Ext<O>;
    fn reset(&mut self);

    // Static information
    // This could be done with associated functions (type-associated data),
    // but methods are more flexible as it will allow transducer implementations
    // which do not encode the # of states and transitions as part of the type
    fn is_restartable(&self) -> bool;
    fn n_states(&self) -> usize;
    fn n_transs(&self) -> usize;

    /* DERIVED FUNCTIONALITY */

    // Spawn an empty copy of the transducer: one that is in the initial
    // state and prior to any .init() updates
    // Note: this implementation is most efficient if self has not been modified;
    // if self has a lot of state it clones that state unnecessarily.
    // However this is only used in testing right now, so not worth optimizing.
    fn spawn_empty(&self) -> Self {
        let mut result = self.clone();
        result.reset();
        result
    }

    // Process an input stream (plus an initial value)
    fn process_stream<'a, Strm>(
        &'a mut self,
        i: I,
        mut strm: Strm,
    ) -> Box<dyn Iterator<Item = Ext<O>> + 'a>
    // Sad output type because 'impl Iterator' is not allowed here :(
    where
        Strm: Iterator<Item = D> + 'a,
        Self: Sized,
        O: 'a,
    {
        let y0 = self.init(i);
        Box::new(iter::once(y0).chain(iter::from_fn(move || {
            strm.next().map(|item| self.update(item))
        })))
    }

    // Process an input stream with "restart" events (initial values),
    // processing such events using one transducer and .init()
    fn process_rstream_single<'a, Strm>(
        &'a mut self,
        mut strm: Strm,
    ) -> Box<dyn Iterator<Item = Ext<O>> + 'a>
    where
        Strm: Iterator<Item = RInput<I, D>> + 'a,
        Self: Sized + 'a,
    {
        Box::new(iter::from_fn(move || {
            strm.next().map(|item| match item {
                RInput::Restart(i) => self.init(i),
                RInput::Item(item) => self.update(item),
            })
        }))
    }

    // Process an input stream with "restart" events, processing such
    // events by spawning many transducers
    // Doesn't use &self for any computation; instead
    // uses .spawn_empty() to get an initial state for each new transducer.
    // This is used mainly for testing in restartability_holds_for below.
    fn process_rstream_multi<'a, Strm>(
        &'a self,
        mut strm: Strm,
    ) -> Box<dyn Iterator<Item = Ext<O>> + 'a>
    where
        Strm: Iterator<Item = RInput<I, D>> + 'a,
        Self: Sized,
        I: Debug,
        D: Clone + Debug,
        O: Debug,
    {
        let mut transducers: Vec<Self> = Vec::new();
        Box::new(iter::from_fn(move || {
            strm.next().map(|item| match item {
                RInput::Restart(i) => {
                    println!("Restart: {:?}", i);
                    transducers.push(self.spawn_empty());
                    let out = transducers.last_mut().unwrap().init(i);
                    println!("--> output: {:?}", out);
                    out
                }
                RInput::Item(item) => {
                    println!("Item: {:?}", item);
                    let mut out = Ext::None;
                    for transducer in transducers.iter_mut() {
                        out += transducer.update(item.clone());
                    }
                    println!("--> output: {:?}", out);
                    out
                }
            })
        }))
    }

    // Having defined the above, now we can write a function which tests whether
    // the restartability property holds on a given input stream
    fn restartability_holds_for<'a, Strm>(&'a self, strm: Strm) -> bool
    where
        Strm: Iterator<Item = RInput<I, D>> + Clone + 'a,
        Self: Sized,
        I: Debug,
        D: Clone + Debug,
        O: Debug + Eq,
    {
        let mut self1 = self.spawn_empty();
        let strm1 = strm.clone();
        let single_out = self1.process_rstream_single(strm1);
        let multi_out = self.process_rstream_multi(strm);
        single_out.eq(multi_out)
    }
}
