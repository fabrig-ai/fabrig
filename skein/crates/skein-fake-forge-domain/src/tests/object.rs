use crate::{commit_bytes, commit_count};

#[test]
fn commit_bytes_keep_the_sha1_spelling_and_zero_padding() {
    for (count, expected) in [
        (
            0,
            [
                0xe2, 0x20, 0xa8, 0x39, 0x7b, 0x1d, 0xcd, 0xaf, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ],
        ),
        (
            1,
            [
                0x91, 0x0a, 0x2d, 0xec, 0x89, 0x02, 0x5c, 0xc1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ],
        ),
        (
            2,
            [
                0x97, 0x58, 0x35, 0xde, 0x1c, 0x97, 0x56, 0xce, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ],
        ),
        (
            u64::MAX,
            [
                0xe4, 0xd9, 0x71, 0x77, 0x1b, 0x65, 0x2c, 0x20, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        ),
    ] {
        assert_eq!(commit_bytes(count), expected, "count {count}");
    }
}

#[test]
fn commit_count_only_reads_the_fake_spelling() {
    assert_eq!(commit_count(&[0; 32]), None, "zero bytes name no commit");
    for count in [0, 1, 2, u64::MAX] {
        let bytes = commit_bytes(count);
        assert_eq!(commit_count(&bytes), Some(count), "count {count}");
        for at in 0..32 {
            let mut changed = bytes;
            *changed.get_mut(at).expect("commit byte") ^= 1;
            assert_eq!(commit_count(&changed), None, "count {count}, byte {at}");
        }
    }
}
