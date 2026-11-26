//! A landing keeps the run, not its dispatch's room (domain/landing-proof.md, 2.1; domain/landing.md, 15).
use super::{
    Domain, LandingEntry, LandingRow, Operation, ReceiptKey, Request, gate_key, input_commit, repository, runs,
};
use crate::{LandingDispatch, LandingFollow, Limits};
use alloc::boxed::Box;
use skein_forge::api;
use skein_forge_client::Entry;
use skein_lib::{List, Queue, Time};
use temper_engine_domain_forge_landing as policy;

fn new(entry: LandingEntry, dispatch: LandingDispatch, try_number: u16) -> LandingFollow {
    LandingFollow {
        entry,
        dispatch: Some(dispatch),
        start: None,
        since: None,
        wall: None,
        try_number,
        failed: false,
        reruns: Box::new([]),
    }
}

pub(super) fn keep(d: &Domain, row: &mut LandingRow, item: LandingEntry, write: &api::Write) {
    let mut owned = None;
    for (key, flight) in &d.landing.flights {
        if flight.task == row.landing.task && flight.entry == item.entry {
            owned = Some((key, flight));
            break;
        }
    }
    let Some((key, flight)) = owned else { return };
    let Some(crate::purpose::EffectPurpose::Landing { attempt, .. }) =
        crate::purpose::EffectPurpose::decode(&flight.purpose)
    else {
        unreachable!("a landing run has a landing purpose");
    };
    let mut kept = List::with_capacity(super::follow_slots(&d.landing.limits).expect("landing follow bound"));
    let mut current = None;
    for follow in row.follows.take().unwrap_or(Box::new([])) {
        if matches_key(d, row, &follow, key) {
            current = Some(follow);
        } else if follow.dispatch.is_some() || !same_position(follow.entry, item) {
            kept.push(follow).expect("bounded follows and marks");
        }
    }
    let follow = if let api::Write::Rerun { attempt: job } = write {
        let mut follow = current.expect("rerun of a followed dispatch");
        rerun(&mut follow, item, *job, &d.landing.limits);
        follow.try_number = attempt;
        follow
    } else {
        new(
            item,
            LandingDispatch {
                workflow: flight.workflow.clone(),
                branch: flight.branch.clone(),
                inputs: flight.inputs.clone(),
            },
            attempt,
        )
    };
    kept.push(follow).expect("bounded follows and marks");
    row.keep_follows(kept.into_boxed());
}

fn same_position(left: LandingEntry, right: LandingEntry) -> bool {
    left.member == right.member && left.gate == right.gate && left.preparation == right.preparation
}

fn matches_key(d: &Domain, row: &LandingRow, follow: &LandingFollow, key: &ReceiptKey) -> bool {
    let Some(dispatch) = &follow.dispatch else { return false };
    let Some((found, _)) = identify(d, row, follow.entry, dispatch) else { return false };
    found == *key
}

fn identify(
    d: &Domain,
    row: &LandingRow,
    item: LandingEntry,
    dispatch: &LandingDispatch,
) -> Option<(ReceiptKey, api::Commit)> {
    let key = input_commit(&dispatch.inputs, b"temper_key")?;
    let candidate = input_commit(&dispatch.inputs, b"temper_commit")?;
    let member = row.landing.members.get(usize::from(item.member))?;
    let gate = d.train_gates(row, repository(member)).get(usize::from(item.gate))?;
    Some((gate_key(row, item.member, item.gate, &gate.name, key), candidate))
}

fn rerun(follow: &mut LandingFollow, item: LandingEntry, job: api::JobAttempt, limits: &Limits) {
    let mut jobs = List::with_capacity(limits.client.rows);
    for old in &follow.reruns {
        if old.run == job.run && old.job != job.job {
            jobs.push(*old).expect("one attempt per run job");
        }
    }
    jobs.push(job).expect("one attempt per run job");
    follow.entry = item;
    follow.reruns = jobs.into_boxed();
    follow.since = None;
    follow.wall = None;
    follow.failed = false;
}

pub(super) fn progress(follow: &mut LandingFollow, entry: &Entry) {
    if follow.entry.operation == Operation::Dispatch
        && let Some(start) = entry.start
    {
        follow.start = Some(start.at);
    }
    if let Some(attempt) = entry.attempt {
        follow.since = Some(attempt.sent);
        follow.wall = Some(attempt.wall);
    }
}

