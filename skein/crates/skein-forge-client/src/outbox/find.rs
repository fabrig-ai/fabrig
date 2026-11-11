use super::{Answer, Condition, Domain, Env, Error, Limits, Made, Outcome, Phase, Queue, Request, Time, api};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct Listing {
    pub(super) more: bool,
    pub(super) cut: bool,
    pub(super) now: Time,
}
pub(super) fn listed(
    d: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    phase: Phase,
    items: &[api::Summary],
    info: Listing,
    out: &mut Queue<Request>,
) {
    let Listing { more, cut, now } = info;
    match phase {
        Phase::Clock => {
            d.outbox.entries.get_mut(&number).expect("entry remains").clock = now;
            let phase = super::check_phase(&d.outbox.entries.get(&number).expect("entry remains").entry);
            super::next(d, number, phase);
        }
        Phase::Find { after, .. } => {
            let expected = super::key(&d.outbox.entries.get(&number).expect("entry remains").entry.effect.write)
                .expect("issue creation has key");
            let mut found = None;
            for item in items {
                if crate::identity::writer(
                    d,
                    d.outbox.entries.get(&number).expect("entry remains").entry.repository,
                    item.author,
                ) && super::same_key(item.key.as_deref(), expected)
                    && crate::identity::historical(d, expected)
                {
                    found = Some(item.number);
                    break;
                }
            }
            match found {
                Some(item) => super::finished(d, env, number, Made::Created(item), true, out),
                None if more => {
                    let w = d.outbox.entries.get_mut(&number).expect("entry remains");
                    let position = w.items.as_mut().expect("issue search has a position");
                    if position.next(items, cut, now).is_some() {
                        let page = position.page;
                        super::next(d, number, Phase::Find { page, after, skip: 0 });
                    } else {
                        super::retry::not_found(d, env, number, out);
                    }
                }
                None => super::retry::not_found(d, env, number, out),
            }
        }
        Phase::Queued
        | Phase::Check
        | Phase::Make
        | Phase::DispatchRuns { .. }
        | Phase::Ancestry { .. }
        | Phase::AfterUpdate
        | Phase::AfterMerge { .. } => {
            unreachable!("items answer only clock or issue find")
        }
    }
}
pub(super) fn comments_found(
    d: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    phase: Phase,
    comments: &[api::Comment],
    more: bool,
    out: &mut Queue<Request>,
) {
    let w = d.outbox.entries.get(&number).expect("find owns entry");
    let expected = super::key(&w.entry.effect.write).expect("comment creation has key");
    let mut found = None;
    let mut last = match phase {
        Phase::Find { after, .. } => after,
        Phase::Clock
        | Phase::Check
        | Phase::Queued
        | Phase::Make
        | Phase::DispatchRuns { .. }
        | Phase::Ancestry { .. }
        | Phase::AfterUpdate
        | Phase::AfterMerge { .. } => {
            unreachable!("comments answer find")
        }
    };
    for comment in comments {
        last = last.max(comment.id);
        if original(comment.provenance)
            && crate::identity::writer(d, w.entry.repository, comment.author)
            && super::same_key(comment.key.as_deref(), expected)
            && crate::identity::historical(d, expected)
        {
            found = Some(comment.id);
            break;
        }
    }
    match found {
        Some(comment) => super::finished(d, env, number, Made::Commented(comment), true, out),
        None if more => super::next(d, number, Phase::Find { page: 1, after: last, skip: 0 }),
        None => super::retry::not_found(d, env, number, out),
    }
}
pub(super) fn reviews_found(
    d: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    phase: Phase,
    reviews: &[api::Review],
    paging: (bool, bool),
    out: &mut Queue<Request>,
) {
    let (more, cut) = paging;
    let w = d.outbox.entries.get(&number).expect("find owns entry");
    let start = w.entry.start.expect("a recovery search has its durable start");
    let expected = super::key(&w.entry.effect.write).expect("review creation has key");
    let mut found = None;
    for review in reviews {
        if review.id > start.review
            && original(review.provenance)
            && crate::identity::writer(d, w.entry.repository, review.author)
            && super::same_key(review.key.as_deref(), expected)
            && crate::identity::historical(d, expected)
        {
            let head = match w.entry.effect.condition {
                Condition::Review { head } => head,
                Condition::None | Condition::Merge { .. } | Condition::Update { .. } => {
                    unreachable!("review condition admitted")
                }
            };
            if review.commit != head {
                super::finish(
                    d,
                    env,
                    number,
                    Outcome::Raced { made: Made::Reviewed(review.id), why: Error::Stale },
                    out,
                );
                return;
            }
            found = Some(review.id);
            break;
        }
    }
    let (page, skip) = match phase {
        Phase::Find { page, skip, .. } => (page, skip),
        Phase::Clock
        | Phase::Check
        | Phase::Queued
        | Phase::Make
        | Phase::DispatchRuns { .. }
        | Phase::Ancestry { .. }
        | Phase::AfterUpdate
        | Phase::AfterMerge { .. } => {
            unreachable!("reviews answer find")
        }
    };
    match found {
        Some(review) => super::finished(d, env, number, Made::Reviewed(review), true, out),
        None if more => super::next(
            d,
            number,
            Phase::Find {
                page: if cut { page } else { page.saturating_add(1) },
                after: 0,
                skip: if cut { skip.saturating_add(u32::try_from(reviews.len()).expect("bounded reviews")) } else { 0 },
            },
        ),
        None => super::retry::not_found(d, env, number, out),
    }
}
fn original(provenance: api::Provenance) -> bool {
    match provenance {
        api::Provenance::Original => true,
        api::Provenance::Revised | api::Provenance::Unknown => false,
    }
}
pub(super) fn ancestry_answered(
    d: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    phase: Phase,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let head = match phase {
        Phase::Ancestry { head, .. } => head,
        Phase::DispatchRuns { .. }
        | Phase::Clock
        | Phase::Check
        | Phase::Queued
        | Phase::Make
        | Phase::Find { .. }
        | Phase::AfterUpdate
        | Phase::AfterMerge { .. } => {
            unreachable!("ancestry answers update recovery")
        }
    };
    let contained = match result {
        Ok(Answer::Commits { commits, more }) => {
            if !commits.is_empty() {
                false
            } else if more {
                super::retry::failed(d, env, number, phase, Error::InvalidAnswer, out);
                return;
            } else {
                true
            }
        }
        Err(Error::TooLarge(_)) => false,
        result => {
            let error = match result {
                Err(error) => error,
                Ok(_) => Error::InvalidAnswer,
            };
            super::retry::failed(d, env, number, phase, error, out);
            return;
        }
    };
    if contained {
        super::finished(d, env, number, Made::Updated(head), true, out);
    } else {
        super::retry::not_found(d, env, number, out);
    }
}

