mod batch;
mod fixtures;
mod invocation;
mod listing;
mod rev_parse;

mod hunks;

mod tail;

#[test]
fn each_decoder_bound_is_its_heap_sum_or_none_on_overflow() {
    use crate::{
        batch, hunks,
        limits::{Decoder, Limits, decoder_bound},
        listing,
    };
    use core::mem::size_of;

    const SMALL: Limits = Limits {
        tail_lines: 2,
        tail_line: 7,
        entries: 3,
        path: 11,
        paths_total: 23,
        chunk: 17,
        hunk_lines: 5,
        hunk_file: 31,
    };
    // Retained items and text, plus each decoder's delivery buffers.
    const TAIL: usize = 2 * size_of::<Box<[u8]>>() + 14 + 14;
    const LISTING: usize = 3 * size_of::<listing::Entry>() + 23 + 100 + 11;
    const DIFF: usize = 5 * size_of::<hunks::Line>() + 93 + 17;
    const LARGE_CHUNK: u32 = 127;
    let rows = [
        (Decoder::Tail, SMALL, Some(u64::try_from(TAIL).expect("small tail heap"))),
        (Decoder::Listing, SMALL, Some(u64::try_from(LISTING).expect("small listing heap"))),
        (Decoder::Diff, SMALL, Some(u64::try_from(DIFF).expect("small diff heap"))),
        (Decoder::Batch, SMALL, Some(u64::from(batch::HEADER))),
        (Decoder::Batch, Limits { chunk: LARGE_CHUNK, ..SMALL }, Some(u64::from(LARGE_CHUNK))),
        (Decoder::Tail, Limits { tail_lines: u32::MAX, tail_line: u32::MAX, ..SMALL }, None),
    ];
    for (decoder, limits, expected) in rows {
        assert_eq!(decoder_bound(decoder, &limits), expected, "{decoder:?}: {limits:?}");
    }
}
