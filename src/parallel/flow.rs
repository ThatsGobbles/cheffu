#![macro_use]

use std::collections::VecDeque;
use std::iter::{IntoIterator, FromIterator};

use super::gate::{Slot, Gate};
use super::scope::Scope;
use super::split::{Split, SplitSet};
use token::Token;

pub struct FlowSlotChoices {
    choices: Vec<VecDeque<Slot>>
}

impl FlowSlotChoices {
    pub fn pop(&mut self, scope_index: usize) -> Option<Slot> {
        if scope_index >= self.choices.len() {
            None
        } else {
            self.choices[scope_index].pop_front()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.choices.iter().all(|slot_queue| slot_queue.is_empty())
    }
}

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

    pub fn find_walks(&self, mut slot_stack: &mut Vec<Slot>) -> Vec<Vec<&Token>> {
        let mut results: Vec<Vec<&Token>> = vec![];
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

                    // TODO: Make this an error instead of a panic.
                    let target_slot = opt_target_slot.unwrap();

                    let mut split_set_walks = split_set.find_walks(target_slot, &mut slot_stack);
                },
            }
        }

        // let helper = |curr_flow_item: &FlowItem, curr_walk_path: &Vec<&Token>| {
        //     // It is possible to have more than one new path output from this helper.
        //     let next_walk_paths: Vec<Vec<&Token>> = match curr_flow_item {
        //         &FlowItem::Token(ref token) => {
        //             let mut result = curr_walk_path.clone();
        //             result.push(token);
        //             vec![result]
        //         },
        //         &FlowItem::Split(ref split_set) => {
        //             let mut results: Vec<Vec<&Token>> = vec![];

        //             let mut split_walks = split_set.find_walks(target_slot);
        //             for mut split_walk in &mut split_walks {
        //                 let mut result = curr_walk_path.clone();
        //                 result.append(&mut split_walk);
        //                 results.push(result);
        //             }

        //             results
        //         },
        //     };
        // };

        vec![]
    }
}