/// A dispatch has no forge key: its writer, event and start time identify an observed run.
pub(super) fn runs_found(
    d: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    phase: Phase,
    runs: &[api::Run],
    more: bool,
    out: &mut Queue<Request>,
) {
    let w = d.outbox.entries.get(&number).expect("dispatch find entry");
    let start = w.entry.start.expect("find has start").at;
    for run in runs {
        if run.event == api::Event::Dispatch
            && crate::identity::writer(d, w.entry.repository, run.actor)
            && run.created >= start
        {
            super::finished(d, env, number, Made::Set, true, out);
            return;
        }
    }
    let Phase::DispatchRuns { head, page } = phase else {
        unreachable!("dispatch runs phase");
    };
    if more {
        super::next(d, number, Phase::DispatchRuns { head, page: page.saturating_add(1) });
    } else {
        super::retry::not_found(d, env, number, out);
    }
}
pub(super) fn rerun_found(d: &mut Domain, env: &Env<Limits>, number: u64, run: &api::Run, out: &mut Queue<Request>) {
    let w = d.outbox.entries.get(&number).expect("rerun find entry");
    let api::Write::Rerun { attempt: expected } = w.entry.effect.write else {
        unreachable!("rerun lookup");
    };
    if run.commit == expected.head {
        for job in &run.jobs {
            if job.id == expected.job && job.attempt > expected.attempt {
                super::finished(d, env, number, Made::Set, true, out);
                return;
            }
        }
    }
    super::retry::not_found(d, env, number, out);
}
