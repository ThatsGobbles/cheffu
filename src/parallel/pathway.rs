use std::collections::{HashMap, BTreeSet};

use super::gate::{Slot, Gate};
use token::Token;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum PathwayItem {
    Token(Token),
    Split(SplitSet),
}

pub type PathwayItemSeq = Vec<PathwayItem>;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Split {
    path_items: PathwayItemSeq,
    active_gate: Gate,
}

pub type SplitSet = BTreeSet<Split>;

/// Contains the tokens and splits that comprise all the variants of a single recipe.
pub struct Procedure(PathwayItemSeq);

impl Procedure {
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
            let coverage_ac = Split{ path_items: PathwayItemSeq::new(), active_gate: union_gate.invert() };
            split_seq.push(coverage_ac);
        }

        // Drop any split choices that have a block-all gate.
        split_seq.retain(|ref ac| !ac.active_gate.is_block_all());

        // Recurse to normalize nested split choices.
        for mut ac in &mut split_seq {
            for mut path_item in &mut ac.path_items {
                match path_item {
                    &mut PathwayItem::Token(_) => {},
                    &mut PathwayItem::Split(ref mut acs) => {
                        *acs = Procedure::normalize_splits(acs);
                    },
                };
            }
        }

        // If any split choices are identical, combine their gates.
        let mut path_items_to_gate: HashMap<&PathwayItemSeq, Gate> = hashmap![];
        for ac in &split_seq {
            let path_items = &ac.path_items;
            let active_gate = &ac.active_gate;
            let entry = path_items_to_gate.entry(path_items).or_insert(Gate::block_all());
            *entry = entry.union(active_gate);
        }

        path_items_to_gate.into_iter().map(|(pi, ag)| Split{ path_items: pi.to_vec(), active_gate: ag }).collect::<SplitSet>()
    }

    pub fn normalize(&mut self) {
        for mut pi in &mut self.0 {
            match pi {
                &mut PathwayItem::Token(_) => {},
                &mut PathwayItem::Split(ref mut ss) => {
                    let normed_ss = Procedure::normalize_splits(&ss);

                    // TODO: If normalized splits has only one element (and therefore, has an allow-all gate),
                    //       convert into a subsequence of Tokens.

                    *ss = normed_ss;
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Split, PathwayItem, Procedure};

    use super::super::gate::Gate;
    use token::Token;

    #[test]
    fn test_normalize_splits() {
        let inputs_and_expected = vec![
            (
                btreeset![
                    Split{ path_items: vec![], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    Split{ path_items: vec![], active_gate: Gate::Block(btreeset![]) },
                ],
            ),
            (
                btreeset![
                    Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![2, 3, 4]) },
                ],
                btreeset![
                    Split{ path_items: vec![], active_gate: Gate::Block(btreeset![0, 1, 2, 3, 4]) },
                    Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2, 3, 4]) },
                ],
            ),
            (
                btreeset![
                    Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![]) },
                    Split{ path_items: vec![PathwayItem::Token(Token), PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    Split{ path_items: vec![], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                    Split{ path_items: vec![PathwayItem::Token(Token), PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
            ),
            (
                btreeset![
                    Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                    Split{ path_items: vec![PathwayItem::Token(Token), PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                    Split{ path_items: vec![PathwayItem::Token(Token), PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
            ),
            (
                btreeset![],
                btreeset![
                    Split{ path_items: vec![], active_gate: Gate::Block(btreeset![]) },
                ],
            ),
            (
                btreeset![
                    Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    Split{ path_items: vec![PathwayItem::Split(btreeset![
                        Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                        Split{ path_items: vec![], active_gate: Gate::Allow(btreeset![5]) },
                    ]), PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    Split{ path_items: vec![PathwayItem::Split(btreeset![
                        Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                        Split{ path_items: vec![], active_gate: Gate::Allow(btreeset![5]) },
                    ]), PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    Split{ path_items: vec![], active_gate: Gate::Block(btreeset![0, 1, 2, 7]) },
                ],
            ),
            (
                btreeset![
                    Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    Split{ path_items: vec![PathwayItem::Split(btreeset![
                        Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                        Split{ path_items: vec![PathwayItem::Token(Token), PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![5]) },
                    ]), PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    Split{ path_items: vec![PathwayItem::Split(btreeset![
                        Split{ path_items: vec![PathwayItem::Token(Token)], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                        Split{ path_items: vec![PathwayItem::Token(Token), PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![5]) },
                        Split{ path_items: vec![], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    ]), PathwayItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    Split{ path_items: vec![], active_gate: Gate::Block(btreeset![0, 1, 2, 7]) },
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Procedure::normalize_splits(&input);
            assert_eq!(expected, produced);
        }
    }
}
