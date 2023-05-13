#![allow(dead_code)]

use {
    crate::var::ContinuousDomain,
    std::{cell::RefCell, rc::Rc},
};

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

pub trait FunctionOn<'a, D: ContinuousDomain + Default> {
    fn new(arrow: Option<Box<dyn Fn(D) -> D>>, coarrow: Option<Box<dyn Fn(D) -> D>>) -> Self;
    fn coterminal(value: Vec<D>) -> Self;
    fn is_coterminal(&'a self) -> bool;
    fn apply_f(&self);
    fn apply_b(&self);
    fn propagate_f(&self);
    fn propagate_b(&self);
    fn link_to(&'a self, other: &'a Self);
    // fn propagate_from(&'a self, v: &[&'a Self]);
    // fn propagate_backward(&'a self, base: D);
    // fn set_backward(self, function: Box<dyn Fn(D) -> D>) -> Self;
    // fn followed_by(&self, other: &Self) -> Self;
    // fn numerical_diff(&'a mut self, x: &'a Variable<D>, eps: &D) -> D;
    // fn inputs(&'a self) -> Iter<'a, &'a D>;
    // fn outputs(&self) -> Iter<&'a D>;
    // fn set_grad(&self, val: D);
}

#[allow(clippy::complexity)]
#[derive(Default)]
pub struct Arrow<'a, D: ContinuousDomain + Default> {
    domain: Vec<Connection<'a, D>>,
    arrow: Option<Rc<Box<dyn Fn(D) -> D>>>,
    values: Vec<D>,
    codomain: Vec<Connection<'a, D>>,
}

impl<D: ContinuousDomain + Default> Clone for Arrow<'_, D> {
    fn clone(&self) -> Self {
        Arrow {
            domain: Vec::new(),
            arrow: self.arrow.clone(),
            values: self.values.clone(),
            codomain: Vec::new(),
        }
    }
}

impl<'a, D: ContinuousDomain + Default> Arrow<'a, D> {
    fn new(function: Option<Box<dyn Fn(D) -> D>>) -> Self {
        Arrow {
            arrow: function.map(Rc::new),
            ..Arrow::default()
        }
    }
    fn coterminal(values: Vec<D>) -> Self {
        Arrow {
            values,
            ..Arrow::default()
        }
    }
    fn is_terminal(&self) -> bool {
        self.arrow.is_none() && self.codomain.is_empty()
    }
    fn is_coterminal(&self) -> bool {
        self.arrow.is_none() && self.domain.is_empty() && !self.values.is_empty()
    }
    fn inputs(&'a self) -> Vec<D> {
        self.domain
            .iter()
            .map(|l| l.0.borrow().value.clone())
            .collect::<Vec<_>>()
    }
    fn outputs(&'a self) -> &[D] {
        &self.values
    }
    fn apply(&mut self) {
        if let Some(f) = &self.arrow {
            self.values = self
                .domain
                .iter()
                .map(|c| f(c.0.borrow().value.clone()))
                .collect::<Vec<_>>();
        }
    }
    fn propagate(&self) {
        for (i, v) in self.values.iter().enumerate() {
            self.codomain[i].0.borrow_mut().value = v.clone();
        }
    }
}

#[allow(clippy::complexity)]
#[derive(Default)]
pub struct FunctionBody<'a, D: ContinuousDomain + Default> {
    f: Arrow<'a, D>,
    b: Arrow<'a, D>,
}

pub struct Function<'a, D: ContinuousDomain + Default>(RefCell<FunctionBody<'a, D>>);

impl<D: ContinuousDomain + Default> Clone for Function<'_, D> {
    fn clone(&self) -> Self {
        Function(RefCell::new(FunctionBody {
            f: self.0.borrow().f.clone(),
            b: self.0.borrow().b.clone(),
        }))
    }
}

