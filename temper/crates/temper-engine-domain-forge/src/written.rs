//! Order a late write record against a design head beside it (domain/forge/branches.md, 3).
use super::read_line::{self, Waiter};
use super::{
    Domain, Env, Limits, List, LostReadState, LostWriterKind, Name, PendingLost, Queue, Request, Token,
    ancestry_status, branch_written, child, confirm_written_head, design, lost_release, save_hold, to_client,
};
use skein_forge::api::{Answer, Commit, Error};

pub(super) fn defer(
    d: &mut Domain,
    env: &Env<Limits>,
    name: &Name,
    commit: Commit,
    task: u64,
    out: &mut Queue<Request>,
) -> bool {
    let Some(hold) = d.holds.get(name) else { return false };
    let head = match hold.beside {
        Some(beside) => beside.head,
        None => {
            if super::repository_fresh(d, super::repository_reads::repository(name)) {
                return false;
            }
            let Some(head) = d.branch_head(name) else { return false };
            head
        }
    };
    if d.branch_head(name) != Some(head) || commit == head {
        return false;
    }
    let hold = d.holds.get_mut(name).expect("held branch");
    hold.beside = Some(crate::HeadBeside { head, waiting: Some(commit) });
    save_hold(out, hold.clone());
    if let Some(resource) = to_client(name, &env.limits) {
        child(d, env, skein_forge_client::Event::Writer { resource, taken: true }, out);
    }
    compare(d, env, name, task, out);
    true
}

fn compare(d: &mut Domain, env: &Env<Limits>, name: &Name, task: u64, out: &mut Queue<Request>) {
    let Some(hold) = d.holds.get(name) else { return };
    let Some(beside) = hold.beside else { return };
    let Some(write) = beside.waiting else { return };
    let Some(resource) = to_client(name, &env.limits) else { return };
    let sequence = d.sequence.checked_add(1).expect("written-head comparison sequence");
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 62));
    d.lost
        .insert(
            owner,
            PendingLost {
                task,
                attempt: 0,
                name: name.clone(),
                baseline: Some(beside.head),
                comparison: Some((write, beside.head)),
                state: LostReadState::Asked,
                last_answer: None,
                kind: LostWriterKind::Written,
            },
        )
        .expect("one waiting write per held branch");
    child(
        d,
        env,
        skein_forge_client::Event::Read {
            owner,
            repository: resource.repository,
            read: skein_forge::api::Read::Commits { before: beside.head, after: write, page: 1 },
        },
        out,
    );
}

pub(super) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut pending: PendingLost,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let Some(hold) = d.holds.get(&pending.name) else { return };
    if !design::pending_writer(hold, &pending) {
        return;
    }
    let waiting = match &result {
        Err(Error::Busy) => Some(LostReadState::WaitingForRoom),
        Err(Error::NotConnected) => Some(LostReadState::WaitingForConnection),
        Ok(_) | Err(_) => None,
    };
    if let Some(state) = waiting {
        pending.state = state;
        d.lost.insert(owner, pending).expect("waiting comparison keeps its slot");
        if state == LostReadState::WaitingForRoom {
            read_line::join(d, Waiter::Lost(owner), env.now, out);
        }
        return;
    }
    let Some((write, _)) = pending.comparison else { unreachable!("written comparison has both heads") };
    let Some(resource) = to_client(&pending.name, &env.limits) else { return };
    if ancestry_status(result) != temper_engine_domain_forge_change::Status::Passed {
        confirm_written_head(d, env, resource.clone(), pending.name.clone(), write, pending.task, out);
    }
    let hold = d.holds.get_mut(&pending.name).expect("comparison's hold");
    hold.beside = None;
    save_hold(out, hold.clone());
    let taken = branch_written(d, &pending.name, None);
    child(d, env, skein_forge_client::Event::Writer { resource, taken }, out);
    lost_release(d, env, &pending.name, pending.task, out);
}

pub(super) fn cleanup(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut ended = List::with_capacity(env.limits.holds);
    for (name, hold) in &d.holds {
        if hold.beside.is_some() && !design::waiting(hold) && !design::other_written(d, name, None) {
            ended.push(name.clone()).expect("bounded holds");
        }
    }
    for name in &ended {
        let hold = d.holds.get_mut(name).expect("collected hold");
        hold.beside = None;
        save_hold(out, hold.clone());
    }
}

pub(super) fn restart(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    cleanup(d, env, out);
    let mut waiting = List::with_capacity(env.limits.holds);
    for (name, hold) in &d.holds {
        if design::waiting(hold) {
            waiting.push((name.clone(), hold.task)).expect("bounded holds");
        }
    }
    for (name, task) in &waiting {
        compare(d, env, name, *task, out);
    }
}
