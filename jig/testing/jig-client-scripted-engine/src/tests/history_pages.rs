use super::*;
fn semantic() -> HistoryRow {
    let task = task();
    let before = AmendmentState {
        addressed: None,
        taken_by: None,
        instructions: None,
        spec: task.spec,
        authority: task.authority,
        wake: task.wake,
        dependencies: List::with_capacity(1),
        cut: false,
    };
    let mut after = before.clone();
    after.instructions = Some(DocumentText::complete(text(b"")));
    HistoryRow::Change(Box::new(HistoryLine {
        task: 8,
        revision: u64::MAX,
        by: Requester::Deployment { project: u32::MAX },
        change: HistoryChange::Amended,
        reason: DocumentText { bytes: text(b"source reason"), cut: true },
        proposal: Some(ProposalHistory {
            reason: DocumentText::complete(text(b"proposal reason")),
            number: u64::MAX,
            proposer: u64::MAX,
            action: RetainedAction::Release { task: 8 },
            as_holder: true,
            state: ProposalHistoryState::Rejected {
                by: Requester::Person(u64::MAX),
                reason: DocumentText::complete(text(b"independent proposal reason")),
            },
        }),
        amendment: Some((Box::new(before), Box::new(after))),
        at: None,
    }))
}
fn attempt() -> HistoryRow {
    HistoryRow::Attempt(Box::new(AttemptLine {
        task: 8,
        attempt: u64::MAX,
        claimed: None,
        started: Some(Wall::EPOCH),
        ended: Some(Wall::from_nanos(u64::MAX)),
        end: Some(RunEnd::Finished),
        phases: Some(u32::MAX),
    }))
}
fn hold() -> HistoryRow {
    HistoryRow::Hold(Box::new(HoldLine {
        task: 8,
        position: u64::MAX,
        at: Wall::EPOCH,
        before: PriorPhase::Closing(ClosingStage::Run { attempt: u64::MAX }),
        why: HoldCategory::Budget,
        source: Some(Part { connector: u16::MAX, kind: PartKind::Resource, key: PartKey::Number(u64::MAX) }),
        ending: Some(ChipEnding::Done(ResultValue::OwnerVerdict {
            verdict: u32::MAX,
            words: DocumentText { bytes: text(b""), cut: true },
            fields: List::with_capacity(1),
        })),
    }))
}

#[test]
fn scripted_history_pages_keep_change_attempt_and_hold_rows_under_opaque_source_continuations() {
    let values = vec![semantic(), attempt(), hold()];
    let mut setup = live_script();
    setup.bounds.rows = 1;
    setup.truth.sources.push(Source {
        party: REQUESTER,
        read: Read::History { task: 8, subtree: false, after: None },
        rows: Rows::History(values.clone()),
    });
    let mut engine = Engine::new(setup, 209).expect("valid script");
    let mut cursor = None;
    for (index, expected) in values.iter().enumerate() {
        let now = Time::from_nanos(10_000_000);
        engine
            .read(
                now,
                REQUESTER_SIGN_IN,
                Token::new(u64::try_from(index).expect("fixture index fits u64") + 20),
                Read::History { task: 8, subtree: false, after: cursor },
            )
            .expect("typed read fits");
        let out = engine.poll(now);
        let Up::Read { terminal: ReadResult::Value(ReadValue::History(page)), .. } = &out[0] else {
            panic!("whole supplied history page");
        };
        assert_eq!(page.rows.as_slice(), core::slice::from_ref(expected));
        assert_eq!(page.before.is_some(), index != 0);
        assert_eq!(page.after.is_some(), index.saturating_add(1) < values.len());
        assert!(!page.cut);
        cursor = page.after.clone();
    }
}
