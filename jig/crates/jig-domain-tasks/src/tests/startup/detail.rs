//! Stored rows exercise the selected fit diagnostics beside admission controls (engine.md, 6).
use super::*;
use crate::{RestoreBound as B, RestoreCause as C, RestoreFailure, RestoreLlmWhy as Why};

fn empty(limits: &crate::Limits, selected: bool) -> Harness {
    let mut cold = Harness::new();
    cold.env.limits = *limits;
    cold.domain = Domain::new(limits, 91, 1, crate::tests::charters(), Box::new([0]));
    cold.stored.clear();
    if selected {
        cold.domain.start_restore();
    }
    assert_eq!(cold.domain.restore_failure(), None);
    cold
}

fn row() -> Stored {
    one_task().stored[&Key::Live(DUE)].clone()
}

fn live(row: &mut Stored) -> &mut crate::TaskRecord {
    let Stored::Live(row) = row else { panic!("live fixture") };
    row
}

fn bound(which: B, held: u64, limit: u64) -> C {
    C::Bound { which, held, limit }
}

fn refused(cold: &mut Harness, row: Stored, cause: C) {
    let key = row.key();
    refusal(&cold.send(Event::Restore { record: row }), Refusal::Restore);
    assert_eq!(cold.domain.restore_failure(), Some(RestoreFailure { key, cause }));
    assert!(!cold.domain.ready());
}

#[test]
fn exact_spec_bytes_fit_and_one_over_keeps_the_first_outer_key() {
    let mut exact = row();
    live(&mut exact).spec.words = vec![1; 64].into_boxed_slice();
    let mut cold = empty(&LIMITS, true);
    assert!(cold.send(Event::Restore { record: exact.clone() }).is_empty());
    assert_eq!(cold.domain.restore_failure(), None);
    live(&mut exact).number = 73;
    live(&mut exact).spec.words = vec![1; 65].into_boxed_slice();
    refused(&mut cold, exact, bound(B::SpecBytes, 65, 64));
    let first = cold.domain.restore_failure();
    let mut other = row();
    live(&mut other).numbers.spent = u64::MAX;
    live(&mut other).numbers.spent_below = 1;
    assert!(cold.send(Event::Restore { record: other }).is_empty());
    assert!(cold.send(Event::Check).is_empty());
    assert!(cold.send(Event::Restored).is_empty());
    assert_eq!(cold.domain.restore_failure(), first);
    assert!(!cold.domain.checked());
}

#[test]
fn nested_proposal_bytes_keep_person_proposal_key() {
    let mut member = task(71, &[]);
    member.spec.words = vec![1; 65].into_boxed_slice();
    refused(
        &mut empty(&LIMITS, true),
        Stored::PersonProposal(Box::new(crate::PersonProposal {
            number: 81,
            proposer: PERSON,
            project: 1,
            action: crate::PersonAction::Batch(Box::new([member])),
            reason: Box::new([]),
            state: crate::PersonProposalState::Pending { since: skein_lib::Wall::EPOCH },
        })),
        bound(B::SpecBytes, 65, 64),
    );
}

fn name(number: u64) -> Name {
    Name { connector: 1, path: Box::new([number.to_le_bytes().as_slice().into()]) }
}

#[test]
fn writer_pool_and_ledger_occupancy_preserve_incoming_keys() {
    let mut cold = empty(&LIMITS, true);
    let capacity = cold.domain.writers.capacity();
    for number in 1..=u64::from(capacity) {
        assert!(
            cold.send(Event::Restore {
                record: Stored::Writer(WriterSlot {
                    number,
                    resource: name(number),
                    writer: Writer::Effect { entry: number },
                    lost: false,
                })
            })
            .is_empty()
        );
    }
    refused(
        &mut cold,
        Stored::Writer(WriterSlot {
            number: 900,
            resource: name(900),
            writer: Writer::Effect { entry: 900 },
            lost: false,
        }),
        bound(B::Writers, u64::from(capacity) + 1, u64::from(capacity)),
    );

    let mut cold = empty(&LIMITS, true);
    let capacity = cold.domain.pools.capacity();
    for number in 1..=u64::from(capacity) {
        assert!(
            cold.send(Event::Restore {
                record: Stored::Pool(crate::PoolSlots { number, pool: name(number), slots: 0 })
            })
            .is_empty()
        );
    }
    refused(
        &mut cold,
        Stored::Pool(crate::PoolSlots { number: 901, pool: name(901), slots: 0 }),
        bound(B::Pools, u64::from(capacity) + 1, u64::from(capacity)),
    );

    let mut cold = empty(&LIMITS, true);
    let capacity = cold.domain.funding.capacity();
    for period in 1..=u64::from(capacity) {
        assert!(cold.send(Event::Restore { record: ledger(period) }).is_empty());
    }
    refused(&mut cold, ledger(902), bound(B::Funders, u64::from(capacity) + 1, u64::from(capacity)));
}

