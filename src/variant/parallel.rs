use std::collections::{HashMap, BTreeSet};

use variant::gate::{Slot, Gate};
use token::Token;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ProcedureItem {
    Token(Token),
    Split(AltChoiceSet),
}

pub type ProcedureItemSeq = Vec<ProcedureItem>;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct AltChoice {
    proc_items: ProcedureItemSeq,
    active_gate: Gate,
}

pub type AltChoiceSet = BTreeSet<AltChoice>;

/// Contains the edges, tokens, and gates that comprise all the variants of a single recipe.
pub struct ProcedureGraph(ProcedureItemSeq);

impl ProcedureGraph {
    /// Processes alt choices to coalesce identical alt choices, and to ensure that the union of all of its
    /// contained gates allows all slots (i.e. is an allow-all gate).
    pub fn normalize_alt_choices(alt_choice_set: &AltChoiceSet) -> AltChoiceSet {
        // Calculate the union gate, which allows all slots allowed in any of the alt choices.
        let union_gate = alt_choice_set.into_iter().fold(Gate::block_all(), |red, ref ac| red.union(&ac.active_gate));

        // Clone and collect into a sequence for easier mutation later on.
        let mut alt_choice_seq: Vec<AltChoice> = alt_choice_set.into_iter().cloned().collect();

        // If union gate is not allow-all, append an empty branch with the inverse of the union gate.
        // This provides an "escape hatch" for a case when a slot does not match any provided gate.
        if !union_gate.is_allow_all() {
            let coverage_ac = AltChoice{ proc_items: ProcedureItemSeq::new(), active_gate: union_gate.invert() };
            alt_choice_seq.push(coverage_ac);
        }

        // Drop any alt choices that have a block-all gate.
        alt_choice_seq.retain(|ref ac| !ac.active_gate.is_block_all());

        // Recurse to normalize nested alt choices.
        for mut ac in &mut alt_choice_seq {
            for mut proc_item in &mut ac.proc_items {
                match proc_item {
                    &mut ProcedureItem::Token(_) => {},
                    &mut ProcedureItem::Split(ref mut acs) => {
                        *acs = ProcedureGraph::normalize_alt_choices(acs);
                    },
                };
            }
        }

        // If any alt choices are identical, combine their gates.
        let mut proc_items_to_gate: HashMap<&ProcedureItemSeq, Gate> = hashmap![];
        for ac in &alt_choice_seq {
            let proc_items = &ac.proc_items;
            let active_gate = &ac.active_gate;
            let entry = proc_items_to_gate.entry(proc_items).or_insert(Gate::block_all());
            *entry = entry.union(active_gate);
        }

        proc_items_to_gate.into_iter().map(|(pi, ag)| AltChoice{ proc_items: pi.to_vec(), active_gate: ag }).collect::<AltChoiceSet>()
    }
}

#[cfg(test)]
mod tests {
    use super::{AltChoice, ProcedureItem, ProcedureGraph};

    use variant::gate::Gate;
    use token::Token;

    #[test]
    fn test_normalize_alt_choices() {
        let inputs_and_expected = vec![
            (
                btreeset![
                    AltChoice{ proc_items: vec![], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    AltChoice{ proc_items: vec![], active_gate: Gate::Block(btreeset![]) },
                ],
            ),
            (
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![2, 3, 4]) },
                ],
                btreeset![
                    AltChoice{ proc_items: vec![], active_gate: Gate::Block(btreeset![0, 1, 2, 3, 4]) },
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2, 3, 4]) },
                ],
            ),
            (
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![]) },
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    AltChoice{ proc_items: vec![], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
            ),
            (
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
            ),
            (
                btreeset![],
                btreeset![
                    AltChoice{ proc_items: vec![], active_gate: Gate::Block(btreeset![]) },
                ],
            ),
            (
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    AltChoice{ proc_items: vec![ProcedureItem::Split(btreeset![
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                        AltChoice{ proc_items: vec![], active_gate: Gate::Allow(btreeset![5]) },
                    ]), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    AltChoice{ proc_items: vec![ProcedureItem::Split(btreeset![
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                        AltChoice{ proc_items: vec![], active_gate: Gate::Allow(btreeset![5]) },
                    ]), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    AltChoice{ proc_items: vec![], active_gate: Gate::Block(btreeset![0, 1, 2, 7]) },
                ],
            ),
            (
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    AltChoice{ proc_items: vec![ProcedureItem::Split(btreeset![
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![5]) },
                    ]), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    AltChoice{ proc_items: vec![ProcedureItem::Split(btreeset![
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![5]) },
                        AltChoice{ proc_items: vec![], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    ]), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    AltChoice{ proc_items: vec![], active_gate: Gate::Block(btreeset![0, 1, 2, 7]) },
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = ProcedureGraph::normalize_alt_choices(&input);
            assert_eq!(expected, produced);
        }
    }
}
