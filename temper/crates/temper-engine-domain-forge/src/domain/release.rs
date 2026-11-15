//! Whole repository retirement and link-preserving pages (domain/forge/adoption.md, 1).
use super::{Domain, Env, Key, Limits, List, Map, Queue, Request, Stored, Token, Topic, child, emit};
use alloc::boxed::Box;
use skein_forge::api::Repository;
use skein_lib::Set;

pub(crate) const PAGE: u32 = 512;

#[derive(Hash, Debug)]
pub(crate) struct State {
    repositories: Map<Repository, Pending>,
    projects: Map<u32, Queue<Key>>,
    replies: Queue<(u32, Token)>,
    forges: Set<u16>,
    started: bool,
    watch_dirty: bool,
    first_writes: u32,
    pub(super) withdrawing: Option<u64>,
}
#[derive(Hash, Debug)]
struct Pending {
    project: u32,
    groups: Queue<Box<[Key]>>,
}
impl State {
    pub(crate) fn new(l: &Limits) -> Self {
        Self {
            repositories: Map::with_capacity(l.repositories),
            projects: Map::uncapped(),
            replies: Queue::uncapped(),
            forges: Set::with_capacity(l.forges),
            started: false,
            watch_dirty: false,
            first_writes: 0,
            withdrawing: None,
        }
    }
}

pub(crate) fn first_write(d: &mut Domain) {
    d.releasing.first_writes = d.releasing.first_writes.checked_add(1).expect("bounded first-decision writes");
}
pub(crate) fn reserved(d: &Domain) -> u32 {
    d.releasing.repositories.len()
}
pub(crate) fn contains(d: &Domain, repository: Repository) -> bool {
    d.releasing.repositories.contains_key(&repository)
}

pub(crate) fn pending(d: &Domain) -> bool {
    !d.releasing.repositories.is_empty() || !d.releasing.projects.is_empty() || !d.releasing.forges.is_empty()
}
pub(crate) fn forge_pending(d: &Domain, forge: u16) -> bool {
    for (repository, _) in &d.releasing.repositories {
        if repository.forge == forge {
            return true;
        }
    }
    false
}
pub(crate) fn forge(d: &mut Domain, forge: u16) {
    for (repository, _) in &d.repositories {
        if repository.forge == forge {
            return;
        }
    }
    d.releasing.forges.insert(forge).expect("bounded removed forges");
    d.releasing.started = true;
}
pub(crate) fn project(
    d: &mut Domain,
    project: u32,
    request: Token,
    mut writes: Queue<Request>,
    out: &mut Queue<Request>,
) {
    let mut erasures = d.releasing.projects.remove(&project).unwrap_or(Queue::uncapped());
    let mut groups = Queue::uncapped();
    let saves = outputs(&mut writes, &mut groups, out, true);
    d.releasing.first_writes = d.releasing.first_writes.checked_add(saves).expect("bounded first-decision writes");
    for _ in 0..groups.len() {
        for key in groups.pop().expect("counted project erasures") {
            erasures.push(key);
        }
    }
    if !erasures.is_empty() {
        d.releasing.projects.insert(project, erasures).expect("internal project releases");
    }
    d.releasing.replies.push((project, request));
    d.releasing.started = true;
}

pub(crate) fn first_page(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if core::mem::replace(&mut d.releasing.watch_dirty, false) {
        super::admission::refresh(d, env, out);
    }
    if d.releasing.started {
        page(d, env, out);
    } else {
        d.releasing.first_writes = 0;
    }
}

