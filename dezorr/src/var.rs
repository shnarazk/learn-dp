pub trait ContinuousDomain:
    'static
    + Clone
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Div<Output = Self>
{
}

impl ContinuousDomain for usize {}
impl ContinuousDomain for u32 {}
impl ContinuousDomain for f64 {}
impl ContinuousDomain for f32 {}

#[derive(Clone, Debug)]
pub struct Variable<D: ContinuousDomain> {
    pub data: D,
    pub grad: Option<D>,
}

impl<D: ContinuousDomain> Variable<D> {
    pub fn new(data: D) -> Self {
        Variable { data, grad: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_step_1_2() {
        let v1: Variable<usize> = Variable::new(0usize);
        assert_eq!(v1.data, 0);
        let mut v2: Variable<f32> = Variable::new(1.0f32);
        assert_eq!(v2.data, 1.0);
        v2.data = 2.0;
        assert_eq!(v2.data, 2.0);
    }
}
