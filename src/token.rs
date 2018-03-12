#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Token;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TToken {
    Ingredient(String),
    Modifier(String),
    Annotation(String),
    Action(String),
    Combination(String),
}