pub(crate) fn page(d: &mut Domain, _env: &Env<Limits>, out: &mut Queue<Request>) {
    d.releasing.started = false;
    let mut room = PAGE.saturating_sub(core::mem::replace(&mut d.releasing.first_writes, 0));
    let mut projects = core::mem::replace(&mut d.releasing.projects, Map::uncapped());
    for _ in 0..projects.len() {
        let project = *projects.first().expect("counted release rows").0;
        let mut keys = projects.remove(&project).expect("counted release rows");
        for _ in 0..keys.len().min(room) {
            let key = keys.pop().expect("counted project erasures");
            emit(out, Request::Erase { key });
            room = room.checked_sub(1).expect("page room");
        }
        if !keys.is_empty() {
            d.releasing.projects.insert(project, keys).expect("internal project releases");
        }
    }
    let capacity = d.releasing.repositories.capacity();
    let mut repositories = core::mem::replace(&mut d.releasing.repositories, Map::with_capacity(capacity));
    for _ in 0..repositories.len() {
        let repository = *repositories.first().expect("counted release rows").0;
        let mut pending = repositories.remove(&repository).expect("counted release rows");
        for _ in 0..pending.groups.len() {
            let count = match pending.groups.iter().next() {
                Some(group) => u32::try_from(group.len()).expect("bounded erase group"),
                None => break,
            };
            if count > room {
                break;
            }
            let group = pending.groups.pop().expect("counted erase groups");
            for key in group {
                emit(out, Request::Erase { key });
            }
            room = room.checked_sub(count).expect("page room");
        }
        if !pending.groups.is_empty() {
            d.releasing.repositories.insert(repository, pending).expect("replaces releasing repository");
        }
    }
    let mut finished = List::with_capacity(d.releasing.forges.len());
    for forge in &d.releasing.forges {
        let mut kept = forge_pending(d, *forge);
        for (repository, _) in &d.repositories {
            kept |= repository.forge == *forge;
        }
        if !kept && room > 0 {
            finished.push(*forge).expect("bounded removed forges");
            room = room.checked_sub(1).expect("page room");
        }
    }
    for forge in finished.into_boxed() {
        d.releasing.forges.remove(&forge);
        crate::forges::erase(d, forge, out);
    }
    for _ in 0..d.releasing.replies.len() {
        let (project, request) = d.releasing.replies.pop().expect("counted release replies");
        let mut waiting = d.releasing.projects.contains_key(&project);
        for (_, pending) in &d.releasing.repositories {
            waiting |= pending.project == project;
        }
        if waiting {
            d.releasing.replies.push((project, request));
        } else {
            emit(out, Request::ProjectReleased { request });
            crate::history::sweep::remove(d, project, out);
        }
    }
}

pub(crate) fn owns(name: &crate::Name, repository: Repository) -> bool {
    name.forge == repository.forge && name.repository == repository.repository
}
pub(crate) fn topic(topic: &Topic, repository: Repository) -> bool {
    match topic {
        Topic::Findings { .. } => false,
        Topic::Ci { repository: owner, .. }
        | Topic::Landings { repository: owner, .. }
        | Topic::Pull { repository: owner, .. }
        | Topic::Participation { repository: owner, .. } => *owner == repository,
    }
}
fn erase(groups: &mut Queue<Box<[Key]>>, key: Key) {
    groups.push(Box::new([key]));
}

pub(crate) fn repository(d: &mut Domain, env: &Env<Limits>, repository: Repository, out: &mut Queue<Request>) {
    let Some(row) = d.repositories.get(&repository) else { return };
    let project = row.project;
    let mut groups = Queue::uncapped();
    retire(d, repository);
    // Settlement clears a release's pending name; select its ownership first.
    let mut released = Queue::uncapped();
    release_rows(d, repository, &mut released);
    entries(d, env, repository, &mut groups, out);
    for _ in 0..released.len() {
        groups.push(released.pop().expect("counted release rows"));
    }
    work(d, env, repository, &mut groups, out);
    observations(d, repository, &mut groups);
    names(d, repository, &mut groups, out);
    let mut writes = Queue::uncapped();
    crate::landing_log::erased(d, repository, &mut writes);
    erasures(&mut writes, &mut groups, out);
    crate::landing::erase_repository_receipts(d, repository, &mut groups);
    crate::drafts::erase_repository(d, repository, &mut writes);
    erasures(&mut writes, &mut groups, out);
    applications(d, repository, &mut groups);
    d.repositories.remove(&repository);
    erase(&mut groups, Key::Client(skein_forge_client::Key::Repository(repository)));
    erase(&mut groups, Key::Repository(repository));
    d.releasing
        .repositories
        .insert(repository, Pending { project, groups })
        .expect("one release per adopted repository");
    d.releasing.started = true;
    d.releasing.watch_dirty = true;
    emit(out, Request::MarkedForges);
}

