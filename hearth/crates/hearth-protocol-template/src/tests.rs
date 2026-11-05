mod fixtures;
mod interpolation;
mod jumps;
mod literals;
mod lookups;
mod operators;
mod statements;
mod templates;
mod whitespace;

#[test]
fn the_decoder_bounds_cover_each_entry_points_workspace() {
    use crate::{
        Instruction, Operation, expression,
        limits::{self, Decoder, Limits},
    };
    const EXPRESSION: u32 = 17;
    const TEMPLATE: usize = 43;
    const STRING: usize = 29;
    const STATEMENTS: usize = 3;
    const TEMPLATE_BYTES: usize = 2 * (3 * TEMPLATE + 1) * size_of::<Instruction>()
        + TEMPLATE * size_of::<Operation>()
        + TEMPLATE
        + 2 * (TEMPLATE + 1) * size_of::<u32>()
        + STATEMENTS * size_of::<crate::template::statement::Block>()
        + TEMPLATE * size_of::<u32>();
    const STRING_BYTES: usize =
        2 * (3 * STRING + 1) * size_of::<Instruction>() + STRING * size_of::<Operation>() + STRING;
    let limits = Limits {
        expression: EXPRESSION,
        nesting: 5,
        template: u32::try_from(TEMPLATE).expect("small template"),
        statements: u32::try_from(STATEMENTS).expect("small statement stack"),
    };
    let expression = expression::decoder_bound(EXPRESSION).expect("small expression");
    for (decoder, bytes) in [
        (Decoder::Expression, 0),
        (Decoder::Template, TEMPLATE_BYTES),
        (Decoder::Interpolated { length: u32::try_from(STRING).expect("small string") }, STRING_BYTES),
    ] {
        let expected = expression.checked_add(u64::try_from(bytes).expect("small workspace")).expect("small bound");
        assert_eq!(limits::decoder_bound(decoder, &limits), Some(expected), "{decoder:?}");
    }
}

#[test]
fn the_decoder_bounds_keep_checked_capacity_arithmetic() {
    use crate::{
        Instruction, Operation, expression,
        limits::{self, Decoder, Limits},
    };
    const BYTES_PER_BYTE: usize = 2 * size_of::<Instruction>() + size_of::<Operation>() + 1;
    let limits = Limits { expression: u32::MAX, template: u32::MAX, statements: u32::MAX, nesting: u32::MAX };
    let expression = expression::decoder_bound(u32::MAX).expect("u32-sized expression workspace fits u64");
    let compiled = u64::from(u32::MAX)
        .checked_mul(u64::try_from(BYTES_PER_BYTE).expect("item sizes fit u64"))
        .expect("u32-sized instruction workspace fits u64")
        .checked_add(expression)
        .expect("u32-sized decoder workspace fits u64");
    for (decoder, expected) in [
        (Decoder::Expression, Some(expression)),
        (Decoder::Interpolated { length: u32::MAX }, Some(compiled)),
        // A template's line table needs template + 1 entries, which overflows u32.
        (Decoder::Template, None),
    ] {
        assert_eq!(limits::decoder_bound(decoder, &limits), expected, "{decoder:?}");
    }
}
