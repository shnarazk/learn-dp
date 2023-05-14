pub trait ContinuousDomain:
    'static
    + Clone
    + std::fmt::Debug
    + Default
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
