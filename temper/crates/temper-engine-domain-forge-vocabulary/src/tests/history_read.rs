use crate::history::Cursor;

#[test]
fn history_cursor_is_exactly_version_one_and_two_big_endian_numbers() {
    let cursor = Cursor { upper: 0x0102_0304_0506_0708, after: 0x1112_1314_1516_1718 };
    let bytes = [1, 1, 2, 3, 4, 5, 6, 7, 8, 17, 18, 19, 20, 21, 22, 23, 24];
    assert_eq!(cursor.encode().as_ref(), bytes);
    assert_eq!(Cursor::decode(&bytes), Some(cursor));
    for length in 0..17 {
        assert_eq!(Cursor::decode(&bytes[..length]), None);
    }
    let mut bad = [0; 18];
    for (target, byte) in bad.iter_mut().zip(bytes) {
        *target = byte;
    }
    assert_eq!(Cursor::decode(&bad), None);
    for version in [0, 2, 255] {
        let mut bad = bytes;
        bad[0] = version;
        assert_eq!(Cursor::decode(&bad), None);
    }
}
