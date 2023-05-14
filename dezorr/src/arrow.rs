#![allow(dead_code)]

use {
    crate::{func::Function, types::ContinuousDomain},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
pub struct ConnectionBody<'a, D: ContinuousDomain> {
    pub value: Option<D>,
    source: &'a Function<'a, D>,
    target: &'a Function<'a, D>,
}

#[derive(Clone)]
pub struct Connection<'a, D: ContinuousDomain>(Rc<RefCell<ConnectionBody<'a, D>>>);

impl<'a, D: ContinuousDomain + std::fmt::Debug> std::fmt::Debug for Connection<'a, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let binding = self.0.borrow();
        f.debug_struct("Connection")
            .field("vaule", &binding.value)
            .finish()
    }
}
impl<'a, D: ContinuousDomain> Connection<'a, D> {
    pub fn new(value: Option<D>, source: &'a Function<'a, D>, target: &'a Function<'a, D>) -> Self {
        Connection(Rc::new(RefCell::new(ConnectionBody {
            value,
            source,
            target,
        })))
    }
    pub fn get_value(&self) -> Option<D> {
        self.0.borrow().value.clone()
    }
    pub fn set_value(&self, val: Option<D>) {
        self.0.borrow_mut().value = val;
    }
}

#[allow(clippy::complexity)]
#[derive(Default)]
pub struct Arrow<'a, D: ContinuousDomain> {
    pub domain: Vec<Connection<'a, D>>,
    pub arrow: Option<Rc<Box<dyn Fn(D) -> D>>>,
    pub values: Vec<D>,
    pub codomain: Vec<Connection<'a, D>>,
}

impl<D: ContinuousDomain> Clone for Arrow<'_, D> {
    fn clone(&self) -> Self {
        Arrow {
            domain: Vec::new(),
            arrow: self.arrow.clone(),
            values: self.values.clone(),
            codomain: Vec::new(),
        }
    }
}

impl<'a, D: ContinuousDomain> Arrow<'a, D> {
    pub fn new(function: Option<Box<dyn Fn(D) -> D>>) -> Self {
        Arrow {
            arrow: function.map(Rc::new),
            ..Arrow::default()
        }
    }
    pub fn coterminal(values: Vec<D>) -> Self {
        Arrow {
            values,
            ..Arrow::default()
        }
    }
    pub fn terminal() -> Self {
        Arrow::default()
    }
    fn is_terminal(&self) -> bool {
        self.arrow.is_none() && self.codomain.is_empty()
    }
    pub fn is_coterminal(&self) -> bool {
        self.arrow.is_none() && self.domain.is_empty() && !self.values.is_empty()
    }
    pub fn add_input(&mut self, connection: Connection<'a, D>) {
        self.domain.push(connection);
    }
    fn inputs(&'a self) -> Vec<Option<D>> {
        self.domain
            .iter()
            .map(|l| l.0.borrow().value.clone())
            .collect::<Vec<_>>()
    }
    fn outputs(&'a self) -> &[D] {
        &self.values
    }
    pub fn apply(&mut self) {
        if let Some(f) = &self.arrow {
            assert!(self.is_appliable());
            self.values = self
                .domain
                .iter()
                .map(|c| f(c.0.borrow().value.as_ref().unwrap().clone()))
                .collect::<Vec<_>>();
        }
    }
    pub fn is_appliable(&self) -> bool {
        self.domain.iter().all(|x| x.0.borrow().value.is_some())
    }
    pub fn is_applied(&self) -> bool {
        self.is_appliable() && (self.is_coterminal() || (self.domain.len() == self.values.len()))
    }
    pub fn propagate(&mut self) -> Option<Vec<&'a Function<'a, D>>> {
        (self.is_coterminal() || (!self.is_applied() && self.is_appliable())).then(|| {
            self.apply();
            assert_eq!(self.values.len(), self.codomain.len());
            for (i, v) in self.values.iter().enumerate() {
                self.codomain[i].0.borrow_mut().value = Some(v.clone());
            }
            println!("propagated");
            self.codomain
                .iter()
                .map(|c| c.0.borrow().target)
                .collect::<Vec<_>>()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::*;
    #[test]
    fn test_connection_basic() {
        let f0: Function<f64> = Function::coterminal(vec![0.0f64]);
        let c0 = Connection::new(Some(0.0f64), &f0, &f0);
        let c1 = c0.clone();
        c0.set_value(Some(10.0));
        assert_eq!(c1.get_value(), Some(10.0));
    }
    #[test]
    fn test_arrow_basic() {
        let mut a0: Arrow<f64> = Arrow::coterminal(vec![0.0f64]);
        let mut a1: Arrow<f64> = Arrow::terminal();
        let mut _a2: Arrow<f64> = Arrow::new(Some(Box::new(|x| x + 1.0)));
        let mut _a3: Arrow<f64> = Arrow::new(Some(Box::new(|x| x - 1.0)));
        let f0: Function<f64> = Function::coterminal(vec![0.0f64]);
        let f1: Function<f64> = Function::terminal(vec![1.0f64]);
        let c0 = Connection::new(Some(0.0f64), &f0, &f1);
        a0.codomain.push(c0.clone());
        assert!(a0.is_coterminal());
        assert!(a0.is_appliable());
        assert!(a0.is_applied());
        assert!(a1.is_terminal());
        a1.add_input(c0);
        // let _c = Connection::new(0.0f64, &c0);
    }
}
