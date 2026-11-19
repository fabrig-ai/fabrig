use super::w;
use skein_lib::Duration;
use skein_lib::List;
#[must_use]
pub fn run_state_record() -> w::RunState {
    w::RunState { task: 7, attempt: None, committed_turn: 7, writing: None, call: None }
}
#[must_use]
pub fn run_text_record() -> w::RunText {
    w::RunText {
        writing: None,
        attempt: 7,
        began: None,
        turn: w::ProvisionalTurnName { attempt: 7, number: 7, retry: 7 },
        from: 7,
        through: 7,
        text: Box::from([]),
        cut: false,
        call: None,
    }
}
#[must_use]
pub fn current_call_record() -> w::CurrentCall {
    w::CurrentCall {
        identity: w::CallIdentity { owner: 7, sequence: 7, retry: 7, id: Box::from([]) },
        tool: Box::from([]),
        input: Box::from([]),
        cut: false,
        began: None,
    }
}
#[must_use]
pub fn tasks_record() -> w::Tasks {
    w::Tasks { ended_mark: 0, rows: Box::from([]), more: false }
}
#[must_use]
pub fn owner_deployment_record() -> w::Deployment {
    w::Deployment {
        worker_settings: List::with_capacity(0),
        cut: false,
        connectors_cut: false,
        providers_cut: false,
        readiness_cut: false,
        providers: Box::new([]),
        key_bytes: 256,
        readiness: Box::new([]),
        store: w::StoreFill { bytes: 7, mark: 7 },
        census: None,
        hosts: Box::from([]),
        attempts: Box::from([]),
        accounts: Box::from([]),
        tasks: w::Capacity { used: 7, limit: 7 },
        workers: w::Capacity { used: 7, limit: 7 },
        attempts_capacity: w::Capacity { used: 7, limit: 7 },
        waiting_attempts: 7,
        proposals: w::Capacity { used: 7, limit: 7 },
        projects: List::with_capacity(0),
        grace_bound: Duration::from_nanos(19),
        hello_refusals: List::with_capacity(0),
        hosts_forgotten: 7,
        hello_refusals_forgotten: 7,
        person_room_refusals: List::with_capacity(0),
        person_room_refusals_forgotten: 7,
        person_answer_retention: Duration::from_nanos(19),
        task_room_refusals: List::with_capacity(0),
        task_room_refusals_forgotten: 7,
        connectors: List::with_capacity(0),
    }
}
#[must_use]
pub fn owner_tree_record() -> w::Tree {
    w::Tree {
        root: super::chip_record(),
        rows: Box::from([]),
        folded: List::with_capacity(0),
        held_below: 19,
        next: None,
    }
}
#[must_use]
pub fn owner_inbox_record() -> w::Inbox {
    w::Inbox {
        party: 19,
        needs: Box::from([]),
        updates: Box::from([]),
        next: None,
        cached_needs: 7,
        cached_updates: w::Count { value: 9, limited: true },
        results_seen: 13,
        updates_time_complete: false,
        selection: Some(Box::from([])),
    }
}
