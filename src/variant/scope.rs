use failure::Error;

use variant::gate::Gate;

#[derive(Debug, Fail, PartialEq, Eq)]
pub enum ScopeError {
    #[fail(display = "no common slots between expected and provided gates; expected {}, provided: {}", expected, provided)]
    NoIntersection {
        expected: Gate,
        provided: Gate,
    },
}

pub struct ScopeManager {
    // When beginning a scope, and going from depth `i` to depth `i+1`,
    // the `i`-th element of this is used and updated.
    // When closing a scope, and going from depth `j` to depth `j-1`,
    // every `k`-th element, where `k > j`, of this is deleted.
    cached_gates: Vec<Gate>,
    depth: usize,
}

impl ScopeManager {
    pub fn begin(&mut self, provided_gate: &Gate) -> Result<(), Error> {
        let curr_depth = self.depth;
        let next_depth = curr_depth + 1;

        // Ensure we have a cache available.
        if self.cached_gates.len() <= next_depth {
            self.cached_gates.resize(next_depth, Gate::allow_all());
        }

        // Perform gate intersection.
        let intersection_gate = {
            let expected_gate = &self.cached_gates[curr_depth];
            let intersection_gate = expected_gate.intersection(provided_gate);

            ensure!(!intersection_gate.is_block_all(), ScopeError::NoIntersection{ expected: expected_gate.clone(), provided: provided_gate.clone() });
            intersection_gate
        };
        self.cached_gates[curr_depth] = intersection_gate;

        // Update depth.
        self.depth = next_depth;

        Ok(())
    }

    pub fn close(&mut self) -> Result<(), Error> {
        let curr_depth = self.depth;
        let next_depth = curr_depth - 1;

        // Need to forget about orphaned scopes.
        // Remove any past caches from those scopes.
        self.cached_gates.truncate(curr_depth + 1);

        // Update depth.
        self.depth = next_depth;

        Ok(())
    }
}
