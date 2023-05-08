use crate::var::Variable;

pub struct Function<D> {
    function: Box<dyn Fn(D) -> D>,
}

impl<D> Function<D> {
    pub fn new(function: Box<dyn Fn(D) -> D>) -> Self {
        Function { function }
    }
    pub fn apply(&self, v: Variable<D>) -> Variable<D> {
        let f = &self.function;
        let n = f(v.data);
        Variable::new(n)
    }
}
