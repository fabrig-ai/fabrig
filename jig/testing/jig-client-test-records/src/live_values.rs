use jig_client_protocol_wire::Record as _;
use super::{owner_chip_records, w};
use skein_lib::{Duration, List, Writer};
pub(super) fn rows<T>(items: impl IntoIterator<Item = T>) -> List<T> {
    List::from_boxed(items.into_iter().collect::<Vec<_>>().into_boxed_slice())
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
    w::Budget {
            numbers: w::Numbers { budget: 7, spent: 7, spent_below: 7, reserved: 7 },
            left: 7,
            funder: w::Funder::Task(w::FunderTask { value: 7 }),
            deadline: None,
        }
}
#[must_use]
pub fn live_call_record() -> w::CurrentCall {
    w::CurrentCall {
            identity: w::CallIdentity { owner: 19, sequence: 23, retry: 3, id: Box::from(b"call".as_slice()) },
            tool: Box::from(b"tool".as_slice()),
            input: Box::from("é".as_bytes()),
            cut: true,
            began: Some(29),
        }
}
#[must_use]
pub fn writing_record() -> w::RunText {
    w::RunText {
            writing: None,
            attempt: 31,
            began: None,
            turn: w::ProvisionalTurnName { attempt: 31, number: 37, retry: 5 },
            from: 41,
            through: 43,
            text: Box::from("é".as_bytes()),
            cut: true,
            call: Some(w::LiveCall::Raw(
                w::LiveCallRaw { value: live_call_record() },
            )),
        }
}
#[must_use]
pub fn run_writing_record() -> w::RunState {
    w::RunState {
            task: 7,
            attempt: Some(31),
            committed_turn: 2,
            writing: Some(writing_record()),
            call: Some(live_call_record()),
        }
}
#[must_use]
pub fn live_cursor_record() -> w::Cursor {
    w::Cursor { value: Box::from([255, 0, 3]) }
}
#[must_use]
pub fn phases_record() -> w::PhaseCounts {
    w::PhaseCounts {
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
        }
}
#[must_use]
pub fn task_line_record() -> w::TaskLine {
    w::TaskLine {
            parts: List::with_capacity(0),
            chip: live_chip_record(),
            project: 7,
            tracked: Some(19),
            label: 0,
            flags: w::TaskFlags {
                    ended_by: w::EndedBy::Result,
                    conversation: false,
                    listed: true,
                    retention: w::Retention::Kept,
                },
            last_activity: 29,
            ended_at: None,
            budget: live_budget_record(),
            waits_on: w::WaitsOn::Nothing,
            phases: phases_record(),
            delegates_done: 9,
            delegates_total: 11,
            held_below: 0,
            conversation: None,
            requester: w::Requester::Person(
                w::RequesterPerson { value: 23 },
            ),
            changed: 29,
        }
}
#[must_use]
pub fn tasks_rows_record() -> w::Tasks {
    let row = task_line_record();
    w::Tasks { ended_mark: 0, rows: pieces([row.encode(&crate::limits()).expect("literal record encodes")]), more: true }
}
#[must_use]
pub fn stored_link_record() -> w::TaskLink {
    w::TaskLink::Stored(
        w::TaskLinkStored { number: 7, status: Some(w::TaskStatus::Failed) },
    )
}
#[must_use]
pub fn tree_rows_record() -> w::Tree {
    let dependency = w::TaskLink::Stored(
        w::TaskLinkStored { number: 19, status: Some(w::TaskStatus::Failed) },
    );
    let row = w::TreeRow {
            chip: live_chip_record(),
            parent: 7,
            depth: 1,
            proposals: rows([w::ProposalKey {
                    proposer: w::Proposer::Party(
                        w::ProposerParty { value: 11 },
                    ),
                    number: 13,
                }]),
            spent: 17,
            dependencies: pieces([dependency.encode(&crate::limits()).expect("literal record encodes")]),
            cut: true,
        };
    w::Tree {
            root: live_chip_record(),
            rows: pieces([row.encode(&crate::limits()).expect("literal record encodes")]),
            folded: rows([w::TreeFold { parent: 7, phases: phases_record(), held_below: 23 }]),
            held_below: 29,
            next: Some(live_cursor_record()),
        }
}
#[must_use]
pub fn live_inbox_cursor_record() -> w::InboxCursor {
    w::InboxCursor::Inbox(
        w::InboxCursorInbox {
                task: 7,
                object: w::InboxObject::Turn(
                    w::InboxObjectTurn { message: 11 },
                ),
                at: Some(13),
            },
    )
}
#[must_use]
pub fn inbox_both_groups_record() -> w::Inbox {
    let need = w::InboxEntry {
            projects: List::with_capacity(0),
            reply: None,
            task: stored_link_record(),
            project: Some(7),
            object: w::InboxObject::PersonTask,
            activity: Some(17),
            at: Some(13),
            unread: Some(true),
            cursor: live_inbox_cursor_record(),
        };
    let update = w::InboxEntry {
            projects: List::with_capacity(0),
            reply: Some(task_line_record()),
            task: stored_link_record(),
            project: Some(7),
            object: w::InboxObject::Turn(
                w::InboxObjectTurn { message: 11 },
            ),
            activity: Some(17),
            at: Some(13),
            unread: Some(true),
            cursor: live_inbox_cursor_record(),
        };
    let selection = w::InboxSelection { cursor: live_inbox_cursor_record(), goals: rows([37, 41]) };
    w::Inbox {
            party: 19,
            needs: pieces([need.encode(&crate::limits()).expect("literal record encodes")]),
            updates: pieces([update.encode(&crate::limits()).expect("literal record encodes")]),
            next: Some(live_inbox_cursor_record()),
            cached_needs: 23,
            cached_updates: w::Count { value: 29, limited: true },
            results_seen: 31,
            updates_time_complete: false,
            selection: Some(pieces([selection.encode(&crate::limits()).expect("literal record encodes")])),
        }
}
#[must_use]
pub fn host_run_record() -> w::HostRun {
    w::HostRun {
            task: stored_link_record(),
            attempt: 11,
            kept: 13,
            listed: true,
            state: w::AttemptState::Kept,
            location: w::RunLocation::Adrift(
                w::RunLocationAdrift {
                        channel: Some(17),
                        until: Some(19),
                        remaining: Some(Duration::from_nanos(23)),
                    },
            ),
            grace_until: Some(29),
        }
}
#[must_use]
pub fn bounded_store_census_record() -> w::StoreCensus {
    let mut keyspaces = List::with_capacity(129);
    for n in 0_u8..129 {
        let mut heads = List::with_capacity(8);
        for h in 0_u16..8 {
            heads
                .push(
                    w::CensusHead { head: if h == 0 { None } else { Some(h) }, rows: u64::from(h) + 10 },
                )
                .expect("eight heads");
        }
        keyspaces
            .push(
                w::CensusKeyspace {
                        prefix: Box::new([n; 8]),
                        name: Box::new([b'n'; 48]),
                        rows: u64::from(n),
                        keys: 11,
                        values: 13,
                        heads,
                        puts: 17,
                        erases: 19,
                        written: u64::MAX,
                    },
            )
            .expect("129 keyspaces");
    }
    w::StoreCensus { commit: u64::MAX, since: 679_000, keyspaces }
}

