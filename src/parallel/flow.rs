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

    // TODO: This will probably need either a Gate or Slot as input.
    pub fn find_walks(&self) -> Vec<Vec<&Token>> {
        fn helper(curr_flow_item: &FlowItem, curr_walk_path: &Vec<&Token>) {
            // It is possible to have more than one new path output from this helper.
            let next_walk_paths: Vec<Vec<&Token>> = match curr_flow_item {
                &FlowItem::Token(ref token) => {
                    let mut result = curr_walk_path.clone();
                    result.push(token);
                    vec![result]
                },
                &FlowItem::Split(ref split_set) => {
                    let mut results: Vec<Vec<&Token>> = vec![];

                    let mut split_walks = split_set.find_walks();
                    for mut split_walk in &mut split_walks {
                        let mut result = curr_walk_path.clone();
                        result.append(&mut split_walk);
                        results.push(result);
                    }

                    results
                },
            };
        }

        vec![]
    }
}