fn entries(
    d: &mut Domain,
    env: &Env<Limits>,
    repository: Repository,
    groups: &mut Queue<Box<[Key]>>,
    out: &mut Queue<Request>,
) {
    let mut numbers = List::with_capacity(d.entries.len());
    for (number, row) in &d.entries {
        if row.entry.repository == repository {
            numbers.push(*number).expect("bounded entries");
        }
    }
    for number in numbers.into_boxed() {
        let task = d.entries.get(&number).expect("collected entry").entry.task;
        let mut settled = Queue::uncapped();
        d.named_outcome(number, skein_forge_client::Outcome::Withdrawn);
        super::settled_facts(d, env, number, task, skein_forge_client::Outcome::Withdrawn, &mut settled);
        super::outage::forget_entry(d, number);
        d.entries.remove(&number);
        if d.flights.remove(&(task, number)).is_some() {
            erase(groups, Key::Flight { task, entry: number });
        }
        erase(groups, Key::Entry(number));
        // Settlement's intermediate work rows are erased below, never saved anew.
        erasures(&mut settled, groups, out);
        d.releasing.withdrawing = Some(number);
        child(d, env, skein_forge_client::Event::Withdraw { entry: number }, out);
        d.releasing.withdrawing = None;
        emit(out, Request::NotMade { entry: number, task });
    }
}
fn erasures(writes: &mut Queue<Request>, groups: &mut Queue<Box<[Key]>>, out: &mut Queue<Request>) {
    outputs(writes, groups, out, false);
}

fn outputs(
    writes: &mut Queue<Request>,
    groups: &mut Queue<Box<[Key]>>,
    out: &mut Queue<Request>,
    keep_saves: bool,
) -> u32 {
    let mut saves = 0_u32;
    for _ in 0..writes.len() {
        let request = writes.pop().expect("counted retirement outputs");
        match request {
            Request::Erase { key } => erase(groups, key),
            Request::Save { .. } => {
                if keep_saves {
                    saves = saves.checked_add(1).expect("bounded saves");
                    emit(out, request);
                }
            }
            other @ (Request::LoadHistory { .. }
            | Request::HistoryPage { .. }
            | Request::Decided { .. }
            | Request::MakingCopied { .. }
            | Request::MakingAct { .. }
            | Request::StatusRun { .. }
            | Request::FollowRun { .. }
            | Request::CancelFollow { .. }
            | Request::TierRun { .. }
            | Request::FileRedFinding { .. }
            | Request::TierDecision { .. }
            | Request::ProjectReleased { .. }
            | Request::ForgeKept(_)
            | Request::ForgeGone { .. }
            | Request::BoxGroup { .. }
            | Request::BoxRemove { .. }
            | Request::LandingNews { .. }
            | Request::LandingDecision { .. }
            | Request::Settled { .. }
            | Request::Bound { .. }
            | Request::PlaygroundActs { .. }
            | Request::ResetAct { .. }
            | Request::SetUp { .. }
            | Request::Connect { .. }
            | Request::Manifest { .. }
            | Request::Accept { .. }
            | Request::Install { .. }
            | Request::AdoptAccess { .. }
            | Request::Reachable { .. }
            | Request::CarriedUnused { .. }
            | Request::EraseCarried { .. }
            | Request::Marked { .. }
            | Request::MarkedForges
            | Request::ApplicationPart { .. }
            | Request::ApplicationDone { .. }
            | Request::ApplicationAgain { .. }
            | Request::StepDependents { .. }
            | Request::CloserChecked { .. }
            | Request::WakeProcedure { .. }
            | Request::Resource { .. }
            | Request::GoalUntopic { .. }
            | Request::BriefClient { .. }
            | Request::BriefReady { .. }
            | Request::BriefSized { .. }
            | Request::BriefReadRefused { .. }
            | Request::BriefTaken { .. }
            | Request::Adopted { .. }
            | Request::Taken { .. }
            | Request::Refused { .. }
            | Request::ResourceRefused { .. }
            | Request::HeadRefused { .. }
            | Request::MakeFailed { .. }
            | Request::LostReadFailed { .. }
            | Request::Outcome { .. }
            | Request::ProjectionOutcome { .. }
            | Request::NotMade { .. }
            | Request::ContinueRelease { .. }
            | Request::ContinueGoalBranch { .. }
            | Request::EffectsSettled { .. }
            | Request::Released { .. }
            | Request::Retained { .. }
            | Request::ReleaseFailed { .. }
            | Request::ProjectionEffect { .. }
            | Request::ProjectAfter { .. }
            | Request::ProjectionSettled { .. }
            | Request::ProjectionFailed { .. }
            | Request::Reviewed { .. }
            | Request::ChangeDecision { .. }
            | Request::News { .. }
            | Request::Drift { .. }
            | Request::Call { .. }
            | Request::Read { .. }
            | Request::RestartDone { .. }
            | Request::Replace { .. }
            | Request::LinkNoted { .. }) => emit(out, other),
        }
    }
    saves
}

