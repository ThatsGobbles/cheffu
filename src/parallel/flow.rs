#![macro_use]

use std::collections::{BTreeSet, HashMap};
use std::iter::IntoIterator;
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

/** FlowItem **/

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum FlowItem<'a> {
    Token(Token),
    Split(SplitSet<'a>),
}

/** Flow **/

/// Contains the tokens and splits that comprise all the variants of a single recipe.
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Flow<'a>(Vec<FlowItem<'a>>);

impl<'a> IntoIterator for &'a Flow<'a> {
    type Item = &'a FlowItem<'a>;
    type IntoIter = <&'a Vec<FlowItem<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Flow<'a> {
    type Item = &'a mut FlowItem<'a>;
    type IntoIter = <&'a mut Vec<FlowItem<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<'a> IntoIterator for Flow<'a> {
    type Item = FlowItem<'a>;
    type IntoIter = <Vec<FlowItem<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> From<Flow<'a>> for Cow<'a, Flow<'a>> {
    fn from(flow: Flow<'a>) -> Self {
        Cow::Owned(flow)
    }
}

impl<'a> From<&'a Flow<'a>> for Cow<'a, Flow<'a>> {
    fn from(flow: &'a Flow<'a>) -> Self {
        Cow::Borrowed(flow)
    }
}

impl<'a> Flow<'a> {
    pub fn new(flow: Vec<FlowItem<'a>>) -> Self {
        Flow(flow)
    }

    fn find_walks(&self, mut slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
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

    pub fn walks(&self, slot_stack: Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
        let mut slot_stack = slot_stack.clone();

        let results = self.find_walks(&mut slot_stack)?;

        ensure!(slot_stack.is_empty(), SlotStackError::Leftover{leftover: slot_stack});

        Ok(results)
    }
}

/** Split **/

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Split<'a> {
    flow: Cow<'a, Flow<'a>>,
    gate: Cow<'a, Gate>,
}

impl<'a> Split<'a> {
    pub fn new<F, G>(flow: F, gate: G) -> Self
    where F: Into<Cow<'a, Flow<'a>>>,
          G: Into<Cow<'a, Gate>>,
    {
        Split { flow: flow.into(), gate: gate.into() }
    }

    fn find_walks(&self, target_slot: Slot, slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
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

/** SplitSet **/

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct SplitSet<'a>(BTreeSet<Split<'a>>);

impl<'a> SplitSet<'a> {
    pub fn new<II>(splits: II) -> Self
    where II: IntoIterator<Item = Split<'a>>
    {
        SplitSet(splits.into_iter().collect())
    }

    // pub fn normalize_splits<'b, II>(splits: II) -> BTreeSet<Split<'b>>
    // where II: IntoIterator<Item = Split<'b>>
    // {
    //     // Collect into a vector for easier mutation later on.
    //     let mut split_seq: Vec<_> = splits.into_iter().collect();

    //     // Calculate the union gate, which allows all slots allowed in any of the splits.
    //     let union_gate = &split_seq.iter().fold(Gate::block_all(), |red, ref s| red.union(&s.gate));

    //     // If union gate is not allow-all, append an empty branch with the inverse of the union gate.
    //     // This provides an "escape hatch" for a case when a slot does not match any provided gate.
    //     if !union_gate.is_allow_all() {
    //         split_seq.push(Split::new(flow![], union_gate.invert()));
    //     }

    //     // Drop any splits that have a block-all gate.
    //     split_seq.retain(|ref s| !s.gate.is_block_all());

    //     // NOTE: Recursing is not needed if this is always built in a bottom up style, but nice to have.
    //     // TODO: Fix to work with `Cow`.
    //     // // Recurse to normalize nested splits.
    //     // for mut ac in &mut split_seq {
    //     //     for mut path_item in &mut ac.flow.to_mut() {
    //     //         match path_item {
    //     //             &mut FlowItem::Token(_) => {},
    //     //             &mut FlowItem::Split(ref mut splits) => {
    //     //                 *splits = Flow::normalize_splits(splits);
    //     //             },
    //     //         };
    //     //     }
    //     // }

    //     // If any splits have identical flows, combine/union their gates.
    //     let mut flow_to_gate: HashMap<Cow<Flow>, Cow<Gate>> = hashmap![];

    //     for split in split_seq {
    //         let flow = split.flow;
    //         let gate = split.gate;

