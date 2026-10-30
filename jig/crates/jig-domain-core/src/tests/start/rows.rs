//! Stored core rows load independently of admission (domain/engine.md, 6).
#![expect(clippy::disallowed_macros, reason = "dynamic unit fixtures exercise stored collection boundaries")]
use super::{Core, Failure, Key, Wall, call_key, config, limits, proof, tasks};
use crate::start::LiveSet;
use crate::{CallPart, CallRecord, CoreKey, CoreRecord, Limits, Restored, SettledAnswer, SettledCall};
use jig_domain_authority as authority;
use jig_domain_notes as notes;
use skein_lib::List;

fn loaded(selected: bool) -> (Core, Limits) {
    let limits = limits();
    let mut core = Core::new(config(301), &limits).expect("configured core");
    if selected {
        let _begin = core.start_begin(false, Wall::EPOCH);
    }
    let mut header = core.counters.deployment();
    header.tasks = 100;
    header.runs = 100;
    header.calls = 100;
    header.people = 100;
    header.messages = 100;
    header.connector_rows = 100;
    header.commits = 1;
    assert_eq!(core.restore_core(CoreRecord::Deployment(header), &limits), Restored::Deployment { commits: 1 });
    core.proofs.insert(1, proof()).expect("one proof");
    (core, limits)
}

fn row(part: CallPart) -> CallRecord {
    CallRecord { key: call_key(), part, settled: None }
}
fn settled(serial: u64, tool: &[u8]) -> SettledCall {
    SettledCall { serial, tool: tool.into(), answer: SettledAnswer::Host { error: false, body: Box::new([]) } }
}
fn note() -> notes::Entry {
    notes::Entry {
        name: 1,
        revision: 1,
        author: notes::Author::Party { party: 1 },
        scope: notes::Scope::Deployment,
        description: b"note".as_slice().into(),
        body: Box::new([]),
        references: List::with_capacity(0),
        changed: Wall::EPOCH,
    }
}

fn retained(record: CallRecord) {
    let (mut default, limits) = loaded(false);
    assert_eq!(default.restore_core(CoreRecord::Call(record.clone()), &limits), Restored::Rejected);
    let (mut selected, limits) = loaded(true);
    assert_eq!(selected.restore_core(CoreRecord::Call(record.clone()), &limits), Restored::Live);
    assert_eq!(selected.call_record(record.key), Some(record));
    assert_eq!(selected.start_failure(), None);
}

#[test]
fn stored_calls_ignore_historical_serials_and_tool_shape() {
    for call in [settled(0, b"tool"), settled(101, b"tool"), settled(1, b"")] {
        let mut record = row(CallPart::Controlled);
        record.settled = Some(call);
        retained(record);
    }
    for key in [
        crate::CallKey { task: 0, ..call_key() },
        crate::CallKey { attempt: 0, ..call_key() },
        crate::CallKey { completion: 0, ..call_key() },
        crate::CallKey { task: 101, ..call_key() },
        crate::CallKey { attempt: 101, ..call_key() },
    ] {
        let mut record = row(CallPart::Controlled);
        record.key = key;
        retained(record);
    }
}

#[test]
fn settled_serials_are_not_record_keys() {
    for selected in [false, true] {
        let (mut core, limits) = loaded(selected);
        let mut record = row(CallPart::Controlled);
        record.settled = Some(settled(1, b"tool"));
        assert_eq!(core.restore_core(CoreRecord::Call(record.clone()), &limits), Restored::Live);
        record.key.position = 1;
        assert_eq!(
            core.restore_core(CoreRecord::Call(record), &limits),
            if selected { Restored::Live } else { Restored::Rejected }
        );
    }
}

