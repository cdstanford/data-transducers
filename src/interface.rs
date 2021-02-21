/*
    Interface for data transducer implementations.
*/

use super::ext_value::Ext;
use std::iter;

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

    // Restartability
    fn is_restartable(&self) -> bool;

    // Size information
    fn n_states(&self) -> usize;
    fn n_transs(&self) -> usize;

    /* DERIVED FUNCTIONALITY */

    fn process_stream<I>(
        i: Self::Init,
        mut strm: I,
    ) -> Box<dyn Iterator<Item = Ext<Self::Output>>>
    // ^^ Sad output type because 'impl Iterator' is not allowed here :(
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
}

/*
*/