#[must_use]
pub fn deployment_rows_record() -> w::Deployment {
    let run = host_run_record();
    let chip = live_chip_record();
    let host = w::Host {
            rule: w::HostRule::Open,
            awaiting_logins: false,
            draining: true,
            connected: false,
            cut: true,
            name: None,
            programs: Box::new([]),
            channel: 17,
            kind: w::HostKind::Engine,
            slots: 2,
            used: 1,
            free: 1,
            grace_until: Some(19),
            grace_remaining: Some(Duration::from_nanos(23)),
            runs: pieces([run.encode(&crate::limits()).expect("literal record encodes")]),
        };
    let account = w::Account {
            list: None,
            facts: super::account_fact_records(),
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
                w::AccountStateLater { until: Some(47) },
            ),
            plan: false,
            usable: false,
            expiring: true,
            attention: false,
            cut: true,
            spent_until: Some(53),
            runs: pieces([chip.encode(&crate::limits()).expect("literal record encodes")]),
            waiting: pieces([chip.encode(&crate::limits()).expect("literal record encodes")]),
        };
    w::Deployment {
            worker_settings: List::with_capacity(0),
            cut: false,
            connectors_cut: true,
            providers_cut: false,
            readiness_cut: false,
            providers: Box::new([]),
            key_bytes: 256,
            readiness: Box::new([]),
            store: w::StoreFill { bytes: 31, mark: 37 },
            census: None,
            hosts: pieces([host.encode(&crate::limits()).expect("literal record encodes")]),
            attempts: pieces([run.encode(&crate::limits()).expect("literal record encodes")]),
            accounts: pieces([account.encode(&crate::limits()).expect("literal record encodes")]),
            tasks: w::Capacity { used: 1, limit: 2 },
            workers: w::Capacity { used: 3, limit: 4 },
            attempts_capacity: w::Capacity { used: 5, limit: 6 },
            waiting_attempts: 7,
            proposals: w::Capacity { used: 8, limit: 9 },
            projects: rows([w::ProjectCapacity {
                    project: 7,
                    tasks: w::Capacity { used: 11, limit: 13 },
                    proposals: 17,
                }]),
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
        }
}
fn hello_refusal_record() -> w::HelloRefusalLine {
    w::HelloRefusalLine {
            channel: 61,
            at: 67,
            slots: 2,
            stop_bound: Duration::from_nanos(71),
            why: w::HelloRefusal::Agent(
                w::HelloRefusalAgent {
                        why: w::AgentShortfall::Window(
                            w::AgentShortfallWindow { charter: 73, model: 79 },
                        ),
                    },
            ),
        }
}
fn person_refusal_record() -> w::PersonCapacityRefusal {
    w::PersonCapacityRefusal {
            key: w::RequestKey { person: 97, key: [3; 16] },
            project: Some(7),
            action: w::PersonCapacityAction::Proposal(
                w::PersonCapacityActionProposal {
                        proposer: w::Proposer::Party(
                            w::ProposerParty { value: 101 },
                        ),
                        number: 103,
                    },
            ),
            at: 107,
            cause: w::PersonCapacityCause::Limit(
                w::PersonCapacityCauseLimit { which: w::AdmissionLimit::LiveTasks },
            ),
        }
}
fn task_refusal_record() -> w::TaskCapacityRefusal {
    w::TaskCapacityRefusal {
            request: w::TaskCapacityRequest {
                    project: Some(7),
                    by: w::Requester::Deployment(
                        w::RequesterDeployment { project: 7 },
                    ),
                    action: w::TaskCapacityAction::Source(
                        w::TaskCapacityActionSource {
                                funder: w::Funder::Task(
                                    w::FunderTask { value: 127 },
                                ),
                            },
                    ),
                    cause: w::TaskCapacityCause::Inbox(
                        w::TaskCapacityCauseInbox { task: 131, messages: 2, bytes: 137 },
                    ),
                },
            at: 139,
        }
}

/// A page fixture with defaults for every field, before a test selects its facts.
#[must_use]
pub fn task_page_record() -> w::TaskPage {
    w::TaskPage { ended_mark: 0, live: Box::new([]), live_after: None, ended: w::EndedGroup::None }
}
