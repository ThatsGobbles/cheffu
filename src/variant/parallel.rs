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
    pub fn apply(&self, stack: &mut Vec<Gate>) -> Result<(), Error> {
        if let &Some(ref gate) = &self.start {
            stack.push(gate.clone());
        }

        if let &Some(ref expected) = &self.close {
            // let produced: Gate = stack.pop().ok_or(GateHopError::EmptyStack)?;

            let opt_produced = stack.pop();

            if let Some(produced) = opt_produced {
                // We expect that the top of the stack should match our expected close gate.
                ensure!(*expected == produced, GateHopError::StackMismatch{expected: expected.clone(), produced: produced.clone()});
            }
            else {
                bail!(GateHopError::EmptyStack);
            }
        }

        Ok(())
    }
}