#[test]
fn stored_parts_ignore_historical_counters_and_current_rules() {
    let problem = tasks::Problem { task: Some(101), why: tasks::Refusal::Grace, blocked_by: None, capacity: None };
    let parts = [
        CallPart::SettingsRefused(crate::people::Refusal::Stale { newest: 0 }),
        CallPart::HoldOwner { task: 101, hold: 1, revision: 1, owner: crate::people::HoldOwner::Party { person: 1 } },
        CallPart::HoldOwner { task: 1, hold: 0, revision: 1, owner: crate::people::HoldOwner::Party { person: 1 } },
        CallPart::HoldOwner { task: 1, hold: 101, revision: 1, owner: crate::people::HoldOwner::Task { task: 1 } },
        CallPart::HoldOwner { task: 1, hold: 1, revision: 0, owner: crate::people::HoldOwner::Party { person: 1 } },
        CallPart::HoldOwner { task: 1, hold: 1, revision: 1, owner: crate::people::HoldOwner::Party { person: 101 } },
        CallPart::HoldOwner { task: 1, hold: 1, revision: 1, owner: crate::people::HoldOwner::Task { task: 101 } },
        CallPart::Again { task: 101, required: true },
        CallPart::PassedUp { task: 101 },
        CallPart::Revised { task: 101, revision: 0 },
        CallPart::Moved { task: 101, now: crate::people::Moved::Spec { revision: 0 } },
        CallPart::Effect { connector: 1, entry: 101, deadline: Wall::EPOCH, outcome: None },
        CallPart::Proposed { proposal: 101, holder: tasks::ProposalHolder::Person(1) },
        CallPart::ProposalDecided { proposal: 101, outcome: tasks::ProposalOutcome::Accepted },
        CallPart::EscalationDecided { task: 101, revision: 1, outcome: tasks::EscalationOutcome::Released },
        CallPart::EscalationDecided { task: 1, revision: 0, outcome: tasks::EscalationOutcome::Released },
        CallPart::Sent { message: 101 },
        CallPart::NoteWritten { name: 101, revision: 1 },
        CallPart::NoteWritten { name: 1, revision: 0 },
        CallPart::Subscribed { subscription: 101 },
        CallPart::Delegated(Box::new([])),
        CallPart::Delegated(Box::new([crate::Made { task: 101, revision: 1, given: None }])),
        CallPart::EffectDenied { answer: authority::Answer::Allow, findings: Box::new([]) },
        CallPart::ToolDenied { answer: authority::Answer::Allow, findings: Box::new([]) },
        CallPart::DelegationDenied { answer: authority::Answer::Allow, findings: Box::new([]) },
        CallPart::EffectRefused(problem.clone()),
        CallPart::MessageRefused(problem.clone()),
        CallPart::ProposalRefused(problem.clone()),
        CallPart::EscalationRefused(problem.clone()),
        CallPart::SubscriptionRefused(problem.clone()),
        CallPart::DelegationRefused(problem.clone()),
        CallPart::ControlRefused(problem),
    ];
    for part in parts {
        retained(row(part));
    }
    let duplicate = row(CallPart::Delegated(Box::new([crate::Made { task: 1, revision: 1, given: None }; 2])));
    let (mut default, mut limits) = loaded(false);
    limits.tasks.batch = 2;
    assert_eq!(default.restore_core(CoreRecord::Call(duplicate.clone()), &limits), Restored::Rejected);
    let (mut selected, _) = loaded(true);
    assert_eq!(selected.restore_core(CoreRecord::Call(duplicate.clone()), &limits), Restored::Live);
    assert_eq!(selected.call_record(duplicate.key), Some(duplicate.clone()));
}

#[test]
fn recalled_notes_keep_their_stored_content_without_admission() {
    let mut cases = List::with_capacity(6);
    let mut n = note();
    n.name = 0;
    cases.push(n).expect("six note fixtures");
    let mut n = note();
    n.revision = 0;
    cases.push(n).expect("six note fixtures");
    let mut n = note();
    n.author = notes::Author::Party { party: 0 };
    cases.push(n).expect("six note fixtures");
    let mut n = note();
    n.author = notes::Author::Task { task: 1, attempt: 0 };
    cases.push(n).expect("six note fixtures");
    let mut n = note();
    n.description = b"two\nlines".as_slice().into();
    cases.push(n).expect("six note fixtures");
    let mut n = note();
    n.references = List::with_capacity(1);
    n.references.push(0).expect("reference room");
    cases.push(n).expect("six note fixtures");
    for n in &cases {
        retained(row(CallPart::NoteRecalled { entries: Box::new([n.clone()]), more: false }));
    }
}

fn call_bound(part: CallPart, held: u64, limit: u64) {
    let (mut core, limits) = loaded(true);
    let record = row(part);
    assert_eq!(core.restore_core(CoreRecord::Call(record), &limits), Restored::Rejected);
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet { set: LiveSet::Calls, key: Key::Core(CoreKey::Call(call_key())), held, limit })
    );
}

