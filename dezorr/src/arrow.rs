use {
    crate::{func::Function, types::ContinuousDomain, var::Variable},
    std::rc::Rc,
};

pub type ArrowType<D> = Box<dyn Fn(&[D]) -> Vec<D>>;

#[allow(clippy::complexity)]
#[derive(Default)]
pub struct Arrow<'a, D: ContinuousDomain> {
    domain: Vec<Variable<'a, D>>,
    pub arrow: Option<Rc<ArrowType<D>>>,
    values: Vec<D>,
    codomain: Vec<Variable<'a, D>>,
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

impl<'a, D: ContinuousDomain + std::fmt::Debug> std::fmt::Debug for Arrow<'a, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct(if self.arrow.is_some() {
            "Arrow"
        } else {
            "Variable"
        })
        .field("domain", &self.domain)
        .field("values", &self.values)
        .field("codomain", &self.codomain)
        .finish()
    }
}

impl<'a, D: ContinuousDomain> Arrow<'a, D> {
    pub fn new(function: Option<ArrowType<D>>) -> Self {
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
    pub fn add_input(&mut self, connection: Variable<'a, D>) {
        self.domain.push(connection);
    }
    pub fn add_output(&mut self, connection: Variable<'a, D>) {
        self.codomain.push(connection);
    }
    pub fn inputs(&self) -> Vec<Option<D>> {
        self.domain
            .iter()
            .map(|l| l.0.borrow().value.clone())
            .collect::<Vec<Option<D>>>()
    }
    pub fn outputs(&self) -> Vec<D> {
        self.values.to_vec()
    }
    pub fn is_applicable(&self) -> bool {
        self.domain.iter().all(|x| x.0.borrow().value.is_some())
    }
    pub fn is_applied(&self) -> bool {
        // self.is_applicable() && (self.domain.len() == self.values.len())
        !self.values.is_empty()
    }
    pub fn apply_f(&mut self) {
        if let Some(f) = &self.arrow {
            // normal arrow
            assert!(self.is_applicable());
            // self.values = self
            //     .domain
            //     .iter()
            //     .map(|c| f(c.0.borrow().value.as_ref().unwrap().clone()))
            let data = self
                .domain
                .iter()
                .map(|c| c.0.borrow().value.as_ref().unwrap().clone())
                .collect::<Vec<D>>();
            self.values = f(&data);
        } else if !self.domain.is_empty() && self.values.is_empty() {
            // terminal
            self.values = self
                .domain
                .iter()
                .map(|c| c.0.borrow().value.as_ref().unwrap().clone())
                .collect::<Vec<_>>();
        }
    }
    pub fn apply_b(&mut self, forward: &[&D]) {
        if let Some(f) = &self.arrow {
            // normal arrow
            assert!(self.is_applicable());
            // self.values = self
            //     .domain
            //     .iter()
            //     .enumerate()
            //     .map(|(i, k)| k.0.borrow().value.as_ref().unwrap().clone() * f(forward[i].clone()))
            //     .collect::<Vec<_>>();
            let data = forward.iter().map(|v| (*v).clone()).collect::<Vec<_>>();
            let output = f(&data);
            self.values = output
                .into_iter()
                .enumerate()
                .map(|(i, x)| self.domain[i].0.borrow().value.as_ref().unwrap().clone() * x)
                .collect::<Vec<_>>();
        } else if !self.domain.is_empty() && self.values.is_empty() {
            // terminal
            self.values = self
                .domain
                .iter()
                .map(|c| c.0.borrow().value.as_ref().unwrap().clone())
                .collect::<Vec<_>>();
        }
    }
    pub fn propagate_forward(&mut self) -> Option<Vec<&'a Function<'a, D>>> {
        (self.is_coterminal()
            || (self.is_terminal() && (!self.is_applied()) || self.is_applicable()))
        .then(|| {
            self.apply_f();
            assert!(self.is_terminal() || self.values.len() == self.codomain.len());
            for (i, t) in self.codomain.iter().enumerate() {
                t.0.borrow_mut().value = Some(self.values[i].clone());
            }
            self.codomain
                .iter()
                .map(|c| c.0.borrow().target)
                .collect::<Vec<_>>()
        })
    }
    pub fn propagate_backward(&mut self, forward: &[&D]) -> Option<Vec<&'a Function<'a, D>>> {
        (self.is_coterminal()
            || (self.is_terminal() && (!self.is_applied()) || self.is_applicable()))
        .then(|| {
            self.apply_b(forward);
            assert!(self.is_terminal() || self.values.len() == self.codomain.len());
            for (i, t) in self.codomain.iter().enumerate() {
                t.0.borrow_mut().value = Some(self.values[i].clone());
            }
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
    use crate::{func::*, DFN};
    #[test]
    fn test_connection_basic() {
        let f0: Function<f64> = Function::coterminal(vec![0.0f64]);
        let c0 = Variable::new(Some(0.0f64), &f0, &f0);
        let c1 = c0.clone();
        c0.set_value(Some(10.0));
        assert_eq!(c1.get_value(), Some(10.0));
    }
    #[test]
    fn test_arrow_basic() {
        let mut a0: Arrow<f64> = Arrow::coterminal(vec![0.0f64]);
        let mut a1: Arrow<f64> = Arrow::terminal();
        let mut _a2: Arrow<f64> = Arrow::new(DFN!(|x| x + 1.0));
        let mut _a3: Arrow<f64> = Arrow::new(DFN!(|x| x - 1.0));
        let f0: Function<f64> = Function::coterminal(vec![0.0f64]);
        let f1: Function<f64> = Function::terminal(vec![1.0f64]);
        let c0 = Variable::new(Some(0.0f64), &f0, &f1);
        a0.codomain.push(c0.clone());
        assert!(a0.is_coterminal());
        assert!(a0.is_applicable());
        assert!(a0.is_applied());
        assert!(a1.is_terminal());
        a1.add_input(c0);
        // let _c = Variable::new(0.0f64, &c0);
    }
}
