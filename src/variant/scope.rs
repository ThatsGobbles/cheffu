use failure::Error;

use variant::gate::{Gate, Slot};

type ScopeDepth = usize;

#[derive(Debug, Fail, PartialEq, Eq)]
pub enum ScopeError {
    #[fail(display = "no common slots between expected and provided gates; expected {}, provided: {}", expected, provided)]
    NoIntersection {
        expected: Gate,
        provided: Gate,
    },

    #[fail(display = "cannot raise when depth already at zero")]
    CannotRaise,

    #[fail(display = "finished at non-zero depth; depth: {}", depth)]
    StillInScope {
        depth: ScopeDepth,
    },
}

pub struct ScopeManager {
    // When lowering into a scope, and going from depth `i` to depth `i+1`,
    // the `i`-th element of this is used and updated.
    // When raising out of a scope, and going from depth `j` to depth `j-1`,
    // every `k`-th element, where `k > j`, of this is deleted.
    cached_gates: Vec<Gate>,

    breadcrumbs: Vec<Vec<Gate>>,

    // The current depth of the manager.
    depth: ScopeDepth,
}

impl ScopeManager {
    pub fn new() -> Self {
        ScopeManager {
            cached_gates: vec![],
            breadcrumbs: vec![],
            depth: 0,
        }
    }

    pub fn lower(&mut self, provided_gate: &Gate) -> Result<(), Error> {
        let curr_depth = self.depth;
        let next_depth = curr_depth + 1;

        // Ensure we have a cache available.
        if self.cached_gates.len() <= next_depth {
            self.cached_gates.resize(next_depth, Gate::allow_all());
        }

        // Ensure we have a breadcrumb bucket available.
        if self.breadcrumbs.len() <= next_depth {
            self.breadcrumbs.resize(next_depth, vec![Gate::allow_all()]);
        }

        // Perform gate intersection.
        let intersection_gate = {
            let expected_gate = &self.cached_gates[curr_depth];
            let intersection_gate = expected_gate.intersection(provided_gate);

            ensure!(!intersection_gate.is_block_all(), ScopeError::NoIntersection{ expected: expected_gate.clone(), provided: provided_gate.clone() });
            intersection_gate
        };
        self.cached_gates[curr_depth] = intersection_gate.clone();
        self.breadcrumbs[curr_depth].pop();
        self.breadcrumbs[curr_depth].push(intersection_gate);

        // Update depth.
        self.depth = next_depth;

        Ok(())
    }

    pub fn raise(&mut self) -> Result<(), Error> {
        // Return error if trying to raise when depth is zero.
        ensure!(self.depth > 0, ScopeError::CannotRaise);

        let curr_depth = self.depth;
        let next_depth = curr_depth - 1;

        // Need to forget about orphaned scopes.
        // Remove any past caches from those scopes.
        // Do NOT remove any breadcrumbs!
        self.cached_gates.truncate(curr_depth + 1);

        // Update depth.
        self.depth = next_depth;

        Ok(())
    }

    pub fn close(mut self) -> Result<Vec<Vec<Gate>>, Error> {
        ensure!(self.depth == 0, ScopeError::StillInScope{ depth: self.depth.clone() });

        Ok(self.breadcrumbs)
    }
}

pub struct Scope {
    // The active slot is used to determine the path(s) to take when spelunking into one of this scope's subscopes.
    active_slot: Slot,

    // A sequence of subscopes contined in this scope.
    // Note that this is NOT a horizontal fanout, but the number of branchouts within a given vertical scope level!
    subscopes: Vec<Scope>,
}

use std::collections::BTreeMap;
use token::Token;

pub enum PathwayItem {
    Token(Token),
    Branch(BTreeMap<Gate, Pathway>),
}

pub type Pathway = Vec<PathwayItem>;

#[cfg(test)]
mod tests {
    use super::ScopeManager;

    use variant::gate::Gate;

    #[test]
    fn test_new() {
        let scope_manager = ScopeManager::new();

        assert!(scope_manager.cached_gates.is_empty());
        assert_eq!(scope_manager.depth, 0);
    }

