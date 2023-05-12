use {
    crate::var::{ContinuousDomain, Variable},
    std::{cell::RefCell, rc::Rc},
};

pub trait FunctionOn<'a, D: ContinuousDomain + Default> {
    fn new(function: Box<dyn Fn(D) -> D>) -> Self;
    fn constant(value: D) -> Self;
    fn propagate_to(&'a self, v: &'a Self);
    fn propagate(&'a self, v: &'a Self);
    fn back_propagate(&self) -> D;
    fn set_backward(self, function: Box<dyn Fn(D) -> D>) -> Self;
    fn followed_by(&self, other: &Self) -> Self;
    fn numerical_diff(&'a mut self, x: &'a Variable<D>, eps: &D) -> D;
    fn seed(&'a self) -> Option<&'a Function<D>>;
    fn input(&self) -> Option<D>;
    fn value(&self) -> Option<D>;
    fn grad(&self) -> Option<D>;
    fn set_grad(&self, val: D);
}

#[allow(clippy::complexity)]
pub struct FunctionBody<'a, D: ContinuousDomain + Default> {
    seed: Option<&'a Function<'a, D>>,
    forward: Rc<Box<dyn Fn(D) -> D>>,
    backward: Option<Rc<Box<dyn Fn(D) -> D>>>,
    output: Variable<D>,
}

pub struct Function<'a, D: ContinuousDomain + Default>(RefCell<FunctionBody<'a, D>>);

impl<D: ContinuousDomain + Default> Clone for Function<'_, D> {
    fn clone(&self) -> Self {
        Function(RefCell::new(FunctionBody {
            seed: None,
            forward: self.0.borrow().forward.clone(),
            backward: self.0.borrow().backward.clone(),
            output: Variable::default(),
        }))
    }
}

impl<'a, D: ContinuousDomain + Default> FunctionOn<'a, D> for Function<'a, D> {
    fn new(function: Box<dyn Fn(D) -> D>) -> Self {
        Function(RefCell::new(FunctionBody {
            seed: None,
            forward: Rc::new(function),
            backward: None,
            output: Variable::default(),
        }))
    }
    fn constant(value: D) -> Self {
        Function(RefCell::new(FunctionBody {
            seed: None,
            forward: Rc::new(Box::new(|x: D| x)),
            backward: None,
            output: Variable::new(value),
        }))
    }
    fn propagate_to(&'a self, seed: &'a Self) {
        seed.propagate(self)
    }
    fn propagate(&'a self, seed: &'a Self) {
        let mut binding = self.0.borrow_mut();
        let f = &binding.forward;
        let Some(input) = &seed.0.borrow().output.data else { panic!(); };
        let p = f(input.clone());
        binding.seed = Some(seed);
        binding.output.data = Some(p);
    }
    fn back_propagate(&self) -> D {
        let my_binding = self.0.borrow_mut();
        let my_output = &my_binding.output;
        let Some(factor) = my_output.grad.as_ref() else { panic!(); };
        let Some(b) = &my_binding.backward else { panic!(); };
        let Some(seed) = my_binding.seed else { panic!(); };
        let mut seed_binding = seed.0.borrow_mut();
        let seed_output = &seed_binding.output;
        let Some(i) = seed_output.data.as_ref() else { panic!(); };
        let input = i.clone();
        let g = factor.clone() * b(input);
        seed_binding.output.grad = Some(g.clone());
        g
    }
    fn set_backward(self, function: Box<dyn Fn(D) -> D>) -> Self {
        self.0.borrow_mut().backward = Some(Rc::new(function));
        self
    }
    /// step 3: function composition
    fn followed_by(&self, other: &Self) -> Self {
        let f = self.0.borrow().forward.clone();
        let g = other.0.borrow().forward.clone();
        Function::<D>::new(Box::new(move |x: D| g(f(x))))
    }
    fn numerical_diff(&'a mut self, x: &'a Variable<D>, eps: &D) -> D {
        let Variable { data: Some(input), .. } = x else { panic!(); };
        let f = &*self.0.borrow().forward;
        let x0 = input.clone() - eps.clone();
        let x1 = input.clone() + eps.clone();
        let y0 = f(x0);
        let y1 = f(x1);
        (y1 - y0) / (eps.clone() + eps.clone())
    }
    fn input(&self) -> Option<D> {
        self.0
            .borrow()
            .seed
            .and_then(|seed| seed.0.borrow().output.data.clone())
    }
    fn seed(&'a self) -> Option<&'a Function<D>> {
        self.0.borrow().seed
    }
    fn value(&self) -> Option<D> {
        self.0.borrow().output.data.clone()
    }
    fn grad(&self) -> Option<D> {
        self.0.borrow().output.grad.clone()
    }
    fn set_grad(&self, val: D) {
        self.0.borrow_mut().output.grad = Some(val);
    }
}

