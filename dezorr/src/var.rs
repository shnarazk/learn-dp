use crate::types::ContinuousDomain;

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
