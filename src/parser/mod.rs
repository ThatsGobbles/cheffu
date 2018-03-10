use std::str::FromStr;
use std::num::ParseIntError;

mod parser_funcs {
    use nom;

    use token::TToken;

    const INGREDIENT_SIGIL: char = '*';
    const MODIFIER_SIGIL: char = ',';
    const ANNOTATION_SIGIL: char = ';';
    const ACTION_SIGIL: char = '=';
    const COMBINATION_SIGIL: char = '/';

    named!(pub integer_repr<&str, &str>,
        recognize!(nom::digit)
    );

    named!(pub nz_integer_repr<&str, &str>,
        verify!(integer_repr, |ds: &str| !ds.chars().all(|c| c == '0'))
    );

    named!(pub decimal_repr<&str, &str>,
        recognize!(complete!(tuple!(
            integer_repr,
            tag!("."),
            integer_repr
        )))
    );

    named!(pub nz_decimal_repr<&str, &str>,
        recognize!(alt!(
            complete!(tuple!(
                nz_integer_repr,
                tag!("."),
                integer_repr
            ))
            | complete!(tuple!(
                integer_repr,
                tag!("."),
                nz_integer_repr
            ))
        ))
    );

    named!(pub rational_repr<&str, &str>,
        recognize!(complete!(tuple!(
            integer_repr,
            tag!("/"),
            nz_integer_repr
        )))
    );

    named!(pub nz_rational_repr<&str, &str>,
        recognize!(complete!(tuple!(
            nz_integer_repr,
            tag!("/"),
            nz_integer_repr
        )))
    );

    named!(pub phrase<&str, &str>,
        // A sequence of whitespace separated alphanumerics.
        ws!(recognize!(separated_nonempty_list_complete!(nom::space, nom::alphanumeric)))
    );

    named!(pub ingredient_token<&str, TToken>,
        ws!(do_parse!(
            char!(INGREDIENT_SIGIL) >>
            value: phrase >>
            (TToken::Ingredient(value.to_string()))
        ))
    );

    named!(pub modifier_token<&str, TToken>,
        ws!(do_parse!(
            char!(MODIFIER_SIGIL) >>
            value: phrase >>
            (TToken::Modifier(value.to_string()))
        ))
    );

    named!(pub annotation_token<&str, TToken>,
        ws!(do_parse!(
            char!(ANNOTATION_SIGIL) >>
            value: phrase >>
            (TToken::Annotation(value.to_string()))
        ))
    );
}

#[cfg(test)]
mod tests {
    use super::parser_funcs;

    use nom::{IResult, ErrorKind};

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
            let produced = parser_funcs::integer_repr(input);
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
            let produced = parser_funcs::nz_integer_repr(input);
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
            let produced = parser_funcs::decimal_repr(input);
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
            let produced = parser_funcs::nz_decimal_repr(input);
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
            let produced = parser_funcs::rational_repr(input);
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
            let produced = parser_funcs::nz_rational_repr(input);
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
            let produced = parser_funcs::phrase(input);
            assert_eq!(expected, produced);
        }
    }
}
