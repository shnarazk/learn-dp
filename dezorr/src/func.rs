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

#[macro_export]
macro_rules! VARIABLE {
    ($($c: expr),+) => {
        Function::coterminal(vec![$($c),*])
    };
}

#[macro_export]
macro_rules! TERMINAL {
    ($($c: expr),+) => {
        Function::terminal(vec![$($c),*])
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
    fn followed_by(&'a self, other: &'a Self) -> &Self;
    fn numerical_diff(&self, x: &D, eps: &D) -> D;
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
    fn propagate_f(&'a self) -> Option<Vec<&'a Function<'a, D>>> {
        self.0.borrow_mut().f.propagate_forward()
    }
    fn propagate_b(&'a self) -> Option<Vec<&'a Function<'a, D>>> {
        let inputs = self.on_f(|a| a.inputs());
        let inputs = inputs
            .iter()
            .map(|x| x.as_ref().unwrap())
            .collect::<Vec<&D>>();
        self.0.borrow_mut().b.propagate_backward(&inputs)
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
            // assert!(source_binding.codomain.len()< source_binding.values.len());
            let link = Connection::new(None, self, target);
            source_binding.add_output(link.clone());
            dist_binding.add_input(link);
        }
        {
            // backward bonding
            let source_binding = &mut target.0.borrow_mut().b;
            let dist_binding = &mut self.0.borrow_mut().b;
            let link = Connection::new(None, target, self);
            source_binding.add_output(link.clone());
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
    fn followed_by(&'a self, other: &'a Self) -> &'a Self {
        self.link_to(other);
        other
    }
    fn numerical_diff(&self, x: &D, eps: &D) -> D {
        (self.on_f(|a| (a.arrow.as_ref().unwrap())(x.clone() + eps.clone()))
            - self.on_f(|a| a.arrow.as_ref().unwrap()(x.clone() - eps.clone())))
            / (eps.clone() + eps.clone())
    }
}

fn square<'a, D: ContinuousDomain>(_lifetime: &'a Function<'a, D>) -> Function<'a, D> {
    Function::new(DFN!(|x: D| x.clone() * x), DFN!(|x| x.clone() + x))
}

fn exp_f32<'a>(_lifetime_designator: &'a Function<'a, f32>) -> Function<'a, f32> {
    Function::<f32>::new(DFN!(|x: f32| x.exp()), DFN!(|x| x.exp()))
}

fn exp_f64<'a>(_lifetime_designator: &'a Function<'a, f64>) -> Function<'a, f64> {
    Function::<f64>::new(DFN!(|x: f64| x.exp()), DFN!(|x| x.exp()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_step_2_base1() {
        let c0: Function<usize> = VARIABLE!(0);
        let f0: Function<usize> = Function::<usize>::new(DFN!(|x| x + 1), DFN!(|_| 1));
        let y0: Function<usize> = TERMINAL!(1);
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
    #[test]
    fn test_step_2_base2() {
        let c0: Function<usize> = VARIABLE!(0);
        let f0: Function<usize> = Function::<usize>::new(DFN!(|x| x + 1), DFN!(|_| 1));
        let y0: Function<usize> = TERMINAL!(1);
        c0.link_to(&f0);
        f0.link_to(&y0);
        c0.propagate_forward();
        assert_eq!(f0.on_f(|a| a.outputs()), vec![1]);
        assert_eq!(y0.on_f(|a| a.inputs()), vec![Some(1)]);
    }
    #[test]
    fn test_step_2_base3() {
        let x: Function<f64> = VARIABLE!(2.0, -1.0);
        let y: Function<f64> = TERMINAL!(1.0, 1.0);
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
    }
    #[test]
    fn test_step_2_base4() {
        let x: Function<f64> = VARIABLE!(1.0, 2.0);
        let y: Function<f64> = TERMINAL!(1.0, 1.0);
        let fa: Function<f64> = Function::new(DFN!(|x| 2.0 * x), DFN!(|_| 2.0));
        let fb: Function<f64> = Function::new(DFN!(|x| 1.0 / x), DFN!(|x| -1.0 * x.powi(-2)));
        // let f1 = fa.followed_by(&fb);
        x.link_to(&fa);
        x.link_to(&fa);
        fa.link_to(&fb);
        fa.link_to(&fb);
        fb.link_to(&y);
        fb.link_to(&y);
        x.propagate_forward();
        y.propagate_backward();
        assert_eq!(y.on_f(|a| a.outputs()), vec![0.5f64, 0.25f64]);
        assert_eq!(
            x.on_b(|a| a.outputs()),
            vec![2.0f64 * -1.0 / 4.0, 2.0f64 * -1.0 / 16.0]
        );
    }
    #[test]
    fn test_step_2_base5() {
        let x: Function<f64> = VARIABLE!(1.0f64, 0.0);
        let y: Function<f64> = TERMINAL!(1.0, 1.0);
        let f0: Function<f64> = Function::new(DFN!(|x| x.exp()), DFN!(|x| x.exp()));
        let f1: Function<f64> = Function::new(DFN!(|x| x.exp()), DFN!(|x| x.exp()));
        x.link_to(&f0);
        x.link_to(&f0);
        f0.link_to(&f1);
        f0.link_to(&f1);
        f1.link_to(&y);
        f1.link_to(&y);
        x.propagate_forward();
        y.propagate_backward();
        assert_eq!(
            y.on_f(|a| a.outputs()),
            vec![1.0f64.exp().exp(), 0.0f64.exp().exp()]
        );
        assert_eq!(
            x.on_b(|a| a.outputs()),
            vec![
                1.0f64.exp() * 1.0f64.exp().exp(),
                0.0f64.exp() * 0.0f64.exp().exp()
            ]
        );
    }
    #[test]
    fn test_step_2_2() {
        let x: Function<usize> = VARIABLE!(10);
        let f: Function<usize> = square::<usize>(&x);
        let y: Function<usize> = TERMINAL!(1);
        x.followed_by(&f).followed_by(&y);
        x.propagate_forward();
        y.propagate_backward();
        assert_eq!(y.on_f(|a| a.outputs()), vec![100]);
        assert_eq!(x.on_b(|a| a.outputs()), vec![20]);
    }
    #[test]
    fn test_step_3_2() {
        let x: Function<f64> = VARIABLE!(0.5);
        let y: Function<f64> = TERMINAL!(1.0);
        let a: Function<f64> = square::<f64>(&x);
        let b: Function<f64> = exp_f64(&x);
        let c: Function<f64> = square::<f64>(&x);
        x.followed_by(&a)
            .followed_by(&b)
            .followed_by(&c)
            .followed_by(&y);
        x.propagate_forward();
        y.propagate_backward();
        assert!((y.on_f(|a| a.outputs())[0] - 1.64872127).abs() < 0.0001);
    }
    /*
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