fn ledger(period: u64) -> Stored {
    Stored::Ledger(crate::FundingRecord {
        funder: crate::Funder::Period { project: 1, period },
        period: None,
        parent: None,
        numbers: crate::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
        made: 0,
        closed: false,
    })
}

#[test]
fn writer_and_pool_nested_names_keep_row_keys() {
    let mut resource = name(1);
    resource.path = Box::new([vec![0; 65].into_boxed_slice()]);
    refused(
        &mut empty(&LIMITS, true),
        Stored::Writer(WriterSlot {
            number: 42,
            resource: resource.clone(),
            writer: Writer::Effect { entry: 1 },
            lost: false,
        }),
        bound(B::HoldBytes, 65, 64),
    );
    refused(
        &mut empty(&LIMITS, true),
        Stored::Pool(crate::PoolSlots { number: 43, pool: resource, slots: 0 }),
        bound(B::HoldBytes, 65, 64),
    );
}

#[test]
fn task_and_ledger_numbers_refuse_only_unrepresentable_sums() {
    for reserved in [false, true] {
        let mut record = ledger(17);
        let Stored::Ledger(ref mut ledger) = record else { panic!("ledger") };
        ledger.numbers.spent = u64::MAX;
        if reserved {
            ledger.numbers.reserved = 1;
        } else {
            ledger.numbers.spent_below = 1;
        }
        refused(&mut empty(&LIMITS, true), record, C::Accounting);
        let mut record = row();
        live(&mut record).numbers.spent = u64::MAX;
        if reserved {
            live(&mut record).numbers.reserved = 1;
        } else {
            live(&mut record).numbers.spent_below = 1;
        }
        refused(&mut empty(&LIMITS, true), record, C::Accounting);
    }
    let mut record = row();
    live(&mut record).numbers.budget = 0;
    live(&mut record).numbers.spent = 1;
    assert!(empty(&LIMITS, true).send(Event::Restore { record }).is_empty(), "no budget rule");
}

#[test]
fn eventual_posting_overflow_names_the_destination_ledger() {
    let mut source = one_task();
    let Some(Stored::Live(task)) = source.stored.get_mut(&Key::Live(DUE)) else { panic!("task") };
    task.numbers.spent = 1;
    let funder = task.funder;
    let Some(Stored::Ledger(ledger)) = source.stored.get_mut(&Key::Ledger(funder)) else { panic!("ledger") };
    ledger.numbers.budget = u64::MAX;
    ledger.numbers.reserved = 0;
    ledger.numbers.spent_below = u64::MAX;
    let mut cold = cold_load_fit_link(&source);
    refusal(&cold.send(Event::Check), Refusal::Restore);
    assert_eq!(cold.domain.restore_failure(), Some(RestoreFailure { key: Key::Ledger(funder), cause: C::Accounting }));
    assert!(cold.send(Event::Restored).is_empty());
}

#[test]
fn project_aggregate_names_the_inspected_live_row() {
    let mut source = Harness::new();
    source.make(vec![task(10, &[]), task(20, &[])]);
    for (limits, cause) in [
        (crate::Limits { project_tasks: 1, ..LIMITS }, bound(B::ProjectTasks, 2, 1)),
        (crate::Limits { project_goals: 1, ..LIMITS }, bound(B::ProjectGoals, 2, 1)),
    ] {
        let mut cold = empty(&limits, true);
        for mut record in source.stored.values().filter(|row| row.key().live()).cloned() {
            if let Stored::Live(row) = &mut record {
                row.tracked = Some(1);
            }
            assert!(cold.send(Event::Restore { record }).is_empty());
        }
        refusal(&cold.send(Event::Check), Refusal::Restore);
        assert_eq!(cold.domain.restore_failure(), Some(RestoreFailure { key: Key::Live(10), cause }));
    }
}

