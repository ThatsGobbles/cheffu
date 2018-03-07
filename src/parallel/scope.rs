use failure::Error;

use super::gate::{Gate, Slot};
use super::flow::{FlowItem, Flow};

#[derive(Clone, PartialEq, Eq)]
pub struct Scope {
    // The active gate is used to determine what slots can be chosen when spelunking into one of this scope's subscopes.
    active_gate: Gate,

    // A sequence of subscopes contined in this scope.
    // Each of these are keyed to this containing scope's active slot.
    // Note that this is NOT a horizontal fanout, but the number of branchouts within a given vertical scope level!
    subscopes: Vec<Scope>,
}

impl Scope {
    pub fn new(active_gate: Gate, subscopes: Vec<Scope>) -> Self {
        Scope { active_gate, subscopes }
    }
}

#[cfg(test)]
mod tests {
}
