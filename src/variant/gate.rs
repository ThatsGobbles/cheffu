use std::collections::HashSet;

/// An identifier for a unique variant pathway through a recipe.
pub type Slot = u8;

/// Represents a filter on a recipe's logical variant pathway, allowing or restricting certain variants from proceeding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    Whitelist(HashSet<Slot>),
    Blacklist(HashSet<Slot>),
}

impl Gate {
    pub fn invert(&self) -> Gate {
        match *self {
            Gate::Whitelist(ref s) => Gate::Blacklist(s.clone()),
            Gate::Blacklist(ref s) => Gate::Whitelist(s.clone()),
        }
    }

    pub fn allows_slot(&self, slot: Slot) -> bool {
        match *self {
            Gate::Whitelist(ref s) => s.contains(&slot),
            Gate::Blacklist(ref s) => !s.contains(&slot),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Gate;

    #[test]
    fn test_invert() {
        let slot_sets = vec![
            hashset![0, 1, 2],
            hashset![],
            hashset![27],
        ];

        for slot_set in slot_sets {
            let input = Gate::Whitelist(slot_set.clone());
            let produced = input.invert();
            let expected = Gate::Blacklist(slot_set.clone());
            assert_eq!(expected, produced);

            let input = Gate::Blacklist(slot_set.clone());
            let produced = input.invert();
            let expected = Gate::Whitelist(slot_set.clone());
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_allows_slot() {
        let inputs_and_expected = vec![
            ((Gate::Whitelist(hashset![0, 1, 2]), 1), true),
            ((Gate::Whitelist(hashset![0, 1, 2]), 3), false),
            ((Gate::Blacklist(hashset![0, 1, 2]), 1), false),
            ((Gate::Blacklist(hashset![0, 1, 2]), 3), true),
            ((Gate::Whitelist(hashset![]), 0), false),
            ((Gate::Blacklist(hashset![]), 0), true),
        ];

        for ((gate, slot), expected) in inputs_and_expected {
            let produced = gate.allows_slot(slot);
            assert_eq!(expected, produced);
        }
    }
}
