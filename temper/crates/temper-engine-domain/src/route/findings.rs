//! Findings are routed in the deciding decision (temper/docs/design/domain/areas.md, 3 to 7).
#[cfg(test)]
mod tests;
use super::{Decision, Domain, Env, Limits, Work, forge, forge_route, tasks};
use alloc::boxed::Box;
use forge::{Filer, FindingRefusal, Holder, settings::AreaOwner};
use skein_lib::{Decimal, List, Map, Queue};

fn live(row: &tasks::TaskRecord, project: u32) -> bool {
    if row.project != project {
        return false;
    }
    match row.phase {
        tasks::Phase::Waiting
        | tasks::Phase::Active(_)
        | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. } => true,
        tasks::Phase::Closing(_) | tasks::Phase::Ended(_) | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => {
            false
        }
    }
}
#[must_use]
pub fn stands(domain: &Domain, project: u32, owner: AreaOwner) -> Option<Holder> {
    match owner {
        AreaOwner::Task(number) => {
            let row = domain.core.tasks.task(number)?;
            if live(row, project) { Some(Holder::Task(number)) } else { None }
        }
        AreaOwner::Party(number) => {
            if domain.core.people.role(number, project).is_some() {
                Some(Holder::Party(number))
            } else {
                None
            }
        }
        AreaOwner::Role(number) => Some(Holder::Role(number)),
        AreaOwner::Charter(charter) => {
            let mut lowest = None;
            for view in domain.core.tasks.view_tasks() {
                let Some(row) = domain.core.tasks.task(view.number) else { continue };
                if row.label == 3 && row.executor == (tasks::Executor::Agent { charter }) && live(row, project) {
                    lowest = Some(match lowest {
                        Some(n) => row.number.min(n),
                        None => row.number,
                    });
                }
            }
            match lowest {
                Some(number) => Some(Holder::Task(number)),
                None => None,
            }
        }
    }
}
#[must_use]
pub fn holder(domain: &Domain, project: u32, area: Option<&[u8]>) -> Holder {
    let areas = domain.forge.project_areas(project);
    let triager = domain.forge.project_triager(project);
    let mut standing = List::with_capacity(17);
    for area in &areas.areas {
        if let Some(holder) = stands(domain, project, area.owner) {
            standing.push((area.owner, holder)).expect("sixteen areas");
        }
    }
    if let Some(owner) = triager
        && let Some(holder) = stands(domain, project, owner)
    {
        standing.push((owner, holder)).expect("one triager beside areas");
    }
    forge::owner(area, &areas, triager, standing.as_slice())
}
/// Resolve the follow owner's first standing area where liveness is known
/// (temper/docs/design/domain/forge/follows.md, 6).
#[must_use]
pub(super) fn follow_area(domain: &Domain, project: u32, follow: u32) -> Option<Box<[u8]>> {
    let follow = domain.forge.follow(project, follow)?;
    let owner = match follow.owner {
        forge::settings::FollowOwner::Task(task) => Holder::Task(task),
        forge::settings::FollowOwner::Person(party) => Holder::Party(party),
    };
    for area in domain.forge.project_areas(project).areas {
        if stands(domain, project, area.owner) == Some(owner) {
            return Some(area.name);
        }
    }
    None
}
#[must_use]
pub fn mine(domain: &Domain, project: u32, party: u64, holder: Holder) -> bool {
    match holder {
        Holder::Task(_) => false,
        Holder::Party(number) => number == party,
        Holder::Role(number) => match domain.core.people.role(party, project) {
            Some(role) => role.number() == number,
            None => false,
        },
    }
}
fn permitted(domain: &Domain, project: u32, number: u32, by: Filer) -> Result<(), FindingRefusal> {
    let row = domain.forge.finding(project, number).ok_or(FindingRefusal::NoFinding)?;
    let held = holder(domain, project, row.area.as_deref());
    let allowed = match by {
        Filer::Task(number) => {
            held == Holder::Task(number) && stands(domain, project, AreaOwner::Task(number)).is_some()
        }
        Filer::Party(party) => {
            domain.core.people.role(party, project) == Some(super::people::Role::Owner)
                || mine(domain, project, party, held)
        }
    };
    if allowed { Ok(()) } else { Err(FindingRefusal::NotYours(held)) }
}