/// The last completed mark of a position in this round is enough for the proof view.
pub(super) fn prune(row: &mut LandingRow, limits: &Limits) {
    let mut kept = List::with_capacity(super::follow_slots(limits).expect("landing follow bound"));
    let follows = row.follows.take().unwrap_or(Box::new([]));
    for follow in &follows {
        if follow.dispatch.is_none() {
            if follow.entry.run.is_none() || follow.entry.preparation != Some(row.landing.preparation) {
                continue;
            }
            let mut newer = false;
            for other in &follows {
                newer |= other.dispatch.is_none()
                    && other.entry.run.is_some()
                    && same_position(other.entry, follow.entry)
                    && other.entry.entry > follow.entry.entry;
            }
            if newer {
                continue;
            }
        }
        kept.push(follow.clone()).expect("bounded follows and marks");
    }
    row.keep_follows(kept.into_boxed());
}

pub(super) fn stop(follow: &mut LandingFollow) {
    follow.dispatch = None;
    follow.start = None;
    follow.since = None;
    follow.wall = None;
    follow.reruns = Box::new([]);
}

pub(crate) fn rebuild(d: &mut Domain) {
    let mut flights = List::with_capacity(super::slots(&d.landing.limits).expect("flight bound"));
    let mut upgraded = List::with_capacity(d.landing.limits.landings);
    for (_, row) in &d.landing.rows {
        if row.landing.phase == policy::Phase::Cancelled {
            continue;
        }
        for record in row.follows() {
            if let Some(flight) = rebuild_one(d, row, record) {
                flights.push(flight).expect("bounded flights");
            }
        }
        if let Some(row) = legacy(d, row, &mut flights) {
            upgraded.push(row).expect("bounded landings");
        }
    }
    for (key, flight) in flights.into_boxed() {
        d.landing.flights.insert(key, flight).expect("bounded flights");
    }
    // The next settlement/observation writes the upgraded row. If the start
    // cuts before then, the same outbox and row are still available to read.
    for row in upgraded.into_boxed() {
        d.landing.remember(row).expect("retained landing");
    }
}

fn rebuild_one(d: &Domain, row: &LandingRow, record: &LandingFollow) -> Option<(ReceiptKey, runs::Follow)> {
    let dispatch = record.dispatch.as_ref()?;
    let item = record.entry;
    let (key, candidate) = identify(d, row, item, dispatch)?;
    let purpose = crate::purpose::EffectPurpose::Landing {
        task: row.landing.task,
        member: item.member,
        gate: item.gate,
        operation: super::operation(item.operation),
        preparation: item.preparation.unwrap_or(row.landing.preparation),
        attempt: record.try_number,
    }
    .identity()
    .expect("landing purpose");
    let mut follow = runs::follow(
        row.landing.task,
        item.entry,
        purpose,
        key.repository,
        dispatch.workflow.clone(),
        dispatch.branch.clone(),
        candidate,
        dispatch.inputs.clone(),
    );
    follow.writes = Box::new([]);
    follow.start = record.start;
    follow.since = record.since;
    follow.wall = record.wall;
    if let Some(kept) = d.entries.get(&item.entry) {
        if item.operation == Operation::Dispatch
            && let Some(start) = kept.entry.start
        {
            follow.start = Some(start.at);
        }
        if let Some(attempt) = kept.entry.attempt {
            follow.since = Some(attempt.sent);
            follow.wall = Some(attempt.wall);
        }
        follow.waiting =
            !kept.held && (kept.entry.attempt.is_none() || d.flights.contains_key(&(row.landing.task, item.entry)));
    }
    let mut run = match item.run {
        Some(mark) => {
            follow.clock = mark.clock;
            Some(mark.id)
        }
        None => None,
    };
    if let Some(job) = record.reruns.last() {
        run = Some(job.run);
    }
    if record.failed {
        follow.outcome = Some(policy::Verdict::Fault(policy::Fault::Lost));
    }
    for rerun in &row.flake_reruns {
        if rerun.key != key || rerun.ended {
            continue;
        }
        let mut pending = List::with_capacity(16);
        for attempt in &rerun.pending {
            pending.push(api::Write::Rerun { attempt: *attempt }).expect("bounded jobs");
        }
        follow.writes = pending.into_boxed();
        if !rerun.pending.is_empty() {
            run = Some(rerun.run);
        }
        if rerun.redispatch {
            follow.outcome = Some(policy::Verdict::Fault(policy::Fault::Lost));
            runs::retry(&mut follow, 2);
            run = None;
        }
    }
    if let Some(id) = run {
        follow.run = Some(api::Run {
            id,
            number: 0,
            workflow: follow.workflow.clone(),
            commit: follow.commit,
            event: api::Event::Dispatch,
            actor: 0,
            state: api::RunState::Running,
            conclusion: None,
            url: Box::new([]),
            created: follow.start.unwrap_or(Time::ZERO),
            jobs: Box::new([]),
        });
    }
    Some((key, follow))
}

