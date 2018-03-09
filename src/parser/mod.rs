use std::str::FromStr;
use std::num::ParseIntError;

use nom;

const INGR_SIGIL: char = '*';
const ACTN_SIGIL: char = '=';
const COMB_SIGIL: char = '/';

named!(pub integer_repr<&str, &str>,
    recognize!(complete!(nom::digit))
);

named!(pub nz_integer_repr<&str, &str>,
    verify!(integer_repr, |ds: &str| !ds.chars().all(|c| c == '0'))
);

named!(pub decimal_repr<&str, &str>,
    recognize!(tuple!(
        integer_repr,
        tag!("."),
        integer_repr
    ))
);

named!(pub nz_decimal_repr<&str, &str>,
    recognize!(alt!(
        tuple!(
            nz_integer_repr,
            opt!(complete!(
                tuple!(
                    tag!("."),
                    integer_repr
                )
            ))
        )
        | tuple!(
            integer_repr,
            opt!(complete!(
                tuple!(
                    tag!("."),
                    nz_integer_repr
                )
            ))
        )
    ))
);

#[cfg(test)]
mod tests {
    use super::{integer_repr, nz_integer_repr, decimal_repr};

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
            let produced = integer_repr(input);
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
            let produced = nz_integer_repr(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_decimal_repr() {
        let inputs_and_expected = vec![
            ("1234.0", IResult::Done("", "1234.0")),
            ("0.1234", IResult::Done("", "0.1234")),
            (".1234", IResult::Error(ErrorKind::Digit)),
            ("1234.", IResult::Error(ErrorKind::Complete)),
            ("0.0", IResult::Done("", "0.0")),
            (".0", IResult::Error(ErrorKind::Digit)),
            ("0.", IResult::Error(ErrorKind::Complete)),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = decimal_repr(input);
            assert_eq!(expected, produced);
        }
    }
}
