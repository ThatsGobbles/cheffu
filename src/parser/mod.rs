use std::str::FromStr;
use std::num::ParseIntError;

use nom;
use failure::Error;

use token::Token;
use parallel::flow::{Flow, FlowItem, Split, SplitSet};
use parallel::gate::{Gate, Slot};

const INGREDIENT_SIGIL: char = '*';
const MODIFIER_SIGIL: char = ',';
const ANNOTATION_SIGIL: char = ';';
const ACTION_SIGIL: char = '=';
const COMBINATION_SIGIL: char = '/';

const VAR_SPLIT_START_SIGIL: char = '[';
const VAR_SPLIT_CLOSE_SIGIL: char = ']';
const VAR_SPLIT_SEP_SIGIL: char = '|';
const VAR_SPLIT_TAG_SIGIL: char = '#';
const VAR_SPLIT_SLOT_SEP_SIGIL: char = ',';
const VAR_SPLIT_INV_SLOT_FLAG_SIGIL: char = '!';

pub struct Parsers;

impl Parsers {

    /** Primitive types **/

    named!(pub integer_repr<&str, &str>,
        recognize!(nom::digit)
    );

    named!(pub nz_integer_repr<&str, &str>,
        verify!(Self::integer_repr, |ds: &str| !ds.chars().all(|c| c == '0'))
    );

    named!(pub decimal_repr<&str, &str>,
        recognize!(complete!(tuple!(
            call!(Self::integer_repr),
            tag!("."),
            call!(Self::integer_repr)
        )))
    );

    named!(pub nz_decimal_repr<&str, &str>,
        recognize!(alt!(
            complete!(tuple!(
                call!(Self::nz_integer_repr),
                tag!("."),
                call!(Self::integer_repr)
            ))
            | complete!(tuple!(
                call!(Self::integer_repr),
                tag!("."),
                call!(Self::nz_integer_repr)
            ))
        ))
    );

    named!(pub rational_repr<&str, &str>,
        recognize!(complete!(tuple!(
            call!(Self::integer_repr),
            tag!("/"),
            call!(Self::nz_integer_repr)
        )))
    );

    named!(pub nz_rational_repr<&str, &str>,
        recognize!(complete!(tuple!(
            call!(Self::nz_integer_repr),
            tag!("/"),
            call!(Self::nz_integer_repr)
        )))
    );

    named!(pub phrase<&str, &str>,
        // A sequence of whitespace-separated alphanumerics.
        ws!(recognize!(separated_nonempty_list_complete!(nom::space, nom::alphanumeric)))
    );

    /** Tokens **/

    named!(pub ingredient_token<&str, Token>,
        ws!(do_parse!(
            char!(INGREDIENT_SIGIL) >>
            value: call!(Self::phrase) >>
            (Token::Ingredient(value.to_string()))
        ))
    );

    named!(pub action_token<&str, Token>,
        ws!(do_parse!(
            char!(ACTION_SIGIL) >>
            value: call!(Self::phrase) >>
            (Token::Action(value.to_string()))
        ))
    );

    named!(pub combination_token<&str, Token>,
        ws!(do_parse!(
            char!(COMBINATION_SIGIL) >>
            value: call!(Self::phrase) >>
            (Token::Combination(value.to_string()))
        ))
    );

    named!(pub modifier_token<&str, Token>,
        ws!(do_parse!(
            char!(MODIFIER_SIGIL) >>
            value: call!(Self::phrase) >>
            (Token::Modifier(value.to_string()))
        ))
    );

    named!(pub annotation_token<&str, Token>,
        ws!(do_parse!(
            char!(ANNOTATION_SIGIL) >>
            value: call!(Self::phrase) >>
            (Token::Annotation(value.to_string()))
        ))
    );

    /** Token sequences **/

    named!(pub token<&str, Token>,
        alt!(
            call!(Self::ingredient_token)
            | call!(Self::action_token)
            | call!(Self::combination_token)
            | call!(Self::modifier_token)
            | call!(Self::annotation_token)
        )
    );

    /** Gates **/

    named!(pub slot<&str, Slot>,
        map_res!(nom::digit, Slot::from_str)
    );