fn hold(number: u64, owner: crate::Owner) -> Stored {
    Stored::PlacedHold(Box::new(crate::PlacedHold {
        number,
        project: 1,
        owner,
        mode: crate::Mode::Stop,
        scope: crate::Scope::Project,
        reason: Box::new([]),
        placed_at: 0,
        revision: 1,
        exempt: Box::new([]),
        lifted: false,
    }))
}

#[test]
fn placed_aggregate_names_the_inspected_hold_and_applies_party_cases() {
    for (limits, cause) in [
        (crate::Limits { project_placed_holds: 1, ..LIMITS }, bound(B::ProjectPlacedHolds, 2, 1)),
        (crate::Limits { party_placed_holds: 1, ..LIMITS }, bound(B::PartyPlacedHolds, 2, 1)),
    ] {
        let mut cold = empty(&limits, true);
        for number in [31, 32] {
            assert!(cold.send(Event::Restore { record: hold(number, crate::Owner::Party { person: 9 }) }).is_empty());
        }
        refusal(&cold.send(Event::Check), Refusal::Restore);
        assert_eq!(cold.domain.restore_failure(), Some(RestoreFailure { key: Key::PlacedHold(31), cause }));
    }
}

#[test]
fn configured_choice_failure_preserves_main_or_subagent_and_reason() {
    for (chosen, why, efforts, models) in [
        (crate::ModelEffort { model: 7, effort: Some(3) }, Why::Unconfigured, vec![0], vec![0]),
        (crate::ModelEffort { model: 1, effort: None }, Why::Charter, vec![0, 0], vec![0]),
        (crate::ModelEffort { model: 0, effort: Some(2) }, Why::Effort, vec![2], vec![0]),
        (crate::ModelEffort { model: 0, effort: None }, Why::Effort, vec![2], vec![0]),
    ] {
        for subagent in [false, true] {
            let mut source = one_task();
            let Some(Stored::Live(row)) = source.stored.get_mut(&Key::Live(DUE)) else { panic!("row") };
            row.llm = Some(crate::LlmChoice {
                account: None,
                main: if subagent {
                    crate::ModelEffort { model: 0, effort: if efforts[0] == 0 { None } else { Some(0) } }
                } else {
                    chosen
                },
                subagents: if subagent { Box::new([chosen]) } else { Box::new([]) },
            });
            let mut cold = empty(&LIMITS, true);
            cold.domain.efforts = efforts.clone().into_boxed_slice();
            cold.domain.charters = Box::new([crate::CharterModels {
                number: 1,
                revision: 1,
                models: Some(models.clone().into_boxed_slice()),
            }]);
            for record in source.stored.values().filter(|row| row.key().live()) {
                assert!(cold.send(Event::Restore { record: record.clone() }).is_empty());
            }
            refusal(&cold.send(Event::Check), Refusal::Restore);
            assert_eq!(
                cold.domain.restore_failure(),
                Some(RestoreFailure { key: Key::Live(DUE), cause: C::LlmChoice { chosen, why: why.clone() } })
            );
        }
    }
}

#[test]
fn nonagent_choice_is_kept_only_by_selected_restore() {
    for executor in
        [crate::Executor::Procedure { connector: 1, code: 1 }, crate::Executor::Person(crate::PersonAddress::Person(9))]
    {
        let mut source = one_task();
        let Some(Stored::Live(row)) = source.stored.get_mut(&Key::Live(DUE)) else { panic!("row") };
        row.executor = executor;
        row.llm = Some(crate::LlmChoice {
            account: None,
            main: crate::ModelEffort { model: 7, effort: Some(9) },
            subagents: Box::new([]),
        });
        let mut selected = cold_load_fit_link(&source);
        assert!(selected.send(Event::Check).is_empty());
        assert!(selected.domain.checked());
        assert_eq!(selected.domain.restore_failure(), None);
        let mut admission = cold(&source);
        admission.send(Event::Check);
        assert!(!admission.domain.checked());
        assert_eq!(admission.domain.restore_failure(), None);
    }
}

