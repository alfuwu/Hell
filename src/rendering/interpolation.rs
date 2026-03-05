#[repr(u8)]
#[derive(Clone, PartialEq, Debug)]
pub enum Interpolation {
    Linear = 0,
    Quadratic = 1,
    Cubic = 2,
    None = 3
}
impl From<u8> for Interpolation {
    fn from(value: u8) -> Interpolation {
        match value {
            1 => Interpolation::Quadratic,
            2 => Interpolation::Cubic,
            3 => Interpolation::None,
            _ => Interpolation::Linear
        }
    }
}