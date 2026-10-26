use super::fixtures::{DIRECTORIES, content, facts};
use crate::{Content, Directory, files};

#[test]
fn the_digest_is_the_secrets() {
    for (row, secret, digits, digest, demo_key, key_digits) in [
        (
            "zero",
            [0; 32],
            b"0000000000000000000000000000000000000000000000000000000000000000\n",
            b"60e05bd1b195af2f94112fa7197a5c88289058840ce7c6df9693756bc6250f55\n",
            [0xff; 32],
            b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\n",
        ),
        (
            "ascending",
            [
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
                28, 29, 30, 31,
            ],
            b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\n",
            b"6c86c6aac5fb24bcf5d9939cb7d7d5645ce39418f449e03b262dd4fa14b4b92b\n",
            [0xab; 32],
            b"abababababababababababababababababababababababababababababababab\n",
        ),
    ] {
        let answered = files(&facts(), &DIRECTORIES, &secret, &demo_key).expect("valid facts");
        assert_eq!(content(&answered, b"worker.secret"), digits, "{row}");
        assert_eq!(content(&answered, b"worker.digest"), digest, "{row}");
        assert_eq!(content(&answered, b"demo/llm.key"), key_digits, "{row}");
    }
}

#[test]
fn the_files_are_named_in_order() {
    let answered = files(&facts(), &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("valid facts");
    let expected = [
        ("secret", b"worker.secret".as_slice(), Directory::Data, Some(true)),
        ("digest", b"worker.digest".as_slice(), Directory::Data, Some(true)),
        ("demo key", b"demo/llm.key".as_slice(), Directory::Data, Some(true)),
        ("demo git", b"demo/bin/git".as_slice(), Directory::Data, None),
        ("agent", b"agent.json".as_slice(), Directory::Config, Some(false)),
        ("document last", b"temper.json".as_slice(), Directory::Config, Some(false)),
    ];
    assert_eq!(answered.files.len(), expected.len());
    for (actual, (row, name, directory, private)) in answered.files.iter().zip(expected) {
        assert_eq!((actual.name, actual.directory), (name, directory), "{row}");
        match &actual.content {
            Content::Bytes { private: actual, .. } => assert_eq!(Some(*actual), private, "{row}"),
            Content::Link { target } => {
                assert_eq!(private, None, "{row}");
                assert_eq!(target.as_ref(), facts().binary, "{row}");
            }
        }
    }
}
