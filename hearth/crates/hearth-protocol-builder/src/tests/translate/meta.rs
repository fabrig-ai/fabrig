use crate::render::diff;

const ERROR_SIDES: &[Option<diff::Side>] = &[None, Some(diff::Side::From), Some(diff::Side::To)];
#[test]
fn an_errors_side_crosses_to_records() {
    for (row, &side) in ERROR_SIDES.iter().enumerate() {
        let mut error = crate::tests::fixtures::decode_error();
        error.side = side;
        assert_eq!(crate::translate::errors::error(error).side, side, "row {row}: error side");
    }
}
