#![macro_use]

use std::collections::BTreeSet;
use std::fmt;
use std::borrow::Cow;

/// An identifier for a unique variant pathway through a recipe.
pub type Slot = u8;
pub type SlotSet = BTreeSet<Slot>;

macro_rules! allow {
    ( $($slot:expr),* $(,)? ) => (Gate::allow(vec!($($slot),*)));
}

macro_rules! block {
    ( $($slot:expr),* $(,)? ) => (Gate::block(vec!($($slot),*)));
}

/// Represents the type of gate, whether its slots are to be marked as allowed or blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GateType {
    Allow,
    Block,
}

impl GateType {
    pub fn is_allow(&self) -> bool {
        self == &GateType::Allow
    }

    pub fn is_block(&self) -> bool {
        self == &GateType::Block
    }

    pub fn invert(&self) -> Self {
        match self {
            &GateType::Allow => GateType::Block,
            &GateType::Block => GateType::Allow,
        }
    }
}

impl fmt::Display for GateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self.is_allow() {
            true => "ALLOW",
            false => "BLOCK",
        };

        write!(f, "{}", name)
    }
}

/// Represents a filter on a recipe's logical variant pathway, allowing or restricting certain variants from proceeding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Gate(GateType, SlotSet);

impl<'a> From<Gate> for Cow<'a, Gate> {
    fn from(gate: Gate) -> Self {
        Cow::Owned(gate)
    }
}

impl<'a> From<&'a Gate> for Cow<'a, Gate> {
    fn from(gate: &'a Gate) -> Self {
        Cow::Borrowed(gate)
    }
}

impl Gate {
    pub fn new<II: IntoIterator<Item = Slot>>(gate_type: GateType, slots: II) -> Self {
        Gate(gate_type, slots.into_iter().collect())
    }

    pub fn allow<II: IntoIterator<Item = Slot>>(slots: II) -> Self {
        Gate::new(GateType::Allow, slots)
    }

    pub fn block<II: IntoIterator<Item = Slot>>(slots: II) -> Self {
        Gate::new(GateType::Block, slots)
    }

    /// Creates a gate that allows every slot.
    pub fn allow_all() -> Self {
        Self::block(vec![])
    }

    /// Creates a gate that blocks every slot.
    pub fn block_all() -> Self {
        Self::allow(vec![])
    }

    /// Checks if a gate has 'allow' semantics (is a white list).
    pub fn is_allow(&self) -> bool {
        self.0.is_allow()
    }

    /// Checks if a gate has 'block' semantics (is a black list).
    pub fn is_block(&self) -> bool {
        self.0.is_block()
    }

    /// Checks if a gate is 'allow-all', blocking no slots.
    pub fn is_allow_all(&self) -> bool {
        self.is_block() && self.1.is_empty()
    }

    /// Checks if a gate is 'block-all', allowing no slots.
    pub fn is_block_all(&self) -> bool {
        self.is_allow() && self.1.is_empty()
    }

    pub fn slots(&self) -> &SlotSet {
        &self.1
    }

    /// Inverts a gate.
    /// The resulting gate allows any slots blocked by the input gate, and vice versa.
    pub fn invert(&self) -> Self {
        Gate(self.0.invert(), self.1.clone())
    }

    pub fn allows_slot(&self, slot: Slot) -> bool {
        self.1.contains(&slot) == self.is_allow()
    }

    pub fn blocks_slot(&self, slot: Slot) -> bool {
        !self.allows_slot(slot)
    }

    /// Combines two gates using a union operation.
    /// The resulting gate allows any slots allowed by either of the input gates.
    pub fn union(&self, gate: &Self) -> Self {
        let ls: &SlotSet = self.slots();
        let rs: &SlotSet = gate.slots();

        match (self.0, gate.0) {
            (GateType::Allow, GateType::Allow) => Gate::allow(ls.union(rs).cloned()),
            (GateType::Allow, GateType::Block) => Gate::block(rs.difference(ls).cloned()),
            (GateType::Block, GateType::Allow) => Gate::block(ls.difference(rs).cloned()),
            (GateType::Block, GateType::Block) => Gate::block(ls.intersection(rs).cloned()),
        }
    }

    /// Combines two gates using an intersection operation.
    /// The resulting gate allows any slots allowed by both of the input gates.
    pub fn intersection(&self, gate: &Self) -> Self {
        let ls: &SlotSet = self.slots();
        let rs: &SlotSet = gate.slots();

        match (self.0, gate.0) {
            (GateType::Allow, GateType::Allow) => Gate::allow(ls.intersection(rs).cloned()),
            (GateType::Allow, GateType::Block) => Gate::allow(ls.difference(rs).cloned()),
            (GateType::Block, GateType::Allow) => Gate::allow(rs.difference(ls).cloned()),
            (GateType::Block, GateType::Block) => Gate::block(ls.union(rs).cloned()),
        }
    }

    /// Combines two gates using a difference operation.
    /// The resulting gate allows any slots allowed by the first, but not the second, input gate.
    pub fn difference(&self, gate: &Self) -> Self {
        self.intersection(&gate.invert())
    }

    /// Combines two gates using a symmetric difference operation.
    /// The resulting gate allows any slots allowed by exactly one of the input gates.
    pub fn sym_difference(&self, gate: &Self) -> Self {
        let sym_diff_slots = self.slots().symmetric_difference(&gate.slots()).cloned();

        match (self.0, gate.0) {
            (GateType::Allow, GateType::Allow) | (GateType::Block, GateType::Block) => Gate::allow(sym_diff_slots),
            (GateType::Allow, GateType::Block) | (GateType::Block, GateType::Allow) => Gate::block(sym_diff_slots),
        }
    }
}

