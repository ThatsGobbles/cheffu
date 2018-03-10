#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Token;


pub enum TToken {
    Ingredient(String),
    Modifier(String),
    Annotation(String),
    Action(String),
    Combination(String),
}
