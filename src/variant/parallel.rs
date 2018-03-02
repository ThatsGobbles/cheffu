use std::collections::{HashMap, HashSet, BTreeSet};

use failure::Error;

use variant::gate::{Slot, Gate};
use token::{Token, TokenSeq};

pub type Nodule = u32;

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

#[derive(Debug, Fail, PartialEq, Eq)]
pub enum GateStackError {
    #[fail(display = "stack is empty")]
    Empty,
    #[fail(display = "top of stack does not match; expected: {}, produced: {}", expected, produced)]
    Mismatch {
        expected: Gate,
        produced: Gate,
    },
    #[fail(display = "leftover items in stack; found: {:?}", leftover)]
    Leftover {
        leftover: Vec<Gate>,
    },
}

/// Represents an item in a start-to-finish walk through a procedure graph.
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum WalkItem<'a> {
    Token(&'a Token),
    Push(&'a Gate),
    Pop(&'a Gate),
}

/// Represents a start-to-finish walk through a procedure graph.
pub struct WalkItemSeq<'a>(Vec<WalkItem<'a>>);

impl<'a> WalkItemSeq<'a> {
    pub fn process(&self) -> Result<Vec<&Token>, Error> {
        let mut gate_stack: Vec<&Gate> = vec![];
        let mut tokens: Vec<&Token> = vec![];

        for walk_item in &self.0 {
            // LEARN: In here, `walk_item` is a reference.
            match walk_item {
                &WalkItem::Token(token) => {
                    tokens.push(token);
                },
                &WalkItem::Push(gate) => {
                    gate_stack.push(gate);
                },
                &WalkItem::Pop(gate) => {
                    let popped: &Gate = gate_stack.pop().ok_or(GateStackError::Empty)?;

                    // We expect that the top of the stack should match our expected close gate.
                    ensure!(gate == popped, GateStackError::Mismatch{expected: gate.clone(), produced: popped.clone()});
                },
            }
        }

        // LEARN: `.cloned()` calls `.clone()` on each element of an iterator.
        ensure!(gate_stack.is_empty(), GateStackError::Leftover{leftover: gate_stack.into_iter().cloned().collect()});

        Ok(tokens)
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
