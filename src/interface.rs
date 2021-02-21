/*
    Interface for data transducer implementations.
*/

use super::ext_value::Ext;
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
pub enum RInput<Init, Input> {
    Restart(Init),
    Item(Input),
}

trait Transducer<Init, Input, Output> {
    /*
        TYPES:
        Init: initial input
        Input: input data stream
        Output: output produced for each input
    */

    /* FUNCTIONALITY TO IMPLEMENT */

    // Computation
    fn init(&mut self, i: Init) -> Ext<Output>;
    fn update(&mut self, item: Input) -> Ext<Output>;
    // Reset all computation back to the original state
    fn reset(&mut self);
    // Spawn an empty copy of the transducer: one that is in the initial
    // state and prior to any .init() updates
    // If Self: Clone, this should be roughly equivalent to self.clone().reset()
    fn spawn_empty(&self) -> Self;

    // Static information
    // This could be done with associated functions (type-associated data),
    // but methods are more flexible as it will allow transducer implementations
    // which do not encode the # of states and transitions as part of the type
    fn is_restartable(&self) -> bool;
    fn n_states(&self) -> usize;
    fn n_transs(&self) -> usize;

    /* DERIVED FUNCTIONALITY */

    // Process an input stream (plus an initial value)
    fn process_stream<'a, I>(
        &'a mut self,
        i: Init,
        mut strm: I,
    ) -> Box<dyn Iterator<Item = Ext<Output>> + 'a>
    // Sad output type because 'impl Iterator' is not allowed here :(
    where
        I: Iterator<Item = Input> + 'a,
        Self: Sized,
        Output: 'a,
    {
        let y0 = self.init(i);
        Box::new(iter::once(y0).chain(iter::from_fn(move || {
            strm.next().map(|item| self.update(item))
        })))
    }

    // Process an input stream with "restart" events (initial values),
    // processing such events using one transducer and .init()
    fn process_rstream_single<'a, I>(
        &'a mut self,
        mut strm: I,
    ) -> Box<dyn Iterator<Item = Ext<Output>> + 'a>
    where
        I: Iterator<Item = RInput<Init, Input>> + 'a,
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
    // uses .spawn_empty() to get an initial state for each new transducer
    fn process_rstream_multi<'a, I>(
        &'a self,
        mut strm: I,
    ) -> Box<dyn Iterator<Item = Ext<Output>> + 'a>
    where
        I: Iterator<Item = RInput<Init, Input>> + 'a,
        Self: Sized,
        Input: Clone,
    {
        let mut transducers: Vec<Self> = Vec::new();
        Box::new(iter::from_fn(move || {
            strm.next().map(|item| match item {
                RInput::Restart(i) => {
                    transducers.push(self.spawn_empty());
                    transducers.last_mut().unwrap().init(i)
                }
                RInput::Item(item) => {
                    let mut out = Ext::None;
                    for transducer in transducers.iter_mut() {
                        out += transducer.update(item.clone());
                    }
                    out
                }
            })
        }))
    }

    // Having defined the above, now we can write a function which tests whether
    // the restartability property correctly holds
    fn check_restartability_for<'a, I>(&'a self, strm: I)
    where
        I: Iterator<Item = RInput<Init, Input>> + Clone + 'a,
        Self: Sized,
        Input: Clone,
        Output: Eq,
    {
        if self.is_restartable() {
            let mut self1 = self.spawn_empty();
            let strm1 = strm.clone();
            let single_out = self1.process_rstream_single(strm1);
            let multi_out = self.process_rstream_multi(strm);
            assert!(single_out.eq(multi_out));
        } else {
            eprintln!(
                "Warning: tried to check restartability for \
                non-restartable transducer"
            );
        }
    }
}
