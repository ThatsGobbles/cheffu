use std::collections::HashSet;

/// Represents a filter on a recipe's logical variant pathway, allowing or restricting certain variants from proceeding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    Whitelist(HashSet<u8>),
    Blacklist(HashSet<u8>),
}

impl Gate {
    pub fn invert(&self) -> Gate {
        match *self {
            Gate::Whitelist(ref s) => Gate::Blacklist(s.clone()),
            Gate::Blacklist(ref s) => Gate::Whitelist(s.clone()),
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
}
