//! The checkpoint act's framing and both terminal forms (temper/docs/design/protocol/checkpoints.md, 3).
use crate::v1 as w;
use crate::v1::Record;
use skein_lib::Reader;

#[test]
fn checkpoints_carry_the_request_and_both_terminal_forms_at_current_version() {
<<<<<<< HEAD
    let request = w::Requesting::new(&w::CEILINGS, w::RequestingParts { request: w::Request::Checkpoint }).unwrap();
    let [high, low] = w::VERSION.to_be_bytes();
    let request_bytes = [high, low, 26];
=======
    let request = w::Requesting { request: w::Request::Checkpoint };
    let request_bytes = [0, 39, 26];
>>>>>>> refs/hand6/10247
    assert_eq!(crate::requesting(&request).as_ref(), request_bytes);
    assert_eq!(crate::decode_requesting(&request_bytes, &w::CEILINGS), Some(request));
    let checkpoint =
        w::Checkpointed::decode(&w::CEILINGS, &mut Reader::new(super::golden("record_checkpointed_smallest.bin")))
            .unwrap();
    let success = w::Outcome::Checkpointed(checkpoint);
    let golden = super::golden("enum_outcome_checkpointed_smallest.bin");
    let mut writer = skein_lib::Writer::new(golden.len().checked_add(2).unwrap());
    writer.put(&[high, low]).unwrap();
    writer.put(golden).unwrap();
    let success_bytes = writer.finish();
<<<<<<< HEAD
    let refused = w::Outcome::CheckpointRefused(
        w::CheckpointRefused::new(&w::CEILINGS, w::CheckpointRefusedParts { why: w::CheckpointWhy::NotOwner }).unwrap(),
    );
    for (outcome, bytes) in [(success, success_bytes), (refused, Box::from([high, low, 41, 0]))] {
        let answer = w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap();
=======
    let refused = w::Outcome::CheckpointRefused(w::CheckpointRefused { why: w::CheckpointWhy::NotOwner });
    for (outcome, bytes) in [(success, success_bytes), (refused, Box::from([0, 39, 41, 0]))] {
        let answer = w::Answer { outcome };
>>>>>>> refs/hand6/10247
        assert_eq!(crate::answer(&answer).as_ref(), bytes.as_ref());
        let mut reader = Reader::new(&bytes);
        assert_eq!(w::Answer::decode(&w::CEILINGS, &mut reader), Ok(answer));
        assert!(reader.is_empty());
    }
    let [high, low] = w::VERSION.checked_sub(1).unwrap().to_be_bytes();
    assert_eq!(
        crate::decode_requesting(&[high, low, 26], &w::CEILINGS),
        None,
        "the previous client layout is not bridged"
    );
}
