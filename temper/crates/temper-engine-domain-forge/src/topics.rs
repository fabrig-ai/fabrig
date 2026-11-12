//! Goal topic facts (jig's domain/connectors.md, 5.4; domain/forge/README.md, 5).
//! Subtree membership is durable in subscribers. File lists are disposable,
//! bounded and pinned by fresh pull reads around all pages. Missing facts
//! make overlap unknown. A head change refreshes the facts.
use crate::{
    Class, Limits, News, Request, Subscriber, Topic,
    domain::{self, Domain},
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Token};
use temper_engine_domain_forge_change as change;

/// One bounded file list pinned to an open pull's head.
#[derive(Hash, Debug)]
pub(crate) struct Files {
    pub head: skein_forge::api::Commit,
    pub paths: Option<Box<[Box<[u8]>]>>,
}
#[derive(Clone, Copy, Hash, Debug)]
enum Stage {
    Before,
    Page(u32, u32),
    After,
}
/// One coherent pull-file read chain; at most one per live change.
#[derive(Hash, Debug)]
pub(crate) struct Pending {
    pub(crate) task: u64,
    pub(crate) repository: skein_forge::api::Repository,
    pull: u64,
    head: skein_forge::api::Commit,
    stage: Stage,
    paths: List<Box<[u8]>>,
}

pub(crate) fn contains(tasks: &[u64], task: u64) -> bool {
    for number in tasks {
        if *number == task {
            return true;
        }
    }
    false
}

pub(crate) fn refresh(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let Some(row) = d.changes.get(&task) else { return };
    if d.forge_ended(row.repository.forge) {
        return;
    }
    let Some(head) = row.change.last_head else { return };
    let repository = row.repository;
    let pull = row.pull;
    let mut wanted = false;
    for (_, subscriber) in &d.subscriptions {
        if let Some(tasks) = &subscriber.goal_tasks
            && contains(tasks, task)
        {
            match &subscriber.topic {
                Topic::Landings { repository: home, .. } if *home == repository => wanted = true,
                Topic::Landings { .. } | Topic::Ci { .. } | Topic::Pull { .. } | Topic::Participation { .. } => {}
            }
        }
    }
    if !wanted {
        return;
    }
    let Some(pull) = pull else { return };
    if let Some(files) = d.goal_files.get(&task)
        && files.head == head
    {
        return;
    }
    for (_, pending) in &d.goal_file_reads {
        if pending.task == task {
            return;
        }
    }
    if d.goal_file_reads.len() == env.limits.changes {
        return;
    }
    let Some(sequence) = d.sequence.checked_add(1) else { return };
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 63));
    let pending = Pending {
        task,
        repository,
        pull,
        head,
        stage: Stage::Before,
        paths: List::with_capacity(env.limits.paths_per_subscription),
    };
    d.goal_file_reads.insert(owner, pending).expect("one read per change");
    domain::child(
        d,
        env,
        skein_forge_client::Event::Read {
            owner,
            repository,
            read: skein_forge::api::Read::Pull { number: pull, ci: false },
        },
        out,
    );
}

pub(crate) fn answer(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<skein_forge::api::Answer, skein_forge::api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = d.goal_file_reads.remove(&owner) else { return };
    let next = match pending.stage {
        Stage::Before => match result {
            Ok(skein_forge::api::Answer::Pull(pull))
                if pull.number == pending.pull
                    && pull.commit == pending.head
                    && pull.state == skein_forge::api::State::Open =>
            {
                pending.stage = Stage::Page(1, 0);
                Some(skein_forge::api::Read::PullFiles {
                    number: pending.pull,
                    head: pending.head,
                    page: 1,
                    skip: 0,
                    sides: false,
                })
            }
            Ok(_) | Err(_) => None,
        },
        Stage::Page(page, skip) => match result {
            Ok(skein_forge::api::Answer::PullFiles { head, files, more, cut }) if head == pending.head => {
                let empty = files.is_empty();
                let kept = u32::try_from(files.len()).expect("bounded files");
                for file in files {
                    if file.path.len() > skein_lib::usize_of(env.limits.name_bytes)
                        || pending.paths.push(file.path).is_err()
                    {
                        finish(d, pending, None);
                        return;
                    }
                }
                if more {
                    if (!cut && page >= env.limits.paths_per_subscription) || empty {
                        finish(d, pending, None);
                        return;
                    }
                    let Some(page) = (if cut { Some(page) } else { page.checked_add(1) }) else {
                        finish(d, pending, None);
                        return;
                    };
                    let skip = if cut { skip.saturating_add(kept) } else { 0 };
                    pending.stage = Stage::Page(page, skip);
                    Some(skein_forge::api::Read::PullFiles { number: pending.pull, head, page, skip, sides: false })
                } else {
                    pending.stage = Stage::After;
                    Some(skein_forge::api::Read::Pull { number: pending.pull, ci: false })
                }
            }
            Ok(_) | Err(_) => None,
        },
        Stage::After => {
            let paths = match result {
                Ok(skein_forge::api::Answer::Pull(pull))
                    if pull.number == pending.pull
                        && pull.commit == pending.head
                        && pull.state == skein_forge::api::State::Open =>
                {
                    Some(core::mem::replace(&mut pending.paths, List::with_capacity(0)).into_boxed())
                }
                Ok(_) | Err(_) => None,
            };
            finish(d, pending, paths);
            return;
        }
    };
    match next {
        Some(read) => {
            let repository = pending.repository;
            d.goal_file_reads.insert(owner, pending).expect("replaces pending read");
            domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
        }
        None => finish(d, pending, None),
    }
}
fn finish(d: &mut Domain, pending: Pending, paths: Option<Box<[Box<[u8]>]>>) {
    d.goal_files.insert(pending.task, Files { head: pending.head, paths }).expect("one file list per change");
}

