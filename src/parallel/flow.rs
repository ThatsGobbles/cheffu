use std::collections::{HashMap, BTreeSet};
use std::collections::btree_set::Iter as BTreeSetIter;
use std::iter::{IntoIterator, FromIterator};

use super::gate::{Slot, Gate};
use super::scope::Scope;
use token::Token;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum FlowItem {
    Token(Token),
    Split(SplitSet),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Split {
    subflow: Flow,
    active_gate: Gate,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct SplitSet(BTreeSet<Split>);

macro_rules! splitset {
    ( $($split:expr),* $(,)? ) => (SplitSet(btreeset!($($split),*)));
}

impl<'a> IntoIterator for &'a SplitSet {
    type Item = &'a Split;
    type IntoIter = <&'a BTreeSet<Split> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for SplitSet {
    type Item = Split;
    type IntoIter = <BTreeSet<Split> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Contains the tokens and splits that comprise all the variants of a single recipe.
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Flow(Vec<FlowItem>);

macro_rules! flow {
    ( $($split:expr),* $(,)? ) => (Flow(vec!($($split),*)));
}

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
    /// Processes split choices to coalesce identical split choices, and to ensure that the union of all of its
    /// contained gates allows all slots (i.e. is an allow-all gate).
    pub fn normalize_splits(splits: &SplitSet) -> SplitSet {
        // Calculate the union gate, which allows all slots allowed in any of the split choices.
        let union_gate = splits.into_iter().fold(Gate::block_all(), |red, ref ac| red.union(&ac.active_gate));

        // Clone and collect into a sequence for easier mutation later on.
        let mut split_seq: Vec<Split> = splits.into_iter().cloned().collect();

        // If union gate is not allow-all, append an empty branch with the inverse of the union gate.
        // This provides an "escape hatch" for a case when a slot does not match any provided gate.
        if !union_gate.is_allow_all() {
            let coverage_ac = Split{ subflow: flow![], active_gate: union_gate.invert() };
            split_seq.push(coverage_ac);
        }

        // Drop any split choices that have a block-all gate.
        split_seq.retain(|ref ac| !ac.active_gate.is_block_all());

        // Recurse to normalize nested split choices.
        for mut ac in &mut split_seq {
            for mut path_item in &mut ac.subflow.0 {
                match path_item {
                    &mut FlowItem::Token(_) => {},
                    &mut FlowItem::Split(ref mut acs) => {
                        *acs = Flow::normalize_splits(acs);
                    },
                };
            }
        }

        // If any split choices are identical, combine their gates.
        let mut subflow_to_gate: HashMap<&Flow, Gate> = hashmap![];
        for ac in &split_seq {
            let subflow = &ac.subflow;
            let active_gate = &ac.active_gate;
            let entry = subflow_to_gate.entry(subflow).or_insert(Gate::block_all());
            *entry = entry.union(active_gate);
        }

        SplitSet(subflow_to_gate.into_iter().map(|(pi, ag)| Split{ subflow: pi.clone(), active_gate: ag }).collect::<BTreeSet<Split>>())
    }

    fn normalize(&mut self) {
        for mut pi in &mut self.0 {
            match pi {
                &mut FlowItem::Token(_) => {},
                &mut FlowItem::Split(ref mut ss) => {
                    let normed_ss = Flow::normalize_splits(&ss);

                    // TODO: If normalized splits has only one element (and therefore, has an allow-all gate),
                    //       convert into a subsequence of Tokens.

                    *ss = normed_ss;
                },
            }
        }
    }

    pub fn new(flow: Vec<FlowItem>) -> Self {
        let mut procedure = Flow(flow);
        procedure.normalize();
        procedure
    }

    pub fn create_scopes(&self) -> Scope {
        for flow_item in &self.0 {
        }

        Scope::new(Gate::allow_all(), vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::{Split, FlowItem, Flow, SplitSet};

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
            (
                splitset![
                    Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    Split{ subflow: flow![FlowItem::Split(splitset![
                        Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                        Split{ subflow: flow![FlowItem::Token(Token), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![5]) },
                    ]), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                splitset![
                    Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    Split{ subflow: flow![FlowItem::Split(splitset![
                        Split{ subflow: flow![FlowItem::Token(Token)], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                        Split{ subflow: flow![FlowItem::Token(Token), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![5]) },
                        Split{ subflow: flow![], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    ]), FlowItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    Split{ subflow: flow![], active_gate: Gate::Block(btreeset![0, 1, 2, 7]) },
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Flow::normalize_splits(&input);
            assert_eq!(expected, produced);
        }
    }
}
