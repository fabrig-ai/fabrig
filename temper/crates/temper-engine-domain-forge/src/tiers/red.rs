//! A red reaches its goals with bounded log ends (temper/docs/design/domain/after-landing.md, 7 and 10).
use crate::{Class, Domain, JobLog, Limits, News, RedNews, RedOn, Request, TierRow, Topic};
use alloc::boxed::Box;
use skein_forge::api::{self, Commit, Repository};
use skein_lib::{Env, List, Queue, Token};
use temper_engine_domain_forge_tiers as p;

/// Disposable reads: the kept final receipt starts them again after a restart.
#[derive(Debug)]
pub(super) enum Fetch {
    Run { id: u64 },
    Jobs { attempts: Box<[api::JobAttempt]>, logs: Box<[JobLog]>, next: usize, share: u32 },
    Done(Box<[JobLog]>),
}

pub(super) fn landings(d: &Domain, row: &TierRow, verdict: &p::Verdict) -> Box<[(Commit, p::Failures)]> {
    let run = row.record.run.as_ref().expect("judged range");
    let mut reds = List::with_capacity(u32::try_from(run.landings.len()).expect("bounded range"));
    for (landing, blame) in run.landings.iter().zip(&verdict.landings) {
        let failing = match blame {
            p::Blame::Cleared => continue,
            p::Blame::Owns(failing) | p::Blame::Suspect(failing) => failing,
        };
        let mut prior_red = false;
        if let Some(verification) = d.verifications.get(&(row.repository, landing.commit)) {
            for tier in &verification.tiers {
                if tier.tier != row.tier {
                    continue;
                }
                if let Some(verdict) = &tier.verdict {
                    match verdict.blame {
                        p::Blame::Owns(_) | p::Blame::Suspect(_) => prior_red = true,
                        p::Blame::Cleared => {}
                    }
                }
            }
        }
        if !prior_red {
            reds.push((landing.commit, failing.clone())).expect("one red per landing");
        }
    }
    reds.into_boxed()
}

pub(super) fn logs(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &TierRow,
    out: &mut Queue<Request>,
) -> Option<Box<[JobLog]>> {
    let key = (row.repository, row.tier.clone());
    match d.tier_ops.reds.remove(&key) {
        Some(Fetch::Done(logs)) => Some(logs),
        Some(fetch) => {
            d.tier_ops.reds.insert(key, fetch).expect("replaces red reads");
            None
        }
        None => {
            let id = row.result.as_ref().expect("kept final receipt").run.id;
            d.tier_ops.reds.insert(key.clone(), Fetch::Run { id }).expect("one red read chain per tier");
            ask(d, env, key, api::Read::Run { run: id }, out);
            None
        }
    }
}

fn ask(d: &mut Domain, env: &Env<Limits>, key: (Repository, Box<[u8]>), read: api::Read, out: &mut Queue<Request>) {
    d.sequence = d.sequence.checked_add(1).expect("tier read sequence");
    let owner = Token::new(d.sequence | (1_u64 << 59));
    let repository = key.0;
    d.tier_ops.reads.insert(owner, key).expect("one log read per tier alongside its key walk");
    crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
}

pub(super) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    key: (Repository, Box<[u8]>),
    answer: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let fetch = d.tier_ops.reds.remove(&key).expect("red read chain");
    if d.stopping {
        return;
    }
    let fetch = match fetch {
        Fetch::Run { id } => match answer {
            Ok(api::Answer::Run(run)) if run.id == id => jobs(run, env.limits.brief_bytes),
            Ok(_) | Err(_) => Fetch::Done(Box::new([])),
        },
        Fetch::Jobs { attempts, mut logs, next, share } => {
            let attempt = attempts.get(next).expect("pending job attempt");
            let end = match answer {
                Ok(api::Answer::Job { attempt: read, log, .. }) if read == *attempt && !log.is_empty() => {
                    let start = log.len().saturating_sub(usize::try_from(share).expect("log share fits usize"));
                    Some(Box::from(log.get(start..).expect("log end")))
                }
                Ok(_) | Err(_) => None,
            };
            logs.get_mut(next).expect("pending job log").end = end;
            Fetch::Jobs { attempts, logs, next: next.checked_add(1).expect("bounded jobs"), share }
        }
        Fetch::Done(_) => unreachable!("completed logs have no read in flight"),
    };
    let (fetch, read) = match fetch {
        Fetch::Jobs { attempts, logs, next, share } => match attempts.get(next) {
            Some(attempt) => {
                let read = api::Read::Job { attempt: *attempt, max_bytes: share };
                (Fetch::Jobs { attempts, logs, next, share }, Some(read))
            }
            None => (Fetch::Done(logs), None),
        },
        Fetch::Done(logs) => (Fetch::Done(logs), None),
        Fetch::Run { .. } => unreachable!("run answer starts jobs"),
    };
    d.tier_ops.reds.insert(key.clone(), fetch).expect("replaces red read chain");
    if let Some(read) = read {
        ask(d, env, key, read, out);
    } else if let Some(row) = d.tiers.get(&key)
        && let Some(task) = row.task
    {
        out.push(Request::WakeProcedure { task });
    }
}

