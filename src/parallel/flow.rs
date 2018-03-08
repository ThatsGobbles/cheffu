#![macro_use]

use std::collections::btree_set::Iter as BTreeSetIter;
use std::iter::{IntoIterator, FromIterator};

use super::gate::{Slot, Gate};
use super::scope::Scope;
use super::split::{Split, SplitSet};
use token::Token;

macro_rules! flow {
    ( $($split:expr),* $(,)? ) => (Flow::new(vec!($($split),*)));
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum FlowItem {
    Token(Token),
    Split(SplitSet),
}

/// Contains the tokens and splits that comprise all the variants of a single recipe.
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Flow(Vec<FlowItem>);

impl<'a> IntoIterator for &'a Flow {
    type Item = &'a FlowItem;
    type IntoIter = <&'a Vec<FlowItem> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for Flow {
    type Item = FlowItem;
    type IntoIter = <Vec<FlowItem> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Flow {
    pub fn new(flow: Vec<FlowItem>) -> Self {
        Flow(flow)
    }
}
