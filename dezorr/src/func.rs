use {
    crate::var::{ContinuousDomain, Variable},
    std::rc::Rc,
};

pub trait FunctionOn<D: ContinuousDomain> {
    fn new(function: Box<dyn Fn(D) -> D>) -> Self;
    fn apply(&self, v: Variable<D>) -> Variable<D>;
    fn set_backward(self, function: Box<dyn Fn(D) -> D>) -> Self;
    fn followed_by(&self, other: &Self) -> Self;
    fn numerical_diff(&self, x: &Variable<D>, eps: &D) -> D;
}

#[allow(clippy::complexity)]
pub struct Function<D: ContinuousDomain> {
    forward: Rc<Box<dyn Fn(D) -> D>>,
    input: Option<D>,
    backward: Option<Rc<Box<dyn Fn(D) -> D>>>,
}

impl<D: ContinuousDomain> Clone for Function<D> {
    fn clone(&self) -> Self {
        Function {
            forward: self.forward.clone(),
            input: self.input.clone(),
            backward: self.backward.clone(),
        }
    }
}
impl<D: ContinuousDomain> FunctionOn<D> for Function<D> {
    fn new(function: Box<dyn Fn(D) -> D>) -> Self {
        Function {
            forward: Rc::new(function),
            input: None,
            backward: None,
        }
    }
    fn apply(&self, v: Variable<D>) -> Variable<D> {
        let f = &*self.forward;
        let n = f(v.data);
        Variable::new(n)
    }
    fn set_backward(mut self, function: Box<dyn Fn(D) -> D>) -> Self {
        self.backward = Some(Rc::new(function));
        self
    }
    /// step 3: function composition
    fn followed_by(&self, other: &Self) -> Self {
        let f = self.forward.clone();
        let g = other.forward.clone();
        Function::<D>::new(Box::new(move |x: D| g(f(x))))
    }
    fn numerical_diff(&self, x: &Variable<D>, eps: &D) -> D {
        let x0 = Variable::new(x.data.clone() - eps.clone());
        let x1 = Variable::new(x.data.clone() + eps.clone());
        (self.apply(x1).data - self.apply(x0).data) / (eps.clone() + eps.clone())
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
        assert!((f2.apply(v1.clone()).data - 7.389056).abs() < 0.001);
        let f3: Function<f32> = f2.clone();
        assert_eq!(f3.apply(v1.clone()).data, f2.apply(v1).data);
    }
    #[test]
    fn test_step_3() {
        let v1: Variable<f32> = Variable::new(1.0f32);
        assert_eq!(v1.data, 1.0);
        let f: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x + 1.0));
        assert_eq!(f.apply(v1.clone()).data, 2.0);
        let g: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.exp()));
        assert!((g.apply(v1.clone()).data - std::f32::consts::E).abs() < 0.001);
        let fg_or_gf: Function<f32> = f.followed_by(&g);
        assert!((fg_or_gf.apply(v1).data - std::f32::consts::E.powi(2)).abs() < 0.001);
    }
    #[test]
    fn test_step_3_2() {
        let x: Variable<f32> = Variable::new(0.5f32);
        let a: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let b: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.exp()));
        let c: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let chain: Function<f32> = a.followed_by(&b).followed_by(&c);
        assert!((chain.apply(x).data - 1.6487212).abs() < 0.001);
    }
    #[test]
    fn test_step_4_2() {
        let x: Variable<f32> = Variable::new(2.0f32);
        let s: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        assert!((s.numerical_diff(&x, &0.0001) - 4.0f32).abs() < 0.005);
    }
    #[test]
    fn test_step_4_3() {
        let x: Variable<f32> = Variable::new(0.5f32);
        let a: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let b: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.exp()));
        let c: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let chain: Function<f32> = a.followed_by(&b).followed_by(&c);
        assert!((chain.numerical_diff(&x, &0.0001) - 3.2974426).abs() < 0.001);
    }
}
