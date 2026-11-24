use super::{Extraction, extract_with};
use skein_lib::sha256::Sha256;
use std::io;

#[test]
fn extraction_rows() {
    const PIECE: usize = 16_384;
    const BOUND: u64 = 32_768;
    let output = vec![b'x'; 2 * PIECE + 1];
    let at_bound = &output[..2 * PIECE];
    let digest = || {
        let mut hash = Sha256::new();
        hash.update(at_bound);
        Extraction::Digest(hash.finish())
    };
    for (row, stdout, clean_exit, bound, expected, written, effects) in [
        ("binary digest at the bound", at_bound, true, BOUND, digest(), 2 * PIECE, ["wait"].as_slice()),
        ("one byte past the bound", &output[..], true, BOUND, Extraction::Bytes, 2 * PIECE, &["kill", "wait"]),
        ("extraction bound", at_bound, true, BOUND - 1, Extraction::Bytes, PIECE, &["kill", "wait"]),
        ("missing member", b"", false, BOUND, Extraction::Failed, 0, &["wait"]),
        ("empty member", b"", true, BOUND, Extraction::Failed, 0, &["wait"]),
        ("corrupt archive", b"", false, BOUND, Extraction::Failed, 0, &["wait"]),
        ("failed exit after stdout", at_bound, false, BOUND, Extraction::Failed, 2 * PIECE, &["wait"]),
    ] {
        // A cursor gives the copier each piece at the size of its next read.
        let mut stdout = io::Cursor::new(stdout);
        let mut result = Vec::new();
        let mut child = Vec::new();
        let verdict = extract_with(
            &mut stdout,
            &mut result,
            bound,
            &mut child,
            |child| {
                child.push("kill");
                Ok(())
            },
            |child| {
                child.push("wait");
                Ok(clean_exit)
            },
        );
        assert_eq!(verdict, expected, "{row}");
        assert_eq!(result, &output[..written], "{row}: only admitted bytes were written");
        assert_eq!(child, effects, "{row}: child teardown");
    }
}