    #[test]
    fn test_lower() {
        let gate_a = Gate::Allow(btreeset![0, 1, 2]);
        let gate_b = Gate::Allow(btreeset![1, 2, 3]);
        let gate_c = Gate::Block(btreeset![0, 1]);
        let gate_d = Gate::Allow(btreeset![3, 4, 5]);

        let mut scope_manager = ScopeManager::new();

        assert!(scope_manager.lower(&gate_a).is_ok());
        assert_eq!(&scope_manager.cached_gates[0], &gate_a);
        assert_eq!(&scope_manager.breadcrumbs, &vec![vec![gate_a.clone()]]);
        assert_eq!(scope_manager.depth, 1);

        assert!(scope_manager.lower(&gate_b).is_ok());
        assert_eq!(&scope_manager.cached_gates[1], &gate_b);
        assert_eq!(&scope_manager.breadcrumbs,
            &vec![
                vec![gate_a.clone()],
                vec![gate_b.clone()],
            ]
        );
        assert_eq!(scope_manager.depth, 2);

        // Manual intervention for testing purposes.
        scope_manager.depth = 1;

        assert!(scope_manager.lower(&gate_a).is_ok());
        assert_eq!(&scope_manager.cached_gates[1], &Gate::Allow(btreeset![1, 2]));
        assert_eq!(&scope_manager.breadcrumbs,
            &vec![
                vec![gate_a.clone()],
                vec![gate_b.clone(), Gate::Allow(btreeset![1, 2])],
            ]
        );
        assert_eq!(scope_manager.depth, 2);

        // Manual intervention for testing purposes.
        scope_manager.depth = 0;

        assert!(scope_manager.lower(&gate_d).is_err());
        assert_eq!(&scope_manager.cached_gates[0], &gate_a);
        assert_eq!(&scope_manager.breadcrumbs,
            &vec![
                vec![gate_a.clone()],
                vec![gate_b.clone(), Gate::Allow(btreeset![1, 2])],
            ]
        );
        assert_eq!(scope_manager.depth, 0);

        assert!(scope_manager.lower(&gate_c).is_ok());
        assert_eq!(&scope_manager.cached_gates[0], &Gate::Allow(btreeset![2]));
        assert_eq!(&scope_manager.breadcrumbs,
            &vec![
                vec![gate_a.clone(), Gate::Allow(btreeset![2])],
                vec![gate_b.clone(), Gate::Allow(btreeset![1, 2])],
            ]
        );
        assert_eq!(scope_manager.depth, 1);
    }

    #[test]
    fn test_raise() {
        let gate_a = Gate::Allow(btreeset![0, 1, 2]);
        let gate_b = Gate::Allow(btreeset![1, 2, 3]);
        let gate_c = Gate::Block(btreeset![0, 1]);
        let gate_d = Gate::Allow(btreeset![3, 4, 5]);

        let all_gates = vec![
            gate_a.clone(),
            gate_b.clone(),
            gate_c.clone(),
            gate_d.clone(),
        ];

        let mut scope_manager = ScopeManager::new();

        assert!(scope_manager.raise().is_err());
        assert_eq!(scope_manager.depth, 0);

        // Manual intervention for testing purposes.
        scope_manager.cached_gates = all_gates.clone();
        scope_manager.depth = scope_manager.cached_gates.len();

        assert!(scope_manager.raise().is_ok());
        assert_eq!(scope_manager.cached_gates, all_gates);
        assert_eq!(scope_manager.depth, 3);

        assert!(scope_manager.raise().is_ok());
        assert_eq!(scope_manager.cached_gates, all_gates);
        assert_eq!(scope_manager.depth, 2);

        assert!(scope_manager.raise().is_ok());
        assert_eq!(scope_manager.cached_gates, all_gates[..3].to_vec());
        assert_eq!(scope_manager.depth, 1);

        assert!(scope_manager.raise().is_ok());
        assert_eq!(scope_manager.cached_gates, all_gates[..2].to_vec());
        assert_eq!(scope_manager.depth, 0);

        assert!(scope_manager.raise().is_err());
        assert_eq!(scope_manager.cached_gates, all_gates[..2].to_vec());
        assert_eq!(scope_manager.depth, 0);
    }

    #[test]
    fn test_close() {
        let gate_a = Gate::Allow(btreeset![0, 1, 2]);
        let gate_b = Gate::Allow(btreeset![1, 2, 3]);
        let gate_c = Gate::Block(btreeset![0, 1]);
        let gate_d = Gate::Allow(btreeset![3, 4, 5]);

        let all_gates = vec![
            gate_a.clone(),
            gate_b.clone(),
            gate_c.clone(),
            gate_d.clone(),
        ];

        let mut scope_manager = ScopeManager::new();

        assert!(scope_manager.close().is_ok());

        let mut scope_manager = ScopeManager::new();

        // Manual intervention for testing purposes.
        scope_manager.cached_gates = all_gates.clone();
        scope_manager.depth = scope_manager.cached_gates.len();

        assert!(scope_manager.close().is_err());

        let mut scope_manager = ScopeManager::new();

        // Manual intervention for testing purposes.
        scope_manager.cached_gates = all_gates.clone();
        scope_manager.depth = 0;

        assert!(scope_manager.close().is_ok());
    }
}
