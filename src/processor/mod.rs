use failure::Error;

use token::Token;
use types::{Portion, Quantity};

/*
Ingredient
Tool
Container
Appliance

Verb
Combine
Transfer
Measure
Take
Leave
Place
Remove
Configure
Meld
Discard
Empty
TagSet
TagGet

Simultaneous
Modifier
Annotation
Time
Until
*/

////////////////////////////////////////////////////////////////////////////////////////////////////
// Element types

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ingredient(String, Vec<String>, Vec<String>);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Tool(String, Vec<String>, Vec<String>);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Container(String, Vec<String>, Vec<String>);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Appliance(String, Vec<String>, Vec<String>);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Verb(String, Vec<String>, Vec<String>);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Combine(String, Vec<String>, Vec<String>);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Transfer(String, Vec<String>, Vec<String>);

////////////////////////////////////////////////////////////////////////////////////////////////////
// Derived types

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Mixture {
    Ingredient(Ingredient),
    Compound(Box<Mixture>, Box<Mixture>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Vessel {
    Container(Container),
    Appliance(Appliance),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct System(Vessel, Mixture);

////////////////////////////////////////////////////////////////////////////////////////////////////
// Stack items

/// The possible elements remaining after meta processing.
pub enum Operatable {
    Ingredient(Ingredient),
    Tool(Tool),
    Container(Container),
    Appliance(Appliance),
    Verb(Verb),
    Combine(Combine),
    Transfer(Transfer),
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
}

impl Operatable {
    pub fn create_operatable_stack<II: IntoIterator<Item = Token>>(tokens: II) -> Vec<Operatable> {
        tokens
            .into_iter()
            .map(|_| Operatable::Place)
            .collect::<Vec<_>>()
    }
}

/// The possible elements remaining after operator processing.
pub enum Concrete {
    Ingredient(Ingredient),
    Tool(Tool),
    Container(Container),
    Appliance(Appliance),
    Mixture(Mixture),
    Vessel(Vessel),
    System(System),
}
