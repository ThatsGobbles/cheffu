// use failure::Error;

// use super::gate::{Slot, Gate};
// use token::Token;

// #[derive(Debug, Fail, PartialEq, Eq)]
// pub enum GateStackError {
//     #[fail(display = "stack is empty")]
//     Empty,

//     #[fail(display = "top of stack does not match; expected: {}, produced: {}", expected, produced)]
//     Mismatch {
//         expected: Gate,
//         produced: Gate,
//     },

//     #[fail(display = "leftover items in stack; found: {:?}", leftover)]
//     Leftover {
//         leftover: Vec<Gate>,
//     },
// }

// #[derive(Debug, Fail, PartialEq, Eq)]
// pub enum SlotError {
//     // TODO: Make error message more clear.
//     #[fail(display = "not enough slot choices provided")]
//     Insufficient,

//     #[fail(display = "expected slot not allowed by gate; gate: {}, slot: {}", gate, slot)]
//     Mismatch {
//         gate: Gate,
//         slot: Slot,
//     },

//     // TODO: Make error message more clear.
//     #[fail(display = "too many slot choices provided")]
//     Leftover,
// }

// /// Represents an item in a start-to-finish walk through a procedure graph.
// #[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
// pub enum WalkItem<'a> {
//     Token(&'a Token),
//     Push(&'a Gate),
//     Pop(&'a Gate),
// }

// /// Represents a start-to-finish walk through a procedure graph.
// pub struct WalkItemSeq<'a>(Vec<WalkItem<'a>>);

// impl<'a> WalkItemSeq<'a> {
//     pub fn process<II>(&self, slot_iter: II) -> Result<Vec<&Token>, Error>
//     where II: IntoIterator<Item = Slot>,
//     {
//         let mut gate_stack: Vec<&Gate> = vec![];
//         let mut tokens: Vec<&Token> = vec![];

//         let mut slot_iter = slot_iter.into_iter();

//         for walk_item in &self.0 {
//             // LEARN: In here, `walk_item` is a reference.
//             match walk_item {
//                 &WalkItem::Token(token) => {
//                     tokens.push(token);
//                 },
//                 &WalkItem::Push(gate) => {
//                     // Get the next expected slot.
//                     // let next_slot = slot_iter.next().ok_or(SlotError::Insufficient)?;

//                     gate_stack.push(gate);
//                 },
//                 &WalkItem::Pop(gate) => {
//                     let popped: &Gate = gate_stack.pop().ok_or(GateStackError::Empty)?;

//                     // We expect that the top of the stack should match our expected close gate.
//                     ensure!(gate == popped, GateStackError::Mismatch{expected: gate.clone(), produced: popped.clone()});
//                 },
//             }
//         }

//         // LEARN: `.cloned()` calls `.clone()` on each element of an iterator.
//         ensure!(gate_stack.is_empty(), GateStackError::Leftover{leftover: gate_stack.into_iter().cloned().collect()});

//         Ok(tokens)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::{WalkItem, WalkItemSeq};
//     use super::super::gate::Gate;
//     use token::Token;

//     #[test]
//     fn test_process() {
//         let token = Token;
//         let gate_a = Gate::Allow(btreeset![0, 1, 2]);
//         let gate_b = Gate::Allow(btreeset![3, 4, 5]);

//         let inputs_and_expected = vec![
//             (WalkItemSeq(vec![]), Some(vec![])),
//             (WalkItemSeq(vec![WalkItem::Token(&token)]), Some(vec![&token])),
//             (WalkItemSeq(vec![
//                 WalkItem::Token(&token),
//                 WalkItem::Token(&token),
//                 WalkItem::Token(&token),
//             ]), Some(vec![&token, &token, &token])),
//             (WalkItemSeq(vec![
//                 WalkItem::Push(&gate_a),
//                 WalkItem::Token(&token),
//                 WalkItem::Pop(&gate_a),
//             ]), Some(vec![&token])),
//             (WalkItemSeq(vec![
//                 WalkItem::Push(&gate_a),
//                 WalkItem::Pop(&gate_a),
//             ]), Some(vec![])),
//             (WalkItemSeq(vec![
//                 WalkItem::Push(&gate_a),
//             ]), None),
//             (WalkItemSeq(vec![
//                 WalkItem::Pop(&gate_a),
//             ]), None),
//             (WalkItemSeq(vec![
//                 WalkItem::Push(&gate_a),
//                 WalkItem::Pop(&gate_b),
//             ]), None),
//             (WalkItemSeq(vec![
//                 WalkItem::Pop(&gate_a),
//                 WalkItem::Push(&gate_a),
//             ]), None),
//             (WalkItemSeq(vec![
//                 WalkItem::Push(&gate_a),
//                 WalkItem::Push(&gate_b),
//                 WalkItem::Pop(&gate_a),
//             ]), None),
//             (WalkItemSeq(vec![
//                 WalkItem::Push(&gate_a),
//                 WalkItem::Push(&gate_b),
//                 WalkItem::Pop(&gate_a),
//                 WalkItem::Pop(&gate_b),
//             ]), None),
//         ];

//         for (input, expected) in inputs_and_expected {
//             let produced = input.process(vec![]).ok();

//             assert_eq!(expected, produced);
//         }
//     }
// }
