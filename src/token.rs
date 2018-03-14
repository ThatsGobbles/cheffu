use std::borrow::Cow;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Token {
    Ingredient(String),
    Modifier(String),
    Annotation(String),
    Action(String),
    Combination(String),
}