    named!(pub gate<&str, Gate>,
        do_parse!(
            char!(VAR_SPLIT_TAG_SIGIL) >>
            inv_flag: map!(opt!(char!(VAR_SPLIT_INV_SLOT_FLAG_SIGIL)), |o| o.is_some()) >>
            slots: separated_nonempty_list_complete!(char!(VAR_SPLIT_SLOT_SEP_SIGIL), call!(Self::slot)) >>
            (match inv_flag {
                true => Gate::block(slots),
                false => Gate::allow(slots),
            })
        )
    );

    /** Flows **/

    named!(pub flow_item<&str, FlowItem>,
        alt!(
            do_parse!(
                token_val: call!(Self::token) >>
                (FlowItem::Token(token_val))
            )
            | do_parse!(
                // TODO: This needs obvious fixing.
                split_val: call!(Self::split_set) >>
                (FlowItem::Split(SplitSet::new(btreeset![])))
            )
        )
    );

    named!(pub flow<&str, Flow>,
        do_parse!(
            flow_items: many0!(call!(Self::flow_item)) >>
            (Flow::new(flow_items))
        )
    );

    named!(pub split<&str, Split>,
        do_parse!(
            flow: call!(Self::flow) >>
            gate: map!(opt!(call!(Self::gate)), |g| g.unwrap_or(block!())) >>
            (Split::new(flow, gate))
        )
    );

    named!(pub split_set<&str, SplitSet>,
        // A bracketed sequence of pipe-separated variants.
        ws!(delimited!(
            char!(VAR_SPLIT_START_SIGIL),
            // call!(Self::phrase),
            do_parse!(
                splits: separated_nonempty_list_complete!(char!(VAR_SPLIT_SEP_SIGIL), call!(Self::split)) >>
                (SplitSet::new(splits))
            ),
            char!(VAR_SPLIT_CLOSE_SIGIL)
        ))
        // ws!(recognize!(separated_nonempty_list_complete!(nom::space, nom::alphanumeric)))
    );
}

#[cfg(test)]
mod tests {
    use super::Parsers;

    use nom::{IResult, ErrorKind};

    use token::Token;

