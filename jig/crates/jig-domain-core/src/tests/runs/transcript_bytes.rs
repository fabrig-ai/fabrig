use crate::tests::*;
use skein_lib::length::len_u64;

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
    core.begin_transcript(TASK, ATTEMPT, None);
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
        retained += len_u64(body.len() + text.len());
        assert_eq!(core.transcript_bytes(), allowance, "partial loads keep their allowance");
    }
    let load = fixture_window(&mut core, TASK, (ATTEMPT, FIRST_TURN));
    let _requests = watch_store_outputs(
        &mut core,
        &limits,
        engine_store::In::Loaded {
            load,
            result: engine_store::Answer::Rows(engine_store::Rows::Turns { turns: Box::new([]), more: false }),
        },
    );
    assert_eq!(core.transcript_bytes(), retained);

    let stale = watch_store_outputs(
        &mut core,
        &limits,
        engine_store::In::Loaded { load, result: engine_store::Answer::Failed(engine_store::Failure::Failed) },
    );
    assert!(stale.is_empty(), "a late terminal cannot fail the brief");
    assert_eq!(core.transcript_bytes(), retained);
    let _requests = watch_outputs(
        &mut core,
        &limits,
        Event::BriefAssembled {
            task: TASK,
            failure: Some(brief::GatherFailure {
                fate: brief::GatherFate::Transient,
                section: b"fixture",
                words: b"brief stopped".as_slice().into(),
            }),
        },
    );
    assert_eq!(core.transcript_bytes(), 0);
}
