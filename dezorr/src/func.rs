use std::rc::Rc;

use crate::var::Variable;

pub struct Function<D> {
    function: Rc<Box<dyn Fn(D) -> D>>,
}

impl<F> Clone for Function<F> {
    fn clone(&self) -> Self {
        Function {
            function: self.function.clone(),
        }
    }
}
impl<D> Function<D> {
    pub fn new(function: Box<dyn Fn(D) -> D>) -> Self {
        Function {
            function: Rc::new(function),
        }
    }
    pub fn apply(&self, v: Variable<D>) -> Variable<D> {
        let f = &*self.function;
        let n = f(v.data);
        Variable::new(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_step_2() {
        let v0: Variable<usize> = Variable::new(0usize);
        assert_eq!(v0.data, 0);
        let f0: Function<usize> = Function::<usize>::new(Box::new(|x: usize| x + 1));
        assert_eq!(f0.apply(v0).data, 1);

        let v1: Variable<f32> = Variable::new(2.0f32);
        assert_eq!(v1.data, 2.0);
        let f1: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x + 1.0));
        assert_eq!(f1.apply(v1.clone()).data, 3.0);
        let f2: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.exp()));
        assert!((f2.apply(v1).data - 7.389056).abs() < 0.001);
    }
}
