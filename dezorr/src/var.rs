#![allow(dead_code)]

use {
    crate::{func::Function, types::ContinuousDomain},
    std::{
        cell::{Ref, RefCell, RefMut},
        rc::Rc,
    },
};

#[macro_export]
macro_rules! DFN {
    ($f: expr) => {
        Some(Box::new(|xs| {
            xs.iter().map(|r| $f(r.clone())).collect::<Vec<_>>()
        }))
    };
}

#[macro_export]
macro_rules! TFN {
    ($f: expr) => {
        Some(Box::new($f))
    };
}

#[derive(Clone)]
pub struct VariableBody<'a, D: ContinuousDomain> {
    pub value: Option<D>,
    pub source: &'a Function<'a, D>,
    pub target: &'a Function<'a, D>,
}

#[derive(Clone)]
pub struct Variable<'a, D: ContinuousDomain>(pub Rc<RefCell<VariableBody<'a, D>>>);

impl<'a, D: ContinuousDomain + std::fmt::Debug> std::fmt::Debug for Variable<'a, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let binding = self.0.borrow();
        f.debug_struct("Variable")
            .field("vaule", &binding.value)
            .finish()
    }
}

impl<'a, D: ContinuousDomain> Variable<'a, D> {
    pub fn new(value: Option<D>, source: &'a Function<'a, D>, target: &'a Function<'a, D>) -> Self {
        Variable(Rc::new(RefCell::new(VariableBody {
            value,
            source,
            target,
        })))
    }
    pub fn get(&'a self) -> Ref<'a, VariableBody<'a, D>> {
        self.0.borrow()
    }
    pub fn set(&'a self) -> RefMut<'a, VariableBody<'a, D>> {
        self.0.borrow_mut()
    }
    pub fn get_value(&self) -> Option<D> {
        self.0.borrow().value.clone()
    }
    pub fn set_value(&self, val: Option<D>) {
        self.0.borrow_mut().value = val;
    }
}
/*

#[derive(Clone, Default)]
pub struct Variable<D: ContinuousDomain> {
    pub val: D,
    pub grad: D,
}

impl<D: ContinuousDomain> Variable<D> {
    pub fn new(val: D) -> Self {
        Variable {
            val,
            grad: D::default(),
        }
    }
    pub fn set_grad(&mut self, g: D) -> &Self {
        self.grad = g;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_step_1_2() {
        let v1: Variable<usize> = Variable::new(0usize);
        assert_eq!(v1.val, 0);
        let mut v2: Variable<f32> = Variable::new(1.0f32);
        assert_eq!(v2.val, 1.0);
        v2.val = 2.0;
        assert_eq!(v2.val, 2.0);
    }
}
*/