#[test]
fn call_payload_capacity_failures_keep_the_outer_record_key() {
    let limits = limits();
    let count = limits.tasks.batch + 1;
    call_bound(
        CallPart::Delegated(vec![crate::Made { task: 1, revision: 1, given: None }; size(count)].into_boxed_slice()),
        u64::from(count),
        u64::from(limits.tasks.batch),
    );
    let limit = authority::stored_finding_limit(&limits.authority).expect("finding bound");
    for part in [
        CallPart::EffectDenied {
            answer: authority::Answer::Allow,
            findings: vec![authority::Finding::Oversized; size(limit + 1)].into_boxed_slice(),
        },
        CallPart::ToolDenied {
            answer: authority::Answer::Allow,
            findings: vec![authority::Finding::Oversized; size(limit + 1)].into_boxed_slice(),
        },
        CallPart::DelegationDenied {
            answer: authority::Answer::Allow,
            findings: vec![authority::Finding::Oversized; size(limit + 1)].into_boxed_slice(),
        },
    ] {
        call_bound(part, u64::from(limit) + 1, u64::from(limit));
    }
    call_bound(
        CallPart::NoteRecalled {
            entries: vec![note(); size(limits.notes.recalled + 1)].into_boxed_slice(),
            more: false,
        },
        u64::from(limits.notes.recalled) + 1,
        u64::from(limits.notes.recalled),
    );
    for body in [false, true] {
        let mut n = note();
        let limit = if body { limits.notes.body_bytes } else { limits.notes.description_bytes };
        let bytes = vec![b'x'; size(limit + 1)].into_boxed_slice();
        if body {
            n.body = bytes;
        } else {
            n.description = bytes;
        }
        call_bound(
            CallPart::NoteRecalled { entries: Box::new([n]), more: false },
            u64::from(limit) + 1,
            u64::from(limit),
        );
    }
    let mut n = note();
    n.references = List::with_capacity(limits.notes.references + 1);
    call_bound(
        CallPart::NoteRecalled { entries: Box::new([n]), more: false },
        u64::from(limits.notes.references) + 1,
        u64::from(limits.notes.references),
    );
    for segments in [false, true] {
        let limit = limits.notes.pattern_bytes;
        let empty: Box<[u8]> = Box::new([]);
        let pattern = if segments {
            notes::Pattern {
                segments: vec![empty; size(limit + 1)].into_boxed_slice(),
                last: notes::Last::Exact(Box::new([])),
            }
        } else {
            notes::Pattern {
                segments: Box::new([]),
                last: notes::Last::Open(vec![b'x'; size(limit + 1)].into_boxed_slice()),
            }
        };
        let mut n = note();
        n.scope = notes::Scope::Resources { project: 1, connector: 1, pattern };
        call_bound(
            CallPart::NoteRecalled { entries: Box::new([n]), more: false },
            u64::from(limit) + 1,
            u64::from(limit),
        );
    }
}

#[test]
fn settled_bytes_and_call_table_capacity_keep_the_outer_key() {
    let (mut core, limits) = loaded(true);
    let mut record = row(CallPart::Controlled);
    record.settled = Some(SettledCall {
        serial: 0,
        tool: Box::new([]),
        answer: SettledAnswer::Delivery {
            outcome: crate::DeliveryOutcome::Delivered,
            evidence: vec![b'x'; size(limits.call_answer_bytes + 1)].into_boxed_slice(),
        },
    });
    assert_eq!(core.restore_core(CoreRecord::Call(record), &limits), Restored::Rejected);
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet {
            set: LiveSet::Calls,
            key: Key::Core(CoreKey::Call(call_key())),
            held: u64::from(limits.call_answer_bytes) + 1,
            limit: u64::from(limits.call_answer_bytes)
        })
    );
    let (mut core, limits) = loaded(true);
    for position in 0..limits.call_records {
        let mut record = row(CallPart::Controlled);
        record.key.position = position;
        assert_eq!(core.restore_core(CoreRecord::Call(record), &limits), Restored::Live);
    }
    let mut record = row(CallPart::Controlled);
    record.key.position = limits.call_records;
    let key = record.key;
    assert_eq!(core.restore_core(CoreRecord::Call(record), &limits), Restored::Rejected);
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet {
            set: LiveSet::Calls,
            key: Key::Core(CoreKey::Call(key)),
            held: u64::from(limits.call_records) + 1,
            limit: u64::from(limits.call_records)
        })
    );
}

