use std::collections::BTreeSet;
use std::fmt;

/// An identifier for a unique variant pathway through a recipe.
pub type Slot = u8;
pub type SlotSet = BTreeSet<Slot>;

/// Represents a filter on a recipe's logical variant pathway, allowing or restricting certain variants from proceeding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    Allow(SlotSet),
    Block(SlotSet),
}

impl Gate {
    pub fn allow_all() -> Self {
        Gate::Block(btreeset![])
    }

    pub fn block_all() -> Self {
        Gate::Allow(btreeset![])
    }

    /// Checks if a gate has 'allow' semantics (is a white list).
    pub fn is_allow(&self) -> bool {
        match *self {
            Gate::Allow(_) => true,
            _ => false,
        }
    }

    /// Checks if a gate has 'block' semantics (is a black list).
    pub fn is_block(&self) -> bool {
        !self.is_allow()
    }

    /// Checks if a gate is 'allow-all', blocking no slots.
    pub fn is_allow_all(&self) -> bool {
        match *self {
            Gate::Block(ref s) => s.is_empty(),
            _ => false,
        }
    }

    /// Checks if a gate is 'block-all', allowing no slots.
    pub fn is_block_all(&self) -> bool {
        match *self {
            Gate::Allow(ref s) => s.is_empty(),
            _ => false,
        }
    }

    pub fn slots(&self) -> &SlotSet {
        match self {
            &Gate::Allow(ref s) => s,
            &Gate::Block(ref s) => s,
        }
    }

    /// Inverts a gate.
    /// The resulting gate allows any slots blocked by the input gate, and vice versa.
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

    /// Combines two gates using a union operation.
    /// The resulting gate allows any slots allowed by either of the input gates.
    pub fn union(&self, gate: &Self) -> Self {
        match (self, gate) {
            (&Gate::Allow(ref ls), &Gate::Allow(ref rs)) => Gate::Allow(ls.union(rs).cloned().collect()),
            (&Gate::Allow(ref ls), &Gate::Block(ref rs)) => Gate::Block(rs.difference(ls).cloned().collect()),
            (&Gate::Block(ref ls), &Gate::Allow(ref rs)) => Gate::Block(ls.difference(rs).cloned().collect()),
            (&Gate::Block(ref ls), &Gate::Block(ref rs)) => Gate::Block(ls.intersection(rs).cloned().collect()),
        }
    }

    /// Combines two gates using an intersection operation.
    /// The resulting gate allows any slots allowed by both of the input gates.
    pub fn intersection(&self, gate: &Self) -> Self {
        match (self, gate) {
            (&Gate::Allow(ref ls), &Gate::Allow(ref rs)) => Gate::Allow(ls.intersection(rs).cloned().collect()),
            (&Gate::Allow(ref ls), &Gate::Block(ref rs)) => Gate::Allow(ls.difference(rs).cloned().collect()),
            (&Gate::Block(ref ls), &Gate::Allow(ref rs)) => Gate::Allow(rs.difference(ls).cloned().collect()),
            (&Gate::Block(ref ls), &Gate::Block(ref rs)) => Gate::Block(ls.union(rs).cloned().collect()),
        }
    }

    /// Combines two gates using a difference operation.
    /// The resulting gate allows any slots allowed by the first, but not the second, input gate.
    pub fn difference(&self, gate: &Self) -> Self {
        self.intersection(&gate.invert())
    }

    /// Combines two gates using a symmetric difference operation.
    /// The resulting gate allows any slots allowed by exactly one of input gates.
    pub fn sym_difference(&self, gate: &Self) -> Self {
        let slots: SlotSet = self.slots().symmetric_difference(&gate.slots()).cloned().collect();
        match (self, gate) {
            (&Gate::Allow(_), &Gate::Allow(_)) | (&Gate::Block(_), &Gate::Block(_)) => Gate::Allow(slots),
            (&Gate::Allow(_), &Gate::Block(_)) | (&Gate::Block(_), &Gate::Allow(_)) => Gate::Block(slots),
        }
    }
}

impl fmt::Display for Gate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self.is_allow() {
            true => "ALLOW",
            false => "BLOCK",
        };

        let slots = match *self {
            Gate::Allow(ref s) => s,
            Gate::Block(ref s) => s,
        };

        write!(f, "{}({:?})", name, slots)
    }
}

#[cfg(test)]
mod tests {
    use super::Gate;

    #[test]
    fn test_allow_all() {
        let expected = Gate::Block(btreeset![]);
        let produced = Gate::allow_all();

        assert_eq!(expected, produced);
    }

    #[test]
    fn test_block_all() {
        let expected = Gate::Allow(btreeset![]);
        let produced = Gate::block_all();

        assert_eq!(expected, produced);
    }

