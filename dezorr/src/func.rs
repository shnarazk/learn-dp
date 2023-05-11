use {
    crate::var::{ContinuousDomain, Variable},
    std::rc::Rc,
};

// pub struct FuctionLayer<D: ContinuousDomain> {
//     pub function: HashMap<String, Function<D>>,
// }

pub trait FunctionOn<'a, D: ContinuousDomain> {
    fn new(function: Box<dyn Fn(D) -> D>) -> Self;
    fn apply(&mut self, v: &Variable<D>) -> Variable<D>;
    fn back_propagate(&self, v: Variable<D>) -> D;
    fn bind(&'a mut self, seed: &'a Self);
    fn set_backward(self, function: Box<dyn Fn(D) -> D>) -> Self;
    fn followed_by(&self, other: &Self) -> Self;
    fn numerical_diff(&mut self, x: &Variable<D>, eps: &D) -> D;
}

#[allow(clippy::complexity)]
pub struct Function<'a, D: ContinuousDomain> {
    seed: Option<&'a Function<'a, D>>,
    forward: Rc<Box<dyn Fn(D) -> D>>,
    input: Option<D>,
    backward: Option<Rc<Box<dyn Fn(D) -> D>>>,
}

impl<D: ContinuousDomain> Clone for Function<'_, D> {
    fn clone(&self) -> Self {
        Function {
            seed: None,
            forward: self.forward.clone(),
            input: self.input.clone(),
            backward: self.backward.clone(),
        }
    }
}
impl<'a, D: ContinuousDomain> FunctionOn<'a, D> for Function<'a, D> {
    fn new(function: Box<dyn Fn(D) -> D>) -> Self {
        Function {
            seed: None,
            forward: Rc::new(function),
            input: None,
            backward: None,
        }
    }
    fn apply(&mut self, v: &Variable<D>) -> Variable<D> {
        let f = &*self.forward;
        self.input = Some(v.data.clone());
        let p = f(v.data.clone());
        Variable::new(p)
    }
    fn bind(&'a mut self, seed: &'a Self) {
        self.seed = Some(seed);
    }
    fn back_propagate(&self, v: Variable<D>) -> D {
        let Some(b) = &self.backward else { panic!(); };
        let Some(x) = &self.input else { panic!(); };
        let Some(g) = v.grad else { panic!(); };
        g * b(x.clone())
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
    fn numerical_diff(&mut self, x: &Variable<D>, eps: &D) -> D {
        let x0 = Variable::new(x.data.clone() - eps.clone());
        let x1 = Variable::new(x.data.clone() + eps.clone());
        (self.apply(&x1).data - self.apply(&x0).data) / (eps.clone() + eps.clone())
    }
}

pub fn function_square<D: ContinuousDomain>() -> Function<'static, D> {
    Function::<'static, D>::new(Box::new(|x: D| x.clone() * x))
        .set_backward(Box::new(|x| x.clone() + x))
}

pub fn function_exp_f32() -> Function<'static, f32> {
    Function::<'static, f32>::new(Box::new(|x: f32| x.exp())).set_backward(Box::new(|x| x.exp()))
}

pub fn function_exp_f64() -> Function<'static, f64> {
    Function::<'static, f64>::new(Box::new(|x: f64| x.exp())).set_backward(Box::new(|x| x.exp()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_step_2() {
        let v0: Variable<usize> = Variable::new(0usize);
        assert_eq!(v0.data, 0);
        let mut f0: Function<usize> = Function::<usize>::new(Box::new(|x: usize| x + 1));
        assert_eq!(f0.apply(&v0).data, 1);

        let v1: Variable<f32> = Variable::new(2.0f32);
        assert_eq!(v1.data, 2.0);
        let mut f1: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x + 1.0));
        assert_eq!(f1.apply(&v1).data, 3.0);
        let mut f2: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.exp()));
        assert!((f2.apply(&v1).data - 7.389056).abs() < 0.001);
        let mut f3: Function<f32> = f2.clone();
        assert_eq!(f3.apply(&v1).data, f2.apply(&v1).data);
    }
    #[test]
    fn test_step_3() {
        let v1: Variable<f32> = Variable::new(1.0f32);
        assert_eq!(v1.data, 1.0);
        let mut f: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x + 1.0));
        assert_eq!(f.apply(&v1).data, 2.0);
        let mut g: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.exp()));
        assert!((g.apply(&v1).data - std::f32::consts::E).abs() < 0.001);
        let mut fg_or_gf: Function<f32> = f.followed_by(&g);
        assert!((fg_or_gf.apply(&v1).data - std::f32::consts::E.powi(2)).abs() < 0.001);
    }
    #[test]
    fn test_step_3_2() {
        let x: Variable<f32> = Variable::new(0.5f32);
        let a: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let b: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.exp()));
        let c: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let mut chain: Function<f32> = a.followed_by(&b).followed_by(&c);
        assert!((chain.apply(&x).data - 1.6487212).abs() < 0.001);
    }
    #[test]
    fn test_step_4_2() {
        let x: Variable<f32> = Variable::new(2.0f32);
        let mut s: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        assert!((s.numerical_diff(&x, &0.0001) - 4.0f32).abs() < 0.005);
    }
    #[test]
    fn test_step_4_3() {
        let x: Variable<f32> = Variable::new(0.5f32);
        let a: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let b: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.exp()));
        let c: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let mut chain: Function<f32> = a.followed_by(&b).followed_by(&c);
        assert!((chain.numerical_diff(&x, &0.0001) - 3.2974426).abs() < 0.001);
    }
    #[test]
    fn test_step_6_3() {
        let mut fa: Function<f64> = function_square::<f64>();
        let mut fb: Function<f64> = function_exp_f64();
        let mut fc: Function<f64> = function_square::<f64>();
        let mut x: Variable<f64> = Variable::new(0.5f64);
        let mut a: Variable<f64> = fa.apply(&x);
        assert_eq!(x.data, fa.input.unwrap());
        let mut b: Variable<f64> = fb.apply(&a);
        assert_eq!(a.data, fb.input.unwrap());
        let mut y: Variable<f64> = fc.apply(&b);
        assert_eq!(b.data, fc.input.unwrap());
        assert!((y.data - 1.6487212).abs() < 0.001);
        y.set_grad(1.0);
        b.set_grad(fc.back_propagate(y));
        a.set_grad(fb.back_propagate(b));
        x.set_grad(fa.back_propagate(a));
        assert!((x.grad.unwrap() - 3.29744).abs() < 0.0001);
    }
}
