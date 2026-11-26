//! Findings are routed in the deciding decision (temper/docs/design/domain/areas.md, 3 to 7).
#[cfg(test)]
mod tests;
use super::{Decision, Domain, Env, Limits, Work, forge, forge_route, tasks};
use alloc::boxed::Box;
pub(crate) use forge::findings::{cut_words, news_words, ownership_words};
use forge::{Filer, FindingRefusal, Holder, settings::AreaOwner};
use skein_lib::{List, Map, Queue};
mod repairing;

pub(crate) fn goal_ended(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    task: u64,
    ending: forge::ReleaseEnding,
) {
    let Some(row) = domain.engine.core().tasks.task(task) else { return };
    let project = row.project;
    let mut out = Queue::uncapped();
    domain.forge.judge_goal_findings(&super::environment_forge(env), project, task, ending, &mut out);
    forge_route::outputs(domain, env, decision, &mut out);
}
pub(crate) fn fail_repair(domain: &mut Domain, project: u32, task: u64, until: skein_lib::Wall) {
    let Some(row) = domain.engine.core().tasks.task(task) else { return };
    let hours = until.as_nanos().saturating_sub(row.created_at.as_nanos()).div_euclid(3_600_000_000_000);
    let mut words = List::uncapped();
    for part in [b"past its repair deadline of ".as_slice(), skein_lib::Decimal::of(hours).as_bytes(), b" hours"] {
        for &byte in part {
            words.push(byte).expect("bounded deadline reason");
        }
    }
    domain.work.push(Work::Tasks(tasks::Event::Fail {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(task)),
        by: tasks::Party::Deployment { project },
        operation: tasks::Fail { task, reason: words.into_boxed() },
    }));
}
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
            let row = domain.engine.core().tasks.task(number)?;
            if live(row, project) { Some(Holder::Task(number)) } else { None }
        }
        AreaOwner::Party(number) => {
            if domain.engine.core().people.role(number, project).is_some() {
                Some(Holder::Party(number))
            } else {
                None
            }
        }
        AreaOwner::Role(number) => Some(Holder::Role(number)),
        AreaOwner::Charter(charter) => {
            let mut lowest = None;
            for view in domain.engine.core().tasks.view_tasks() {
                let Some(row) = domain.engine.core().tasks.task(view.number) else { continue };
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
        Holder::Role(number) => match domain.engine.core().people.role(party, project) {
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
            domain.engine.core().people.role(party, project) == Some(super::people::Role::Owner)
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
/// Trusted landing-tier filing, with the follow's standing area (domain/red-findings.md, 2 and 9).
pub fn file_verdict(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    task: u64,
    filing: forge::VerdictFiling,
) -> forge::FileFindingAnswer {
    let result = domain.forge.prepare_verdict_filing(&env.limits.forge, env.wall, Filer::Task(task), &filing);
    let result = match result {
        Ok(mut write) => {
            if let Some(follow) = filing.follow {
                write.row.area = follow_area(domain, filing.project, follow);
            }
            Ok(write)
        }
        Err(why) => Err(why),
    };
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
    match &call.act {
        forge::Triage::Repair(act) => {
            return match repairing::repair(domain, env, decision, by, call.project, call.number, act) {
                Ok(goal) => forge::TriageFindingAnswer::Repaired(goal),
                Err(why) => forge::TriageFindingAnswer::Refused(why),
            };
        }
        forge::Triage::Route(_) | forge::Triage::Backlog | forge::Triage::Drop(_) | forge::Triage::Blame(_) => {}
    }
    let blame = match &call.act {
        forge::Triage::Blame(_) => true,
        forge::Triage::Route(_) | forge::Triage::Backlog | forge::Triage::Drop(_) | forge::Triage::Repair(_) => false,
    };
    let result = prepare_triage(domain, by, call);
    let result = if blame {
        match result {
            Ok(write) => Ok(commit_without_news(domain, env, decision, write)),
            Err(why) => Err(why),
        }
    } else {
        commit_prepared(domain, env, decision, result, false)
    };
    match result {
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
    match &call.act {
        forge::Triage::Route(goal) => {
            let Some(row) = domain.engine.core().tasks.task(*goal) else { return Err(FindingRefusal::NoGoal) };
            if !live(row, call.project) || row.label != 2 || row.tracked.is_none() {
                return Err(FindingRefusal::NoGoal);
            }
        }
        forge::Triage::Backlog | forge::Triage::Drop(_) | forge::Triage::Repair(_) | forge::Triage::Blame(_) => {}
    }
    domain.forge.prepare_triage_finding(by, call)
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
#[expect(clippy::unnecessary_wraps, reason = "keep the finding commit result used by forge routes")]
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
    if let Some(task) = recipient
        && let Some(subscription) = subscribe(domain, env, decision, task, row.project)
        && let Some(event) = domain.engine.core_mut().connector_news(
            task,
            subscription,
            tasks::NewsClass::Wakes,
            news_words(row, hand, env.limits.tasks.message_bytes),
            env.wall,
        )
    {
        domain.work.push(Work::Tasks(event));
    }
    Ok(commit_without_news(domain, env, decision, write))
}
fn commit_without_news(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    write: forge::FindingWrite,
) -> u32 {
    let number = write.row.number;
    let mut output = Queue::uncapped();
    domain.forge.commit_finding(&env.limits.forge, write, &mut output);
    forge_route::outputs(domain, env, decision, &mut output);
    number
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
            if !row.state.ended() && !row.needs_no_triage() {
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
    if let Some(subscription) = subscribe(domain, env, decision, task, project)
        && let Some(event) =
            domain.engine.core_mut().connector_news(task, subscription, tasks::NewsClass::Wakes, words, env.wall)
    {
        domain.work.push(Work::Tasks(event));
    }
}

fn subscribe(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, task: u64, project: u32) -> Option<u64> {
    let topic = forge::Topic::Findings { project };
    let existing = domain.forge.topic_subscriber(task, &topic);
    let number = if let Some(row) = existing {
        if subscribed(domain, task, row.number) {
            return Some(row.number);
        }
        row.number
    } else {
        crate::fresh(&mut domain.engine.core_mut().counters, crate::Family::Message).expect("admitted message number")
    };
    let existing = existing.is_some();
    let subscriber =
        forge::Subscriber { task, number, topic, own_change: None, goal_tasks: None, paths: Box::default() };
    if !existing && !domain.forge.can_subscribe(&env.limits.forge, &subscriber) {
        return None;
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
    if !subscribed(domain, task, number) {
        return None;
    }
    if !existing {
        super::forge_event(domain, env, decision, forge::Event::Subscribe { subscription: subscriber });
    }
    Some(number)
}
fn subscribed(domain: &Domain, task: u64, number: u64) -> bool {
    if let Some(row) = domain.engine.core().tasks.task(task) {
        for sub in &row.subscriptions {
            if sub.number == number {
                return true;
            }
        }
    }
    false
}
#[must_use]
pub fn document(domain: &Domain, project: u32, party: u64, filter: forge::FindingsFilter) -> forge::Findings {
    let mut rows = List::with_capacity(128);
    let mut cut = false;
    for (_, row) in domain.forge.findings(project).rev() {
        let held = holder(domain, project, row.area.as_deref());
        let keep = match filter {
            forge::FindingsFilter::Triage { state, mine: only_mine } => {
                let state_kept = match state {
                    Some(state) => state == row.state,
                    None => true,
                };
                state_kept && (!only_mine || (!row.needs_no_triage() && mine(domain, project, party, held)))
            }
            forge::FindingsFilter::Goal(_) | forge::FindingsFilter::Milestone(_) => blamed_on(row, filter),
        };
        if !keep {
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
fn blamed_on(row: &forge::Finding, filter: forge::FindingsFilter) -> bool {
    let Some(blame) = &row.blame else { return false };
    for landing in &blame.landings {
        for goal in &landing.goals {
            let found = match filter {
                forge::FindingsFilter::Goal(number) => goal.goal == number,
                forge::FindingsFilter::Milestone(number) => goal.milestone == Some(number),
                forge::FindingsFilter::Triage { .. } => false,
            };
            if found {
                return true;
            }
        }
    }
    false
}
#[must_use]
pub fn read(domain: &Domain, project: u32, number: u32) -> Option<forge::FindingAnswer> {
    let row = domain.forge.finding(project, number)?;
    Some(forge::FindingAnswer {
        row: row.row(holder(domain, project, row.area.as_deref())),
        body: row.body.clone(),
        filer: row.filer,
        blame: row.blame.clone(),
    })
}