fn jobs(run: api::Run, bytes: u32) -> Fetch {
    let count = u32::try_from(run.jobs.len()).expect("bounded run jobs");
    let mut attempts = List::with_capacity(count);
    let mut logs = List::with_capacity(count);
    for job in run.jobs {
        if job.conclusion == Some(api::Conclusion::Failure) {
            attempts
                .push(api::JobAttempt { head: run.commit, run: run.id, job: job.id, attempt: job.attempt })
                .expect("one attempt per failed job");
            logs.push(JobLog { name: job.name, end: None }).expect("one log per failed job");
        }
    }
    let share = bytes.checked_div(attempts.len().max(1)).expect("positive failed job count");
    Fetch::Jobs { attempts: attempts.into_boxed(), logs: logs.into_boxed(), next: 0, share }
}

pub(super) fn landed(
    d: &Domain,
    row: &TierRow,
    reds: &[(Commit, p::Failures)],
    logs: &[JobLog],
    out: &mut Queue<Request>,
) {
    let tier = super::configured(d, row).expect("configured tier");
    let branch = super::followed(d, row.repository, &tier).expect("landing tier branch");
    let topic = Topic::Landings { repository: row.repository, branch: branch.clone() };
    for (commit, failing) in reds {
        let Some(verification) = d.verifications.get(&(row.repository, *commit)) else { continue };
        let mut goals = List::with_capacity(d.subscriptions.capacity());
        for (_, subscriber) in &d.subscriptions {
            if subscriber.topic != topic {
                continue;
            }
            if let Some(tasks) = &subscriber.goal_tasks {
                for change in &verification.changes {
                    if tasks.contains(change) {
                        goals.push(subscriber.task).expect("bounded subscribers");
                        break;
                    }
                }
            }
        }
        let result = row.result.as_ref().expect("kept final receipt");
        let news = News::Red(RedNews {
            tier: row.tier.clone(),
            run: row.record.run.as_ref().expect("judged range").number,
            url: if result.run.url.is_empty() { None } else { Some(result.run.url.clone()) },
            on: RedOn::Landing { commit: *commit, branch: branch.clone(), failing: failing.clone() },
            logs: Box::from(logs),
        });
        tell(d, row.repository, &branch, goals.as_slice(), news, Class::Wakes, out);
    }
}

/// Tell each task through its subscription, independent of the news's red variant.
pub(crate) fn tell(
    d: &Domain,
    repository: Repository,
    branch: &[u8],
    subscribers: &[u64],
    news: News,
    class: Class,
    out: &mut Queue<Request>,
) {
    let topic = Topic::Landings { repository, branch: Box::from(branch) };
    for (_, subscriber) in &d.subscriptions {
        if subscriber.topic == topic && subscribers.contains(&subscriber.task) {
            out.push(Request::News {
                task: subscriber.task,
                subscription: subscriber.number,
                topic: topic.clone(),
                news: news.clone(),
                class,
            });
        }
    }
}

pub(super) fn forget(d: &mut Domain, row: &TierRow) {
    let key = (row.repository, row.tier.clone());
    let reading = match d.tier_ops.reds.remove(&key) {
        Some(Fetch::Run { .. } | Fetch::Jobs { .. }) => true,
        Some(Fetch::Done(_)) | None => false,
    };
    if reading {
        let mut walker = None;
        if let Some(walk) = d.tier_ops.walks.get(&key) {
            walker = Some(walk.owner);
        }
        let mut owner = None;
        for (token, read_key) in &d.tier_ops.reads {
            if read_key == &key && Some(*token) != walker {
                owner = Some(*token);
            }
        }
        if let Some(owner) = owner {
            d.tier_ops.reads.remove(&owner);
        }
    }
}
