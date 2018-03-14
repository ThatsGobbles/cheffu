#![macro_use]

use std::collections::{BTreeSet, HashMap};
use std::iter::{IntoIterator, FromIterator};
use std::borrow::Cow;

use failure::Error;

use super::gate::{Slot, Gate};
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

    pub fn find_walks(&self, mut slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
        let mut results: Vec<Vec<&Token>> = vec![vec![]];
        let mut opt_target_slot: Option<Slot> = None;

        // Iterate through all items in this flow.
        for flow_item in &self.0 {
            match flow_item {
                &CowFlowItem::Token(ref token) => {
                    // Append this token to each result.
                    for mut result in &mut results {
                        result.push(token);
                    }
                },
                &CowFlowItem::Split(ref split_set) => {
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

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct CowSplit<'a> {
    flow: Cow<'a, CowFlow<'a>>,
    gate: Cow<'a, Gate>,
}

impl<'a> CowSplit<'a> {
    pub fn new<F, G>(flow: F, gate: G) -> Self
    where F: Into<Cow<'a, CowFlow<'a>>>,
          G: Into<Cow<'a, Gate>>,
    {
        CowSplit { flow: flow.into(), gate: gate.into() }
    }

    pub fn find_walks(&self, target_slot: Slot, slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
        // Check if the slot is allowed by the active gate.
        if !self.gate.allows_slot(target_slot) {
            // NOTE: This is a single-element result.
            // TODO: This should never happen with proper normalization, might be better to error.
            Ok(vec![vec![]])
        }
        else {
            // Find all walks on the contained flow.
            self.flow.find_walks(slot_stack)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct CowSplitSet<'a>(BTreeSet<CowSplit<'a>>);

impl<'a> CowSplitSet<'a> {
    pub fn new<II>(splits: II) -> Self
    where II: IntoIterator<Item = CowSplit<'a>>
    {
        CowSplitSet(splits.into_iter().collect())
    }

    pub fn normalize_splits<'b, II>(splits: II) -> BTreeSet<CowSplit<'b>>
    where II: IntoIterator<Item = CowSplit<'b>>
    {
        // Collect into a vector for easier mutation later on.
        let mut split_seq: Vec<_> = splits.into_iter().collect();

        // Calculate the union gate, which allows all slots allowed in any of the splits.
        let union_gate = &split_seq.iter().fold(Gate::block_all(), |red, ref s| red.union(&s.gate));

        // If union gate is not allow-all, append an empty branch with the inverse of the union gate.
        // This provides an "escape hatch" for a case when a slot does not match any provided gate.
        if !union_gate.is_allow_all() {
            split_seq.push(CowSplit::new(cflow![], union_gate.invert()));
        }

        // Drop any splits that have a block-all gate.
        split_seq.retain(|ref s| !s.gate.is_block_all());

        // NOTE: Recursing is not needed if this is always built in a bottom up style, but nice to have.
        // TODO: Fix to work with `Cow`.
        // // Recurse to normalize nested splits.
        // for mut ac in &mut split_seq {
        //     for mut path_item in &mut ac.flow.to_mut() {
        //         match path_item {
        //             &mut FlowItem::Token(_) => {},
        //             &mut FlowItem::Split(ref mut splits) => {
        //                 *splits = Flow::normalize_splits(splits);
        //             },
        //         };
        //     }
        // }

        // If any splits have identical flows, combine/union their gates.
        let mut flow_to_gate: HashMap<Cow<CowFlow>, Cow<Gate>> = hashmap![];

        for split in split_seq {
            let flow = split.flow;
            let gate = split.gate;

            flow_to_gate
                .entry(flow)
                .and_modify(|present| { *present = Cow::Owned(gate.union(&present)) })
                .or_insert(gate);
        }

        flow_to_gate.into_iter().map(|(f, g)| CowSplit::new(f, g)).collect::<BTreeSet<CowSplit>>()
    }

    /// Produces all walks through the contained splits that allow a given slot.
    pub fn find_walks(&self, target_slot: Slot, slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
        let mut results: Vec<Vec<&Token>> = vec![];
        for split in &self.0 {
            let mut split_result = split.find_walks(target_slot, &mut slot_stack.clone())?;
            results.append(&mut split_result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::{CowFlow, CowFlowItem, CowSplit, CowSplitSet};

    use super::super::gate::{Gate, Slot};
    use token::Token;

    #[test]
    fn test_find_walks() {
        let token_a = Token::Ingredient("apple".to_string());
        let token_b = Token::Ingredient("banana".to_string());
        let token_c = Token::Ingredient("cherry".to_string());
        let token_d = Token::Ingredient("date".to_string());

        let inputs_and_expected = vec![
            ((cflow![CowFlowItem::Token(token_a.clone())], vec![0: Slot]),
                vec![vec![&token_a]]),
            ((cflow![CowFlowItem::Token(token_a.clone()), CowFlowItem::Token(token_b.clone())], vec![0]),
                vec![vec![&token_a, &token_b]]),
            (
                (
                    cflow![
                        CowFlowItem::Token(token_a.clone()),
                        CowFlowItem::Split(
                            csplitset!(
                                CowSplit::new(
                                    cflow!(CowFlowItem::Token(token_b.clone())),
                                    allow!(0),
                                ),
                            ),
                        ),
                        CowFlowItem::Token(token_c.clone())
                    ],
                    vec![0]
                ),
                vec![vec![&token_a, &token_b, &token_c]],
            ),
            (
                (
                    cflow![
                        CowFlowItem::Token(token_a.clone()),
                        CowFlowItem::Split(
                            csplitset!(
                                CowSplit::new(
                                    cflow!(CowFlowItem::Token(token_b.clone())),
                                    allow!(0),
                                ),
                            ),
                        ),
                        CowFlowItem::Token(token_c.clone())
                    ],
                    vec![1]
                ),
                vec![vec![&token_a, &token_c]],
            ),
            (
                (
                    cflow![
                        CowFlowItem::Token(token_a.clone()),
                        CowFlowItem::Split(
                            csplitset!(
                                CowSplit::new(
                                    cflow!(CowFlowItem::Token(token_b.clone())),
                                    allow!(0),
                                ),
                            ),
                        ),
                        CowFlowItem::Token(token_c.clone()),
                        CowFlowItem::Split(
                            csplitset!(
                                CowSplit::new(
                                    cflow!(CowFlowItem::Token(token_d.clone()), CowFlowItem::Token(token_a.clone())),
                                    allow!(1),
                                ),
                            ),
                        ),
                    ],
                    vec![1]
                ),
                vec![vec![&token_a, &token_c, &token_d, &token_a]],
            ),
            (
                (
                    cflow![
                        CowFlowItem::Token(token_a.clone()),
                        CowFlowItem::Split(
                            csplitset!(
                                CowSplit::new(
                                    cflow!(CowFlowItem::Token(token_b.clone())),
                                    allow!(0),
                                ),
                            ),
                        ),
                        CowFlowItem::Token(token_c.clone()),
                        CowFlowItem::Split(
                            csplitset!(
                                CowSplit::new(
                                    cflow!(CowFlowItem::Token(token_d.clone()), CowFlowItem::Token(token_a.clone())),
                                    allow!(1),
                                ),
                            ),
                        ),
                    ],
                    vec![0]
                ),
                vec![vec![&token_a, &token_b, &token_c]],
            ),
            (
                (
                    cflow![
                        CowFlowItem::Token(token_a.clone()),
                        CowFlowItem::Split(
                            csplitset!(
                                CowSplit::new(
                                    cflow!(CowFlowItem::Token(token_b.clone())),
                                    allow!(0),
                                ),
                            ),
                        ),
                        CowFlowItem::Token(token_c.clone()),
                        CowFlowItem::Split(
                            csplitset!(
                                CowSplit::new(
                                    cflow!(CowFlowItem::Token(token_d.clone()), CowFlowItem::Token(token_a.clone())),
                                    allow!(1),
                                ),
                            ),
                        ),
                    ],
                    vec![2]
                ),
                vec![vec![&token_a, &token_c]],
            ),
        ];

        for ((flow, slot_stack), expected) in inputs_and_expected {
            let produced = flow.find_walks(&mut slot_stack.clone()).expect("Unable to find walks");
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_normalize_splits() {
        let token_a = Token::Ingredient("apple".to_string());

        let inputs_and_expected = vec![
            (
                vec![
                    CowSplit::new(cflow![], allow![0, 1, 2]),
                ],
                btreeset![
                    CowSplit::new(cflow![], block![]),
                ],
            ),
            (
                vec![
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], allow![2, 3, 4]),
                ],
                btreeset![
                    CowSplit::new(cflow![], block![0, 1, 2, 3, 4]),
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], allow![0, 1, 2, 3, 4]),
                ],
            ),
            (
                vec![
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], allow![]),
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone()), CowFlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                ],
                btreeset![
                    CowSplit::new(cflow![], block![0, 1, 2]),
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone()), CowFlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                ],
            ),
            (
                vec![
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], block![]),
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone()), CowFlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                ],
                btreeset![
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], block![]),
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone()), CowFlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                ],
            ),
            (
                vec![],
                btreeset![
                    CowSplit::new(cflow![], block![]),
                ],
            ),
            (
                vec![
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], allow![7]),
                    CowSplit::new(cflow![CowFlowItem::Split(csplitset![
                        CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], block![]),
                        CowSplit::new(cflow![], allow![5]),
                    ]), CowFlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                ],
                btreeset![
                    CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], allow![7]),
                    CowSplit::new(cflow![CowFlowItem::Split(csplitset![
                        CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], block![]),
                        CowSplit::new(cflow![], allow![5]),
                    ]), CowFlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                    CowSplit::new(cflow![], block![0, 1, 2, 7]),
                ],
            ),
            // NOTE: This case tests recursive normalization.
            // (
            //     vec![
            //         CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], allow![7]),
            //         CowSplit::new(cflow![CowFlowItem::Split(csplitset![
            //             CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], block![0, 1, 2]),
            //             CowSplit::new(cflow![CowFlowItem::Token(token_a.clone()), CowFlowItem::Token(token_a.clone())], allow![5]),
            //         ]), CowFlowItem::Token(token_a.clone())], allow![0, 1, 2]),
            //     ],
            //     btreeset![
            //         CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], allow![7]),
            //         CowSplit::new(cflow![CowFlowItem::Split(csplitset![
            //             CowSplit::new(cflow![CowFlowItem::Token(token_a.clone())], block![0, 1, 2]),
            //             CowSplit::new(cflow![CowFlowItem::Token(token_a.clone()), CowFlowItem::Token(token_a.clone())], allow![5]),
            //             CowSplit::new(cflow![], allow![0, 1, 2]),
            //         ]), CowFlowItem::Token(token_a.clone())], allow![0, 1, 2]),
            //         CowSplit::new(cflow![], block![0, 1, 2, 7]),
            //     ],
            // ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = CowSplitSet::normalize_splits(input);
            assert_eq!(expected, produced);
        }
    }
}
