use failure::Error;

use super::gate::{Gate, Slot};
use super::pathway::{PathwayItem, PathwayItemSeq};

#[derive(Clone, PartialEq, Eq)]
pub struct Scope {
    // The active slot is used to determine the path(s) to take when spelunking into one of this scope's subscopes.
    active_slot: Slot,

    // A sequence of subscopes contined in this scope.
    // Each of these are keyed to this containing scope's active slot.
    // Note that this is NOT a horizontal fanout, but the number of branchouts within a given vertical scope level!
    subscopes: Vec<Scope>,
}

#[cfg(test)]
mod tests {
}
