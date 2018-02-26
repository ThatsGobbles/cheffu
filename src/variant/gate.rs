use std::collections::HashSet;

/// An identifier for a unique variant pathway through a recipe.
pub type Slot = u8;

/// Represents a filter on a recipe's logical variant pathway, allowing or restricting certain variants from proceeding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    Allow(HashSet<Slot>),
    Block(HashSet<Slot>),
}

impl Gate {
    pub fn allow_all() -> Self {
        Gate::Block(hashset![])
    }
    pub fn block_all() -> Self {
        Gate::Allow(hashset![])
    }

    pub fn is_allow(&self) -> bool {
        match *self {
            Gate::Allow(_) => true,
            _ => false,
        }
    }

    pub fn is_block(&self) -> bool {
        !self.is_allow()
    }

    pub fn invert(&self) -> Self {
        match *self {
            Gate::Allow(ref s) => Gate::Block(s.clone()),
            Gate::Block(ref s) => Gate::Allow(s.clone()),
        }
    }

    pub fn allows_slot(&self, slot: Slot) -> bool {
        match *self {
            Gate::Allow(ref s) => s.contains(&slot),
            Gate::Block(ref s) => !s.contains(&slot),
        }
    }

    pub fn blocks_slot(&self, slot: Slot) -> bool {
        !self.allows_slot(slot)
    }

    // pub fn slots(&self) -> &HashSet<Slot> {
    //     match *self {
    //         Gate::Allow(ref s) => s,
    //         Gate::Block(ref s) => s,
    //     }
    // }

    /// Produces a gate that allows all slots that would pass at least one of the input gates.
    pub fn union(&self, other: &Self) -> Self {
        match (self, other) {
            (&Gate::Allow(ref ls), &Gate::Allow(ref rs)) => Gate::Allow(ls.union(rs).cloned().collect()),
            (&Gate::Allow(ref ls), &Gate::Block(ref rs)) => Gate::Block(rs.difference(ls).cloned().collect()),
            (&Gate::Block(ref ls), &Gate::Allow(ref rs)) => Gate::Block(ls.difference(rs).cloned().collect()),
            (&Gate::Block(ref ls), &Gate::Block(ref rs)) => Gate::Block(ls.intersection(rs).cloned().collect()),
        }
    }

