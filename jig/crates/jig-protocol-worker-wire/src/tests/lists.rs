//! Account list boundaries (jig/docs/design/protocol/channel/records.md, 1).
use super::fixtures::repeated;
use crate::{self as wire, v11};
use skein_lib::{Reader, Writer};

// Channel 5.1's fixed bounds, independent of adjustable codec limits.
const ROWS: u32 = 256;
const NAME: u32 = 128;
const SHOWN: u32 = 64;
const EFFORTS: u32 = 8;
const EFFORT: u32 = 16;
// Read8 + outcome tag1 + count4; each full row is 375 bytes.
const FULL_BODY: u32 = 96013;

fn list_bytes(rows: u32, name: u32, shown: u32, efforts: u32, effort: u32) -> Box<[u8]> {
    // Lengths, three option tags, window/output, and the effort count add23 per row.
    let effort_bytes = efforts.checked_mul(4_u32.checked_add(effort).expect("effort prefix")).expect("efforts");
    let row = 23_u32
        .checked_add(name)
        .expect("name prefix")
        .checked_add(shown)
        .expect("shown prefix")
        .checked_add(effort_bytes)
        .expect("row size");
    let length = 13_u32.checked_add(rows.checked_mul(row).expect("row bytes")).expect("list size");
    let mut bytes = Writer::new(usize::try_from(length).expect("bounded fixture"));
    bytes.put(&u64::MAX.to_be_bytes()).expect("read");
    bytes.put(&[0]).expect("listed tag");
    bytes.put(&rows.to_be_bytes()).expect("row count");
    for _ in 0..rows {
        bytes.put(&name.to_be_bytes()).expect("name length");
        bytes.put(&repeated(b'n', name)).expect("name");
        bytes.put(&[1]).expect("shown present");
        bytes.put(&shown.to_be_bytes()).expect("shown length");
        bytes.put(&repeated(b's', shown)).expect("shown");
        for bound in [u32::MAX, u32::MAX] {
            bytes.put(&[1]).expect("token bound present");
            bytes.put(&bound.to_be_bytes()).expect("token bound");
        }
        bytes.put(&efforts.to_be_bytes()).expect("effort count");
        for _ in 0..efforts {
            bytes.put(&effort.to_be_bytes()).expect("effort length");
            bytes.put(&repeated(b'e', effort)).expect("effort");
        }
    }
    bytes.finish()
}

#[test]
fn list_rows_at_each_bound_round_trip_and_one_past_is_refused() {
    for (rows, name, shown, efforts, effort, problem) in [
        (0, 0, 0, 0, 0, None),
        (ROWS, NAME, SHOWN, EFFORTS, EFFORT, None),
        (ROWS + 1, NAME, SHOWN, EFFORTS, EFFORT, Some(v11::Path::ListedRowsRows)),
        (1, NAME + 1, SHOWN, EFFORTS, EFFORT, Some(v11::Path::ListRowName)),
        (1, NAME, SHOWN + 1, EFFORTS, EFFORT, Some(v11::Path::ListRowShown)),
        (1, NAME, SHOWN, EFFORTS + 1, EFFORT, Some(v11::Path::ListRowEfforts)),
        (1, NAME, SHOWN, EFFORTS, EFFORT + 1, Some(v11::Path::ListRowEfforts)),
    ] {
        let bytes = list_bytes(rows, name, shown, efforts, effort);
        let result = wire::ListRead::decode(&wire::CEILINGS, &mut Reader::new(&bytes));
        match problem {
            Some(path) => assert_eq!(
                result.expect_err("one past bound"),
                v11::Problem { path, reason: skein_codec::Reason::Bound }
            ),
            None => {
                let record = result.expect("within bounds");
                let mut writer = Writer::new(usize::try_from(record.measure()).expect("bounded record"));
                record.encode(&mut writer).expect("encoded");
                assert_eq!(writer.finish(), bytes);
            }
        }
    }
    assert_eq!(wire::ListRead::encoded_bound(&wire::CEILINGS), Some(u64::from(FULL_BODY)));
}