#[test]
fn proof_capacity_failures_keep_the_outer_key() {
    enum Bound {
        Offers,
        Accounts,
        Credentials,
        Terminal,
        Rows,
    }
    let limits = limits();
    for which in [Bound::Offers, Bound::Accounts, Bound::Credentials, Bound::Terminal, Bound::Rows] {
        let (mut core, _) = loaded(true);
        let mut row = proof();
        row.task = 2;
        let limit = match which {
            Bound::Offers => {
                row.outstanding = vec![1; size(limits.tasks.run_inbox + 1)].into_boxed_slice();
                limits.tasks.run_inbox
            }
            Bound::Accounts => {
                row.accounts =
                    vec![authority::AccountBinding { provider: 1, account: 1, source: 1 }; size(tasks::MODELS + 1)]
                        .into_boxed_slice();
                tasks::MODELS
            }
            Bound::Credentials => {
                let limit = core.accounts.credential_limit();
                row.credentials = vec![1; size(limit + 1)].into_boxed_slice();
                limit
            }
            Bound::Terminal => {
                let limit = limits.tasks.result_bytes * 2;
                row.terminal = Some(crate::TerminalRecord {
                    task: 2,
                    attempt: 1,
                    cumulative: 0,
                    end: tasks::End::Finished {
                        result: tasks::TaskResult::Report { words: vec![b'x'; size(limit + 1)].into_boxed_slice() },
                        cancel_delegates: false,
                    },
                });
                limit
            }
            Bound::Rows => {
                for number in 2..=limits.tasks.tasks {
                    let mut p = proof();
                    p.task = u64::from(number);
                    assert_eq!(core.restore_core(CoreRecord::RunProof(p), &limits), Restored::Live);
                }
                row.task = u64::from(limits.tasks.tasks) + 1;
                limits.tasks.tasks
            }
        };
        let key = Key::Core(CoreKey::RunProof(row.task));
        assert_eq!(core.restore_core(CoreRecord::RunProof(row), &limits), Restored::Rejected);
        assert_eq!(
            core.start_failure(),
            Some(Failure::LiveSet {
                set: LiveSet::RunProofs,
                key,
                held: u64::from(limit) + 1,
                limit: u64::from(limit)
            })
        );
    }
}

fn size(count: u32) -> usize {
    usize::try_from(count).expect("small fixture count")
}

fn accepted(part: CallPart) {
    for selected in [false, true] {
        let (mut core, limits) = loaded(selected);
        let record = row(part.clone());
        assert_eq!(core.restore_core(CoreRecord::Call(record.clone()), &limits), Restored::Live);
        assert_eq!(core.call_record(record.key), Some(record));
    }
}

#[test]
fn call_payloads_at_their_capacity_fit_both_paths() {
    let limits = limits();
    accepted(CallPart::Delegated(
        vec![crate::Made { task: 1, revision: 1, given: None }; size(limits.tasks.batch)].into_boxed_slice(),
    ));
    let limit = authority::stored_finding_limit(&limits.authority).expect("finding bound");
    accepted(CallPart::EffectDenied {
        answer: authority::Answer::Refuse,
        findings: vec![authority::Finding::Oversized; size(limit)].into_boxed_slice(),
    });
    accepted(CallPart::ToolDenied {
        answer: authority::Answer::Refuse,
        findings: vec![authority::Finding::Oversized; size(limit)].into_boxed_slice(),
    });
    accepted(CallPart::DelegationDenied {
        answer: authority::Answer::Refuse,
        findings: vec![authority::Finding::Oversized; size(limit)].into_boxed_slice(),
    });
    accepted(CallPart::NoteRecalled {
        entries: vec![note(); size(limits.notes.recalled)].into_boxed_slice(),
        more: false,
    });
    for body in [false, true] {
        let mut n = note();
        if body {
            n.body = vec![b'x'; size(limits.notes.body_bytes)].into_boxed_slice();
        } else {
            n.description = vec![b'x'; size(limits.notes.description_bytes)].into_boxed_slice();
        }
        accepted(CallPart::NoteRecalled { entries: Box::new([n]), more: false });
    }
    let mut n = note();
    n.references = List::with_capacity(limits.notes.references);
    accepted(CallPart::NoteRecalled { entries: Box::new([n]), more: false });
    for segments in [false, true] {
        let empty: Box<[u8]> = Box::new([]);
        let pattern = if segments {
            notes::Pattern {
                segments: vec![empty; size(limits.notes.pattern_bytes)].into_boxed_slice(),
                last: notes::Last::Exact(Box::new([])),
            }
        } else {
            notes::Pattern {
                segments: Box::new([]),
                last: notes::Last::Open(vec![b'x'; size(limits.notes.pattern_bytes)].into_boxed_slice()),
            }
        };
        let mut n = note();
        n.scope = notes::Scope::Resources { project: 1, connector: 1, pattern };
        accepted(CallPart::NoteRecalled { entries: Box::new([n]), more: false });
    }
    for delivery in [false, true] {
        let (mut core, limits) = loaded(true);
        let mut record = row(CallPart::Controlled);
        let bytes = vec![b'x'; size(limits.call_answer_bytes) - 1].into_boxed_slice();
        record.settled = Some(SettledCall {
            serial: 1,
            tool: b"t".as_slice().into(),
            answer: if delivery {
                SettledAnswer::Delivery { outcome: crate::DeliveryOutcome::Delivered, evidence: bytes }
            } else {
                SettledAnswer::Host { error: false, body: bytes }
            },
        });
        assert_eq!(core.restore_core(CoreRecord::Call(record), &limits), Restored::Live);
    }
}

