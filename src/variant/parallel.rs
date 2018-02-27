use error::*;
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

// impl GateHop {
//     pub fn apply(&self, stack: &mut Vec<Gate>) -> Result<()> {
//         if let Some(gate) = self.start {
//             stack.push(gate);
//         }

//         if let Some(gate) = self.close {
//             let result = stack.pop().ok_or()?;
//         }

//         Ok(())
//     }
// }
