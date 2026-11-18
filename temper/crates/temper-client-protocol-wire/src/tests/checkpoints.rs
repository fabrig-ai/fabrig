//! The checkpoint act's framing and both terminal forms (temper/docs/design/protocol/checkpoints.md, 3).
use crate::v1 as w;
use skein_lib::Reader;

#[test]
fn checkpoints_carry_the_request_and_both_terminal_forms_at_current_version() {
    let request = w::Requesting::new(&w::CEILINGS, w::RequestingParts { request: w::Request::Checkpoint }).unwrap();
    let request_bytes = [0, 35, 26];
    assert_eq!(crate::requesting(&request).as_ref(), request_bytes);
    assert_eq!(crate::decode_requesting(&request_bytes, &w::CEILINGS), Some(request));
    let checkpoint =
        w::Checkpointed::decode(&w::CEILINGS, &mut Reader::new(super::golden("record_checkpointed_smallest.bin")))
            .unwrap();
    let success = w::Outcome::Checkpointed(checkpoint);
    let golden = super::golden("enum_outcome_checkpointed_smallest.bin");
    let mut writer = skein_lib::Writer::new(golden.len().checked_add(2).unwrap());
    writer.put(&[0, 35]).unwrap();
    writer.put(golden).unwrap();
    let success_bytes = writer.finish();
    let refused = w::Outcome::CheckpointRefused(
        w::CheckpointRefused::new(&w::CEILINGS, w::CheckpointRefusedParts { why: w::CheckpointWhy::NotOwner }).unwrap(),
    );
    for (outcome, bytes) in [(success, success_bytes), (refused, Box::from([0, 35, 41, 0]))] {
        let answer = w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap();
        assert_eq!(crate::answer(&answer).as_ref(), bytes.as_ref());
        let mut reader = Reader::new(&bytes);
        assert_eq!(w::Answer::decode(&w::CEILINGS, &mut reader), Ok(answer));
        assert!(reader.is_empty());
    }
    assert_eq!(crate::decode_requesting(&[0, 34, 26], &w::CEILINGS), None, "the previous client layout is not bridged");
}
