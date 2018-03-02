use std::collections::{HashMap, HashSet, BTreeSet};

use failure::Error;

use variant::gate::Gate;
use token::{Token, TokenSeq};

pub type UniqueId = u32;

pub type Nodule = UniqueId;
pub type EdgeId = UniqueId;

// /// Cheffu uses an edge-first system design, where edges represent directed connections between nodules.
// /// Edges contain most of the interesting information of the graph, including variant gates and tokens.
// /// It is possible to have multiple edges between a pair of nodules, due to alts and variants.
// pub struct Edge {
//     id: EdgeId,
//     src_nodule: Nodule,
//     dst_nodule: Nodule,
//     token_seq: TokenSeq,
//     gate_op: Option<GateOp>,
//     // dst_gate_op: Option<GateOp>,
// }

// /// Set of edge ids outbound for a (implied) nodule.
// pub type OutEdgeIdSet = HashSet<EdgeId>;

// /// Maps nodules to the ids of edges travelling out from that nodule.
// pub type NoduleOutEdgeMap = HashMap<Nodule, OutEdgeIdSet>;

// /// Maps edge ids to their edge definitions.
// pub type EdgeLookupMap = HashMap<EdgeId, Edge>;

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
pub enum GateOpError {
    #[fail(display = "stack is empty")]
    EmptyStack,
    #[fail(display = "top of stack does not match; expected: {}, produced: {}", expected, produced)]
    StackMismatch {
        expected: Gate,
        produced: Gate,
    },
    #[fail(display = "leftover items in stack; found: {:?}", leftover)]
    StackLeftover {
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
                    let popped: &Gate = gate_stack.pop().ok_or(GateOpError::EmptyStack)?;

                    // We expect that the top of the stack should match our expected close gate.
                    ensure!(gate == popped, GateOpError::StackMismatch{expected: gate.clone(), produced: popped.clone()});
                },
            }
        }

        // LEARN: `.cloned()` calls `.clone()` on each element of an iterator.
        ensure!(gate_stack.is_empty(), GateOpError::StackLeftover{leftover: gate_stack.into_iter().cloned().collect()});

        Ok(tokens)
    }
}

// pub struct EdgeIdGen(EdgeId);

// impl EdgeIdGen {
//     pub fn advance(&mut self) -> EdgeId {
//         let to_return = self.0.clone();
//         self.0 += 1;
//         to_return
//     }
// }

// pub struct NoduleGen(Nodule);

// impl NoduleGen {
//     pub fn advance(&mut self) -> Nodule {
//         let to_return = self.0.clone();
//         self.0 += 1;
//         to_return
//     }
// }

// /// Contains the edges, tokens, and gates that comprise all the variants of a single recipe.
// pub struct ProcedureGraph {
//     nodule_out_edge_map: NoduleOutEdgeMap,
//     edge_lookup_map: EdgeLookupMap,
//     edge_id_gen: EdgeIdGen,
//     nodule_gen: NoduleGen,
// }

// impl ProcedureGraph {
//     /// Creates a new `ProcedureGraph`.
//     pub fn new() -> Self {
//         ProcedureGraph {
//             nodule_out_edge_map: NoduleOutEdgeMap::new(),
//             edge_lookup_map: EdgeLookupMap::new(),
//             edge_id_gen: EdgeIdGen(0),
//             nodule_gen: NoduleGen(0),
//         }
//     }

//     /// Connects two nodules together with an edge.
//     /// This edge will contain information about the tokens present on it, as well as the stack commands on start and close.
//     pub fn connect(
//         &mut self,
//         src_nodule: Nodule,
//         dst_nodule: Nodule,
//         token_seq: TokenSeq,
//         gate_op: Option<GateOp>,
//         // dst_gate_op: Option<GateOp>,
//     )
//     {
//         // Create a new edge id,
//         let new_edge_id = self.edge_id_gen.advance();

//         // A new edge needs to be created.
//         let edge = Edge{
//             id: new_edge_id,
//             src_nodule,
//             dst_nodule,
//             token_seq,
//             gate_op,
//             // dst_gate_op,
//         };

//         // Add edge id to nodule out edge map, creating if not already existing.
//         self.nodule_out_edge_map.entry(src_nodule).or_default().insert(new_edge_id);

//         // Add edge and edge id to edge lookup map.
//         self.edge_lookup_map.insert(new_edge_id, edge);
//     }

//     pub fn process_procedure_item_seq(
//         &mut self,
//         procedure_item_seq: &ProcedureItemSeq,
//         src_nodule: Nodule,
//         dst_nodule: Nodule,
//         gate: Gate,
//     )
//     {
//         // Keep track of the most recent src nodule.
//         let curr_src_nodule = src_nodule.clone();

//         // Collect tokens encountered directly on this procedure path.
//         let mut encountered_tokens: TokenSeq = vec![];

//         // LEARN: In this case `procedure_item_seq` is a reference, so `procedure_item` is as well.
//         for procedure_item in procedure_item_seq {
//             match procedure_item {
//                 &ProcedureItem::Token(ref token) => {
//                     encountered_tokens.push(token.clone());
//                 },
//                 &ProcedureItem::Split(ref alt_choices) => {
//                     // Create new src and dst nodules for the to-be-processed alt choices.
//                     let alt_src_nodule = self.nodule_gen.advance();
//                     let alt_dst_nodule = self.nodule_gen.advance();

//                     // Capture current list of encountered tokens.
//                     // Close off the current path by connecting to the new src nodule.
//                     self.connect(
//                         curr_src_nodule,
//                         alt_src_nodule,
//                         encountered_tokens,
//                         gate_op.clone(),
//                         // dst_gate_op.clone(),
//                     );

//                     // # We only want to put the stack command on the first out path of a branch, not on any further down.
//                     // if start_slot_filter_stack_command is not None:
//                     //     start_slot_filter_stack_command = None

//                     // Reset encountered tokens.
//                     encountered_tokens = vec![];
//                 },
//             };
//         }
//     }

//     pub fn process_alt_choice_set(
//         &mut self,
//         alt_choice_set: &AltChoiceSet,
//         src_nodule: Nodule,
//         dst_nodule: Nodule,

//     )
//     {
//         // Normalize the alt choices.
//         // Each of the resulting alt choices will be 'sandwiched' between the provided src and dst nodules.
//         let alt_choice_set = normalize_alt_choices(alt_choice_set);

//         for alt_choice in alt_choice_set {
//             let gate_op = Some(GateOp::Push(alt_choice.active_gate.clone()));
//             // let dst_gate_op = Some(GateOp::Pop(alt_choice.active_gate.clone()));

//             self.process_procedure_item_seq(
//                 &alt_choice.proc_items,
//                 src_nodule,
//                 dst_nodule,
//                 gate_op,
//                 // dst_gate_op,
//             );
//         }
//     }
// }

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