pub fn function_square<'a, D: ContinuousDomain + Default>(
    _lifetime_designator: &'a Function<'a, D>,
) -> Function<'a, D> {
    Function::<'a, D>::new(Box::new(|x: D| x.clone() * x)).set_backward(Box::new(|x| x.clone() + x))
}

pub fn function_exp_f32<'a>(_lifetime_designator: &'a Function<'a, f32>) -> Function<'static, f32> {
    Function::<f32>::new(Box::new(|x: f32| x.exp())).set_backward(Box::new(|x| x.exp()))
}

pub fn function_exp_f64<'a>(_lifetime_designator: &'a Function<'a, f64>) -> Function<'a, f64> {
    Function::<'a, f64>::new(Box::new(|x: f64| x.exp())).set_backward(Box::new(|x| x.exp()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_step_2() {
        let c0: Function<usize> = Function::constant(0usize);
        assert_eq!(c0.0.borrow().output.data, Some(0));
        let f0: Function<usize> = Function::<usize>::new(Box::new(|x: usize| x + 1));
        f0.propagate(&c0);
        assert_eq!(f0.0.borrow().output.data, Some(1));

        let _v1: Variable<f32> = Variable::new(2.0f32);
        let c1: Function<f64> = Function::constant(2.0f64);
        assert_eq!(c1.value(), Some(2.0));
        let f1: Function<f64> = Function::<f64>::new(Box::new(|x: f64| x + 1.0));
        f1.propagate(&c1);
        assert_eq!(f1.value(), Some(3.0));
        let f2: Function<f64> = Function::<f64>::new(Box::new(|x: f64| x.exp()));
        f2.propagate(&c1);
        assert!((f2.value().unwrap() - 7.389056).abs() < 0.001);
        let f3: Function<f64> = f2.clone();
        f3.propagate(&c1);
        assert_eq!(f3.value(), f2.value());
    }
    #[test]
    fn test_step_3() {
        let _v1: Variable<f32> = Variable::new(1.0f32);
        let c1: Function<f64> = Function::constant(1.0f64);
        assert_eq!(c1.value(), Some(1.0));
        let f: Function<f64> = Function::<f64>::new(Box::new(|x: f64| x + 1.0));
        f.propagate(&c1);
        assert_eq!(f.value(), Some(2.0));
        let c1: Function<f64> = Function::constant(1.0f64);
        let g: Function<f64> = Function::<f64>::new(Box::new(|x: f64| x.exp()));
        g.propagate(&c1);
        assert!((g.value().unwrap() - std::f64::consts::E).abs() < 0.001);
        let c1: Function<f64> = Function::constant(1.0f64);
        let fg_or_gf: Function<f64> = f.followed_by(&g);
        fg_or_gf.propagate(&c1);
        assert!((fg_or_gf.value().unwrap() - std::f64::consts::E.powi(2)).abs() < 0.001);
    }
    #[test]
    fn test_step_3_2() {
        let x: Function<f32> = Function::<f32>::constant(0.5f32);
        let a: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let b: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.exp()));
        let c: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let chain: Function<f32> = a.followed_by(&b).followed_by(&c);
        x.propagate_to(&chain);
        assert!((chain.value().unwrap() - 1.6487212).abs() < 0.001);
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
        let x = Function::constant(0.5f64);
        let fa = function_square(&x);
        let fb = function_exp_f64(&x);
        let fc = function_square::<f64>(&x);
        x.propagate_to(&fa);
        assert_eq!(x.value(), fa.input());
        fa.propagate_to(&fb);
        assert_eq!(fa.value(), fb.input());
        fb.propagate_to(&fc);
        assert_eq!(fb.value(), fc.input());
        assert!((fc.value().unwrap() - 1.6487212).abs() < 0.001);
        fc.set_grad(1.0);
        fc.back_propagate();
        fb.back_propagate();
        fa.back_propagate();
        assert!((x.grad().unwrap() - 3.29744).abs() < 0.0001);
    }
    #[test]
    fn test_step_6_3_backtrace() {
        let x = Function::constant(0.5f64);
        let fa = function_square(&x);
        let fb = function_exp_f64(&x);
        let fc = function_square::<f64>(&x);

        x.propagate_to(&fa);
        fa.propagate_to(&fb);
        fb.propagate_to(&fc);

        fc.set_grad(1.0);
        fc.back_propagate();
        let Some(f_equals_to_fb) = fc.seed() else { panic!(); };
        f_equals_to_fb.back_propagate();
        let Some(f_equals_to_fa) = f_equals_to_fb.seed() else { panic!(); };
        f_equals_to_fa.back_propagate();
        let Some(f_equals_to_x) = f_equals_to_fa.seed() else { panic!(); };
        assert!(f_equals_to_x.seed().is_none());

        assert!((x.grad().unwrap() - 3.29744).abs() < 0.0001);
    }
}