    //         flow_to_gate
    //             .entry(flow)
    //             .and_modify(|present| { *present = Cow::Owned(gate.union(&present)) })
    //             .or_insert(gate);
    //     }

    //     flow_to_gate.into_iter().map(|(f, g)| Split::new(f, g)).collect::<BTreeSet<Split>>()
    // }

    pub fn normalize_splits<'b, II>(splits: II) -> BTreeSet<Split<'b>>
    where II: IntoIterator<Item = Split<'b>>
    {
        let mut flow_to_gate: HashMap<Cow<Flow>, Cow<Gate>> = hashmap![];

        // Iterate over all splits that do not have a block-all gate.
        for split in splits.into_iter().filter(|s| !s.gate.is_block_all()) {
            // Break apart split into flow and gate.
            // LEARN: These cause moves, and the split is no longer usable.
            let flow = split.flow;
            let gate = split.gate;

            // TODO: If doing recursion, logic should live here.
            // Would need to have a method on Flow, which returns a new Flow with normalized Split enums.

            // Store in mapping.
            flow_to_gate
                .entry(flow)
                .and_modify(|present| { *present = Cow::Owned(gate.union(&present)) })
                .or_insert(gate);
        }

        // Calculate the union gate.
        let union_gate = flow_to_gate.values().fold(Gate::block_all(), |acc_g, ref g| acc_g.union(&g));

        // Store/modify empty flow in mapping if the union gate is not allow-all.
        if !union_gate.is_allow_all() {
            let inv_union_gate = union_gate.invert();
            flow_to_gate
                .entry(Cow::Owned(flow![]))
                .and_modify(|present| { *present = Cow::Owned(inv_union_gate.union(&present)) })
                .or_insert(Cow::Owned(inv_union_gate));
        }

        flow_to_gate.into_iter().map(|(f, g)| Split::new(f, g)).collect::<BTreeSet<Split>>()
    }