    /// Produces a gate that allows all slots that would pass all of the input gates.
    pub fn intersection(&self, other: &Self) -> Self {
        match (self, other) {
            (&Gate::Allow(ref ls), &Gate::Allow(ref rs)) => Gate::Allow(ls.intersection(rs).cloned().collect()),
            (&Gate::Allow(ref ls), &Gate::Block(ref rs)) => Gate::Allow(ls.difference(rs).cloned().collect()),
            (&Gate::Block(ref ls), &Gate::Allow(ref rs)) => Gate::Allow(rs.difference(ls).cloned().collect()),
            (&Gate::Block(ref ls), &Gate::Block(ref rs)) => Gate::Block(ls.union(rs).cloned().collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Gate;

    #[test]
    fn test_allow_all() {
        let expected = Gate::Block(hashset![]);
        let produced = Gate::allow_all();

        assert_eq!(expected, produced);
    }

    #[test]
    fn test_block_all() {
        let expected = Gate::Allow(hashset![]);
        let produced = Gate::block_all();

        assert_eq!(expected, produced);
    }

    #[test]
    fn test_is_allow() {
        let gates_and_expected = vec![
            (Gate::Allow(hashset![]), true),
            (Gate::Block(hashset![]), false),
        ];

        for (gate, expected) in gates_and_expected {
            let produced = gate.is_allow();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_is_block() {
        let gates_and_expected = vec![
            (Gate::Allow(hashset![]), false),
            (Gate::Block(hashset![]), true),
        ];

        for (gate, expected) in gates_and_expected {
            let produced = gate.is_block();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_invert() {
        let slot_sets = vec![
            hashset![0, 1, 2],
            hashset![],
            hashset![27],
        ];

        for slot_set in slot_sets {
            let input = Gate::Allow(slot_set.clone());
            let produced = input.invert();
            let expected = Gate::Block(slot_set.clone());
            assert_eq!(expected, produced);

            let input = Gate::Block(slot_set.clone());
            let produced = input.invert();
            let expected = Gate::Allow(slot_set.clone());
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_allows_slot() {
        let inputs_and_expected = vec![
            ((Gate::Allow(hashset![0, 1, 2]), 1), true),
            ((Gate::Allow(hashset![0, 1, 2]), 3), false),
            ((Gate::Block(hashset![0, 1, 2]), 1), false),
            ((Gate::Block(hashset![0, 1, 2]), 3), true),
            ((Gate::Allow(hashset![]), 0), false),
            ((Gate::Block(hashset![]), 0), true),
        ];

        for ((gate, slot), expected) in inputs_and_expected {
            let produced = gate.allows_slot(slot);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_blocks_slot() {
        let inputs_and_expected = vec![
            ((Gate::Allow(hashset![0, 1, 2]), 1), false),
            ((Gate::Allow(hashset![0, 1, 2]), 3), true),
            ((Gate::Block(hashset![0, 1, 2]), 1), true),
            ((Gate::Block(hashset![0, 1, 2]), 3), false),
            ((Gate::Allow(hashset![]), 0), true),
            ((Gate::Block(hashset![]), 0), false),
        ];

        for ((gate, slot), expected) in inputs_and_expected {
            let produced = gate.blocks_slot(slot);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_union() {
        let inputs_and_expected = vec![
            ((Gate::Allow(hashset![0, 1, 2]), Gate::Allow(hashset![2, 3, 4])),
                Gate::Allow(hashset![0, 1, 2, 3, 4])),
            ((Gate::Allow(hashset![0, 1, 2]), Gate::Block(hashset![2, 3, 4])),
                Gate::Block(hashset![3, 4])),
            ((Gate::Block(hashset![0, 1, 2]), Gate::Allow(hashset![2, 3, 4])),
                Gate::Block(hashset![0, 1])),
            ((Gate::Block(hashset![0, 1, 2]), Gate::Block(hashset![2, 3, 4])),
                Gate::Block(hashset![2])),
        ];

        for ((l_gate, r_gate), expected) in inputs_and_expected {
            let produced = l_gate.union(&r_gate);
            assert_eq!(expected, produced);

            // Manually perform the same logic that union should provide.
            for slot in 0u8..10 {
                let l_is_allowed = l_gate.allows_slot(slot);
                let r_is_allowed = r_gate.allows_slot(slot);
                let u_is_allowed = produced.allows_slot(slot);

                assert_eq!(l_is_allowed || r_is_allowed, u_is_allowed);
            }
        }
    }

    #[test]
    fn test_intersection() {
        let inputs_and_expected = vec![
            ((Gate::Allow(hashset![0, 1, 2]), Gate::Allow(hashset![2, 3, 4])),
                Gate::Allow(hashset![2])),
            ((Gate::Allow(hashset![0, 1, 2]), Gate::Block(hashset![2, 3, 4])),
                Gate::Allow(hashset![0, 1])),
            ((Gate::Block(hashset![0, 1, 2]), Gate::Allow(hashset![2, 3, 4])),
                Gate::Allow(hashset![3, 4])),
            ((Gate::Block(hashset![0, 1, 2]), Gate::Block(hashset![2, 3, 4])),
                Gate::Block(hashset![0, 1, 2, 3, 4])),
            ((Gate::Allow(hashset![0, 1, 2]), Gate::Allow(hashset![3, 4, 5])),
                Gate::Allow(hashset![])),
            ((Gate::Allow(hashset![0, 1, 2]), Gate::Block(hashset![0, 1, 2])),
                Gate::Allow(hashset![])),
        ];

        for ((l_gate, r_gate), expected) in inputs_and_expected {
            let produced = l_gate.intersection(&r_gate);
            assert_eq!(expected, produced);

            // Manually perform the same logic that intersection should provide.
            for slot in 0u8..10 {
                let l_is_allowed = l_gate.allows_slot(slot);
                let r_is_allowed = r_gate.allows_slot(slot);
                let u_is_allowed = produced.allows_slot(slot);

                assert_eq!(l_is_allowed && r_is_allowed, u_is_allowed);
            }
        }
    }

    // #[test]
    // fn test_slots() {
    //     // TODO: Complete!
    // }
}