    #[test]
    fn test_is_allow() {
        let gates_and_expected = vec![
            (Gate::Allow(btreeset![]), true),
            (Gate::Block(btreeset![]), false),
        ];

        for (gate, expected) in gates_and_expected {
            let produced = gate.is_allow();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_is_block() {
        let gates_and_expected = vec![
            (Gate::Allow(btreeset![]), false),
            (Gate::Block(btreeset![]), true),
        ];

        for (gate, expected) in gates_and_expected {
            let produced = gate.is_block();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_invert() {
        let slot_sets = vec![
            btreeset![0, 1, 2],
            btreeset![],
            btreeset![27],
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
            ((Gate::Allow(btreeset![0, 1, 2]), 1), true),
            ((Gate::Allow(btreeset![0, 1, 2]), 3), false),
            ((Gate::Block(btreeset![0, 1, 2]), 1), false),
            ((Gate::Block(btreeset![0, 1, 2]), 3), true),
            ((Gate::Allow(btreeset![]), 0), false),
            ((Gate::Block(btreeset![]), 0), true),
        ];

        for ((gate, slot), expected) in inputs_and_expected {
            let produced = gate.allows_slot(slot);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_blocks_slot() {
        let inputs_and_expected = vec![
            ((Gate::Allow(btreeset![0, 1, 2]), 1), false),
            ((Gate::Allow(btreeset![0, 1, 2]), 3), true),
            ((Gate::Block(btreeset![0, 1, 2]), 1), true),
            ((Gate::Block(btreeset![0, 1, 2]), 3), false),
            ((Gate::Allow(btreeset![]), 0), true),
            ((Gate::Block(btreeset![]), 0), false),
        ];

        for ((gate, slot), expected) in inputs_and_expected {
            let produced = gate.blocks_slot(slot);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_union() {
        let inputs_and_expected = vec![
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Allow(btreeset![2, 3, 4])),
                Gate::Allow(btreeset![0, 1, 2, 3, 4])),
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Block(btreeset![2, 3, 4])),
                Gate::Block(btreeset![3, 4])),
            ((Gate::Block(btreeset![0, 1, 2]), Gate::Allow(btreeset![2, 3, 4])),
                Gate::Block(btreeset![0, 1])),
            ((Gate::Block(btreeset![0, 1, 2]), Gate::Block(btreeset![2, 3, 4])),
                Gate::Block(btreeset![2])),
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
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Allow(btreeset![2, 3, 4])),
                Gate::Allow(btreeset![2])),
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Block(btreeset![2, 3, 4])),
                Gate::Allow(btreeset![0, 1])),
            ((Gate::Block(btreeset![0, 1, 2]), Gate::Allow(btreeset![2, 3, 4])),
                Gate::Allow(btreeset![3, 4])),
            ((Gate::Block(btreeset![0, 1, 2]), Gate::Block(btreeset![2, 3, 4])),
                Gate::Block(btreeset![0, 1, 2, 3, 4])),
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Allow(btreeset![3, 4, 5])),
                Gate::Allow(btreeset![])),
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Block(btreeset![0, 1, 2])),
                Gate::Allow(btreeset![])),
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

    #[test]
    fn test_difference() {
        let inputs_and_expected = vec![
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Allow(btreeset![2, 3, 4])),
                Gate::Allow(btreeset![0, 1])),
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Block(btreeset![2, 3, 4])),
                Gate::Allow(btreeset![2])),
            ((Gate::Block(btreeset![0, 1, 2]), Gate::Allow(btreeset![2, 3, 4])),
                Gate::Block(btreeset![0, 1, 2, 3, 4])),
            ((Gate::Block(btreeset![0, 1, 2]), Gate::Block(btreeset![2, 3, 4])),
                Gate::Allow(btreeset![3, 4])),
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Allow(btreeset![3, 4, 5])),
                Gate::Allow(btreeset![0, 1, 2])),
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Block(btreeset![0, 1, 2])),
                Gate::Allow(btreeset![0, 1, 2])),
        ];

        for ((l_gate, r_gate), expected) in inputs_and_expected {
            let produced = l_gate.difference(&r_gate);
            assert_eq!(expected, produced);

            // Manually perform the same logic that difference should provide.
            for slot in 0u8..10 {
                let l_is_allowed = l_gate.allows_slot(slot);
                let r_is_allowed = r_gate.allows_slot(slot);
                let u_is_allowed = produced.allows_slot(slot);

                assert_eq!(l_is_allowed & !r_is_allowed, u_is_allowed);
            }
        }
    }

    #[test]
    fn test_sym_difference() {
        let inputs_and_expected = vec![
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Allow(btreeset![2, 3, 4])),
                Gate::Allow(btreeset![0, 1, 3, 4])),
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Block(btreeset![2, 3, 4])),
                Gate::Block(btreeset![0, 1, 3, 4])),
            ((Gate::Block(btreeset![0, 1, 2]), Gate::Allow(btreeset![2, 3, 4])),
                Gate::Block(btreeset![0, 1, 3, 4])),
            ((Gate::Block(btreeset![0, 1, 2]), Gate::Block(btreeset![2, 3, 4])),
                Gate::Allow(btreeset![0, 1, 3, 4])),
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Allow(btreeset![3, 4, 5])),
                Gate::Allow(btreeset![0, 1, 2, 3, 4, 5])),
            ((Gate::Allow(btreeset![0, 1, 2]), Gate::Block(btreeset![0, 1, 2])),
                Gate::Block(btreeset![])),
        ];

        for ((l_gate, r_gate), expected) in inputs_and_expected {
            let produced = l_gate.sym_difference(&r_gate);
            assert_eq!(expected, produced);

            // Manually perform the same logic that symmetric difference should provide.
            for slot in 0u8..10 {
                let l_is_allowed = l_gate.allows_slot(slot);
                let r_is_allowed = r_gate.allows_slot(slot);
                let u_is_allowed = produced.allows_slot(slot);

                assert_eq!(l_is_allowed ^ r_is_allowed, u_is_allowed);
            }
        }
    }
}
