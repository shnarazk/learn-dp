#[derive(Debug)]
pub struct Variable<D> {
    pub data: D,
}

impl<D> Variable<D> {
    pub fn new(data: D) -> Self {
        Variable { data }
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
