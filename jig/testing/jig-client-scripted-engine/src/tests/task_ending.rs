use super::*;

#[test]
fn scripted_task_ending_refusals_keep_whole_owner_facts_after_keyed_durability_and_replay() {
    let ending = OwnerChipEnding::Cancelled {
        reason: DocumentText { bytes: text(UTF8_TEXTS[0]), cut: true },
        result: Some(ResultValue::Approved {
            state: Box::from([255_u8; 32]),
            words: Some(DocumentText { bytes: text(b""), cut: true }),
        }),
        by: OwnerRequester::Deployment { project: u32::MAX },
        at: Wall::from_nanos(u64::MAX),
    };
    let mut setup = script();
    let mut row = task();
    row.result = Some(TaskResult::Ending(Box::new(ending.clone())));
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    let ask = Ask::Cancel { project: 1, task: 8, reason: text(b"cancel"), grace: None };
    let expected = Answer::Refused { refusal: Refusal::Ended { how: Ending::Task(Box::new(ending)) }, at: None };
    let mut engine = Engine::new(setup, 1).unwrap();
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    send(&mut engine, 1, 12, 1, 2, ask.clone());
    assert!(engine.poll(Time::from_nanos(9_999_999)).is_empty());
    let answers = engine.poll(Time::from_nanos(10_000_001));
    assert_eq!(answers.len(), 2);
    assert_eq!(final_answer(&answers[0]), &expected);
    assert_eq!(final_answer(&answers[1]), &expected);
    assert_eq!(engine.commits().len(), 1);
    assert_eq!(engine.commits()[0].ask, Some(ask.clone()));
    send(&mut engine, 20_000_000, 11, 1, 3, ask);
    let replay = engine.poll(Time::from_nanos(20_000_000));
    assert_eq!(replay.len(), 1);
    assert_eq!(final_answer(&replay[0]), &expected);
    assert_eq!(engine.commits().len(), 1);
    let Answer::Refused { refusal, .. } = expected else { panic!("expected refusal") };
    assert_eq!(EntranceRefusal::new(refusal.clone()), Err(refusal));
}

const UTF8_TEXTS: [&[u8]; 1] = [&[b'!'; 32]];