    /// Produces all walks through the contained splits that allow a given slot.
    fn find_walks(&self, target_slot: Slot, slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
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
    use super::{Flow, FlowItem, Split, SplitSet};

    use super::super::gate::{Gate, Slot};
    use token::Token;

    #[test]
    fn test_find_walks() {
        let token_a = Token::Ingredient("apple".to_string());
        let token_b = Token::Ingredient("banana".to_string());
        let token_c = Token::Ingredient("cherry".to_string());
        let token_d = Token::Ingredient("date".to_string());

        let inputs_and_expected = vec![
            ((flow![FlowItem::Token(token_a.clone())], vec![0: Slot]),
                vec![vec![&token_a]]),
            ((flow![FlowItem::Token(token_a.clone()), FlowItem::Token(token_b.clone())], vec![0]),
                vec![vec![&token_a, &token_b]]),
            (
                (
                    flow![
                        FlowItem::Token(token_a.clone()),
                        FlowItem::Split(
                            splitset!(
                                Split::new(
                                    flow!(FlowItem::Token(token_b.clone())),
                                    allow!(0),
                                ),
                            ),
                        ),
                        FlowItem::Token(token_c.clone())
                    ],
                    vec![0]
                ),
                vec![vec![&token_a, &token_b, &token_c]],
            ),
            (
                (
                    flow![
                        FlowItem::Token(token_a.clone()),
                        FlowItem::Split(
                            splitset!(
                                Split::new(
                                    flow!(FlowItem::Token(token_b.clone())),
                                    allow!(0),
                                ),
                            ),
                        ),
                        FlowItem::Token(token_c.clone())
                    ],
                    vec![1]
                ),
                vec![vec![&token_a, &token_c]],
            ),
            (
                (
                    flow![
                        FlowItem::Token(token_a.clone()),
                        FlowItem::Split(
                            splitset!(
                                Split::new(
                                    flow!(FlowItem::Token(token_b.clone())),
                                    allow!(0),
                                ),
                            ),
                        ),
                        FlowItem::Token(token_c.clone()),
                        FlowItem::Split(
                            splitset!(
                                Split::new(
                                    flow!(FlowItem::Token(token_d.clone()), FlowItem::Token(token_a.clone())),
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
                    flow![
                        FlowItem::Token(token_a.clone()),
                        FlowItem::Split(
                            splitset!(
                                Split::new(
                                    flow!(FlowItem::Token(token_b.clone())),
                                    allow!(0),
                                ),
                            ),
                        ),
                        FlowItem::Token(token_c.clone()),
                        FlowItem::Split(
                            splitset!(
                                Split::new(
                                    flow!(FlowItem::Token(token_d.clone()), FlowItem::Token(token_a.clone())),
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
                    flow![
                        FlowItem::Token(token_a.clone()),
                        FlowItem::Split(
                            splitset!(
                                Split::new(
                                    flow!(FlowItem::Token(token_b.clone())),
                                    allow!(0),
                                ),
                            ),
                        ),
                        FlowItem::Token(token_c.clone()),
                        FlowItem::Split(
                            splitset!(
                                Split::new(
                                    flow!(FlowItem::Token(token_d.clone()), FlowItem::Token(token_a.clone())),
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
                    Split::new(flow![], allow![0, 1, 2]),
                ],
                btreeset![
                    Split::new(flow![], block![]),
                ],
            ),
            (
                vec![
                    Split::new(flow![FlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                    Split::new(flow![FlowItem::Token(token_a.clone())], allow![2, 3, 4]),
                ],
                btreeset![
                    Split::new(flow![], block![0, 1, 2, 3, 4]),
                    Split::new(flow![FlowItem::Token(token_a.clone())], allow![0, 1, 2, 3, 4]),
                ],
            ),
            (
                vec![
                    Split::new(flow![FlowItem::Token(token_a.clone())], allow![]),
                    Split::new(flow![FlowItem::Token(token_a.clone()), FlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                ],
                btreeset![
                    Split::new(flow![], block![0, 1, 2]),
                    Split::new(flow![FlowItem::Token(token_a.clone()), FlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                ],
            ),
            (
                vec![
                    Split::new(flow![FlowItem::Token(token_a.clone())], block![]),
                    Split::new(flow![FlowItem::Token(token_a.clone()), FlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                ],
                btreeset![
                    Split::new(flow![FlowItem::Token(token_a.clone())], block![]),
                    Split::new(flow![FlowItem::Token(token_a.clone()), FlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                ],
            ),
            (
                vec![],
                btreeset![
                    Split::new(flow![], block![]),
                ],
            ),
            (
                vec![
                    Split::new(flow![FlowItem::Token(token_a.clone())], allow![7]),
                    Split::new(flow![FlowItem::Split(splitset![
                        Split::new(flow![FlowItem::Token(token_a.clone())], block![]),
                        Split::new(flow![], allow![5]),
                    ]), FlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                ],
                btreeset![
                    Split::new(flow![FlowItem::Token(token_a.clone())], allow![7]),
                    Split::new(flow![FlowItem::Split(splitset![
                        Split::new(flow![FlowItem::Token(token_a.clone())], block![]),
                        Split::new(flow![], allow![5]),
                    ]), FlowItem::Token(token_a.clone())], allow![0, 1, 2]),
                    Split::new(flow![], block![0, 1, 2, 7]),
                ],
            ),
            // NOTE: This case tests recursive normalization.
            // (
            //     vec![
            //         Split::new(flow![FlowItem::Token(token_a.clone())], allow![7]),
            //         Split::new(flow![FlowItem::Split(splitset![
            //             Split::new(flow![FlowItem::Token(token_a.clone())], block![0, 1, 2]),
            //             Split::new(flow![FlowItem::Token(token_a.clone()), FlowItem::Token(token_a.clone())], allow![5]),
            //         ]), FlowItem::Token(token_a.clone())], allow![0, 1, 2]),
            //     ],
            //     btreeset![
            //         Split::new(flow![FlowItem::Token(token_a.clone())], allow![7]),
            //         Split::new(flow![FlowItem::Split(splitset![
            //             Split::new(flow![FlowItem::Token(token_a.clone())], block![0, 1, 2]),
            //             Split::new(flow![FlowItem::Token(token_a.clone()), FlowItem::Token(token_a.clone())], allow![5]),
            //             Split::new(flow![], allow![0, 1, 2]),
            //         ]), FlowItem::Token(token_a.clone())], allow![0, 1, 2]),
            //         Split::new(flow![], block![0, 1, 2, 7]),
            //     ],
            // ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = SplitSet::normalize_splits(input);
            assert_eq!(expected, produced);
        }
    }
}
