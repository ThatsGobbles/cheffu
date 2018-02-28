use std::collections::{HashMap, HashSet};

use failure::Error;

use variant::gate::Gate;
use token::Token;

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
    gate_hop: GateHop,
}

/// Represents a move from a (implied) nodule along an edge to a new nodule.
pub struct GraphHop {
    edge_id: EdgeId,
    dst_nodule: Nodule,
}

pub type GraphHopSequence = Vec<GraphHop>;

pub struct GraphWalk {
    start_nodule: Nodule,
    hop_seq: GraphHopSequence,
}

pub struct GateHop {
    start: Option<Gate>,
    close: Option<Gate>,
}

#[derive(Debug, Fail)]
pub enum GateHopError {
    #[fail(display = "stack is empty")]
    EmptyStack,
    #[fail(display = "top of stack does not match; expected: {}, produced: {}", expected, produced)]
    StackMismatch{
        expected: Gate,
        produced: Gate,
    },
}

impl GateHop {
    pub fn push(gate: Gate) -> Self {
        GateHop {
            start: Some(gate),
            close: None,
        }
    }

    pub fn pop(gate: Gate) -> Self {
        GateHop {
            start: None,
            close: Some(gate),
        }
    }

    pub fn apply(&self, stack: &mut Vec<Gate>) -> Result<(), Error> {
        if let &Some(ref gate) = &self.start {
            stack.push(gate.clone());
        }

        if let &Some(ref expected) = &self.close {
            let produced: Gate = stack.pop().ok_or(GateHopError::EmptyStack)?;

            // We expect that the top of the stack should match our expected close gate.
            ensure!(*expected == produced, GateHopError::StackMismatch{expected: expected.clone(), produced: produced.clone()});
        }

        Ok(())
    }
}

pub type GateHopSequence = Vec<GateHop>;

/// Set of edge IDs outbound for a (implied) nodule.
pub type OutEdgeIdSet = HashSet<EdgeId>;

/// Maps nodules to the IDs of edges travelling out from that nodule.
pub type NoduleOutEdgeMap = HashMap<Nodule, OutEdgeIdSet>;

/// Maps edge IDs to their edge definitions.
pub type EdgeLookupMap = HashMap<EdgeId, Edge>;

pub enum ProcedureItem {
    Token(Token),
    GatedAltChoices(GatedAltChoices),
}

pub type ProcedureItemSequence = Vec<ProcedureItem>;

pub struct GatedAlt {
    proc_items: ProcedureItemSequence,
    active_gate: Gate,
}

pub type GatedAltChoices = Vec<GatedAlt>;

pub fn normalize_gated_alt_choices(gated_alt_choices: &GatedAltChoices) -> GatedAltChoices {
    GatedAltChoices::new()
}

/// Connects two nodules together with an edge.
/// This edge will contain information about the tokens present on it, as well as the stack commands on start and close.
pub fn connect(
    new_edge_id: EdgeId,
    src_nodule: Nodule,
    dst_nodule: Nodule,
    nodule_out_edge_map: &mut NoduleOutEdgeMap,
    edge_lookup_map: &mut EdgeLookupMap,
    token_seq: Vec<Token>,
    gate_hop: GateHop,
)
{
    // A new edge needs to be created.
    let edge = Edge{
        id: new_edge_id,
        src_nodule,
        dst_nodule,
        token_seq,
        gate_hop,
    };

    // Add edge ID to nodule out edge map, creating if not already existing.
    nodule_out_edge_map.entry(src_nodule).or_default().insert(new_edge_id);

    // Add edge and edge ID to edge lookup map.
    edge_lookup_map.insert(new_edge_id, edge);
}
