use super::{limits, owner_chip_records, w};
use skein_lib::{Duration, List, Writer};
pub(super) fn rows<T>(items: impl IntoIterator<Item = T>) -> List<T> {
    let items: Vec<_> = items.into_iter().collect();
    let mut out = List::with_capacity(u32::try_from(items.len()).expect("fixture size"));
    for item in items {
        out.push(item).ok().expect("fixture rows fit");
    }
    out
}
pub(super) fn encoded(measure: u32, write: impl FnOnce(&mut Writer) -> Result<(), skein_lib::Overflow>) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(measure).expect("fixture size"));
    write(&mut writer).expect("literal record encodes");
    writer.finish()
}
pub(super) fn pieces(values: impl IntoIterator<Item = Box<[u8]>>) -> Box<[u8]> {
    let values: Vec<_> = values.into_iter().collect();
    let size = values.iter().map(|v| v.len().checked_add(4).expect("literal piece size")).sum();
    let mut writer = Writer::new(size);
    for value in values {
        skein_codec::piece::write(&mut writer, &value).expect("sized literal pieces");
    }
    writer.finish()
}
#[must_use]
pub fn live_chip_record() -> w::Chip {
    owner_chip_records().into_iter().next().expect("waiting phase fixture")
}
#[must_use]
pub fn live_budget_record() -> w::Budget {
    let l = limits();
    w::Budget::new(
        &l,
        w::BudgetParts {
            numbers: w::Numbers::new(&l, w::NumbersParts { budget: 7, spent: 7, spent_below: 7, reserved: 7 })
                .expect("literal"),
            left: 7,
            funder: w::Funder::Task(w::FunderTask::new(&l, w::FunderTaskParts { value: 7 }).expect("literal")),
            deadline: None,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn live_call_record() -> w::CurrentCall {
    let l = limits();
    w::CurrentCall::new(
        &l,
        w::CurrentCallParts {
            identity: w::CallIdentity::new(
                &l,
                w::CallIdentityParts { owner: 19, sequence: 23, retry: 3, id: Box::from(b"call".as_slice()) },
            )
            .expect("literal"),
            tool: Box::from(b"tool".as_slice()),
            input: Box::from("é".as_bytes()),
            cut: true,
            began: Some(29),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn writing_record() -> w::RunText {
    let l = limits();
    w::RunText::new(
        &l,
        w::RunTextParts {
            writing: None,
            attempt: 31,
            began: None,
            turn: w::ProvisionalTurnName::new(&l, w::ProvisionalTurnNameParts { attempt: 31, number: 37, retry: 5 })
                .expect("literal"),
            from: 41,
            through: 43,
            text: Box::from("é".as_bytes()),
            cut: true,
            call: Some(w::LiveCall::Raw(
                w::LiveCallRaw::new(&l, w::LiveCallRawParts { value: live_call_record() }).expect("literal"),
            )),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn run_writing_record() -> w::RunState {
    let l = limits();
    w::RunState::new(
        &l,
        w::RunStateParts {
            task: 7,
            attempt: Some(31),
            committed_turn: 2,
            writing: Some(writing_record()),
            call: Some(live_call_record()),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn live_cursor_record() -> w::Cursor {
    w::Cursor::new(&limits(), w::CursorParts { value: Box::from([255, 0, 3]) }).expect("opaque literal")
}
#[must_use]
pub fn phases_record() -> w::PhaseCounts {
    w::PhaseCounts::new(
        &limits(),
        w::PhaseCountsParts {
            waiting: 1,
            idle: 2,
            starting: 3,
            running: 4,
            backing_off: 5,
            closing: 6,
            held: 7,
            done: 8,
            failed: 9,
            cancelled: 10,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn task_line_record() -> w::TaskLine {
    let l = limits();
    w::TaskLine::new(
        &l,
        w::TaskLineParts {
            chip: live_chip_record(),
            project: 7,
            tracked: Some(19),
            label: 0,
            flags: w::TaskFlags::new(
                &l,
                w::TaskFlagsParts {
                    ended_by: w::EndedBy::Result,
                    conversation: false,
                    listed: true,
                    retention: w::Retention::Kept,
                },
            )
            .expect("fixture flags"),
            last_activity: 29,
            ended_at: None,
            budget: live_budget_record(),
            waits_on: w::WaitsOn::Nothing,
            phases: phases_record(),
            held_below: 0,
            conversation: None,
            requester: w::Requester::Person(
                w::RequesterPerson::new(&l, w::RequesterPersonParts { value: 23 }).expect("literal"),
            ),
            changed: 29,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn tasks_rows_record() -> w::Tasks {
    let l = limits();
    let row = task_line_record();
    w::Tasks::new(&l, w::TasksParts { rows: pieces([encoded(row.measure(), |w| row.encode(w))]), more: true })
        .expect("literal")
}
#[must_use]
pub fn stored_link_record() -> w::TaskLink {
    let l = limits();
    w::TaskLink::Stored(
        w::TaskLinkStored::new(&l, w::TaskLinkStoredParts { number: 7, status: Some(w::TaskStatus::Failed) })
            .expect("literal"),
    )
}
#[must_use]
pub fn tree_rows_record() -> w::Tree {
    let l = limits();
    let dependency = w::TaskLink::Stored(
        w::TaskLinkStored::new(&l, w::TaskLinkStoredParts { number: 19, status: Some(w::TaskStatus::Failed) })
            .expect("literal"),
    );
    let row = w::TreeRow::new(
        &l,
        w::TreeRowParts {
            chip: live_chip_record(),
            parent: 7,
            depth: 1,
            proposals: rows([w::ProposalKey::new(
                &l,
                w::ProposalKeyParts {
                    proposer: w::Proposer::Party(
                        w::ProposerParty::new(&l, w::ProposerPartyParts { value: 11 }).expect("literal"),
                    ),
                    number: 13,
                },
            )
            .expect("literal")]),
            spent: 17,
            dependencies: pieces([encoded(dependency.measure(), |w| dependency.encode(w))]),
            cut: true,
        },
    )
    .expect("literal");
    w::Tree::new(
        &l,
        w::TreeParts {
            root: live_chip_record(),
            rows: pieces([encoded(row.measure(), |w| row.encode(w))]),
            folded: rows([w::TreeFold::new(
                &l,
                w::TreeFoldParts { parent: 7, phases: phases_record(), held_below: 23 },
            )
            .expect("literal")]),
            held_below: 29,
            next: Some(live_cursor_record()),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn live_inbox_cursor_record() -> w::InboxCursor {
    let l = limits();
    w::InboxCursor::Inbox(
        w::InboxCursorInbox::new(
            &l,
            w::InboxCursorInboxParts {
                task: 7,
                object: w::InboxObject::Turn(
                    w::InboxObjectTurn::new(&l, w::InboxObjectTurnParts { message: 11 }).expect("literal"),
                ),
                at: Some(13),
            },
        )
        .expect("literal"),
    )
}
#[must_use]
pub fn inbox_both_groups_record() -> w::Inbox {
    let l = limits();
    let need = w::InboxEntry::new(
        &l,
        w::InboxEntryParts {
            reply: None,
            task: stored_link_record(),
            project: Some(7),
            object: w::InboxObject::PersonTask,
            activity: Some(17),
            at: Some(13),
            unread: Some(true),
            cursor: live_inbox_cursor_record(),
        },
    )
    .expect("literal");
    let update = w::InboxEntry::new(
        &l,
        w::InboxEntryParts {
            reply: Some(task_line_record()),
            task: stored_link_record(),
            project: Some(7),
            object: w::InboxObject::Turn(
                w::InboxObjectTurn::new(&l, w::InboxObjectTurnParts { message: 11 }).expect("literal"),
            ),
            activity: Some(17),
            at: Some(13),
            unread: Some(true),
            cursor: live_inbox_cursor_record(),
        },
    )
    .expect("literal");
    let selection = w::InboxSelection::new(
        &l,
        w::InboxSelectionParts { cursor: live_inbox_cursor_record(), goals: rows([37, 41]) },
    )
    .expect("literal");
    w::Inbox::new(
        &l,
        w::InboxParts {
            party: 19,
            needs: pieces([encoded(need.measure(), |w| need.encode(w))]),
            updates: pieces([encoded(update.measure(), |w| update.encode(w))]),
            next: Some(live_inbox_cursor_record()),
            cached_needs: 23,
            cached_updates: w::Count::new(&l, w::CountParts { value: 29, limited: true }).expect("literal"),
            results_seen: 31,
            updates_time_complete: false,
            selection: Some(pieces([encoded(selection.measure(), |w| selection.encode(w))])),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn host_run_record() -> w::HostRun {
    let l = limits();
    w::HostRun::new(
        &l,
        w::HostRunParts {
            task: stored_link_record(),
            attempt: 11,
            kept: 13,
            listed: true,
            state: w::AttemptState::Kept,
            location: w::RunLocation::Adrift(
                w::RunLocationAdrift::new(
                    &l,
                    w::RunLocationAdriftParts {
                        channel: Some(17),
                        until: Some(19),
                        remaining: Some(Duration::from_nanos(23)),
                    },
                )
                .expect("literal"),
            ),
            grace_until: Some(29),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn deployment_rows_record() -> w::Deployment {
    let l = limits();
    let run = host_run_record();
    let chip = live_chip_record();
    let host = w::Host::new(
        &l,
        w::HostParts {
            rule: w::HostRule::Open,
            skein_bools: [false, true, false, true],
            name: None,
            programs: Box::new([]),
            channel: 17,
            kind: w::HostKind::Engine,
            slots: 2,
            used: 1,
            free: 1,
            grace_until: Some(19),
            grace_remaining: Some(Duration::from_nanos(23)),
            runs: pieces([encoded(run.measure(), |w| run.encode(w))]),
        },
    )
    .expect("literal");
    let account = w::Account::new(
        &l,
        w::AccountParts {
            provider_name: None,
            manage_usage: None,
            number: 41,
            place: 0,
            uses: None,
            provider: Some(3),
            display: super::wire_text(b"Deployment account"),
            holder: w::AccountHolder::Deployment,
            sources: Box::from([]),
            generation: Some(43),
            state: w::AccountState::Later(
                w::AccountStateLater::new(&l, w::AccountStateLaterParts { until: Some(47) }).expect("literal"),
            ),
            skein_bools: [false, false, true, false, true],
            spent_until: Some(53),
            runs: pieces([encoded(chip.measure(), |w| chip.encode(w))]),
            waiting: pieces([encoded(chip.measure(), |w| chip.encode(w))]),
        },
    )
    .expect("literal");
    w::Deployment::new(
        &l,
        w::DeploymentParts {
            skein_bools: [false, true, false, false],
            providers: Box::new([]),
            key_bytes: 256,
            readiness: Box::new([]),
            store: w::StoreFill::new(&l, w::StoreFillParts { bytes: 31, mark: 37 }).expect("literal"),
            hosts: pieces([encoded(host.measure(), |w| host.encode(w))]),
            attempts: pieces([encoded(run.measure(), |w| run.encode(w))]),
            accounts: pieces([encoded(account.measure(), |w| account.encode(w))]),
            tasks: w::Capacity::new(&l, w::CapacityParts { used: 1, limit: 2 }).expect("literal"),
            workers: w::Capacity::new(&l, w::CapacityParts { used: 3, limit: 4 }).expect("literal"),
            attempts_capacity: w::Capacity::new(&l, w::CapacityParts { used: 5, limit: 6 }).expect("literal"),
            waiting_attempts: 7,
            proposals: w::Capacity::new(&l, w::CapacityParts { used: 8, limit: 9 }).expect("literal"),
            projects: rows([w::ProjectCapacity::new(
                &l,
                w::ProjectCapacityParts {
                    project: 7,
                    tasks: w::Capacity::new(&l, w::CapacityParts { used: 11, limit: 13 }).expect("literal"),
                    proposals: 17,
                },
            )
            .expect("literal")]),
            grace_bound: Duration::from_nanos(59),
            hello_refusals: rows([hello_refusal_record()]),
            hosts_forgotten: 83,
            hello_refusals_forgotten: 89,
            person_room_refusals: rows([person_refusal_record()]),
            person_room_refusals_forgotten: 109,
            person_answer_retention: Duration::from_nanos(113),
            task_room_refusals: rows([task_refusal_record()]),
            task_room_refusals_forgotten: 149,
            connectors: rows([2, 3]),
        },
    )
    .expect("literal")
}
fn hello_refusal_record() -> w::HelloRefusalLine {
    let l = limits();
    w::HelloRefusalLine::new(
        &l,
        w::HelloRefusalLineParts {
            channel: 61,
            at: 67,
            slots: 2,
            stop_bound: Duration::from_nanos(71),
            why: w::HelloRefusal::Agent(
                w::HelloRefusalAgent::new(
                    &l,
                    w::HelloRefusalAgentParts {
                        why: w::AgentShortfall::Window(
                            w::AgentShortfallWindow::new(&l, w::AgentShortfallWindowParts { charter: 73, model: 79 })
                                .expect("literal"),
                        ),
                    },
                )
                .expect("literal"),
            ),
        },
    )
    .expect("literal")
}
fn person_refusal_record() -> w::PersonCapacityRefusal {
    let l = limits();
    w::PersonCapacityRefusal::new(
        &l,
        w::PersonCapacityRefusalParts {
            key: w::RequestKey::new(&l, w::RequestKeyParts { person: 97, key: [3; 16] }).expect("literal"),
            project: Some(7),
            action: w::PersonCapacityAction::Proposal(
                w::PersonCapacityActionProposal::new(
                    &l,
                    w::PersonCapacityActionProposalParts {
                        proposer: w::Proposer::Party(
                            w::ProposerParty::new(&l, w::ProposerPartyParts { value: 101 }).expect("literal"),
                        ),
                        number: 103,
                    },
                )
                .expect("literal"),
            ),
            at: 107,
            cause: w::PersonCapacityCause::Limit(
                w::PersonCapacityCauseLimit::new(
                    &l,
                    w::PersonCapacityCauseLimitParts { which: w::AdmissionLimit::LiveTasks },
                )
                .expect("literal"),
            ),
        },
    )
    .expect("literal")
}
fn task_refusal_record() -> w::TaskCapacityRefusal {
    let l = limits();
    w::TaskCapacityRefusal::new(
        &l,
        w::TaskCapacityRefusalParts {
            request: w::TaskCapacityRequest::new(
                &l,
                w::TaskCapacityRequestParts {
                    project: Some(7),
                    by: w::Requester::Deployment(
                        w::RequesterDeployment::new(&l, w::RequesterDeploymentParts { project: 7 }).expect("literal"),
                    ),
                    action: w::TaskCapacityAction::Source(
                        w::TaskCapacityActionSource::new(
                            &l,
                            w::TaskCapacityActionSourceParts {
                                funder: w::Funder::Task(
                                    w::FunderTask::new(&l, w::FunderTaskParts { value: 127 }).expect("literal"),
                                ),
                            },
                        )
                        .expect("literal"),
                    ),
                    cause: w::TaskCapacityCause::Inbox(
                        w::TaskCapacityCauseInbox::new(
                            &l,
                            w::TaskCapacityCauseInboxParts { task: 131, messages: 2, bytes: 137 },
                        )
                        .expect("literal"),
                    ),
                },
            )
            .expect("literal"),
            at: 139,
        },
    )
    .expect("literal")
}

/// A page fixture with defaults for every field, before a test selects its facts.
#[must_use]
pub fn task_page_record() -> w::TaskPage {
    w::TaskPage::new(&limits(), w::TaskPageParts { live: Box::new([]), live_after: None, ended: w::EndedGroup::None })
        .expect("empty page fits")
}