#[test]
fn success_and_admission_refusals_have_no_detail() {
    let source = one_task();
    let mut selected = cold_load_fit_link(&source);
    assert!(selected.send(Event::Check).is_empty());
    selected.send(Event::Restored);
    assert!(selected.domain.ready());
    assert_eq!(selected.domain.restore_failure(), None);
    let mut bad = row();
    live(&mut bad).spec.words = vec![0; 65].into_boxed_slice();
    let mut admission = empty(&LIMITS, false);
    refusal(&admission.send(Event::Restore { record: bad }), Refusal::Restore);
    assert_eq!(admission.domain.restore_failure(), None);
}

#[test]
#[should_panic(expected = "duplicate live index")]
fn duplicate_live_is_an_assertion() {
    let mut cold = empty(&LIMITS, true);
    cold.send(Event::Restore { record: row() });
    cold.send(Event::Restore { record: row() });
}

#[test]
#[should_panic(expected = "missing charter 99")]
fn missing_charter_is_a_required_link() {
    let mut source = one_task();
    let Some(Stored::Live(row)) = source.stored.get_mut(&Key::Live(DUE)) else { panic!("row") };
    row.executor = crate::Executor::Agent { charter: 99 };
    cold_load_fit_link(&source).send(Event::Check);
}

#[test]
fn live_stub_proposal_and_revision_capacity_include_the_incoming_row() {
    let mut cold = empty(&LIMITS, true);
    for number in 1..=u64::from(LIMITS.tasks) {
        let mut record = row();
        live(&mut record).number = number;
        assert!(cold.send(Event::Restore { record }).is_empty());
    }
    let mut record = row();
    live(&mut record).number = 300;
    refused(&mut cold, record, bound(B::Tasks, 17, 16));

    let mut cold = empty(&LIMITS, true);
    let capacity = cold.domain.stubs.capacity();
    for number in 1..=u64::from(capacity) {
        assert!(
            cold.send(Event::Restore {
                record: Stored::Stub(Stub { task: number, phase: Status::Done, result: Token::new(number) })
            })
            .is_empty()
        );
    }
    refused(
        &mut cold,
        Stored::Stub(Stub {
            task: u64::from(capacity) + 1,
            phase: Status::Done,
            result: Token::new(u64::from(capacity) + 1),
        }),
        bound(B::Stubs, u64::from(capacity) + 1, u64::from(capacity)),
    );

    let mut cold = empty(&LIMITS, true);
    for number in 1..=u64::from(LIMITS.tasks) {
        assert!(cold.send(Event::Restore { record: proposal(number) }).is_empty());
    }
    refused(&mut cold, proposal(900), bound(B::PersonProposals, 17, 16));

    let mut cold = empty(&LIMITS, true);
    let capacity = cold.domain.holders.capacity();
    for number in 1..=u64::from(capacity) {
        let mut record = row();
        live(&mut record).number = number;
        live(&mut record).settings.task = u32::try_from(number).expect("small revision");
        assert!(cold.send(Event::Restore { record }).is_empty());
    }
    let mut record = row();
    live(&mut record).number = 901;
    // The incoming task needs two distinct revisions not yet held.
    live(&mut record).settings.task = 40;
    live(&mut record).settings.run = Some(41);
    refused(&mut cold, record, bound(B::HeldRevisions, u64::from(capacity) + 2, u64::from(capacity)));
}

fn proposal(number: u64) -> Stored {
    Stored::PersonProposal(Box::new(crate::PersonProposal {
        number,
        proposer: PERSON,
        project: 1,
        action: crate::PersonAction::Batch(Box::new([])),
        reason: Box::new([]),
        state: crate::PersonProposalState::Pending { since: skein_lib::Wall::EPOCH },
    }))
}

fn word(kind: crate::MessageKind, size: usize) -> crate::Word {
    crate::Word {
        number: 1,
        from: crate::Party::Person(9),
        kind,
        words: vec![0; size].into_boxed_slice(),
        at: skein_lib::Wall::EPOCH,
        hits: 1,
        eligible: true,
    }
}

