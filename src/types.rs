#[derive(Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Quantity;

#[derive(Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Portion {
    Pseudo(String),
    Quantity(Quantity),
    Fraction(u8, u8),
}
