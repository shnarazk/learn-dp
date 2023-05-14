#![allow(dead_code)]

use {
    crate::{
        arrow::{Arrow, Connection},
        types::ContinuousDomain,
    },
    std::{cell::RefCell, collections::VecDeque},
};

#[macro_export]
macro_rules! DFN {
    ($f: expr) => {
        Some(Box::new($f))
    };
}

pub trait FunctionOn<'a, D: ContinuousDomain> {
    fn on_f<T>(&self, f: impl Fn(&Arrow<'a, D>) -> T) -> T;
    fn on_b<T>(&self, f: impl Fn(&Arrow<'a, D>) -> T) -> T;
    fn new(arrow: Option<Box<dyn Fn(D) -> D>>, coarrow: Option<Box<dyn Fn(D) -> D>>) -> Self;
    fn coterminal(value: Vec<D>) -> Self;
    fn terminal(value: Vec<D>) -> Self;
    fn is_coterminal(&'a self) -> bool;
    fn link_to(&'a self, other: &'a Self);
    fn propagate_forward(&'a self);
    fn propagate_backward(&'a self);
    fn followed_by(&'a self, other: &'a Self) -> Self;
    // fn numerical_diff(&'a mut self, x: &'a Variable<D>, eps: &D) -> D;
    // fn inputs(&'a self) -> Iter<'a, &'a D>;
    // fn outputs(&self) -> Iter<&'a D>;
    // fn set_grad(&self, val: D);
}

#[derive(Debug, Default)]
struct FunctionBody<'a, D: ContinuousDomain> {
    f: Arrow<'a, D>,
    b: Arrow<'a, D>,
}

#[derive(Debug)]
pub struct Function<'a, D: ContinuousDomain>(RefCell<FunctionBody<'a, D>>);

impl<D: ContinuousDomain> Clone for Function<'_, D> {
    fn clone(&self) -> Self {
        Function(RefCell::new(FunctionBody {
            f: self.0.borrow().f.clone(),
            b: self.0.borrow().b.clone(),
        }))
    }
}

impl<'a, D: ContinuousDomain> Function<'a, D> {
    // fn apply_f(&self) {
    //     self.0.borrow_mut().f.apply();
    // }
    // fn apply_b(&self) {
    //     self.0.borrow_mut().b.apply();
    // }
    fn propagate_f(&'a self) -> Option<Vec<&'a Function<'a, D>>> {
        self.0.borrow_mut().f.propagate()
    }
    fn propagate_b(&'a self) -> Option<Vec<&'a Function<'a, D>>> {
        self.0.borrow_mut().b.propagate()
    }
}

impl<'a, D: ContinuousDomain> FunctionOn<'a, D> for Function<'a, D> {
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
    fn terminal(values: Vec<D>) -> Self {
        Function(RefCell::new(FunctionBody {
            f: Arrow::default(),
            b: Arrow::coterminal(values),
        }))
    }
    fn on_f<T>(&self, f: impl Fn(&Arrow<'a, D>) -> T) -> T {
        let a = &self.0.borrow().f;
        f(a)
    }
    fn on_b<T>(&self, f: impl Fn(&Arrow<'a, D>) -> T) -> T {
        let a = &self.0.borrow().b;
        f(a)
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
    fn link_to(&'a self, target: &'a Self) {
        {
            // forward bonding
            let source_binding = &mut self.0.borrow_mut().f;
            let dist_binding = &mut target.0.borrow_mut().f;
            // let num_used = source_binding.codomain.len();
            // assert!(num_used < source_binding.values.len());
            let link = Connection::new(None, self, target);
            // source_binding.codomain.push(link.clone());
            source_binding.add_output(link.clone());
            // dist_binding.domain.push(link);
            dist_binding.add_input(link);
        }
        {
            // backward bonding
            let source_binding = &mut target.0.borrow_mut().b;
            let dist_binding = &mut self.0.borrow_mut().b;
            // let num_used = source_binding.codomain.len();
            // assert!(num_used < source_binding.values.len());
            let link = Connection::new(None, target, self);
            // source_binding.codomain.push(link.clone());
            source_binding.add_output(link.clone());
            // dist_binding.domain.push(link);
            dist_binding.add_input(link);
        }
    }
    fn propagate_forward(&'a self) {
        let mut to_propagate = VecDeque::new();
        to_propagate.push_front(self);
        while let Some(f) = to_propagate.pop_front() {
            if let Some(fs) = f.propagate_f() {
                for g in fs.iter() {
                    to_propagate.push_back(*g);
                }
            }
        }
    }
    fn propagate_backward(&'a self) {
        let mut to_propagate = VecDeque::new();
        to_propagate.push_front(self);
        while let Some(f) = to_propagate.pop_front() {
            if let Some(fs) = f.propagate_b() {
                for g in fs.iter() {
                    to_propagate.push_back(*g);
                }
            }
        }
    }
    /// step 3: function composition
    fn followed_by(&'a self, other: &'a Self) -> Self {
        let f_f = self.0.borrow().f.arrow.clone();
        let f_g = other.0.borrow().f.arrow.clone();
        let b_f = self.0.borrow().b.arrow.clone();
        let b_g = other.0.borrow().b.arrow.clone();
        Function::new(
            match (f_g, f_f) {
                (Some(g), Some(f)) => DFN!(move |x: D| g(f(x))),
                (Some(g), None) => DFN!(move |x: D| g(x)),
                (None, Some(f)) => DFN!(move |x: D| f(x)),
                (None, None) => None,
            },
            match (b_g, b_f) {
                (Some(g), Some(f)) => DFN!(move |x: D| f(g(x))),
                (Some(g), None) => DFN!(move |x: D| g(x)),
                (None, Some(f)) => DFN!(move |x: D| f(x)),
                (None, None) => None,
            },
        )
    }
    /*
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

fn square<'a, D: ContinuousDomain>(_lifetime: &'a Function<'a, D>) -> Function<'a, D> {
    Function::new(DFN!(|x: D| x.clone() * x), DFN!(|x| x.clone() + x))
}

fn exp_f32<'a>(_lifetime_designator: &'a Function<'a, f32>) -> Function<'static, f32> {
    Function::<f32>::new(DFN!(|x: f32| x.exp()), DFN!(|x| x.exp()))
}

fn exp_f64<'a>(_lifetime_designator: &'a Function<'a, f64>) -> Function<'static, f64> {
    Function::<f64>::new(DFN!(|x: f64| x.exp()), DFN!(|x| x.exp()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_step_2() {
        {
            let c0: Function<usize> = Function::coterminal(vec![0usize]);
            let f0: Function<usize> = Function::<usize>::new(DFN!(|x| x + 1), DFN!(|_| 1));
            let y0: Function<usize> = Function::<usize>::terminal(vec![1usize]);
            c0.link_to(&f0);
            f0.link_to(&y0);
            dbg!(c0.propagate_f());
            assert!(c0.on_f(|a| a.is_applied()));
            assert!(f0.on_f(|a| a.is_applicable()));
            assert!(f0.on_f(|a| !a.is_applied()));
            dbg!(f0.propagate_f());
            assert!(f0.on_f(|a| a.is_applied()));
            assert_eq!(f0.on_f(|a| a.outputs()), vec![1]);
            assert_eq!(y0.on_f(|a| a.inputs()), vec![Some(1)]);
        }
        {
            let c0: Function<usize> = Function::coterminal(vec![0usize]);
            let f0: Function<usize> = Function::<usize>::new(DFN!(|x| x + 1), DFN!(|_| 1));
            let y0: Function<usize> = Function::<usize>::terminal(vec![1usize]);
            c0.link_to(&f0);
            f0.link_to(&y0);
            c0.propagate_forward();
            assert_eq!(f0.on_f(|a| a.outputs()), vec![1]);
            assert_eq!(y0.on_f(|a| a.inputs()), vec![Some(1)]);
        }
        {
            let x: Function<f64> = Function::coterminal(vec![2.0, -1.0]);
            let y: Function<f64> = Function::terminal(vec![1.0, 1.0]);
            let f1: Function<f64> = Function::new(DFN!(|x: f64| x + 1.0), DFN!(|_| 1.0));
            x.link_to(&f1);
            x.link_to(&f1);
            f1.link_to(&y);
            f1.link_to(&y);
            x.propagate_forward();
            y.propagate_backward();
            assert_eq!(f1.on_f(|a| a.outputs()), vec![3.0, 0.0]);
            assert_eq!(y.on_f(|a| a.inputs()), vec![Some(3.0), Some(0.0)]);
            assert_eq!(y.on_f(|a| a.outputs()), vec![3.0, 0.0]);
            assert_eq!(x.on_b(|a| a.outputs()), vec![1.0, 1.0]);
            // c1.propagate_value_to(&f1);
            // assert_eq!(f1.value(), Some(3.0));
            // let f2: Function<f64> = Function::<f64>::new(Box::new(|x: f64| x.exp()));
            // c1.propagate_value_to(&f2);
            // assert!((f2.value().unwrap() - 7.389056).abs() < 0.001);
            // let f3: Function<f64> = f2.clone();
            // c1.propagate_value_to(&f3);
            // assert_eq!(f3.value(), f2.value());
        }
    }
    /*
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
    */
}