// Goes once dev.fabrig.ai has run this build (temper/docs/design/draft/archive/store-floor.md):
// reconstruct follows once from dispatch/rerun entries retained by the old writer.
fn legacy(d: &Domain, row: &LandingRow, flights: &mut List<(ReceiptKey, runs::Follow)>) -> Option<LandingRow> {
    let mut moved = List::with_capacity(super::follow_slots(&d.landing.limits).expect("landing follow bound"));
    // Newest first: an old redispatch for the same key supersedes its previous window.
    for item in row.entries.iter().rev() {
        if item.operation != Operation::Dispatch {
            continue;
        }
        let Some(kept) = d.entries.get(&item.entry) else { continue };
        let api::Write::Dispatch { workflow, branch, inputs } = &kept.entry.effect.write else { continue };
        let dispatch = LandingDispatch { workflow: workflow.clone(), branch: branch.clone(), inputs: inputs.clone() };
        let Some((key, candidate)) = identify(d, row, *item, &dispatch) else { continue };
        let mut known = false;
        for record in row.follows() {
            known |= record.entry.entry == item.entry || matches_key(d, row, record, &key);
        }
        for record in &moved {
            known |= matches_key(d, row, record, &key);
        }
        if known || already_proved(d, row, &key, inputs) {
            continue;
        }
        let member = row.landing.members.get(usize::from(item.member)).expect("dispatch member");
        let mut try_number = 1;
        if let Some(proof) = member.gates.get(usize::from(item.gate))
            && let policy::Proof::Dispatched { key: proof, try_ } = proof.proof
            && proof == key.key
        {
            try_number = try_;
        }
        let mut record = new(*item, dispatch, try_number);
        progress(&mut record, &kept.entry);
        for retry in &row.entries {
            if retry.entry <= item.entry || retry.operation != Operation::Rerun || !same_position(*retry, *item) {
                continue;
            }
            if let Some(kept) = d.entries.get(&retry.entry)
                && let api::Write::Rerun { attempt } = kept.entry.effect.write
                && attempt.head == candidate
            {
                rerun(&mut record, *retry, attempt, &d.landing.limits);
                progress(&mut record, &kept.entry);
            }
        }
        if let Some(flight) = rebuild_one(d, row, &record) {
            flights.push(flight).expect("bounded flights");
            moved.push(record).expect("bounded follows and marks");
        }
    }
    if moved.is_empty() {
        return None;
    }
    for record in row.follows() {
        moved.push(record.clone()).expect("bounded follows and marks");
    }
    let mut row = row.clone();
    row.keep_follows(moved.into_boxed());
    Some(row)
}

fn already_proved(d: &Domain, row: &LandingRow, key: &ReceiptKey, inputs: &[api::Input]) -> bool {
    if let Some(receipt) = d.receipt(key)
        && !super::superseded(d, row, key, receipt)
    {
        match super::tests_value(inputs) {
            None => true,
            Some(selected) => receipt.passed || super::selected_report(selected, receipt.failures.as_ref()),
        }
    } else {
        false
    }
}

pub(super) fn cancelled(d: &mut Domain, row: &mut LandingRow, out: &mut Queue<Request>) {
    let task = row.landing.task;
    let mut keys = List::with_capacity(d.landing.flights.len());
    for (key, follow) in &d.landing.flights {
        if follow.task == task {
            keys.push(key.clone()).expect("bounded flights");
        }
    }
    for key in keys.into_boxed() {
        d.landing.flights.remove(&key);
    }
    let mut faults = List::with_capacity(d.landing.faults.len());
    for (position, _) in &d.landing.faults {
        if position.0 == task {
            faults.push(*position).expect("bounded faults");
        }
    }
    for position in faults.into_boxed() {
        d.landing.faults.remove(&position);
    }
    for follow in row.follows_mut() {
        stop(follow);
    }
    prune(row, &d.landing.limits);
    wake_project(d, row.landing.project, task, out);
}

pub(super) fn wake_project(d: &Domain, project: u32, except: u64, out: &mut Queue<Request>) {
    for (task, row) in &d.landing.rows {
        if *task != except
            && row.landing.project == project
            && row.landing.phase != policy::Phase::Cancelled
            && (row.landing.phase != policy::Phase::Landed || super::following(d, *task))
        {
            super::emit(out, Request::WakeProcedure { task: *task });
        }
    }
}