impl<'a, D: ContinuousDomain + Default> FunctionOn<'a, D> for Function<'a, D> {
    fn new(arrow: Option<Box<dyn Fn(D) -> D>>, coarrow: Option<Box<dyn Fn(D) -> D>>) -> Self {
        Function(RefCell::new(FunctionBody {
            f: Arrow::new(arrow),
            b: Arrow::new(coarrow),
        }))
    }
    fn coterminal(values: Vec<D>) -> Self {
        Function(RefCell::new(FunctionBody {
            f: Arrow::coterminal(values),
            b: Arrow::default(),
        }))
    }
    // fn inputs(&self) -> Iter<&D> {
    //     self.0.borrow().f.domain.iter().map(|l| &l.value)
    // }
    // fn outputs(&self) -> Iter<&D> {
    //     self.0.borrow().f.values.iter()
    // }
    fn is_coterminal(&self) -> bool {
        let binding = self.0.borrow();
        let f = &binding.f;
        f.is_coterminal()
    }
    fn apply_f(&self) {
        self.0.borrow_mut().f.apply();
    }
    fn apply_b(&self) {
        self.0.borrow_mut().b.apply();
    }
    fn propagate_f(&self) {
        self.0.borrow_mut().f.propagate();
    }
    fn propagate_b(&self) {
        self.0.borrow_mut().b.propagate();
    }
    fn link_to(&'a self, other: &'a Self) {
        {
            // forward bind
            let source_binding = &mut self.0.borrow_mut().f;
            let dist_binding = &mut other.0.borrow_mut().f;
            let num_used = source_binding.values.len();
            let link = Connection::new(source_binding.values[num_used].clone(), self);
            source_binding.codomain.push(link.clone());
            dist_binding.domain.push(link);
        }
        {
            // backward bind
            let source_binding = &mut other.0.borrow_mut().b;
            let dist_binding = &mut self.0.borrow_mut().b;
            let num_used = source_binding.values.len();
            let link = Connection::new(source_binding.values[num_used].clone(), self);
            source_binding.codomain.push(link.clone());
            dist_binding.domain.push(link);
        }
    }
    /*
    // fn propagate_value_to(&'a self, target: &'a Self) {
    //     let mut target_binding = target.0.borrow_mut();
    //     let f = &target_binding.forward;
    //     let Some(input) = &self.0.borrow().output.data else { panic!(); };
    //     let p = f(input.clone());
    //     target_binding.seed = Some(self);
    //     target_binding.output.data = Some(p);
    // }
    fn propagate_from(&'a self, sources: &[&'a Self]) {
        let mut self_binding = self.0.borrow_mut();
        let f = &self_binding.forward;
        let vars = sources
            .iter()
            .map(|s| {
                let p = f(s.outbut.data.clone());
                Variable::new(p)
            })
            .collect();
        self_binding.seeds = sources.to_vec();
        self_binding.outputs = vars;
    }
    fn propagate_grad(&self) -> D {
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
    fn propagate_backward(&'a self, base: D) {
        self.set_grad(base);
        let mut stack = vec![self];
        while let Some(f) = stack.pop() {
            if !f.is_constant() {
                f.propagate_grad();
                f.seeds().for_each(|s| stack.push(s));
            }
        }
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
    fn set_grad(&self, val: D) {
        self.0.borrow_mut().outputs.iter_mut().map(|v| v.grad = val);
    }
    */
}
/*
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
        c0.propagate_value_to(&f0);
        assert_eq!(f0.0.borrow().output.data, Some(1));

        let _v1: Variable<f32> = Variable::new(2.0f32);
        let c1: Function<f64> = Function::constant(2.0f64);
        assert_eq!(c1.value(), Some(2.0));
        let f1: Function<f64> = Function::<f64>::new(Box::new(|x: f64| x + 1.0));
        c1.propagate_value_to(&f1);
        assert_eq!(f1.value(), Some(3.0));
        let f2: Function<f64> = Function::<f64>::new(Box::new(|x: f64| x.exp()));
        c1.propagate_value_to(&f2);
        assert!((f2.value().unwrap() - 7.389056).abs() < 0.001);
        let f3: Function<f64> = f2.clone();
        c1.propagate_value_to(&f3);
        assert_eq!(f3.value(), f2.value());
    }
    #[test]
    fn test_step_3() {
        let c1: Function<f64> = Function::constant(1.0f64);
        assert_eq!(c1.value(), Some(1.0));
        let f: Function<f64> = Function::<f64>::new(Box::new(|x: f64| x + 1.0));
        c1.propagate_value_to(&f);
        assert_eq!(f.value(), Some(2.0));
        let c1: Function<f64> = Function::constant(1.0f64);
        let g: Function<f64> = Function::<f64>::new(Box::new(|x: f64| x.exp()));
        c1.propagate_value_to(&g);
        assert!((g.value().unwrap() - std::f64::consts::E).abs() < 0.001);
        let c1: Function<f64> = Function::constant(1.0f64);
        let fg: Function<f64> = f.followed_by(&g);
        c1.propagate_value_to(&fg);
        assert!((fg.value().unwrap() - std::f64::consts::E.powi(2)).abs() < 0.001);
    }
    #[test]
    fn test_step_3_2() {
        let x: Function<f32> = Function::<f32>::constant(0.5f32);
        let a: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let b: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.exp()));
        let c: Function<f32> = Function::<f32>::new(Box::new(|x: f32| x.powi(2)));
        let chain: Function<f32> = a.followed_by(&b).followed_by(&c);
        x.propagate_value_to(&chain);
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
        x.propagate_value_to(&fa);
        assert_eq!(x.value(), fa.input());
        fa.propagate_value_to(&fb);
        assert_eq!(fa.value(), fb.input());
        fb.propagate_value_to(&fc);
        assert_eq!(fb.value(), fc.input());
        assert!((fc.value().unwrap() - 1.6487212).abs() < 0.001);
        fc.set_grad(1.0);
        fc.propagate_grad();
        fb.propagate_grad();
        fa.propagate_grad();
        assert!((x.grad().unwrap() - 3.29744).abs() < 0.0001);
    }
    #[test]
    fn test_step_7_3() {
        let x0 = Function::constant(0.5f64);
        let fa = function_square(&x0);
        let fb = function_exp_f64(&x0);
        let fc = function_square::<f64>(&x0);

        x0.propagate_value_to(&fa);
        fa.propagate_value_to(&fb);
        fb.propagate_value_to(&fc);

        fc.propagate_backward(1.0);

        assert!((x0.grad().unwrap() - 3.2974425).abs() < 0.00001);
    }
    #[test]
    fn test_step_8_3() {
        test_step_7_3();
    }
}
*/
