use super::{
    Alarm, Answer, Box, Commit, Delivery, Domain, Echo, Env, Error, Fresh, Limits, List, Phase, Priority, Progress,
    Queue, Repository, Request, Resource, Scan, Sent, State, Stored, Time, What, api,
};
use skein_lib::usize_of;

pub(crate) fn answered_repository(
    d: &mut Domain,
    env: &Env<Limits>,
    key: Repository,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let polled = super::poll_at(&d.keep, &env.limits, env.now, key, env.limits.poll);
    let row = d.keep.repos.get_mut(&key).expect("call owns repository");
    if !row.active {
        d.keep.repos.remove(&key).expect("inactive repository exists");
        return;
    }
    if core::mem::replace(&mut row.dropping, false) {
        return;
    }
    let mut progress = match row.state {
        Scan::Busy(progress) => progress,
        Scan::Off => return,
        Scan::Idle | Scan::Due(_) | Scan::Waiting(_) => unreachable!("repository terminal while busy"),
    };
    let Ok(answer) = result else {
        wait_repository(d, env, key, progress);
        return;
    };
    match answer {
        Answer::Items { items, more, cut, now } => {
            let row = d.keep.repos.get_mut(&key).expect("repository remains");
            row.record.clock = row.record.clock.max(now);
            let begun = progress.begun.unwrap_or(now);
            progress.begun = Some(begun);
            for item in &items {
                progress.newest = progress.newest.max(item.updated);
                progress.moved |= item.updated >= begun;
                listed(d, env, key, item, now, out);
            }
            let row = d.keep.repos.get_mut(&key).expect("repository remains");
            if more && !progress.bootstrap {
                let Some(next) = next_page(progress, &items, cut, now) else {
                    let restart = Progress::first(row.record.mark);
                    wait_repository(d, env, key, restart);
                    return;
                };
                row.state = Scan::Due(next);
            } else {
                row.record.mark = Some(
                    (if progress.bootstrap {
                        now
                    } else if progress.moved {
                        progress.tied.unwrap_or(progress.newest)
                    } else {
                        progress.newest
                    })
                    .min(now),
                );
                row.retry = skein_lib::Duration::ZERO;
                row.state = Scan::Idle;
                row.fresh = Fresh::Complete;
                let next = if row.hinted { env.now.saturating_add(env.limits.hinted) } else { polled };
                row.hinted = false;
                d.alarms.arm(Alarm::Repository(key), next).expect("one repository alarm");
            }
            out.push(Request::Save { record: Stored::Repository(row.record) });
        }
        Answer::Item { .. }
        | Answer::Pull(_)
        | Answer::Reviews { .. }
        | Answer::Statuses { .. }
        | Answer::Remarks { .. }
        | Answer::Commit(_)
        | Answer::Branches(_)
        | Answer::Compare { .. }
        | Answer::PullFiles { .. }
        | Answer::Checks(_)
        | Answer::Directory { .. }
        | Answer::File { .. }
        | Answer::Job { .. }
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators { .. }
        | Answer::Permission(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Merged(_)
        // Only a merge is answered Queued; it never reaches a read.
        | Answer::Queued
        | Answer::Branch(_)
        | Answer::Runs { .. }
        | Answer::Run(_)
        | Answer::Tree { .. }
            | Answer::Commits { .. }
        | Answer::Located { .. }
        | Answer::Done => unreachable!("listing answer shape"),
    }
}
fn next_page(mut progress: Progress, items: &[api::Summary], cut: bool, now: Time) -> Option<Progress> {
    if !progress.position.next(items, cut, now)? {
        progress.tied = Some(progress.tied.unwrap_or(progress.position.since).min(now));
    }
    Some(progress)
}
fn wait_repository(d: &mut Domain, env: &Env<Limits>, key: Repository, progress: Progress) {
    let row = d.keep.repos.get_mut(&key).expect("repository remains");
    row.state = Scan::Waiting(progress);
    let wait = super::retry_wait(&mut row.retry, &env.limits);
    d.alarms.arm(Alarm::Repository(key), env.now.saturating_add(wait)).expect("one repository alarm");
}
fn listed(
    d: &mut Domain,
    env: &Env<Limits>,
    repo: Repository,
    item: &api::Summary,
    now: Time,
    out: &mut Queue<Request>,
) {
    for key in super::keys(&d.keep) {
        let row = d.keep.live.get_mut(&key).expect("bounded key remains");
        let key = &key;
        if key.repository != repo || !row.active {
            continue;
        }
        let number = match key.what {
            What::Pull(number) | What::Issue(number) => number,
            What::Repository | What::Branch(_) => continue,
        };
        // Inclusive pages can repeat a time; reread an equal timestamp too.
        // The hint/cursor and response equality suppress duplicate news.
        if item.number == number
            && (item.updated > row.record.listed
                || row.record.cached.item.as_ref() != Some(item)
                || item.updated >= now)
        {
            row.record.listed = item.updated;
            out.push(Request::Save { record: Stored::Live(row.record.clone()) });
            super::poll::nudge_resource(&mut d.alarms, env, key, row);
        }
    }
}
pub(crate) fn answered_resource(
    d: &mut Domain,
    env: &Env<Limits>,
    key: &Resource,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let row = d.keep.live.get_mut(key).expect("call owns resource");
    if !row.active {
        d.keep.live.remove(key).expect("inactive resource exists");
        return;
    }
    let phase = match row.state {
        State::Busy(phase, _) => phase,
        State::Idle | State::Due(_, _) | State::Waiting(_, _) => unreachable!("resource terminal while busy"),
    };
    // A head or an absence read before this client's own push cannot judge
    // that push; a fresh read does. Other failures keep their backoff.
    let before_push = core::mem::replace(&mut row.read, Sent::Current) == Sent::BeforePush;
    let stale = match &result {
        Ok(_) => before_push,
        Err(error) => before_push && *error == Error::Missing,
    };
    if stale {
        row.state = State::Due(Phase::Branch, Priority::Fresh);
        return;
    }
    match phase {
        Phase::BranchAncestry { before, after } => {
            branch_ancestry_answered(d, env, key, before, after, result, out);
            return;
        }
        Phase::Object | Phase::Pull | Phase::Reviews(_, _) | Phase::Branch => {}
    }
    let answer = match result {
        Ok(answer) => answer,
        Err(error) => {
            failed_resource(d, env, key, phase, error, out);
            return;
        }
    };
    match answer {
        Answer::Item { item, comments, more } => item_answered(d, env, key, item, comments, more, out),
        Answer::Pull(pull) => {
            let row = d.keep.live.get_mut(key).expect("resource remains");
            super::taken(row, phase);
            row.presence = super::Presence::Present;
            let changed = row.record.cached.pull.as_ref() != Some(&pull);
            row.record.cached.pull = Some(pull.clone());
            out.push(Request::Save { record: Stored::Live(row.record.clone()) });
            if changed {
                row.interval = env.limits.poll;
                out.push(Request::Changed { resource: key.clone(), result: Ok(Answer::Pull(pull)) });
            }
            if row.record.watch.participating && row.inbox.reviews_needed {
                row.state = State::Due(Phase::Reviews(row.record.review_page, 0), super::priority(row));
            } else {
                rest_resource(d, env, key);
            }
        }
        Answer::Reviews { reviews, more, cut } => reviews_answered(d, env, key, phase, reviews, (more, cut), out),
        Answer::Commit(commit) => {
            super::taken(d.keep.live.get_mut(key).expect("resource remains"), phase);
            branch_answered(d, env, key, commit, out);
        }
        Answer::Compare { .. }
        | Answer::Items { .. }
        | Answer::Runs { .. }
        | Answer::Run(_)
        | Answer::Tree { .. }
            | Answer::Commits { .. }
        | Answer::Located { .. }
        | Answer::Branches(_)
        | Answer::Statuses { .. }
        | Answer::Remarks { .. }
        | Answer::PullFiles { .. }
        | Answer::Checks(_)
        | Answer::Directory { .. }
        | Answer::File { .. }
        | Answer::Job { .. }
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators { .. }
        | Answer::Permission(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Merged(_)
        // Only a merge is answered Queued; it never reaches a read.
        | Answer::Queued
        | Answer::Branch(_)
        | Answer::Done => unreachable!("resource operation answer shape"),
    }
}
fn item_answered(
    d: &mut Domain,
    env: &Env<Limits>,
    key: &Resource,
    item: api::Summary,
    comments: Box<[api::Comment]>,
    more: bool,
    out: &mut Queue<Request>,
) {
    let row = d.keep.live.get(key).expect("resource remains");
    let after = row.record.comment_after;
    let participating = row.record.watch.participating;
    if more && participating && row.record.comment_page >= env.limits.inbox {
        failed_resource(d, env, key, Phase::Object, Error::TooLarge(api::Excess::unnamed()), out);
        return;
    }
    let mut stamps = copied_stamps(&row.record.comments, env.limits.inbox);
    if participating {
        for comment in &comments {
            if !remember(&mut stamps, Delivery { id: comment.id, revision: comment.revision }) {
                failed_resource(d, env, key, Phase::Object, Error::TooLarge(api::Excess::unnamed()), out);
                return;
            }
        }
    }
    let mut last = after;
    let mut delivered = List::with_capacity(env.limits.rows);
    if participating {
        for comment in comments {
            last = last.max(comment.id);
            if unseen(&row.record.comments, Delivery { id: comment.id, revision: comment.revision })
                && (known_id(&row.record.comments, comment.id)
                    || !original(comment.provenance)
                    || (!super::echo::is_echo(&row.record.echoes, Echo::Comment(comment.id))
                        && !super::echo::key_echo(d, key.repository, comment.author, comment.key.as_deref())))
            {
                delivered.push(comment).expect("answer rows checked before retention");
            }
        }
    }
    if more && participating && last <= after {
        failed_resource(d, env, key, Phase::Object, Error::InvalidAnswer, out);
        return;
    }
    let row = d.keep.live.get_mut(key).expect("resource remains");
    super::taken(row, Phase::Object);
    if let What::Issue(_) = key.what {
        row.presence = super::Presence::Present;
    }
    let changed = row.record.cached.item.as_ref() != Some(&item);
    row.record.position.comment = row.record.position.comment.max(last);
    row.record.comment_after = if more && participating { last } else { row.record.position.comment };
    row.record.comment_page = if more && participating {
        row.record.comment_page.checked_add(1).expect("scan page admitted below cap")
    } else {
        1
    };
    row.record.comment_scanning = more && participating;
    row.inbox.reviews_needed = changed || row.record.comment_repair || row.record.review_page > 1;
    if !more {
        if row.record.comment_repair || row.record.cached.item.is_none() || super::priority(row) == Priority::Slow {
            row.inbox.repair_at = env.now.saturating_add(env.limits.slow);
        }
        row.record.comment_repair = false;
    }
    row.record.comments = stamps.into_boxed();
    row.record.cached.item = Some(item.clone());
    super::echo::prune_comments(row, last);
    out.push(Request::Save { record: Stored::Live(row.record.clone()) });
    if changed || !delivered.is_empty() {
        out.push(Request::Changed {
            resource: key.clone(),
            result: Ok(Answer::Item { item, comments: delivered.into_boxed(), more }),
        });
    }
    if more && participating {
        row.state = State::Due(Phase::Object, super::priority(row));
        return;
    }
    match key.what {
        What::Pull(_) => row.state = State::Due(Phase::Pull, super::priority(row)),
        What::Issue(_) => rest_resource(d, env, key),
        What::Repository | What::Branch(_) => unreachable!("item resource"),
    }
}
fn reviews_answered(
    d: &mut Domain,
    env: &Env<Limits>,
    key: &Resource,
    phase: Phase,
    reviews: Box<[api::Review]>,
    paging: (bool, bool),
    out: &mut Queue<Request>,
) {
    let (more, cut) = paging;
    let (page, skip) = match phase {
        Phase::Reviews(page, skip) => (page, skip),
        Phase::Object | Phase::Pull | Phase::Branch | Phase::BranchAncestry { .. } => unreachable!("reviews stage"),
    };
    if more && !cut && page >= env.limits.inbox {
        failed_resource(d, env, key, phase, Error::TooLarge(api::Excess::unnamed()), out);
        return;
    }
    let next_skip = if cut { skip.saturating_add(u32::try_from(reviews.len()).expect("bounded reviews")) } else { 0 };
    let next = if cut {
        page
    } else if more {
        match page.checked_add(1) {
            Some(next) => next,
            None => {
                failed_resource(d, env, key, phase, Error::TooLarge(api::Excess::unnamed()), out);
                return;
            }
        }
    } else {
        1
    };
    let row = d.keep.live.get(key).expect("resource remains");
    let mut stamps = copied_stamps(&row.record.reviews, env.limits.inbox);
    for review in &reviews {
        if !remember(&mut stamps, Delivery { id: review.id, revision: review.revision }) {
            failed_resource(d, env, key, phase, Error::TooLarge(api::Excess::unnamed()), out);
            return;
        }
    }
    super::taken(d.keep.live.get_mut(key).expect("resource remains"), phase);
    let mut delivered = List::with_capacity(env.limits.rows);
    for review in reviews {
        let row = d.keep.live.get(key).expect("resource remains");
        let fresh = unseen(&row.record.reviews, Delivery { id: review.id, revision: review.revision });
        let echo = !known_id(&row.record.reviews, review.id)
            && original(review.provenance)
            && (super::echo::is_echo(&row.record.echoes, Echo::Review(review.id))
                || super::echo::key_echo(d, key.repository, review.author, review.key.as_deref()));
        let row = d.keep.live.get_mut(key).expect("resource remains");
        super::echo::remove_echo(row, Echo::Review(review.id));
        row.record.position.review = row.record.position.review.max(review.id);
        if fresh && !echo {
            delivered.push(review).expect("bounded answer rows");
        }
    }
    let row = d.keep.live.get_mut(key).expect("resource remains");
    row.record.reviews = stamps.into_boxed();
    row.record.review_page = next;
    if more {
        row.state = State::Due(Phase::Reviews(next, next_skip), super::priority(row));
    }
    out.push(Request::Save { record: Stored::Live(row.record.clone()) });
    if !delivered.is_empty() {
        out.push(Request::Changed {
            resource: key.clone(),
            result: Ok(Answer::Reviews { reviews: delivered.into_boxed(), more, cut }),
        });
    }
    if !more {
        row.inbox.reviews_needed = false;
        rest_resource(d, env, key);
    }
}
pub(super) fn valid_deliveries(stamps: &[Delivery], cap: u32) -> bool {
    if stamps.len() > usize_of(cap) {
        return false;
    }
    for (index, stamp) in stamps.iter().enumerate() {
        for previous in stamps.get(..index).expect("enumerated stamp index") {
            if previous.id == stamp.id {
                return false;
            }
        }
    }
    true
}
fn copied_stamps(stamps: &[Delivery], cap: u32) -> List<Delivery> {
    let mut copied = List::with_capacity(cap);
    for stamp in stamps {
        copied.push(*stamp).expect("restored stamp table bounded");
    }
    copied
}
fn original(provenance: api::Provenance) -> bool {
    match provenance {
        api::Provenance::Original => true,
        api::Provenance::Revised | api::Provenance::Unknown => false,
    }
}
fn known_id(stamps: &[Delivery], wanted: u64) -> bool {
    for stamp in stamps {
        if stamp.id == wanted {
            return true;
        }
    }
    false
}
fn unseen(stamps: &[Delivery], wanted: Delivery) -> bool {
    for stamp in stamps {
        if *stamp == wanted {
            return false;
        }
    }
    true
}
fn remember(stamps: &mut List<Delivery>, wanted: Delivery) -> bool {
    for index in 0..stamps.len() {
        let stamp = stamps.get_mut(index).expect("bounded stamp index");
        if stamp.id == wanted.id {
            stamp.revision = wanted.revision;
            return true;
        }
    }
    stamps.push(wanted).is_ok()
}
fn branch_answered(d: &mut Domain, env: &Env<Limits>, key: &Resource, commit: Commit, out: &mut Queue<Request>) {
    let row = d.keep.live.get_mut(key).expect("resource remains");
    row.branch_present = true;
    if let Some((before, after)) = row.refused.ancestry
        && (row.record.cached.tip != Some(before) || commit != after)
    {
        row.refused.ancestry = None;
    }
    if row.writer
        && row.record.cached.tip != Some(commit)
        && let Some(before) = row.record.cached.tip
    {
        let row = d.keep.live.get_mut(key).expect("resource remains");
        let phase = Phase::BranchAncestry { before, after: commit };
        if row.refused.contains(phase) {
            rest_resource(d, env, key);
        } else {
            row.state = State::Due(phase, Priority::Fresh);
        }
        return;
    }
    branch_observed(d, env, key, commit, false, out);
}
fn branch_ancestry_answered(
    d: &mut Domain,
    env: &Env<Limits>,
    key: &Resource,
    before: Commit,
    after: Commit,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let row = d.keep.live.get(key).expect("resource remains");
    // A run answer can confirm a newer head while this ancestry read is in flight.
    // The read cannot overwrite that confirmation.
    if row.record.cached.tip != Some(before) {
        let row = d.keep.live.get_mut(key).expect("resource remains");
        row.state = State::Due(Phase::Branch, Priority::Fresh);
        return;
    }
    let forward = match result {
        Ok(Answer::Commits { commits, more }) => {
            if !commits.is_empty() {
                false
            } else if more {
                failed_resource(d, env, key, Phase::BranchAncestry { before, after }, Error::InvalidAnswer, out);
                return;
            } else {
                true
            }
        }
        Err(error @ Error::TooLarge(_)) => {
            failed_resource(d, env, key, Phase::BranchAncestry { before, after }, error, out);
            return;
        }
        result => {
            let error = match result {
                Err(error) => error,
                Ok(_) => Error::InvalidAnswer,
            };
            failed_resource(d, env, key, Phase::BranchAncestry { before, after }, error, out);
            return;
        }
    };
    super::taken(d.keep.live.get_mut(key).expect("resource remains"), Phase::BranchAncestry { before, after });
    branch_observed(d, env, key, after, !forward, out);
}
fn branch_observed(
    d: &mut Domain,
    env: &Env<Limits>,
    key: &Resource,
    commit: Commit,
    backwards: bool,
    out: &mut Queue<Request>,
) {
    let row = d.keep.live.get(key).expect("resource remains");
    let outside = match row.pushed_by {
        Some((head, actor)) if head == commit => !crate::identity::writer(d, key.repository, actor),
        Some(_) | None => false,
    };
    let row = d.keep.live.get_mut(key).expect("resource remains");
    row.presence = super::Presence::Present;
    let changed = row.record.cached.tip != Some(commit);
    let expected = if row.writer { row.record.cached.tip } else { row.record.pushed };
    if (backwards || outside || !row.writer)
        && changed
        && let Some(expected) = expected
        && expected != commit
    {
        out.push(Request::Drift { resource: key.clone(), expected, observed: Some(commit) });
    }
    row.pushed_by = None;
    row.record.cached.tip = Some(commit);
    out.push(Request::Save { record: Stored::Live(row.record.clone()) });
    if changed && !row.writer {
        out.push(Request::Changed { resource: key.clone(), result: Ok(Answer::Commit(commit)) });
    }
    rest_resource(d, env, key);
}
fn failed_resource(
    d: &mut Domain,
    env: &Env<Limits>,
    key: &Resource,
    phase: Phase,
    error: Error,
    out: &mut Queue<Request>,
) {
    let ancestry = match phase {
        Phase::BranchAncestry { .. } => true,
        Phase::Object | Phase::Pull | Phase::Reviews(_, _) | Phase::Branch => false,
    };
    if error == Error::Missing && !ancestry {
        let row = d.keep.live.get_mut(key).expect("resource remains");
        if row.presence != super::Presence::Missing {
            if let What::Branch(_) = key.what
                && let Some(expected) = row.record.cached.tip.take()
            {
                out.push(Request::Drift { resource: key.clone(), expected, observed: None });
                out.push(Request::Save { record: Stored::Live(row.record.clone()) });
            }
            out.push(Request::Changed { resource: key.clone(), result: Err(error) });
        }
        row.presence = super::Presence::Missing;
        super::taken(row, phase);
        rest_resource(d, env, key);
        return;
    }
    out.push(Request::Changed { resource: key.clone(), result: Err(error) });
    let row = d.keep.live.get_mut(key).expect("resource remains");
    if let Error::TooLarge(_) = error {
        if !row.hinted {
            row.refused.set(phase, true);
        }
        skip_resource(d, env, key, phase);
        return;
    }
    row.state = State::Waiting(phase, Priority::Slow);
    let wait = super::retry_wait(&mut row.retry, &env.limits);
    d.alarms.arm(Alarm::Resource(row.alarm), env.now.saturating_add(wait)).expect("one resource alarm");
}
pub(super) fn skip_resource(d: &mut Domain, env: &Env<Limits>, key: &Resource, phase: Phase) {
    let row = d.keep.live.get_mut(key).expect("resource remains");
    match phase {
        Phase::Object => match key.what {
            What::Pull(_) => row.state = State::Due(Phase::Pull, super::priority(row)),
            What::Issue(_) => rest_resource(d, env, key),
            What::Repository | What::Branch(_) => unreachable!("item phase"),
        },
        Phase::Pull if row.record.watch.participating && row.inbox.reviews_needed => {
            row.state = State::Due(Phase::Reviews(row.record.review_page, 0), super::priority(row));
        }
        Phase::Pull | Phase::Reviews(_, _) | Phase::Branch | Phase::BranchAncestry { .. } => rest_resource(d, env, key),
    }
}
fn rest_resource(d: &mut Domain, env: &Env<Limits>, key: &Resource) {
    let row = d.keep.live.get_mut(key).expect("resource remains");
    row.fresh = Fresh::Complete;
    let interval = if row.hinted {
        env.limits.hinted
    } else if row.presence == super::Presence::Missing {
        match key.what {
            What::Branch(_) if !row.branch_present => env.limits.slow.max(env.limits.fallback),
            What::Branch(_) | What::Issue(_) | What::Pull(_) | What::Repository => env.limits.slow,
        }
    } else {
        match key.what {
            What::Pull(_) => {
                let interval = row.interval;
                row.interval = interval.saturating_mul(2).min(env.limits.poll_max);
                interval.min(env.limits.slow)
            }
            What::Branch(_) | What::Issue(_) | What::Repository => env.limits.slow,
        }
    };
    let hinted = row.hinted;
    row.hinted = false;
    row.state = State::Idle;
    let next = env.now.saturating_add(interval);
    // A nonparticipating pull has no inbox repair to shorten its quiet CI rest.
    let repairs = match key.what {
        What::Pull(_) => row.record.watch.participating,
        What::Issue(_) | What::Branch(_) | What::Repository => true,
    };
    let next = if repairs && row.presence != super::Presence::Missing && row.inbox.repair_at > env.now {
        next.min(row.inbox.repair_at)
    } else {
        next
    };
    let alarm = row.alarm;
    let next = if hinted { next } else { super::fallen_back(&d.keep, &env.limits, env.now, key.repository, next) };
    d.alarms.arm(Alarm::Resource(alarm), next).expect("one resource alarm");
}
