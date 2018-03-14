use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap};

use super::gate::{Gate, Slot};
use super::cow_flow::{CowFlow, SlotStackError};

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

//     pub fn find_walks(&self, target_slot: Slot, slot_stack: &mut Vec<Slot>) -> Result<Vec<Vec<&Token>>, Error> {
//         // Check if the slot is allowed by the active gate.
//         if !self.gate.allows_slot(target_slot) {
//             // bail!(SlotStackError::Mismatch{expected: self.gate.clone(), produced: target_slot})
//             Ok(vec![])
//         }
//         else {
//             // Find all walks on the contained flow.
//             self.flow.find_walks(slot_stack)
//         }
//     }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct CowSplitSet<'a>(BTreeSet<CowSplit<'a>>);

impl<'a> CowSplitSet<'a> {
    pub fn new<II>(splits: II) -> Self
    where II: IntoIterator<Item = CowSplit<'a>>
    {
        CowSplitSet(btreeset![])
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

        // NOTE: Recursing is not needed if this is always built in a bottom up style.
        // Recurse to normalize nested splits.
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
}

#[cfg(test)]
mod tests {
    use super::{CowSplit, CowSplitSet};

    use super::super::cow_flow::{CowFlow, CowFlowItem};
    use super::super::gate::Gate;
    use token::Token;

    #[test]
    fn test_normalize_splits() {
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
                    CowSplit::new(cflow![CowFlowItem::Token(Token)], allow![0, 1, 2]),
                    CowSplit::new(cflow![CowFlowItem::Token(Token)], allow![2, 3, 4]),
                ],
                btreeset![
                    CowSplit::new(cflow![], block![0, 1, 2, 3, 4]),
                    CowSplit::new(cflow![CowFlowItem::Token(Token)], allow![0, 1, 2, 3, 4]),
                ],
            ),
            (
                vec![
                    CowSplit::new(cflow![CowFlowItem::Token(Token)], allow![]),
                    CowSplit::new(cflow![CowFlowItem::Token(Token), CowFlowItem::Token(Token)], allow![0, 1, 2]),
                ],
                btreeset![
                    CowSplit::new(cflow![], block![0, 1, 2]),
                    CowSplit::new(cflow![CowFlowItem::Token(Token), CowFlowItem::Token(Token)], allow![0, 1, 2]),
                ],
            ),
            (
                vec![
                    CowSplit::new(cflow![CowFlowItem::Token(Token)], block![]),
                    CowSplit::new(cflow![CowFlowItem::Token(Token), CowFlowItem::Token(Token)], allow![0, 1, 2]),
                ],
                btreeset![
                    CowSplit::new(cflow![CowFlowItem::Token(Token)], block![]),
                    CowSplit::new(cflow![CowFlowItem::Token(Token), CowFlowItem::Token(Token)], allow![0, 1, 2]),
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
                    CowSplit::new(cflow![CowFlowItem::Token(Token)], allow![7]),
                    CowSplit::new(cflow![CowFlowItem::Split(csplitset![
                        CowSplit::new(cflow![CowFlowItem::Token(Token)], block![]),
                        CowSplit::new(cflow![], allow![5]),
                    ]), CowFlowItem::Token(Token)], allow![0, 1, 2]),
                ],
                btreeset![
                    CowSplit::new(cflow![CowFlowItem::Token(Token)], allow![7]),
                    CowSplit::new(cflow![CowFlowItem::Split(csplitset![
                        CowSplit::new(cflow![CowFlowItem::Token(Token)], block![]),
                        CowSplit::new(cflow![], allow![5]),
                    ]), CowFlowItem::Token(Token)], allow![0, 1, 2]),
                    CowSplit::new(cflow![], block![0, 1, 2, 7]),
                ],
            ),
            (
                vec![
                    CowSplit::new(cflow![CowFlowItem::Token(Token)], allow![7]),
                    CowSplit::new(cflow![CowFlowItem::Split(csplitset![
                        CowSplit::new(cflow![CowFlowItem::Token(Token)], block![0, 1, 2]),
                        CowSplit::new(cflow![CowFlowItem::Token(Token), CowFlowItem::Token(Token)], allow![5]),
                    ]), CowFlowItem::Token(Token)], allow![0, 1, 2]),
                ],
                btreeset![
                    CowSplit::new(cflow![CowFlowItem::Token(Token)], allow![7]),
                    CowSplit::new(cflow![CowFlowItem::Split(csplitset![
                        CowSplit::new(cflow![CowFlowItem::Token(Token)], block![0, 1, 2]),
                        CowSplit::new(cflow![CowFlowItem::Token(Token), CowFlowItem::Token(Token)], allow![5]),
                        CowSplit::new(cflow![], allow![0, 1, 2]),
                    ]), CowFlowItem::Token(Token)], allow![0, 1, 2]),
                    CowSplit::new(cflow![], block![0, 1, 2, 7]),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = CowSplitSet::normalize_splits(input);
            assert_eq!(expected, produced);
        }
    }
}