    #[test]
    fn test_integer_repr() {
        let inputs_and_expected = vec![
            ("1234", IResult::Done("", "1234")),
            (" 1234", IResult::Error(ErrorKind::Digit)),
            ("1234 ", IResult::Done(" ", "1234")),
            ("1", IResult::Done("", "1")),
            ("010", IResult::Done("", "010")),
            ("0", IResult::Done("", "0")),
            ("0000", IResult::Done("", "0000")),
            ("0123", IResult::Done("", "0123")),
            ("+1234", IResult::Error(ErrorKind::Digit)),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::integer_repr(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_nz_integer_repr() {
        let inputs_and_expected = vec![
            ("1234", IResult::Done("", "1234")),
            (" 1234", IResult::Error(ErrorKind::Digit)),
            ("1234 ", IResult::Done(" ", "1234")),
            ("1", IResult::Done("", "1")),
            ("010", IResult::Done("", "010")),
            ("0", IResult::Error(ErrorKind::Verify)),
            ("0000", IResult::Error(ErrorKind::Verify)),
            ("0123", IResult::Done("", "0123")),
            ("+1234", IResult::Error(ErrorKind::Digit)),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::nz_integer_repr(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_decimal_repr() {
        let inputs_and_expected = vec![
            ("1234.0", IResult::Done("", "1234.0")),
            ("0.1234", IResult::Done("", "0.1234")),
            ("010.010", IResult::Done("", "010.010")),
            (".1234", IResult::Error(ErrorKind::Digit)),
            ("1234.", IResult::Error(ErrorKind::Complete)),
            ("0.0", IResult::Done("", "0.0")),
            ("0.000", IResult::Done("", "0.000")),
            ("000.000", IResult::Done("", "000.000")),
            (".0", IResult::Error(ErrorKind::Digit)),
            ("0.", IResult::Error(ErrorKind::Complete)),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::decimal_repr(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_nz_decimal_repr() {
        let inputs_and_expected = vec![
            ("1234.0", IResult::Done("", "1234.0")),
            ("0.1234", IResult::Done("", "0.1234")),
            ("010.010", IResult::Done("", "010.010")),
            (".1234", IResult::Error(ErrorKind::Alt)),
            ("1234.", IResult::Error(ErrorKind::Alt)),
            ("0.0", IResult::Error(ErrorKind::Alt)),
            ("0.000", IResult::Error(ErrorKind::Alt)),
            ("000.000", IResult::Error(ErrorKind::Alt)),
            (".0", IResult::Error(ErrorKind::Alt)),
            ("0.", IResult::Error(ErrorKind::Alt)),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::nz_decimal_repr(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_rational_repr() {
        let inputs_and_expected = vec![
            ("1/2", IResult::Done("", "1/2")),
            ("3/2", IResult::Done("", "3/2")),
            ("0/1", IResult::Done("", "0/1")),
            ("000/010", IResult::Done("", "000/010")),
            ("1 /2", IResult::Error(ErrorKind::Tag)),
            (" 1/2", IResult::Error(ErrorKind::Digit)),
            ("1/ 2", IResult::Error(ErrorKind::Digit)),
            ("1", IResult::Error(ErrorKind::Complete)),
            ("1/0", IResult::Error(ErrorKind::Verify)),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::rational_repr(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_nz_rational_repr() {
        let inputs_and_expected = vec![
            ("1/2", IResult::Done("", "1/2")),
            ("3/2", IResult::Done("", "3/2")),
            ("0/1", IResult::Error(ErrorKind::Verify)),
            ("000/010", IResult::Error(ErrorKind::Verify)),
            ("1 /2", IResult::Error(ErrorKind::Tag)),
            (" 1/2", IResult::Error(ErrorKind::Digit)),
            ("1/ 2", IResult::Error(ErrorKind::Digit)),
            ("1", IResult::Error(ErrorKind::Complete)),
            ("1/0", IResult::Error(ErrorKind::Verify)),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::nz_rational_repr(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_phrase() {
        let inputs_and_expected = vec![
            ("apple", IResult::Done("", "apple")),
            (" banana", IResult::Done("", "banana")),
            (" coffee ", IResult::Done("", "coffee")),
            (" apple cinnamon ", IResult::Done("", "apple cinnamon")),
            ("apple 007", IResult::Done("", "apple 007")),
            ("apple/007", IResult::Done("/007", "apple")),
            (" a ", IResult::Done("", "a")),
            (" 7 ", IResult::Done("", "7")),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::phrase(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_ingredient_token() {
        let inputs_and_expected = vec![
            ("* apple", IResult::Done("", Token::Ingredient("apple".to_string()))),
            ("* apple      fritters", IResult::Done("", Token::Ingredient("apple      fritters".to_string()))),
            ("*apple", IResult::Done("", Token::Ingredient("apple".to_string()))),
            (" *apple", IResult::Done("", Token::Ingredient("apple".to_string()))),
            ("* apple, Granny Smith", IResult::Done(", Granny Smith", Token::Ingredient("apple".to_string()))),
            ("apple", IResult::Error(ErrorKind::Char)),
            ("* !!!!", IResult::Error(ErrorKind::AlphaNumeric)),
            ("* apple!!!!", IResult::Done("!!!!", Token::Ingredient("apple".to_string()))),
            ("* apple !!!!", IResult::Done("!!!!", Token::Ingredient("apple".to_string()))),
            ("* APPLE !!!!", IResult::Done("!!!!", Token::Ingredient("APPLE".to_string()))),
            ("* APPLE   007 !!!!", IResult::Done("!!!!", Token::Ingredient("APPLE   007".to_string()))),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::ingredient_token(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_action_token() {
        let inputs_and_expected = vec![
            ("= saute", IResult::Done("", Token::Action("saute".to_string()))),
            ("= saute      in", IResult::Done("", Token::Action("saute      in".to_string()))),
            ("=saute", IResult::Done("", Token::Action("saute".to_string()))),
            (" =saute", IResult::Done("", Token::Action("saute".to_string()))),
            ("= saute, over high heat", IResult::Done(", over high heat", Token::Action("saute".to_string()))),
            ("saute", IResult::Error(ErrorKind::Char)),
            ("= !!!!", IResult::Error(ErrorKind::AlphaNumeric)),
            ("= saute!!!!", IResult::Done("!!!!", Token::Action("saute".to_string()))),
            ("= saute !!!!", IResult::Done("!!!!", Token::Action("saute".to_string()))),
            ("= SAUTE !!!!", IResult::Done("!!!!", Token::Action("SAUTE".to_string()))),
            ("= SAUTE   007 !!!!", IResult::Done("!!!!", Token::Action("SAUTE   007".to_string()))),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::action_token(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_combination_token() {
        let inputs_and_expected = vec![
            ("/ mix", IResult::Done("", Token::Combination("mix".to_string()))),
            ("/ mix      together", IResult::Done("", Token::Combination("mix      together".to_string()))),
            ("/mix", IResult::Done("", Token::Combination("mix".to_string()))),
            (" /mix", IResult::Done("", Token::Combination("mix".to_string()))),
            ("/ mix, over high heat", IResult::Done(", over high heat", Token::Combination("mix".to_string()))),
            ("mix", IResult::Error(ErrorKind::Char)),
            ("/ !!!!", IResult::Error(ErrorKind::AlphaNumeric)),
            ("/ mix!!!!", IResult::Done("!!!!", Token::Combination("mix".to_string()))),
            ("/ mix !!!!", IResult::Done("!!!!", Token::Combination("mix".to_string()))),
            ("/ MIX !!!!", IResult::Done("!!!!", Token::Combination("MIX".to_string()))),
            ("/ MIX   007 !!!!", IResult::Done("!!!!", Token::Combination("MIX   007".to_string()))),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::combination_token(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_modifier_token() {
        let inputs_and_expected = vec![
            (", large", IResult::Done("", Token::Modifier("large".to_string()))),
            (", large      green", IResult::Done("", Token::Modifier("large      green".to_string()))),
            (",large", IResult::Done("", Token::Modifier("large".to_string()))),
            (" ,large", IResult::Done("", Token::Modifier("large".to_string()))),
            (", large, over high heat", IResult::Done(", over high heat", Token::Modifier("large".to_string()))),
            ("large", IResult::Error(ErrorKind::Char)),
            (", !!!!", IResult::Error(ErrorKind::AlphaNumeric)),
            (", large!!!!", IResult::Done("!!!!", Token::Modifier("large".to_string()))),
            (", large !!!!", IResult::Done("!!!!", Token::Modifier("large".to_string()))),
            (", LARGE !!!!", IResult::Done("!!!!", Token::Modifier("LARGE".to_string()))),
            (", LARGE   007 !!!!", IResult::Done("!!!!", Token::Modifier("LARGE   007".to_string()))),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::modifier_token(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_annotation_token() {
        let inputs_and_expected = vec![
            ("; gently", IResult::Done("", Token::Annotation("gently".to_string()))),
            ("; gently      together", IResult::Done("", Token::Annotation("gently      together".to_string()))),
            (";gently", IResult::Done("", Token::Annotation("gently".to_string()))),
            (" ;gently", IResult::Done("", Token::Annotation("gently".to_string()))),
            ("; gently, over high heat", IResult::Done(", over high heat", Token::Annotation("gently".to_string()))),
            ("gently", IResult::Error(ErrorKind::Char)),
            ("; !!!!", IResult::Error(ErrorKind::AlphaNumeric)),
            ("; gently!!!!", IResult::Done("!!!!", Token::Annotation("gently".to_string()))),
            ("; gently !!!!", IResult::Done("!!!!", Token::Annotation("gently".to_string()))),
            ("; GENTLY !!!!", IResult::Done("!!!!", Token::Annotation("GENTLY".to_string()))),
            ("; GENTLY   007 !!!!", IResult::Done("!!!!", Token::Annotation("GENTLY   007".to_string()))),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Parsers::annotation_token(input);
            assert_eq!(expected, produced);
        }
    }
}