fn release_rows(d: &mut Domain, repository: Repository, groups: &mut Queue<Box<[Key]>>) {
    let capacity = d.releases.capacity();
    let mut rows = core::mem::replace(&mut d.releases, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let key = *rows.first().expect("counted release rows").0;
        let row = rows.remove(&key).expect("counted release rows");
        if match &row.pending {
            Some((_, Some(name))) => owns(name, repository),
            Some((_, None)) | None => false,
        } {
            erase(groups, Key::Release(key));
        } else {
            d.releases.insert(key, row).expect("retained releases");
        }
    }
}

fn work(
    d: &mut Domain,
    env: &Env<Limits>,
    repository: Repository,
    groups: &mut Queue<Box<[Key]>>,
    out: &mut Queue<Request>,
) {
    let capacity = d.proposed_effects.capacity();
    let mut rows = core::mem::replace(&mut d.proposed_effects, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let key = *rows.first().expect("counted release rows").0;
        let row = rows.remove(&key).expect("counted release rows");
        if row.entry.repository == repository {
            erase(groups, Key::ProposedEffect(key));
        } else {
            d.proposed_effects.insert(key, row).expect("retained proposed_effects");
        }
    }
    let capacity = d.holds.capacity();
    let mut rows = core::mem::replace(&mut d.holds, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let key = rows.first().expect("counted release rows").0.clone();
        let row = rows.remove(&key).expect("counted release rows");
        if owns(&key, repository) {
            erase(groups, Key::Hold(key));
        } else {
            d.holds.insert(key, row).expect("retained holds");
        }
    }
    let capacity = d.issues.capacity();
    let mut rows = core::mem::replace(&mut d.issues, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let key = *rows.first().expect("counted release rows").0;
        let row = rows.remove(&key).expect("counted release rows");
        if row.repository == repository {
            erase(groups, Key::Issue(key));
        } else {
            d.issues.insert(key, row).expect("retained issues");
        }
    }
    crate::landing::release_repository(d, repository, out, groups);
    crate::tiers::release_repository(d, env, repository, groups);
    let mut changes = List::with_capacity(d.changes.len());
    for (task, row) in &d.changes {
        if row.repository == repository {
            changes.push((*task, row.terminal())).expect("bounded changes");
        }
    }
    for (task, terminal) in changes.into_boxed() {
        if !terminal {
            emit(
                out,
                Request::ChangeDecision {
                    task,
                    decision: temper_engine_domain_forge_change::Decision::Cancel,
                    entry: None,
                    evidence: None,
                },
            );
        }
        let mut writes = Queue::uncapped();
        super::room::drop_released(d, task, false, &mut writes);
        erasures(&mut writes, groups, out);
        // The two parts always share one decision, even at a page boundary.
        groups.push(Box::new([Key::ChangeCold(task), Key::ChangeHot(task)]));
    }
}