impl fmt::Display for Gate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({:?})", self.0, self.slots())
    }
}

#[cfg(test)]
mod tests {
    use super::Gate;

    #[test]
    fn test_allow_all() {
        let expected = block!();
        let produced = Gate::allow_all();

        assert_eq!(expected, produced);
    }

    #[test]
    fn test_block_all() {
        let expected = allow!();
        let produced = Gate::block_all();

        assert_eq!(expected, produced);
    }

    #[test]
    fn test_is_allow() {
        let gates_and_expected = vec![
            (allow!(), true),
            (block!(), false),
        ];

        for (gate, expected) in gates_and_expected {
            let produced = gate.is_allow();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_is_block() {
        let gates_and_expected = vec![
            (allow!(), false),
            (block!(), true),
        ];

        for (gate, expected) in gates_and_expected {
            let produced = gate.is_block();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_is_allow_all() {
        let gates_and_expected = vec![
            (allow!(), false),
            (allow!(0, 1, 2), false),
            (block!(), true),
            (block!(0, 1, 2), false),
        ];

        for (gate, expected) in gates_and_expected {
            let produced = gate.is_allow_all();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_is_block_all() {
        let gates_and_expected = vec![
            (allow!(), true),
            (allow!(0, 1, 2), false),
            (block!(), false),
            (block!(0, 1, 2), false),
        ];

        for (gate, expected) in gates_and_expected {
            let produced = gate.is_block_all();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_slots() {
        let gates_and_expected = vec![
            (allow!(), btreeset![]),
            (allow!(0, 1, 2), btreeset![0, 1, 2]),
            (block!(), btreeset![]),
            (block!(0, 1, 2), btreeset![0, 1, 2]),
        ];

        for (gate, expected) in gates_and_expected {
            let produced = gate.slots();
            assert_eq!(&expected, produced);
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
            let input = Gate::allow(slot_set.clone());
            let produced = input.invert();
            let expected = Gate::block(slot_set.clone());
            assert_eq!(expected, produced);

            let input = Gate::block(slot_set.clone());
            let produced = input.invert();
            let expected = Gate::allow(slot_set.clone());
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_allows_slot() {
        let inputs_and_expected = vec![
            ((allow!(0, 1, 2), 1), true),
            ((allow!(0, 1, 2), 3), false),
            ((block!(0, 1, 2), 1), false),
            ((block!(0, 1, 2), 3), true),
            ((allow!(), 0), false),
            ((block!(), 0), true),
        ];

        for ((gate, slot), expected) in inputs_and_expected {
            let produced = gate.allows_slot(slot);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_blocks_slot() {
        let inputs_and_expected = vec![
            ((allow!(0, 1, 2), 1), false),
            ((allow!(0, 1, 2), 3), true),
            ((block!(0, 1, 2), 1), true),
            ((block!(0, 1, 2), 3), false),
            ((allow!(), 0), true),
            ((block!(), 0), false),
        ];

        for ((gate, slot), expected) in inputs_and_expected {
            let produced = gate.blocks_slot(slot);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_union() {
        let inputs_and_expected = vec![
            ((allow!(0, 1, 2), allow!(2, 3, 4)),
                allow!(0, 1, 2, 3, 4)),
            ((allow!(0, 1, 2), block!(2, 3, 4)),
                block!(3, 4)),
            ((block!(0, 1, 2), allow!(2, 3, 4)),
                block!(0, 1)),
            ((block!(0, 1, 2), block!(2, 3, 4)),
                block!(2)),
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
            ((allow!(0, 1, 2), allow!(2, 3, 4)),
                allow!(2)),
            ((allow!(0, 1, 2), block!(2, 3, 4)),
                allow!(0, 1)),
            ((block!(0, 1, 2), allow!(2, 3, 4)),
                allow!(3, 4)),
            ((block!(0, 1, 2), block!(2, 3, 4)),
                block!(0, 1, 2, 3, 4)),
            ((allow!(0, 1, 2), allow!(3, 4, 5)),
                allow!()),
            ((allow!(0, 1, 2), block!(0, 1, 2)),
                allow!()),
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
            ((allow!(0, 1, 2), allow!(2, 3, 4)),
                allow!(0, 1)),
            ((allow!(0, 1, 2), block!(2, 3, 4)),
                allow!(2)),
            ((block!(0, 1, 2), allow!(2, 3, 4)),
                block!(0, 1, 2, 3, 4)),
            ((block!(0, 1, 2), block!(2, 3, 4)),
                allow!(3, 4)),
            ((allow!(0, 1, 2), allow!(3, 4, 5)),
                allow!(0, 1, 2)),
            ((allow!(0, 1, 2), block!(0, 1, 2)),
                allow!(0, 1, 2)),
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
            ((allow!(0, 1, 2), allow!(2, 3, 4)),
                allow!(0, 1, 3, 4)),
            ((allow!(0, 1, 2), block!(2, 3, 4)),
                block!(0, 1, 3, 4)),
            ((block!(0, 1, 2), allow!(2, 3, 4)),
                block!(0, 1, 3, 4)),
            ((block!(0, 1, 2), block!(2, 3, 4)),
                allow!(0, 1, 3, 4)),
            ((allow!(0, 1, 2), allow!(3, 4, 5)),
                allow!(0, 1, 2, 3, 4, 5)),
            ((allow!(0, 1, 2), block!(0, 1, 2)),
                block!()),
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

