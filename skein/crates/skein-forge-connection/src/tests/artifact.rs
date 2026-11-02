//! A compressed artifact at the shared answer bound fits its component ledger.
use super::{fixtures as f, github_files::CallHeap};
use alloc::boxed::Box;
use skein_forge::{
    api::{Answer, Op, Read},
    forgejo::{document as d, server},
};
use skein_lib::Duration;
const RUN: u64 = 7;
const ARTIFACT: u64 = 11;
const REQUESTS: u32 = 3; // Listing, run, complete ZIP.
const PATH: &[u8] = b"failed.txt";
const RUN_BODY: &[u8] = br#"{"id":7,"index_in_repo":7,"commit_sha":"0000000000000000000000000000000000000000"}"#;
fn warm() -> f::Harness {
    let mut h = f::known();
    h.call(1, f::REPO, Op::Read(Read::Branch { branch: Box::from(f::BRANCH) }));
    h.answer(f::FORGE, &f::ok(f::BRANCH_BODY));
    h.run(Duration::ZERO);
    drop(h.answers.remove(&1));
    h
}
#[test]
#[expect(clippy::print_stdout, clippy::disallowed_macros, reason = "report the artifact's measured component peak")]
fn an_answer_bound_inflated_artifact_fits_the_owned_heap() {
    // Peak 133840 bytes, plus a quarter, rounded up to two significant digits.
    const CEILING: u64 = 170_000;
    let bound = f::shared().answer_bytes;
    assert_eq!(bound, INFLATED);
    let archive = ARCHIVE;
    let listing = server::encode(
        &server::Answer::Artifacts(Box::new([d::Artifact {
            id: ARTIFACT,
            name: Box::from(*b"tests"),
            run_id: RUN,
            size_in_bytes: u64::from(bound),
        }])),
        &crate::limits::documents(&f::limits()),
    )
    .expect("listing fits");
    let mut h = warm();
    h.component_heap = Some(CallHeap { peak: 0, held: 0 });
    h.call(1, f::REPO, Op::Read(Read::Artifact { run: RUN, name: Box::from(*b"tests"), max_bytes: bound }));
    for body in [&listing, RUN_BODY, archive] {
        let response = f::ok(body);
        h.answer(f::FORGE, &response);
        h.run(Duration::ZERO);
    }
    let heap = h.component_heap.take().expect("component metering");
    let answer = h.answered(1).expect("artifact answer");
    let kept = usize::try_from(bound).expect("answer cap").checked_sub(PATH.len()).expect("path fits");
    assert_eq!(
        answer.result,
        Ok(Answer::File { head: f::BRANCH_HEAD, path: Box::from(PATH), bytes: f::filled(kept, b'x'), truncated: true })
    );
    assert!(u64::try_from(heap.peak).expect("peak") <= CEILING, "{} <= {CEILING}", heap.peak);
    println!("artifact component peak={} ceiling={CEILING}", heap.peak);
    assert_eq!(answer.cost, REQUESTS);
}

// Python zlib, one deflated file filling the shared answer bound.
const INFLATED: u32 = 65_536;
const ARCHIVE: &[u8] = &[
    80, 75, 3, 4, 20, 0, 0, 0, 8, 0, 0, 0, 33, 80, 154, 88, 180, 44, 78, 0, 0, 0, 0, 0, 1, 0, 10, 0, 0, 0, 102, 97,
    105, 108, 101, 100, 46, 116, 120, 116, 237, 193, 1, 1, 0, 0, 0, 128, 144, 219, 205, 239, 8, 10, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 106, 80, 75, 1, 2, 20, 3, 20, 0, 0, 0, 8, 0, 0, 0, 33, 80,
    154, 88, 180, 44, 78, 0, 0, 0, 0, 0, 1, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 1, 0, 0, 0, 0, 102, 97, 105,
    108, 101, 100, 46, 116, 120, 116, 80, 75, 5, 6, 0, 0, 0, 0, 1, 0, 1, 0, 56, 0, 0, 0, 118, 0, 0, 0, 0, 0,
];