/// Trusted root filing entry, also used after a party's membership admission.
/// The caller names the filer; automatic filings need no agent run or public request.
pub fn file_finding(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    filer: Filer,
    call: forge::FileFinding,
) -> forge::FileFindingAnswer {
    let result = domain.forge.prepare_file_finding(&env.limits.forge, env.wall, filer, call);
    match commit_prepared(domain, env, decision, result, false) {
        Ok(number) => forge::FileFindingAnswer::Filed { number },
        Err(why) => forge::FileFindingAnswer::Refused(why),
    }
}
pub fn hand_finding(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    by: Filer,
    call: forge::HandFinding,
) -> forge::HandFindingAnswer {
    let result = match permitted(domain, call.project, call.number, by) {
        Ok(()) => domain.forge.prepare_hand_finding(by, call),
        Err(why) => Err(why),
    };
    match commit_prepared(domain, env, decision, result, true) {
        Ok(_) => forge::HandFindingAnswer::Handed,
        Err(why) => forge::HandFindingAnswer::Refused(why),
    }
}
pub fn triage_finding(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    by: Filer,
    call: forge::TriageFinding,
) -> forge::TriageFindingAnswer {
    let result = prepare_triage(domain, by, call);
    match commit_prepared(domain, env, decision, result, false) {
        Ok(_) => forge::TriageFindingAnswer::Triaged,
        Err(why) => forge::TriageFindingAnswer::Refused(why),
    }
}
fn prepare_triage(
    domain: &Domain,
    by: Filer,
    call: forge::TriageFinding,
) -> Result<forge::FindingWrite, FindingRefusal> {
    permitted(domain, call.project, call.number, by)?;
    if let forge::Triage::Route(goal) = call.act {
        let Some(row) = domain.core.tasks.task(goal) else { return Err(FindingRefusal::NoGoal) };
        if !live(row, call.project) || row.label != 2 || row.tracked.is_none() {
            return Err(FindingRefusal::NoGoal);
        }
    }
    domain.forge.prepare_triage_finding(call)
}
fn commit_prepared(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    write: Result<forge::FindingWrite, FindingRefusal>,
    hand: bool,
) -> Result<u32, FindingRefusal> {
    commit(domain, env, decision, write?, hand)
}
pub(crate) fn commit(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    write: forge::FindingWrite,
    hand: bool,
) -> Result<u32, FindingRefusal> {
    let row = &write.row;
    let recipient = match row.state {
        forge::FindingState::New => match holder(domain, row.project, row.area.as_deref()) {
            Holder::Task(task) => Some(task),
            Holder::Party(_) | Holder::Role(_) => None,
        },
        forge::FindingState::Routed => row.goal,
        forge::FindingState::Backlog | forge::FindingState::Dropped => None,
    };
    if let Some(task) = recipient {
        let subscription = subscribe(domain, env, decision, task, row.project)?;
        if let Some(event) = domain.core.connector_news(
            task,
            subscription,
            tasks::NewsClass::Wakes,
            news_words(row, hand, env.limits.tasks.message_bytes),
            env.wall,
        ) {
            domain.work.push(Work::Tasks(event));
        }
    }
    let number = row.number;
    let mut output = Queue::uncapped();
    domain.forge.commit_finding(&env.limits.forge, write, &mut output);
    forge_route::outputs(domain, env, decision, &mut output);
    Ok(number)
}
/// Read the old holders while the effective settings still precede the save.
/// Only the grouped delta crosses the commit barrier (domain/areas.md, 5).
pub(super) fn settings_holders(domain: &Domain, items: &[forge::settings::Asked]) -> List<(u32, u32, Holder)> {
    let mut projects = Map::uncapped();
    for item in items {
        if item.kind == 14 || item.kind == 15 {
            projects.insert(item.scope, ()).expect("one project per settings scope");
        }
    }
    let mut holders = List::uncapped();
    for (&project, ()) in &projects {
        for (_, row) in domain.forge.findings(project) {
            if !row.state.ended() {
                holders
                    .push((project, row.number, holder(domain, project, row.area.as_deref())))
                    .expect("one old holder per open finding");
            }
        }
    }
    holders
}

pub(super) fn settings_saved(
    domain: &Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    before: &[(u32, u32, Holder)],
) {
    let mut groups = Map::<(u32, u64), List<u32>>::uncapped();
    for &(project, number, old) in before {
        let Some(row) = domain.forge.finding(project, number) else { continue };
        let new = holder(domain, project, row.area.as_deref());
        if new == old {
            continue;
        }
        match new {
            Holder::Task(task) => {
                let key = (project, task);
                if !groups.contains_key(&key) {
                    groups.insert(key, List::uncapped()).expect("one group per new task holder");
                }
                groups.get_mut(&key).expect("inserted holder group").push(number).expect("one number per finding");
            }
            Holder::Party(_) | Holder::Role(_) => {}
        }
    }
    for (&(project, task), numbers) in &groups {
        super::emit(
            decision,
            &env.limits,
            crate::Delivery::FindingsOwned {
                project,
                task,
                words: ownership_words(project, numbers.as_slice(), env.limits.tasks.message_bytes),
            },
        );
    }
}