fn observations(d: &mut Domain, repository: Repository, groups: &mut Queue<Box<[Key]>>) {
    let capacity = d.heads.capacity();
    let mut rows = core::mem::replace(&mut d.heads, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let key = rows.first().expect("counted release rows").0.clone();
        let row = rows.remove(&key).expect("counted release rows");
        if owns(&key, repository) {
            erase(groups, Key::BranchHead(key));
        } else {
            d.heads.insert(key, row).expect("retained heads");
        }
    }
    let capacity = d.pulls.capacity();
    let mut rows = core::mem::replace(&mut d.pulls, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let key = rows.first().expect("counted release rows").0.clone();
        let row = rows.remove(&key).expect("counted release rows");
        if owns(&key, repository) {
            erase(groups, Key::PullState(key));
        } else {
            d.pulls.insert(key, row).expect("retained pulls");
        }
    }
    let capacity = d.subscriptions.capacity();
    let mut rows = core::mem::replace(&mut d.subscriptions, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let key = rows.first().expect("counted release rows").0.clone();
        let row = rows.remove(&key).expect("counted release rows");
        if topic(&row.topic, repository) {
            erase(groups, Key::Subscription { task: key.0, topic: key.1 });
        } else {
            d.subscriptions.insert(key, row).expect("retained subscriptions");
        }
    }
    let capacity = d.ci.capacity();
    let mut rows = core::mem::replace(&mut d.ci, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let key = *rows.first().expect("counted release rows").0;
        let row = rows.remove(&key).expect("counted release rows");
        if key.0 == repository {
            erase(groups, Key::Ci { repository, head: key.1 });
        } else {
            d.ci.insert(key, row).expect("retained ci");
        }
    }
    let capacity = d.verifications.capacity();
    let mut rows = core::mem::replace(&mut d.verifications, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let key = *rows.first().expect("counted release rows").0;
        let row = rows.remove(&key).expect("counted release rows");
        if key.0 == repository {
            erase(groups, Key::Verification { repository, commit: key.1 });
        } else {
            d.verifications.insert(key, row).expect("retained verifications");
        }
    }
}

fn applications(d: &mut Domain, repository: Repository, groups: &mut Queue<Box<[Key]>>) {
    let capacity = d.application.kept.capacity();
    let mut rows = core::mem::replace(&mut d.application.kept, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let key = *rows.first().expect("counted release rows").0;
        let row = rows.remove(&key).expect("counted release rows");
        if match &row.1 {
            crate::ApplicationOutcome::Adopted(adopted) => adopted.repository == repository,
            crate::ApplicationOutcome::CoreRefused(_)
            | crate::ApplicationOutcome::ForgeRefused(_)
            | crate::ApplicationOutcome::MakeKept
            | crate::ApplicationOutcome::Follow(_)
            | crate::ApplicationOutcome::MakeRefused(_) => false,
        } {
            erase(groups, Key::Application(key.raw()));
        } else {
            d.application.kept.insert(key, row).expect("retained application.kept");
        }
    }
}

fn names(d: &mut Domain, repository: Repository, groups: &mut Queue<Box<[Key]>>, out: &mut Queue<Request>) {
    let capacity = d.names.capacity();
    let mut rows = core::mem::replace(&mut d.names, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let task = *rows.first().expect("counted release rows").0;
        let names = rows.remove(&task).expect("counted release rows");
        let mut kept = List::with_capacity(u32::try_from(names.len()).expect("bounded names"));
        for name in &names {
            if !owns(name, repository) {
                kept.push(name.clone()).expect("fewer names");
            }
        }
        let resources = kept.into_boxed();
        if resources != names {
            if resources.is_empty() {
                erase(groups, Key::Names(task));
            } else {
                d.releasing.first_writes = d.releasing.first_writes.checked_add(1).expect("bounded name saves");
                emit(
                    out,
                    Request::Save {
                        read_afresh: None,
                        release: None,
                        record: Stored::Names { task, resources: resources.clone() },
                    },
                );
            }
        }
        if !resources.is_empty() {
            d.names.insert(task, resources).expect("retained names");
        }
    }
    let capacity = d.kept.capacity();
    let mut rows = core::mem::replace(&mut d.kept, Map::with_capacity(capacity));
    for _ in 0..rows.len() {
        let task = *rows.first().expect("counted release rows").0;
        let names = rows.remove(&task).expect("counted release rows");
        let mut kept = List::with_capacity(u32::try_from(names.len()).expect("bounded names"));
        for name in &names {
            if !owns(name, repository) {
                kept.push(name.clone()).expect("fewer names");
            }
        }
        let resources = kept.into_boxed();
        if resources != names {
            if resources.is_empty() {
                erase(groups, Key::Names(task));
            } else {
                d.releasing.first_writes = d.releasing.first_writes.checked_add(1).expect("bounded name saves");
                emit(
                    out,
                    Request::Save {
                        read_afresh: None,
                        release: None,
                        record: Stored::Names { task, resources: resources.clone() },
                    },
                );
            }
        }
        if !resources.is_empty() {
            d.kept.insert(task, resources).expect("retained names");
        }
    }
}