#[test]
fn selected_row_duplicates_reject_before_replacement() {
    let (mut core, limits) = loaded(true);
    let record = row(CallPart::Controlled);
    assert_eq!(core.restore_core(CoreRecord::Call(record.clone()), &limits), Restored::Live);
    let duplicate = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
        core.restore_core(CoreRecord::Call(row(CallPart::Introduced)), &limits)
    }));
    let _panic = duplicate.expect_err("duplicate call rejects");
    assert_eq!(core.call_record(record.key), Some(record));
    let header = core.counters.deployment();
    let duplicate = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
        core.restore_core(CoreRecord::Deployment(header), &limits)
    }));
    let _panic = duplicate.expect_err("duplicate header rejects");
    assert_eq!(core.counters.deployment(), header);
    let duplicate = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
        core.restore_core(CoreRecord::RunProof(proof()), &limits)
    }));
    let _panic = duplicate.expect_err("duplicate proof rejects");
    assert_eq!(core.start_failure(), None);
}

#[test]
fn a_proof_at_all_payload_bounds_is_retained() {
    let (mut core, limits) = loaded(true);
    let mut p = proof();
    p.task = 2;
    p.outstanding = vec![1; size(limits.tasks.run_inbox)].into_boxed_slice();
    p.credentials = vec![1; size(core.accounts.credential_limit())].into_boxed_slice();
    p.accounts =
        vec![authority::AccountBinding { provider: 1, account: 1, source: 1 }; size(tasks::MODELS)].into_boxed_slice();
    p.terminal = Some(crate::TerminalRecord {
        task: 2,
        attempt: 1,
        cumulative: 0,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Report {
                words: vec![b'x'; size(limits.tasks.result_bytes * 2)].into_boxed_slice(),
            },
            cancel_delegates: false,
        },
    });
    assert_eq!(core.restore_core(CoreRecord::RunProof(p.clone()), &limits), Restored::Live);
    assert_eq!(core.proofs.get(&2), Some(&p));
    assert_eq!(core.start_failure(), None);
}

#[test]
fn seed_commit_prevents_further_core_loading_until_acknowledged() {
    let limits = limits();
    let env = super::Env { now: super::Time::ZERO, wall: Wall::EPOCH, limits };
    let mut core = Core::new(config(302), &limits).expect("configured seed");
    let _begin = core.start_begin(false, Wall::EPOCH);
    let mut out = super::Queue::uncapped();
    let super::Action::Commit(token) = core.start_done(&env, super::Phase::LoadCore, &mut out) else {
        panic!("seed requires a commit")
    };
    let writes = out.len();
    let rejected = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
        core.restore_core(CoreRecord::Call(row(CallPart::Controlled)), &limits)
    }));
    let _panic = rejected.expect_err("completed loading cannot accept a row during its seed commit");
    assert_eq!(out.len(), writes);
    assert_eq!(core.call_record(call_key()), None);
    super::step_phase(core.start_resume(&env, token, &mut out), super::Phase::LoadInbox);
}

#[test]
fn selected_seed_policy_uses_restore_instead_of_current_admission() {
    let limits = limits();
    let env = super::Env { now: super::Time::ZERO, wall: Wall::EPOCH, limits };
    let mut configured = config(303);
    configured.seed_projects[0].policy.policy.period_spend = 1001;
    let mut core = Core::new(configured, &limits).expect("seed fits configuration bounds");
    let _begin = core.start_begin(false, Wall::EPOCH);
    let action = core.start_done(&env, super::Phase::LoadCore, &mut super::Queue::uncapped());
    let super::Action::Commit(_) = action else { panic!("selected seed kept its policy") };
    assert_eq!(core.authority.policy(1).expect("seeded policy").period_spend, 1001);
    assert_eq!(core.start_failure(), None);
}
