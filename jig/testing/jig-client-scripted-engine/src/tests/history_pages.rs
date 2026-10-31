use super::*;
fn semantic() -> HistoryRow {
    let task = task();
    let before = OwnerAmendmentState {
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
    HistoryRow::Change(Box::new(OwnerHistoryLine {
        task: 8,
        revision: u64::MAX,
        by: OwnerRequester::Deployment { project: u32::MAX },
        change: OwnerHistoryChange::Amended,
        reason: DocumentText { bytes: text(b"source reason"), cut: true },
        proposal: Some(OwnerProposalHistory {
            number: u64::MAX,
            proposer: u64::MAX,
            action: OwnerRetainedAction::Release { task: 8 },
            as_holder: true,
            state: OwnerProposalHistoryState::Rejected {
                by: OwnerRequester::Person(u64::MAX),
                reason: DocumentText::complete(text(b"independent proposal reason")),
            },
        }),
        amendment: Some((Box::new(before), Box::new(after))),
        at: None,
    }))
}
fn milestone() -> HistoryRow {
    HistoryRow::Milestone(Box::new(OwnerMilestoneLine {
        task: 8,
        position: u64::MAX,
        phase: OwnerMilestonePhase::Closing {
            stage: ClosingStage::Run { attempt: u64::MAX },
            ending: OwnerChipEnding::Done(ResultValue::OwnerVerdict {
                verdict: u32::MAX,
                words: DocumentText { bytes: text(b""), cut: true },
                fields: List::with_capacity(1),
            }),
        },
        at: Some(Wall::EPOCH),
    }))
}

#[test]
fn scripted_history_pages_keep_change_and_milestone_rows_under_opaque_source_continuations() {
    let values = vec![semantic(), milestone()];
    let mut setup = live_script();
    setup.bounds.rows = 1;
    setup.truth.sources.push(Source {
        party: 1,
        read: Read::History { task: 8, subtree: false, after: None },
        rows: Rows::History(values.clone()),
    });
    let mut engine = Engine::new(setup, 209).unwrap();
    let mut cursor = None;
    for (index, expected) in values.iter().enumerate() {
        let now = Time::from_nanos(10_000_000);
        engine
            .read(
                now,
                11,
                Token::new(u64::try_from(index).unwrap() + 20),
                Read::History { task: 8, subtree: false, after: cursor },
            )
            .unwrap();
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