#[expect(clippy::too_many_lines, reason = "retirement visits each bounded volatile table once")]
pub(super) fn retire(d: &mut Domain, repository: Repository) {
    let mut keys = List::with_capacity(d.ci_news.len());
    for (key, _) in &d.ci_news {
        if let Some(row) = d.changes.get(&key.0)
            && row.repository == repository
        {
            keys.push(*key).expect("bounded CI news");
        }
    }
    for key in keys.into_boxed() {
        d.ci_news.remove(&key);
    }
    let mut keys = List::with_capacity(d.steps.len());
    for (key, row) in &d.steps {
        if let Some(change) = d.changes.get(&row.task)
            && change.repository == repository
        {
            keys.push(*key).expect("bounded step reads");
        }
    }
    for key in keys.into_boxed() {
        d.steps.remove(&key);
    }
    let mut keys = List::with_capacity(d.projection_due.len());
    for (key, _) in &d.projection_due {
        if let Some(row) = d.issues.get(key)
            && row.repository == repository
        {
            keys.push(*key).expect("bounded projection deadlines");
        }
    }
    for key in keys.into_boxed() {
        d.projection_due.remove(&key);
    }
    crate::adoption::retire_repository(d, repository);

    let mut keys = List::with_capacity(d.pull_mergeable.len());
    for (key, _) in &d.pull_mergeable {
        if owns(key, repository) {
            keys.push(key.clone()).expect("bounded pull_mergeable");
        }
    }
    for key in keys.into_boxed() {
        d.pull_mergeable.remove(&key);
    }
    let mut keys = List::with_capacity(d.lost.len());
    for (key, row) in &d.lost {
        if owns(&row.name, repository) {
            keys.push(*key).expect("bounded lost");
        }
    }
    for key in keys.into_boxed() {
        d.lost.remove(&key);
    }
    let mut keys = List::with_capacity(d.pending_ci.len());
    for (key, row) in &d.pending_ci {
        if row.repository == repository {
            keys.push(*key).expect("bounded pending_ci");
        }
    }
    for key in keys.into_boxed() {
        d.pending_ci.remove(&key);
    }
    let mut keys = List::with_capacity(d.goal_file_reads.len());
    for (key, row) in &d.goal_file_reads {
        if row.repository == repository {
            keys.push(*key).expect("bounded goal_file_reads");
        }
    }
    for key in keys.into_boxed() {
        d.goal_file_reads.remove(&key);
    }
    let mut keys = List::with_capacity(d.landings.len());
    for (key, row) in &d.landings {
        if topic(&row.topic, repository) {
            keys.push(*key).expect("bounded landings");
        }
    }
    for key in keys.into_boxed() {
        d.landings.remove(&key);
    }
    let mut keys = List::with_capacity(d.adoptions.len());
    for (key, row) in &d.adoptions {
        if row.repository == repository {
            keys.push(*key).expect("bounded adoptions");
        }
    }
    for key in keys.into_boxed() {
        d.adoptions.remove(&key);
    }
    let mut keys = List::with_capacity(d.capabilities.len());
    for (key, row) in &d.capabilities {
        if row.repository == repository {
            keys.push(*key).expect("bounded capabilities");
        }
    }
    for key in keys.into_boxed() {
        d.capabilities.remove(&key);
    }
    super::goal_branch::release_repository(d, repository);
    super::rebase::release_repository(d, repository);
    super::outage::release_repository(d, repository);
    crate::brief::release_repository(d, repository);
    crate::brief_document::release_repository(d, repository);
    crate::held::release_repository(d, repository);
    crate::workflows::release_repository(d, repository);
    crate::closer::release_repository(d, repository);
    crate::follows::release_repository(d, repository);
}

#[cfg(test)]
#[path = "../tests/repository_release.rs"]
mod tests;