/// Subscribe and tell the new holder in the decision after the settings commit.
pub(crate) fn tell_owner(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    project: u32,
    task: u64,
    words: Box<[u8]>,
) {
    if let Ok(subscription) = subscribe(domain, env, decision, task, project)
        && let Some(event) = domain.core.connector_news(task, subscription, tasks::NewsClass::Wakes, words, env.wall)
    {
        domain.work.push(Work::Tasks(event));
    }
}

fn ownership_words(project: u32, numbers: &[u32], limit: u32) -> Box<[u8]> {
    let mut out = List::uncapped();
    append(&mut out, b"Findings on project ");
    append(&mut out, Decimal::of(u64::from(project)).as_bytes());
    append(&mut out, b" are yours now:");
    let header = out.as_slice().len();
    for (index, &number) in numbers.iter().enumerate() {
        if index == 0 {
            append(&mut out, b" ");
        } else {
            append(&mut out, b", ");
        }
        append(&mut out, Decimal::of(u64::from(number)).as_bytes());
    }
    let limit = skein_lib::usize_of(limit);
    if out.as_slice().len() <= limit {
        return out.into_boxed();
    }
    let mut end = header;
    let mut kept = header;
    let mut count = 0_usize;
    for (index, &number) in numbers.iter().enumerate() {
        end = end.saturating_add(if index == 0 { 1 } else { 2 });
        end = end.saturating_add(Decimal::of(u64::from(number)).as_bytes().len());
        let remaining = numbers.len().saturating_sub(index).saturating_sub(1);
        let suffix = Decimal::of(skein_lib::len_u64(remaining));
        if end.saturating_add(11).saturating_add(suffix.as_bytes().len()) <= limit {
            kept = end;
            count = index.saturating_add(1);
        }
    }
    let more = Decimal::of(skein_lib::len_u64(numbers.len().saturating_sub(count)));
    let mut words = List::uncapped();
    if count == 0 {
        // A tiny message bound may not fit even the header and the omitted count.
        let prefix = cut_words(
            out.as_slice().get(..header).expect("header prefix"),
            u32::try_from(limit.saturating_sub(10).saturating_sub(more.as_bytes().len())).expect("message bound"),
        );
        append(&mut words, &prefix);
    } else {
        append(&mut words, out.as_slice().get(..kept).expect("fitting number prefix"));
    }
    if count != 0 {
        append(&mut words, b", ");
    } else {
        append(&mut words, b" ");
    }
    append(&mut words, b"and ");
    append(&mut words, more.as_bytes());
    append(&mut words, b" more");
    words.into_boxed()
}

