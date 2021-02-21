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

trait Transducer {
    /* TYPES */

    // Initial input
    type Init;

    // Input and output data streams
    type Input;
    type Output;

    /* FUNCTIONALITY TO IMPLEMENT */

    // Computation
    fn new() -> Self;
    fn init(&mut self, i: Self::Init) -> Ext<Self::Output>;
    fn update(&mut self, item: Self::Input) -> Ext<Self::Output>;

    // Static information (type-associated data)
    fn is_restartable() -> bool;
    fn n_states() -> usize;
    fn n_transs() -> usize;

    /* DERIVED FUNCTIONALITY */

    // Process an input stream (plus an initial value)
    fn process_stream<I>(
        i: Self::Init,
        mut strm: I,
    ) -> Box<dyn Iterator<Item = Ext<Self::Output>>>
    // Sad output type because 'impl Iterator' is not allowed here :(
    where
        I: 'static + Iterator<Item = Self::Input>,
        Self: 'static + Sized,
    {
        let mut transducer = Self::new();
        let y0 = transducer.init(i);
        Box::new(iter::once(y0).chain(iter::from_fn(move || {
            strm.next().map(|item| transducer.update(item))
        })))
    }

    // Process an input stream with "restart" events (initial values),
    // processing such events using one transducer and .init()
    fn process_rstream_single<I>(
        mut strm: I,
    ) -> Box<dyn Iterator<Item = Ext<Self::Output>>>
    where
        I: 'static + Iterator<Item = RInput<Self::Init, Self::Input>>,
        Self: 'static + Sized,
    {
        let mut transducer = Self::new();
        Box::new(iter::from_fn(move || {
            strm.next().map(|item| match item {
                RInput::Restart(i) => transducer.init(i),
                RInput::Item(item) => transducer.update(item),
            })
        }))
    }

    // Process an input stream with "restart" events, processing such
    // events by spawning many transducers
    fn process_rstream_multi<I>(
        mut strm: I,
    ) -> Box<dyn Iterator<Item = Ext<Self::Output>>>
    where
        I: 'static + Iterator<Item = RInput<Self::Init, Self::Input>>,
        Self: 'static + Sized,
        Self::Input: Clone,
    {
        let mut transducers: Vec<Self> = Vec::new();
        Box::new(iter::from_fn(move || {
            strm.next().map(|item| match item {
                RInput::Restart(i) => {
                    transducers.push(Self::new());
                    transducers.last_mut().unwrap().init(i)
                }
                RInput::Item(item) => {
                    let mut out = Ext::None;
                    for transducer in transducers.iter_mut() {
                        out = out + transducer.update(item.clone());
                    }
                    out
                }
            })
        }))
    }

    // Having defined the above, now we can write a function which tests whether
    // the restartability property correctly holds
    fn check_restartability_for<I>(strm: I)
    where
        I: 'static + Iterator<Item = RInput<Self::Init, Self::Input>> + Clone,
        Self: 'static + Sized,
        Self::Input: Clone,
        Self::Output: Eq,
    {
        if Self::is_restartable() {
            let strm2 = strm.clone();
            let single_out = Self::process_rstream_single(strm);
            let multi_out = Self::process_rstream_multi(strm2);
            assert!(single_out.eq(multi_out));
        } else {
            eprintln!(
                "Warning: tried to check restartability for \
                non-restartable transducer"
            );
        }
    }
}
