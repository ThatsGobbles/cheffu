#![macro_use]

use std::collections::VecDeque;
use std::iter::{IntoIterator, FromIterator};
use std::borrow::Cow;

use failure::Error;

use super::gate::{Slot, Gate};
use super::scope::Scope;
use super::cow_split::{CowSplit, CowSplitSet};
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
pub enum CowFlowItem<'a> {
    Token(Token),
    Split(CowSplitSet<'a>),
}

/// Contains the tokens and splits that comprise all the variants of a single recipe.
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct CowFlow<'a>(Vec<CowFlowItem<'a>>);

impl<'a> IntoIterator for &'a CowFlow<'a> {
    type Item = &'a CowFlowItem<'a>;
    type IntoIter = <&'a Vec<CowFlowItem<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut CowFlow<'a> {
    type Item = &'a mut CowFlowItem<'a>;
    type IntoIter = <&'a mut Vec<CowFlowItem<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<'a> IntoIterator for CowFlow<'a> {
    type Item = CowFlowItem<'a>;
    type IntoIter = <Vec<CowFlowItem<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> From<CowFlow<'a>> for Cow<'a, CowFlow<'a>> {
    fn from(flow: CowFlow<'a>) -> Self {
        Cow::Owned(flow)
    }
}

impl<'a> From<&'a CowFlow<'a>> for Cow<'a, CowFlow<'a>> {
    fn from(flow: &'a CowFlow<'a>) -> Self {
        Cow::Borrowed(flow)
    }
}

impl<'a> CowFlow<'a> {
    pub fn new(flow: Vec<CowFlowItem<'a>>) -> Self {
        CowFlow(flow)
    }

    // pub fn find_walks(&self, mut slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
    //     let mut results: Vec<Vec<&Token>> = vec![vec![]];
    //     let mut opt_target_slot: Option<Slot> = None;

    //     // Iterate through all items in this flow.
    //     for flow_item in &self.0 {
    //         match flow_item {
    //             &CowFlowItem::Token(ref token) => {
    //                 // Append this token to each result.
    //                 for mut result in &mut results {
    //                     result.push(token);
    //                 }
    //             },
    //             &CowFlowItem::Split(ref split_set) => {
    //                 // NOTE: This code is in charge of popping off the slots from the slot stack.
    //                 // Since we are about to start a split, set the target slot if not already set,
    //                 // and use the value contained.
    //                 if opt_target_slot.is_none() {
    //                     opt_target_slot = slot_stack.pop();
    //                 }

    //                 let target_slot = opt_target_slot.ok_or(SlotStackError::Empty)?;

    //                 let mut split_set_walks = split_set.find_walks(target_slot, &mut slot_stack)?;

    //                 // For each existing result walk, append each of the split set walks.
    //                 let mut new_results: Vec<Vec<&Token>> = vec![];
    //                 for result in &results {
    //                     for split_set_walk in &split_set_walks {
    //                         let mut a = result.clone();
    //                         let mut b = split_set_walk.clone();
    //                         a.append(&mut b);
    //                         new_results.push(a);
    //                     }
    //                 }

    //                 results = new_results;
    //             },
    //         }
    //     }

    //     Ok(results)
    // }
}

#[cfg(test)]
mod tests {
    use super::{CowFlow, CowFlowItem};

    #[macro_use] use super::super::cow_split::{CowSplit, CowSplitSet};
    use super::super::gate::{Gate, Slot};
    use token::Token;

    // #[test]
    // fn test_find_walks() {
    //     let inputs_and_expected = vec![
    //         ((cflow![CowFlowItem::Token(Token)], vec![0: Slot]),
    //             vec![vec![&Token]]),
    //         ((cflow![CowFlowItem::Token(Token), CowFlowItem::Token(Token)], vec![0]),
    //             vec![vec![&Token, &Token]]),
    //         (
    //             (
    //                 cflow![
    //                     CowFlowItem::Token(Token),
    //                     CowFlowItem::Split(
    //                         csplitset!(
    //                             CowSplit::new(
    //                                 cflow!(CowFlowItem::Token(Token)),
    //                                 allow!(0),
    //                             ),
    //                         ),
    //                     ),
    //                     CowFlowItem::Token(Token)
    //                 ],
    //                 vec![0]
    //             ),
    //             vec![vec![&Token, &Token, &Token]],
    //         ),
    //         // (
    //         //     (
    //         //         cflow![
    //         //             CowFlowItem::Token(Token),
    //         //             CowFlowItem::Split(
    //         //                 csplitset!(
    //         //                     Split::new(
    //         //                         cflow!(CowFlowItem::Token(Token)),
    //         //                         allow!(0),
    //         //                     ),
    //         //                 ),
    //         //             ),
    //         //             CowFlowItem::Token(Token)
    //         //         ],
    //         //         vec![1]
    //         //     ),
    //         //     vec![vec![&Token, &Token]],
    //         // ),
    //     ];

    //     for ((flow, slot_stack), expected) in inputs_and_expected {
    //         let produced = flow.find_walks(&mut slot_stack.clone()).expect("Unable to find walks");
    //         assert_eq!(expected, produced);
    //     }
    // }
}
