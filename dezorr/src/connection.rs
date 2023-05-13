#[derive(Clone)]
struct ConnectionBody<'a, D: ContinuousDomain + Default> {
    value: D,
    source: &'a Function<'a, D>,
}

#[derive(Clone)]
pub struct Connection<'a, D: ContinuousDomain + Default>(RefCell<ConnectionBody<'a, D>>);

impl<'a, D: ContinuousDomain + Default> Connection<'a, D> {
    fn new(value: D, source: &'a Function<'a, D>) -> Self {
        Connection(RefCell::new(ConnectionBody { value, source }))
    }
}