fn subscribe(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    task: u64,
    project: u32,
) -> Result<u64, FindingRefusal> {
    let topic = forge::Topic::Findings { project };
    if let Some(row) = domain.forge.topic_subscriber(task, &topic) {
        return Ok(row.number);
    }
    let number = crate::fresh(&mut domain.core.counters, crate::Family::Message).expect("admitted message number");
    let subscriber =
        forge::Subscriber { task, number, topic, own_change: None, goal_tasks: None, paths: Box::default() };
    if !domain.forge.can_subscribe(&env.limits.forge, &subscriber) {
        return Err(subscription_refused(task, b"forge.subscriptions"));
    }
    let requests = forge_route::step_core(
        domain,
        env,
        super::core::Event::Tasks(tasks::Event::SubscribeTopic {
            reply_to: super::internal(u64::MAX),
            task,
            subscription: tasks::Subscription {
                number,
                kind: tasks::SubscriptionKind::Topic { connector: domain.config.forge_connector, topic: number },
            },
        }),
    );
    super::route_core_requests(domain, env, decision, requests);
    let mut present = false;
    if let Some(row) = domain.core.tasks.task(task) {
        for sub in &row.subscriptions {
            present |= sub.number == number;
        }
    }
    if !present {
        return Err(subscription_refused(task, b"tasks.subscriptions or inbox bound"));
    }
    super::forge_event(domain, env, decision, forge::Event::Subscribe { subscription: subscriber });
    Ok(number)
}
fn subscription_refused(task: u64, bound: &[u8]) -> FindingRefusal {
    let mut words = List::uncapped();
    append(&mut words, b"task ");
    append(&mut words, Decimal::of(task).as_bytes());
    append(&mut words, b" cannot subscribe to findings: ");
    append(&mut words, bound);
    FindingRefusal::Words(words.into_boxed())
}
fn append(out: &mut List<u8>, words: &[u8]) {
    for &byte in words {
        out.push(byte).expect("bounded finding words");
    }
}
fn news_words(row: &forge::Finding, hand: bool, limit: u32) -> Box<[u8]> {
    let mut out = List::uncapped();
    for words in [
        b"Finding ".as_slice(),
        Decimal::of(u64::from(row.project)).as_bytes(),
        b"/",
        Decimal::of(u64::from(row.number)).as_bytes(),
        if row.state == forge::FindingState::Routed { b" for you: ".as_slice() } else { b" is yours: " },
        &row.title,
        b"\n",
    ] {
        append(&mut out, words);
    }
    let kind: &[u8] = match row.kind {
        forge::FindingKind::Qa => b"qa",
        forge::FindingKind::Flake => b"flake",
        forge::FindingKind::Red => b"red",
        forge::FindingKind::Review => b"review",
        forge::FindingKind::Feedback => b"feedback",
    };
    let severity: &[u8] = match row.severity {
        forge::Severity::Low => b"low",
        forge::Severity::Medium => b"medium",
        forge::Severity::High => b"high",
    };
    append(&mut out, kind);
    append(&mut out, b", ");
    append(&mut out, severity);
    append(&mut out, b", ");
    match &row.area {
        Some(area) => {
            append(&mut out, b"area ");
            append(&mut out, area);
        }
        None => append(&mut out, b"no area"),
    }
    append(&mut out, b"\n");
    if hand && let (Some(by), Some(why)) = (row.hander, &row.why) {
        append(&mut out, b"Handed on by ");
        match by {
            Filer::Task(number) => {
                append(&mut out, b"task ");
                append(&mut out, Decimal::of(number).as_bytes());
            }
            Filer::Party(number) => {
                append(&mut out, b"party ");
                append(&mut out, Decimal::of(number).as_bytes());
            }
        }
        append(&mut out, b": ");
        append(&mut out, why);
        append(&mut out, b"\n");
    }
    append(&mut out, &row.body);
    cut_words(out.as_slice(), limit)
}
pub(crate) fn cut_words(words: &[u8], limit: u32) -> Box<[u8]> {
    let limit = skein_lib::usize_of(limit);
    if words.len() <= limit {
        return words.into();
    }
    let marker = b"[cut]";
    let mut end = limit.saturating_sub(marker.len());
    while end > 0
        && match words.get(end) {
            Some(byte) => byte & 0xc0 == 0x80,
            None => false,
        }
    {
        end = end.saturating_sub(1);
    }
    let mut out = List::uncapped();
    append(&mut out, words.get(..end).expect("prefix within message"));
    append(&mut out, marker.get(..marker.len().min(limit)).expect("cut marker prefix"));
    out.into_boxed()
}
#[must_use]
pub fn document(
    domain: &Domain,
    project: u32,
    party: u64,
    state: Option<forge::FindingState>,
    only_mine: bool,
) -> forge::Findings {
    let mut rows = List::with_capacity(128);
    let mut cut = false;
    for (_, row) in domain.forge.findings(project).rev() {
        let held = holder(domain, project, row.area.as_deref());
        if (match state {
            Some(state) => state != row.state,
            None => false,
        }) || (only_mine && !mine(domain, project, party, held))
        {
            continue;
        }
        if rows.push(row.row(held)).is_err() {
            cut = true;
        }
    }
    let areas = domain.forge.project_areas(project);
    let mut listed = List::with_capacity(16);
    for area in areas.areas {
        listed
            .push(forge::AreaRow {
                holder: stands(domain, project, area.owner),
                name: area.name,
                words: area.words,
                owner: area.owner,
            })
            .expect("bounded areas");
    }
    forge::Findings {
        project,
        rows: rows.into_boxed(),
        areas: listed.into_boxed(),
        triager: domain.forge.project_triager(project),
        holder: holder(domain, project, None),
        cut,
    }
}
#[must_use]
pub fn read(domain: &Domain, project: u32, number: u32) -> Option<forge::FindingAnswer> {
    let row = domain.forge.finding(project, number)?;
    Some(forge::FindingAnswer {
        row: row.row(holder(domain, project, row.area.as_deref())),
        body: row.body.clone(),
        filer: row.filer,
        goal: row.goal,
        reason: row.reason.clone(),
    })
}