pub(crate) fn classify(d: &Domain, subscriber: &Subscriber, news: &News) -> Class {
    let (after, files) = match news {
        News::Landing { after, files, .. } => (after, files),
        News::Ci { .. } | News::Changed { .. } | News::Red(_) => return Class::Wakes,
    };
    let Some(tasks) = &subscriber.goal_tasks else { return domain::classify(d, subscriber, news) };
    if let Some(owner) = d.landed.get(after)
        && contains(tasks, *owner)
    {
        return Class::Kept;
    }
    let Some(files) = files else { return Class::Wakes };
    if domain::overlap(files, &subscriber.paths) {
        return Class::Wakes;
    }
    let mut unknown = false;
    for task in tasks {
        let Some(row) = d.changes.get(task) else { continue };
        match row.change.state {
            change::State::Landed { .. }
            | change::State::Red { .. }
            | change::State::Cleared { .. }
            | change::State::Uncleared { .. }
            | change::State::NoChange { .. }
            | change::State::Ended => continue,
            change::State::Producing { .. }
            | change::State::Opening { .. }
            | change::State::Recreating { .. }
            | change::State::Reopening { .. }
            | change::State::Checking { .. }
            | change::State::Judging { .. }
            | change::State::Gating { .. }
            | change::State::CancellingGates { .. }
            | change::State::Queued { .. }
            | change::State::First { .. }
            | change::State::Ready { .. }
            | change::State::Preparing { .. }
            | change::State::Updating { .. }
            | change::State::WaitingProducer { .. }
            | change::State::Resolving { .. }
            | change::State::Repairing { .. }
            | change::State::Landing { .. }
            | change::State::Held { .. } => {}
        }
        if row.pull.is_none() {
            if subscriber.paths.is_empty() {
                unknown = true;
            }
            continue;
        }
        match &subscriber.topic {
            Topic::Landings { repository, branch } if row.repository == *repository && row.base == *branch => {}
            Topic::Landings { .. } | Topic::Ci { .. } | Topic::Pull { .. } | Topic::Participation { .. } => continue,
        }
        let cached = match d.goal_files.get(task) {
            Some(cached) if Some(cached.head) == row.change.last_head => cached.paths.as_ref(),
            Some(_) | None => None,
        };
        match cached {
            Some(paths) if domain::overlap(files, paths) => return Class::Wakes,
            Some(_) => {}
            None => unknown = true,
        }
    }
    if unknown { Class::Wakes } else { Class::Kept }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_forge::api::{self, Answer};

    #[test]
    fn file_collection_resumes_cuts_before_advancing_and_stays_bounded() {
        for capacity in [2, 4] {
            let mut d = crate::tests::fixtures::connected_domain();
            let env = crate::tests::env();
            let owner = Token::new(99);
            let repository = api::Repository { forge: 1, repository: 2 };
            d.goal_file_reads
                .insert(
                    owner,
                    Pending {
                        task: 7,
                        repository,
                        pull: 1,
                        head: [1; 32],
                        stage: Stage::Before,
                        paths: List::with_capacity(capacity),
                    },
                )
                .unwrap();
            let mut out = Queue::uncapped();
            let mut pull = crate::tests::fixtures::open_pull();
            pull.commit = [1; 32];
            answer(&mut d, &env, owner, Ok(Answer::Pull(pull)), &mut out);
            for (path, cut, more, next_page, next_skip) in [
                (b"a", true, true, 1, 1),
                (b"b", true, true, 1, 2),
                (b"c", false, true, 2, 0),
                (b"d", false, false, 0, 0),
            ] {
                let pending = d.goal_file_reads.get(&owner).unwrap();
                let Stage::Page(page, skip) = pending.stage else { panic!("file page") };
                let (call, op) = crate::tests::fixtures::next_call(&mut d);
                assert_eq!(
                    op,
                    api::Op::Read(api::Read::PullFiles { number: 1, head: [1; 32], page, skip, sides: false })
                );
                crate::tests::fixtures::answer(
                    &mut d,
                    call,
                    Ok(Answer::PullFiles {
                        head: [1; 32],
                        files: Box::new([api::File { path: Box::from(path.as_slice()), before: None, after: None }]),
                        more,
                        cut,
                    }),
                );
                if !d.goal_file_reads.contains_key(&owner) {
                    assert_eq!(capacity, 2);
                    assert!(d.goal_files.get(&7).unwrap().paths.is_none());
                    break;
                }
                let pending = d.goal_file_reads.get(&owner).unwrap();
                if next_page == 0 {
                    assert!(matches!(pending.stage, Stage::After));
                } else {
                    assert!(matches!(pending.stage, Stage::Page(page, skip) if page == next_page && skip == next_skip));
                }
            }
        }
    }
}
