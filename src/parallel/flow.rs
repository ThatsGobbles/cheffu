#![macro_use]

use std::collections::VecDeque;
use std::iter::{IntoIterator, FromIterator};
use std::borrow::Cow;

use failure::Error;

use super::gate::{Slot, Gate};
use super::scope::Scope;
use super::split::{Split, SplitSet};
use token::Token;

#[derive(Debug, Fail, PartialEq, Eq)]
pub enum SlotStackError {
    #[fail(display = "stack is empty")]
    Empty,

    #[fail(display = "leftover items in stack; found: {:?}", leftover)]
    Leftover {
        leftover: Vec<Slot>,
    },
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

impl<'a> IntoIterator for &'a mut Flow {
    type Item = &'a mut FlowItem;
    type IntoIter = <&'a mut Vec<FlowItem> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl IntoIterator for Flow {
    type Item = FlowItem;
    type IntoIter = <Vec<FlowItem> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> From<Flow> for Cow<'a, Flow> {
    fn from(flow: Flow) -> Self {
        Cow::Owned(flow)
    }
}

impl<'a> From<&'a Flow> for Cow<'a, Flow> {
    fn from(flow: &'a Flow) -> Self {
        Cow::Borrowed(flow)
    }
}

impl Flow {
    pub fn new(flow: Vec<FlowItem>) -> Self {
        Flow(flow)
    }

    pub fn find_walks(&self, mut slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
        let mut results: Vec<Vec<&Token>> = vec![vec![]];
        let mut opt_target_slot: Option<Slot> = None;

        // Iterate through all items in this flow.
        for flow_item in &self.0 {
            match flow_item {
                &FlowItem::Token(ref token) => {
                    // Append this token to each result.
                    for mut result in &mut results {
                        result.push(token);
                    }
                },
                &FlowItem::Split(ref split_set) => {
                    // NOTE: This code is in charge of popping off the slots from the slot stack.
                    // Since we are about to start a split, set the target slot if not already set,
                    // and use the value contained.
                    if opt_target_slot.is_none() {
                        opt_target_slot = slot_stack.pop();
                    }

                    let target_slot = opt_target_slot.ok_or(SlotStackError::Empty)?;

                    let mut split_set_walks = split_set.find_walks(target_slot, &mut slot_stack)?;

                    // For each existing result walk, append each of the split set walks.
                    let mut new_results: Vec<Vec<&Token>> = vec![];
                    for result in &results {
                        for split_set_walk in &split_set_walks {
                            let mut a = result.clone();
                            let mut b = split_set_walk.clone();
                            a.append(&mut b);
                            new_results.push(a);
                        }
                    }

                    results = new_results;
                },
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::{Flow, FlowItem};

    #[macro_use] use super::super::split::{Split, SplitSet};
    use super::super::gate::{Gate, Slot};
    use token::Token;

    #[test]
    fn test_find_walks() {
        let inputs_and_expected = vec![
            ((flow![FlowItem::Token(Token)], vec![0: Slot]),
                vec![vec![&Token]]),
            ((flow![FlowItem::Token(Token), FlowItem::Token(Token)], vec![0]),
                vec![vec![&Token, &Token]]),
            (
                (
                    flow![
                        FlowItem::Token(Token),
                        FlowItem::Split(
                            splitset!(
                                Split::new(
                                    flow!(FlowItem::Token(Token)),
                                    allow!(0),
                                ),
                            ),
                        ),
                        FlowItem::Token(Token)
                    ],
                    vec![0]
                ),
                vec![vec![&Token, &Token, &Token]],
            ),
            // (
            //     (
            //         flow![
            //             FlowItem::Token(Token),
            //             FlowItem::Split(
            //                 splitset!(
            //                     Split::new(
            //                         flow!(FlowItem::Token(Token)),
            //                         allow!(0),
            //                     ),
            //                 ),
            //             ),
            //             FlowItem::Token(Token)
            //         ],
            //         vec![1]
            //     ),
            //     vec![vec![&Token, &Token]],
            // ),
        ];

        for ((flow, slot_stack), expected) in inputs_and_expected {
            let produced = flow.find_walks(&mut slot_stack.clone()).expect("Unable to find walks");
            assert_eq!(expected, produced);
        }
    }
}
