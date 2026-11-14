use super::{budget, chip};
use jig_domain_core_vocabulary::documents as d;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as t;
use jig_domain_views as v;
use skein_lib::{Duration, List, Token, Wall};
pub fn current_call() -> v::Call {
    v::Call {
        identity: v::CallIdentity { owner: Token::new(19), sequence: 23, retry: 3, id: Box::from(b"call".as_slice()) },
        tool: Box::from(b"tool".as_slice()),
        input: Box::from("é".as_bytes()),
        cut: true,
        began: Some(Wall::from_nanos(29)),
    }
}
pub fn writing() -> v::Writing {
    v::Writing {
        writing: None,
        turn: v::TurnName { attempt: 31, turn: 37, retry: 5 },
        from: 41,
        to: 43,
        text: Box::from("é".as_bytes()),
        cut: true,
        call: Some(current_call()),
    }
}
pub fn run_writing() -> d::Run {
    d::Run { task: 7, attempt: Some(31), committed_turn: 2, writing: Some(writing()), call: Some(current_call()) }
}
pub fn phase_counts() -> d::PhaseCounts {
    d::PhaseCounts {
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
pub fn tasks_document() -> d::Lines {
    d::Lines {
        query: d::TaskQuery {
            project: Some(7),
            requester: None,
            root: None,
            parent: None,
            label: None,
            phase: None,
            executor: None,
            tracked: None,
            order: d::QueryOrder::Changed,
        },
        lines: List::from_boxed(Box::new([d::TaskLine {
            chip: chip(),
            project: 7,
            requester: t::Party::Person(23),
            tracked: Some(19),
            label: 0,
            flags: t::Flags {
                ended_by: t::EndedBy::Result,
                conversation: false,
                listed: true,
                retention: t::Retention::Kept,
            },
            changed: Wall::from_nanos(29),
            last_activity: Wall::from_nanos(29),
            ended_at: None,
            budget: budget(),
            waits_on: d::WaitsOn::Nothing,
            phases: phase_counts(),
            delegates_done: 9,
            delegates_total: 11,
            held_below: 0,
            conversation: None,
        }])),
        next: Some(d::Cursor::Number { number: 7 }),
    }
}
pub fn tree_document() -> d::Tree {
    d::Tree {
        root: chip(),
        rows: List::from_boxed(Box::new([d::TreeRow {
            chip: chip(),
            parent: 7,
            depth: 1,
            proposals: List::from_boxed(Box::new([d::ProposalRef {
                proposer: people::Proposer::Party(11),
                number: 13,
            }])),
            spent: 17,
            dependencies: List::from_boxed(Box::new([d::TaskLink::Stored {
                number: 19,
                status: Some(t::Status::Failed),
            }])),
            cut: true,
        }])),
        folded: List::from_boxed(Box::new([d::FoldedCount { parent: 7, phases: phase_counts(), held_below: 23 }])),
        held_below: 29,
        next: Some(d::Cursor::Number { number: 31 }),
    }
}
pub fn inbox_cursor() -> d::Cursor {
    d::Cursor::Inbox { task: 7, object: d::InboxObject::Reply { message: 11 }, at: Some(Wall::from_nanos(13)) }
}
pub fn inbox_entry() -> d::InboxEntry {
    d::InboxEntry {
        projects: List::with_capacity(0),
        reply: Some(Box::new(tasks_document().lines.get(0).expect("fixture line").clone())),
        task: d::TaskLink::Stored { number: 7, status: Some(t::Status::Failed) },
        project: Some(7),
        object: d::InboxObject::Reply { message: 11 },
        activity: Some(17),
        at: Some(Wall::from_nanos(13)),
        unread: Some(true),
        cursor: inbox_cursor(),
    }
}
pub fn inbox_document() -> d::Inbox {
    let mut need = inbox_entry();
    need.reply = None;
    need.object = d::InboxObject::PersonTask;
    d::Inbox {
        party: 19,
        needs: List::from_boxed(Box::new([need])),
        updates: List::from_boxed(Box::new([inbox_entry()])),
        next: Some(inbox_cursor()),
        cached_needs: 23,
        cached_updates: d::Count::AtLeast(29),
        results_seen: 31,
        updates_time_complete: false,
        selection: Some(List::from_boxed(Box::new([d::InboxSelection {
            cursor: inbox_cursor(),
            goals: List::from_boxed(Box::new([37, 41])),
        }]))),
    }
}
pub fn host_run() -> d::HostRun {
    d::HostRun {
        task: d::TaskLink::Stored { number: 7, status: Some(t::Status::Failed) },
        attempt: 11,
        kept: 13,
        listed: true,
        state: fleet::AttemptState::Kept,
        location: d::RunLocation::Adrift {
            channel: Some(17),
            until: Some(Wall::from_nanos(19)),
            remaining: Some(Duration::from_nanos(23)),
        },
        grace_until: Some(Wall::from_nanos(29)),
    }
}
pub fn deployment_document() -> d::Deployment {
    d::Deployment {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        readiness: List::with_capacity(0),
        readiness_cut: false,
        store: d::StoreFill { bytes: 31, mark: 37 },
        census: None,
        hosts: List::from_boxed(Box::new([d::Host {
            name: None,
            awaiting_logins: false,
            rule: jig_domain_core_vocabulary::HostRule::Open,
            programs: List::with_capacity(0),
            channel: 17,
            kind: fleet::HostKind::Engine,
            slots: 2,
            used: 1,
            free: 1,
            draining: true,
            connected: false,
            grace_until: Some(Wall::from_nanos(19)),
            grace_remaining: Some(Duration::from_nanos(23)),
            runs: List::from_boxed(Box::new([host_run()])),
            cut: true,
        }])),
        attempts: List::from_boxed(Box::new([host_run()])),
        accounts: List::from_boxed(Box::new([d::Account {
            list: None,
            facts: super::super::accounts::fact_rows(),
            provider_name: None,
            manage_usage: None,
            place: 0,
            uses: None,
            plan: false,
            number: 41,
            provider: Some(3),
            display: d::Text { bytes: b"Deployment account".as_slice().into(), cut: false },
            holder: jig_domain_accounts::Holder::Deployment,
            sources: List::with_capacity(0),
            generation: Some(43),
            state: d::AccountState::Later { until: Some(Wall::from_nanos(47)) },
            usable: false,
            expiring: true,
            attention: false,
            spent_until: Some(Wall::from_nanos(53)),
            runs: List::from_boxed(Box::new([chip()])),
            waiting: List::from_boxed(Box::new([chip()])),
            cut: true,
        }])),
        tasks: d::Capacity { used: 1, limit: 2 },
        workers: d::Capacity { used: 3, limit: 4 },
        attempts_capacity: d::Capacity { used: 5, limit: 6 },
        waiting_attempts: 7,
        proposals: d::Capacity { used: 8, limit: 9 },
        projects: List::from_boxed(Box::new([d::ProjectCapacity {
            project: 7,
            tasks: d::Capacity { used: 11, limit: 13 },
            proposals: 17,
        }])),
        grace_bound: Duration::from_nanos(59),
        hello_refusals: List::from_boxed(Box::new([d::HelloRefusalLine {
            channel: 61,
            at: Wall::from_nanos(67),
            slots: 2,
            stop_bound: Duration::from_nanos(71),
            why: fleet::HelloRefusal::Agent { why: fleet::AgentShortfall::Window { charter: 73, model: 79 } },
        }])),
        hosts_forgotten: 83,
        hello_refusals_forgotten: 89,
        person_room_refusals: List::from_boxed(Box::new([people::CapacityRefusal {
            key: people::RequestKey { person: 97, key: [3; 16] },
            project: Some(7),
            action: people::CapacityAction::Proposal { proposer: people::Proposer::Party(101), number: 103 },
            at: Wall::from_nanos(107),
            cause: people::CapacityCause::Limit { which: people::LimitKind::LiveTasks },
        }])),
        person_room_refusals_forgotten: 109,
        person_answer_retention: Duration::from_nanos(113),
        task_room_refusals: List::from_boxed(Box::new([t::CapacityRefusal {
            request: t::Capacity {
                project: Some(7),
                by: t::Party::Deployment { project: 7 },
                action: t::CapacityAction::Source { funder: t::Funder::Task(127) },
                cause: t::CapacityCause::Inbox { task: 131, messages: 2, bytes: 137 },
            },
            at: Wall::from_nanos(139),
        }])),
        task_room_refusals_forgotten: 149,
        connectors: List::from_boxed(Box::new([2, 3])),
        connectors_cut: true,
        cut: false,
    }
}
