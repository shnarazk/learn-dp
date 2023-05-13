#![allow(dead_code)]

use {
    crate::{func::Function, types::ContinuousDomain},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Clone)]
struct ConnectionBody<'a, D: ContinuousDomain> {
    value: D,
    source: &'a Function<'a, D>,
}

#[derive(Clone)]
pub struct Connection<'a, D: ContinuousDomain>(RefCell<ConnectionBody<'a, D>>);

impl<'a, D: ContinuousDomain> Connection<'a, D> {
    pub fn new(value: D, source: &'a Function<'a, D>) -> Self {
        Connection(RefCell::new(ConnectionBody { value, source }))
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
    fn is_terminal(&self) -> bool {
        self.arrow.is_none() && self.codomain.is_empty()
    }
    pub fn is_coterminal(&self) -> bool {
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
    pub fn apply(&mut self) {
        if let Some(f) = &self.arrow {
            self.values = self
                .domain
                .iter()
                .map(|c| f(c.0.borrow().value.clone()))
                .collect::<Vec<_>>();
        }
    }
    pub fn propagate(&self) {
        for (i, v) in self.values.iter().enumerate() {
            self.codomain[i].0.borrow_mut().value = v.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_arrow_basic() {
        let a0: Arrow<f64> = Arrow::coterminal(vec![0.0f64]);
        let _a1: Arrow<f64> = Arrow::new(Some(Box::new(|x| x + 1.0)));
        let _a2: Arrow<f64> = Arrow::new(Some(Box::new(|x| x - 1.0)));
        assert!(a0.is_coterminal());
        assert!(a0.is_terminal());
        // let _c = Connection::new(0.0f64, &c0);
    }
}
