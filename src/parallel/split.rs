#![macro_use]

use std::collections::{HashMap, BTreeSet};

use super::gate::{Gate, Slot};
use super::flow::{Flow};
use token::Token;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Split {
    subflow: Flow,
    active_gate: Gate,
}

impl Split {
    pub fn new(subflow: Flow, active_gate: Gate) -> Self {
        Split { subflow, active_gate }
    }

    pub fn find_walks(&self, target_slot: Slot, slot_stack: &mut Vec<Slot>) -> Vec<Vec<&Token>> {
        // Check if the slot is allowed by the active gate.
        if !self.active_gate.allows_slot(target_slot) {
            vec![]
        }
        else {
            // Find all walks on the contained subflow.
            self.subflow.find_walks(slot_stack)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct SplitSet(BTreeSet<Split>);

impl SplitSet {
    pub fn new<II: IntoIterator<Item = Split>>(splits: II) -> Self {
        SplitSet(SplitSet::normalize_splits(splits))
    }

    /// Processes split choices to coalesce identical split choices, and to ensure that the union of all of its
    /// contained gates allows all slots (i.e. is an allow-all gate).
    pub fn normalize_splits<II: IntoIterator<Item = Split>>(splits: II) -> BTreeSet<Split> {
        // Clone and collect into a sequence for easier mutation later on.
        let mut split_seq: Vec<Split> = splits.into_iter().collect();

        // Calculate the union gate, which allows all slots allowed in any of the split choices.
        let union_gate = &split_seq.iter().fold(Gate::block_all(), |red, ref spl| red.union(&spl.active_gate));

        // If union gate is not allow-all, append an empty branch with the inverse of the union gate.
        // This provides an "escape hatch" for a case when a slot does not match any provided gate.
        if !union_gate.is_allow_all() {
            let coverage_ac = Split{ subflow: flow![], active_gate: union_gate.invert() };
            split_seq.push(coverage_ac);
        }

        // Drop any split choices that have a block-all gate.
        split_seq.retain(|ref ac| !ac.active_gate.is_block_all());

        // NOTE: Recursing is not needed if this is always built in a bottom up style.
        // // Recurse to normalize nested splits.
        // for mut ac in &mut split_seq {
        //     for mut path_item in &mut ac.subflow.0 {
        //         match path_item {
        //             &mut FlowItem::Token(_) => {},
        //             &mut FlowItem::Split(ref mut acs) => {
        //                 *acs = Flow::normalize_splits(acs);
        //             },
        //         };
        //     }
        // }

        // If any split choices are identical, combine their gates.
        let mut subflow_to_gate: HashMap<Flow, Gate> = hashmap![];
        // LEARN: We want a move to occur here.
        for ac in split_seq {
            let subflow = ac.subflow;
            let active_gate = ac.active_gate;
            let entry = subflow_to_gate.entry(subflow).or_insert(Gate::block_all());
            *entry = entry.union(&active_gate);
        }

        subflow_to_gate.into_iter().map(|(pi, ag)| Split{ subflow: pi, active_gate: ag }).collect::<BTreeSet<Split>>()
    }

    /// Produces all walks through the contained splits that allow a given slot.
    pub fn find_walks(&self, target_slot: Slot, slot_stack: &mut Vec<Slot>) -> Vec<Vec<&Token>> {
        self.0.iter().flat_map(|s| s.find_walks(target_slot, &mut slot_stack.clone())).collect()
    }
}

impl<'a> IntoIterator for &'a SplitSet {
    type Item = &'a Split;
    type IntoIter = <&'a BTreeSet<Split> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// LEARN: This doesn't work! But for a good reason; BTreeSet doesn't support mutable iteration!
// impl<'a> IntoIterator for &'a mut SplitSet {
//     type Item = &'a mut Split;
//     type IntoIter = <&'a mut BTreeSet<Split> as IntoIterator>::IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         self.0.iter()
//     }
// }

impl IntoIterator for SplitSet {
    type Item = Split;
    type IntoIter = <BTreeSet<Split> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{Split, SplitSet};

    use super::super::flow::{Flow, FlowItem};
    use super::super::gate::Gate;
    use token::Token;

    #[test]
    fn test_normalize_splits() {
        let inputs_and_expected = vec![
            (
                splitset![
                    Split{ subflow: flow![], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                splitset![
                    Split{ subflow: flow![], active_gate: Gate::Block(btreeset![]) },
                ],
            ),
            (
                splitset![
                    Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![2, 3, 4]) },
                ],
                splitset![
                    Split{ subflow: flow![], active_gate: Gate::Block(btreeset![0, 1, 2, 3, 4]) },
                    Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2, 3, 4]) },
                ],
            ),
            (
                splitset![
                    Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![]) },
                    Split{ subflow: flow![FlowItem::Token(Token), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                splitset![
                    Split{ subflow: flow![], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                    Split{ subflow: flow![FlowItem::Token(Token), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
            ),
            (
                splitset![
                    Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                    Split{ subflow: flow![FlowItem::Token(Token), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                splitset![
                    Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                    Split{ subflow: flow![FlowItem::Token(Token), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
            ),
            (
                splitset![],
                splitset![
                    Split{ subflow: flow![], active_gate: Gate::Block(btreeset![]) },
                ],
            ),
            (
                splitset![
                    Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    Split{ subflow: flow![FlowItem::Split(splitset![
                        Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                        Split{ subflow: flow![], active_gate: Gate::Allow(btreeset![5]) },
                    ]), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                splitset![
                    Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    Split{ subflow: flow![FlowItem::Split(splitset![
                        Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                        Split{ subflow: flow![], active_gate: Gate::Allow(btreeset![5]) },
                    ]), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    Split{ subflow: flow![], active_gate: Gate::Block(btreeset![0, 1, 2, 7]) },
                ],
            ),
            // (
            //     splitset![
            //         Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
            //         Split{ subflow: flow![FlowItem::Split(splitset![
            //             Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Block(btreeset![0, 1, 2]) },
            //             Split{ subflow: flow![FlowItem::Token(Token), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![5]) },
            //         ]), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
            //     ],
            //     splitset![
            //         Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
            //         Split{ subflow: flow![FlowItem::Split(splitset![
            //             Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Block(btreeset![0, 1, 2]) },
            //             Split{ subflow: flow![FlowItem::Token(Token), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![5]) },
            //             Split{ subflow: flow![], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
            //         ]), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
            //         Split{ subflow: flow![], active_gate: Gate::Block(btreeset![0, 1, 2, 7]) },
            //     ],
            // ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = SplitSet(SplitSet::normalize_splits(input));
            assert_eq!(expected, produced);
        }
    }
}
