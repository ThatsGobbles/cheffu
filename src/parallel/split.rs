#![macro_use]

use std::collections::{HashMap, BTreeSet};
use std::borrow::Cow;

use failure::Error;

use super::gate::{Gate, Slot};
use super::flow::{Flow, SlotStackError};
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

    pub fn find_walks(&self, target_slot: Slot, slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
        // Check if the slot is allowed by the active gate.
        if !self.active_gate.allows_slot(target_slot) {
            // bail!(SlotStackError::Mismatch{expected: self.active_gate.clone(), produced: target_slot})
            Ok(vec![])
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

    pub fn cow_normalize_splits<'a, II>(splits: II) -> BTreeSet<Cow<'a, Split>>
    where II: IntoIterator<Item = &'a Split>
    {
        // Collect into a vector for easier mutation later on.
        let mut split_seq: Vec<Cow<'a, Split>> = splits.into_iter().map(|s| Cow::Borrowed(s)).collect();

        // Calculate the union gate, which allows all slots allowed in any of the splits.
        let union_gate = &split_seq.iter().fold(Gate::block_all(), |red, ref s| red.union(&s.active_gate));

        // If union gate is not allow-all, append an empty branch with the inverse of the union gate.
        // This provides an "escape hatch" for a case when a slot does not match any provided gate.
        if !union_gate.is_allow_all() {
            split_seq.push(Cow::Owned(Split::new(flow![], union_gate.invert())));
        }

        // Drop any splits that have a block-all gate.
        split_seq.retain(|ref s| !s.active_gate.is_block_all());

        // If any splits have identical flows, combine/union their gates.
        let mut subflow_to_split: HashMap<&Flow, Cow<Split>> = hashmap![];

        // split_seq
        //     .drain(0..)
        //     .inspect(|c| println!("{:?}", c))
        //     .map(|c_split: Cow<_>| {
        //         subflow_to_split
        //             .entry(&c_split.subflow)
        //             .and_modify(|m_split| {
        //                 *m_split = Cow::Owned(Split::new(c_split.subflow.clone(), c_split.active_gate.union(&m_split.active_gate)))
        //             })
        //             .or_insert(c_split);
        //     });

        // for split in split_seq {
        //     subflow_to_split
        //         .entry(&split.subflow)
        //         .and_modify(|m_split| { *m_split = Cow::Owned(Split::new(split.subflow.clone(), split.active_gate.union(&m_split.active_gate))) })
        //         .or_insert(Cow::Borrowed(&split));
        // }

        let mut subflow_to_gate: HashMap<&Flow, Cow<Gate>> = hashmap![];
        for split in &split_seq {
            let subflow = &split.subflow;
            let active_gate = &split.active_gate;

            subflow_to_gate
                .entry(subflow)
                .and_modify(|g| { *g = Cow::Owned(g.union(active_gate)) })
                .or_insert(Cow::Borrowed(active_gate));
        }

        // subflow_to_gate.into_iter().map(|(sf, ag)| {}).collect::<BTreeSet<Split>>()

        btreeset![]
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
    pub fn find_walks(&self, target_slot: Slot, slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
        // // self.0.iter().flat_map(|s| s.find_walks(target_slot, &mut slot_stack.clone())).collect()
        // let results: Result<Vec<Vec<Vec<&Token>>>, _> = self.0.iter().map(|s| s.find_walks(target_slot, &mut slot_stack.clone())).collect();
        // results.map(|walks| walks.iter().flat_map(|walk| walk))

        let mut results: Vec<Vec<&Token>> = vec![];
        for split in &self.0 {
            let mut split_result = split.find_walks(target_slot, &mut slot_stack.clone())?;
            results.append(&mut split_result);
        }

        Ok(results)
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