#[test]
fn inbox_payloads_and_separate_reserved_lanes_keep_the_live_key() {
    for (kind, size, cause) in [
        (crate::MessageKind::Words, 33, bound(B::MessageBytes, 33, 32)),
        (crate::MessageKind::Result(crate::ResultKind::Report), 33, bound(B::ResultBytes, 33, 32)),
        (crate::MessageKind::Met { dependency: 9, removed: false }, 1, bound(B::ZeroBytes, 1, 0)),
        (
            crate::MessageKind::QuestionOwed,
            crate::questions::OWED_WORDS.len() + 1,
            bound(
                B::ReminderBytes,
                u64::try_from(crate::questions::OWED_WORDS.len()).expect("fixed reminder") + 1,
                u64::try_from(crate::questions::OWED_WORDS.len()).expect("fixed reminder"),
            ),
        ),
    ] {
        let mut record = row();
        live(&mut record).inbox = Box::new([word(kind, size)]);
        refused(&mut empty(&LIMITS, true), record, cause);
    }
    let mut record = row();
    live(&mut record).inbox = vec![word(crate::MessageKind::Amendment { revision: 1 }, 0); 3].into_boxed_slice();
    refused(&mut empty(&LIMITS, true), record, bound(B::AmendmentSlots, 3, 2));
    let mut record = row();
    live(&mut record).inbox =
        vec![word(crate::MessageKind::Met { dependency: 9, removed: false }, 0); 2].into_boxed_slice();
    refused(&mut empty(&LIMITS, true), record, bound(B::MetSlots, 2, 1));
    let mut record = row();
    live(&mut record).inbox = vec![word(crate::MessageKind::Words, 32); 5].into_boxed_slice();
    refused(&mut empty(&LIMITS, true), record, bound(B::InboxBytes, 160, 128));
    let mut record = row();
    live(&mut record).inbox = vec![word(crate::MessageKind::Words, 0); 9].into_boxed_slice();
    refused(&mut empty(&LIMITS, true), record, bound(B::InboxMessages, 9, 8));
}

#[test]
fn nonparty_placed_owners_do_not_acquire_the_party_bound() {
    let mut source = one_task();
    for number in [31, 32] {
        source.stored.insert(Key::PlacedHold(number), hold(number, crate::Owner::Task { task: DUE }));
    }
    let mut cold = empty(&crate::Limits { party_placed_holds: 1, ..LIMITS }, true);
    for record in source.stored.values().filter(|row| row.key().live()) {
        assert!(cold.send(Event::Restore { record: record.clone() }).is_empty());
    }
    assert!(cold.send(Event::Check).is_empty());
    assert!(cold.domain.checked());
    assert_eq!(cold.domain.restore_failure(), None);
}

#[test]
#[should_panic(expected = "historical row in live load")]
fn historical_row_is_mode_misuse() {
    let mut record = row();
    let Stored::Live(task) = &mut record else { panic!("row") };
    let historical = Stored::Ended(task.clone());
    empty(&LIMITS, true).send(Event::Restore { record: historical });
}

#[test]
#[should_panic(expected = "restored completion before link")]
fn completion_before_link_is_mode_misuse() {
    empty(&LIMITS, true).send(Event::Restored);
}

#[test]
#[should_panic(expected = "restore row after load completed")]
fn late_row_is_mode_misuse() {
    let mut cold = empty(&LIMITS, true);
    cold.send(Event::Check);
    cold.send(Event::Restore { record: row() });
}

#[test]
fn retained_choices_do_not_rejudge_authority_or_duplicate_models() {
    let mut source = one_task();
    let Some(Stored::Live(row)) = source.stored.get_mut(&Key::Live(DUE)) else { panic!("row") };
    let chosen = crate::ModelEffort { model: 0, effort: None };
    row.llm = Some(crate::LlmChoice { account: None, main: chosen, subagents: Box::new([chosen, chosen]) });
    row.authority.models = crate::Models::Named(0);
    let mut selected = cold_load_fit_link(&source);
    assert!(selected.send(Event::Check).is_empty());
    assert!(selected.domain.checked());
    assert_eq!(selected.domain.restore_failure(), None);
    let mut admission = cold(&source);
    admission.send(Event::Check);
    assert!(admission.domain.checked());
    assert_eq!(admission.domain.restore_failure(), None);
}
