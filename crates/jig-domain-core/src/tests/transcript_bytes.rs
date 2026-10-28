use super::*;

/// A preparation counts at its allowance until its load is in, then at
/// what it keeps, and at nothing once it ends.
#[test]
fn transcript_bytes_follow_a_preparation_from_allowance_to_zero() {
    const TASK: u64 = 1;
    const ATTEMPT: u64 = 1;
    const FIRST_TURN: u32 = 1;
    const TURNS: [(u32, &[u8], &[u8]); 2] = [
        (FIRST_TURN, b"first opaque turn", b"first turn's text"),
        (FIRST_TURN + 1, b"second opaque turn", b"second turn's longer text"),
    ];
    let (mut core, limits) = core();
    make_task(&mut core, &limits, TASK);
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(TASK)), task: TASK });
    core.contexts.insert(TASK, core.tasks.run_context(TASK).expect("preparing task")).unwrap();
    core.begin_transcript(TASK, ATTEMPT);
    core.transcripts.get_mut(&TASK).expect("begun transcript").window = Some((ATTEMPT, FIRST_TURN));

    let allowance = u64::from(core.settings.resume_bytes) + u64::from(core.settings.tail_bytes);
    assert_eq!(core.transcript_bytes(), allowance);
    let mut retained = 0_u64;
    for (number, body, text) in TURNS {
        assert!(core.append_transcript(
            TASK,
            TurnRecord {
                task: TASK,
                attempt: ATTEMPT,
                turn: number,
                spent: 0,
                read: None,
                opens_window: number == FIRST_TURN,
                at: Wall::EPOCH,
                transcript: body.into(),
            },
            Some(text.into()),
        ));
        retained += u64::try_from(body.len() + text.len()).expect("fixture byte lengths fit u64");
        assert_eq!(core.transcript_bytes(), allowance, "partial loads keep their allowance");
    }
    let _requests = watch_outputs(&mut core, &limits, Event::StartBrief { task: TASK });
    assert_eq!(core.transcript_bytes(), retained);

    let _requests = watch_outputs(&mut core, &limits, Event::PreparationFailed { task: TASK });
    assert_eq!(core.transcript_bytes(), 0);
}
