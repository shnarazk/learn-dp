pub trait ContinuousDomain:
    'static
    + Clone
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
{
}

impl ContinuousDomain for usize {}
impl ContinuousDomain for u32 {}
impl ContinuousDomain for f64 {}
impl ContinuousDomain for f32 {}

#[derive(Clone, Default)]
pub struct Variable<D: ContinuousDomain + Default> {
    pub data: Option<D>,
    pub grad: Option<D>,
}

impl<D: ContinuousDomain + Default> Variable<D> {
    pub fn new(data: D) -> Self {
        Variable {
            data: Some(data),
            grad: None,
        }
    }
    pub fn set_grad(&mut self, g: D) -> &Self {
        self.grad = Some(g);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_step_1_2() {
        let v1: Variable<usize> = Variable::new(0usize);
        assert_eq!(v1.data.unwrap(), 0);
        let mut v2: Variable<f32> = Variable::new(1.0f32);
        assert_eq!(v2.data.unwrap(), 1.0);
        v2.data = Some(2.0);
        assert_eq!(v2.data.unwrap(), 2.0);
    }
}
