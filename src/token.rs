use types::{Portion, Quantity};

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Token {
    Ingredient(String),
    Tool(String),
    Container(String),
    Appliance(String),

    Verb(String),
    Combine(String),
    Transfer(String),
    Measure(Quantity),
    Take(Portion),
    Leave(Portion),
    Place,
    Remove,
    Configure(String),
    Meld(String),
    Discard,
    Empty,
    TagSet(String),
    TagGet(String),

    Modifier(String),
    Annotation(String),
}
