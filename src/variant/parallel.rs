use std::collections::{HashMap, HashSet, BTreeSet};

use failure::Error;

use variant::gate::Gate;
use token::{Token, TokenSeq};

pub type UniqueId = u32;

pub type Nodule = UniqueId;
pub type EdgeId = UniqueId;

/// Cheffu uses an edge-first system design, where edges represent directed connections between nodules.
/// Edges contain most of the interesting information of the graph, including variant gates and tokens.
/// It is possible to have multiple edges between a pair of nodules, due to alts and variants.
pub struct Edge {
    id: EdgeId,
    src_nodule: Nodule,
    dst_nodule: Nodule,
    token_seq: Vec<Token>,
    gate: Gate,
}

/// Represents a move from a (implied) nodule along an edge to a new nodule.
pub struct GraphHop {
    edge_id: EdgeId,
    dst_nodule: Nodule,
}

pub type GraphHopSeq = Vec<GraphHop>;

pub struct GraphWalk {
    start_nodule: Nodule,
    hop_seq: GraphHopSeq,
}

#[derive(Debug, Fail)]
pub enum GateCmdError {
    #[fail(display = "stack is empty")]
    EmptyStack,
    #[fail(display = "top of stack does not match; expected: {}, produced: {}", expected, produced)]
    StackMismatch{
        expected: Gate,
        produced: Gate,
    },
}

pub enum GateCmd {
    Push(Gate),
    Pop(Gate),
}

impl GateCmd {
    pub fn apply(&self, stack: &mut Vec<Gate>) -> Result<(), Error> {
        match self {
            &GateCmd::Push(ref gate) => { stack.push(gate.clone()); },
            &GateCmd::Pop(ref gate) => {
                let popped: Gate = stack.pop().ok_or(GateCmdError::EmptyStack)?;

                // We expect that the top of the stack should match our expected close gate.
                ensure!(*gate == popped, GateCmdError::StackMismatch{expected: gate.clone(), produced: popped.clone()});
            },
        }

        Ok(())
    }
}

/// Set of edge ids outbound for a (implied) nodule.
pub type OutEdgeIdSet = HashSet<EdgeId>;

/// Maps nodules to the ids of edges travelling out from that nodule.
pub type NoduleOutEdgeMap = HashMap<Nodule, OutEdgeIdSet>;

/// Maps edge ids to their edge definitions.
pub type EdgeLookupMap = HashMap<EdgeId, Edge>;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ProcedureItem {
    Token(Token),
    AltChoices(AltChoiceSet),
}

pub type ProcedureItemSeq = Vec<ProcedureItem>;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct AltChoice {
    proc_items: ProcedureItemSeq,
    active_gate: Gate,
}

pub type AltChoiceSet = BTreeSet<AltChoice>;

/// Processes alt choices to remove multiple null alts, and to ensure that the union of all of its
/// contained gates allows all slots (i.e. is an allow-all gate).
pub fn normalize_alt_choices(alt_choice_set: &AltChoiceSet) -> AltChoiceSet {
    // Calculate the value of the else-filter, which contains all slots not explicitly allowed in the alt choice.
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
                &mut ProcedureItem::AltChoices(ref mut acs) => {
                    *acs = normalize_alt_choices(acs);
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

/// Contains the edges, tokens, and gates that comprise all the variants of a single recipe.
pub struct ProcedureGraph {
    nodule_out_edge_map: NoduleOutEdgeMap,
    edge_lookup_map: EdgeLookupMap,
    curr_edge_id: EdgeId,
}

impl ProcedureGraph {
    /// Connects two nodules together with an edge.
    /// This edge will contain information about the tokens present on it, as well as the stack commands on start and close.
    pub fn connect(
        &mut self,
        src_nodule: Nodule,
        dst_nodule: Nodule,
        token_seq: TokenSeq,
        gate: Gate,
    )
    {
        // Create a new edge id,
        let new_edge_id = self.curr_edge_id.clone();
        self.curr_edge_id += 1;

        // A new edge needs to be created.
        let edge = Edge{
            id: new_edge_id,
            src_nodule,
            dst_nodule,
            token_seq,
            gate,
        };

        // Add edge id to nodule out edge map, creating if not already existing.
        self.nodule_out_edge_map.entry(src_nodule).or_default().insert(new_edge_id);

        // Add edge and edge id to edge lookup map.
        self.edge_lookup_map.insert(new_edge_id, edge);
    }
}

#[cfg(test)]
mod tests {
    use super::{AltChoice, ProcedureItem};
    use super::normalize_alt_choices;

    use std::collections::HashSet;

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
                    AltChoice{ proc_items: vec![ProcedureItem::AltChoices(btreeset![
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                        AltChoice{ proc_items: vec![], active_gate: Gate::Allow(btreeset![5]) },
                    ]), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    AltChoice{ proc_items: vec![ProcedureItem::AltChoices(btreeset![
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Block(btreeset![]) },
                        AltChoice{ proc_items: vec![], active_gate: Gate::Allow(btreeset![5]) },
                    ]), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    AltChoice{ proc_items: vec![], active_gate: Gate::Block(btreeset![0, 1, 2, 7]) },
                ],
            ),
            (
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    AltChoice{ proc_items: vec![ProcedureItem::AltChoices(btreeset![
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![5]) },
                    ]), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                ],
                btreeset![
                    AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![7]) },
                    AltChoice{ proc_items: vec![ProcedureItem::AltChoices(btreeset![
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token)], active_gate: Gate::Block(btreeset![0, 1, 2]) },
                        AltChoice{ proc_items: vec![ProcedureItem::Token(Token), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![5]) },
                        AltChoice{ proc_items: vec![], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    ]), ProcedureItem::Token(Token)], active_gate: Gate::Allow(btreeset![0, 1, 2]) },
                    AltChoice{ proc_items: vec![], active_gate: Gate::Block(btreeset![0, 1, 2, 7]) },
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = normalize_alt_choices(&input);
            assert_eq!(expected, produced);
        }
    }
}
