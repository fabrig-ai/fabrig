//! Cold restart/readback (jig/docs/design/domain/engine/world.md, 1;
//! temper/docs/design/testing/scenarios.md, 1.2).
pub(crate) enum Fixture {
    Checked,
    Closed,
    Broken,
    Transferred,
}

pub mod config;
pub mod durable;

use crate::direct::Driver;
use jig_domain_core::{
    self as core,
    reads::{Answer, Read},
};
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token};
use std::{borrow::Cow, collections::BTreeMap};
use temper_engine_domain::{self as engine, Delivery, Key, Record};
use temper_engine_domain_forge::Document as ForgeDocument;

/// The one inventory of rows rebuilt by startup, with the step's reason.
#[must_use]
pub fn rebuilt(key: &Key) -> Option<&'static str> {
    if *key == Key::Core(core::Key::Core(core::CoreKey::Deployment)) {
        Some("root startup commits its start mark; only Deployment.commits is rebuilt")
    } else if matches!(key, Key::Forge(temper_engine_domain_forge::Key::Client(skein_forge_client::Key::Repository(_))))
    {
        Some("keep::answered_repository rebuilds RepositoryRecord.clock and mark during startup's fresh scan")
    } else if matches!(key, Key::Forge(temper_engine_domain_forge::Key::Client(skein_forge_client::Key::Live(_)))) {
        Some(
            "keep::listed advances LiveRecord.listed from the startup listing; an entirely empty Repository Live row is derived startup bookkeeping and compares as absent; delivery positions and cached contents remain durable",
        )
    } else {
        None
    }
}

/// The unprojected readback of a successful closing restart, when requested.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CloseObservation {
    pub queries: Vec<(u64, Read)>,
    pub before: Vec<Answer>,
    pub after: Vec<Answer>,
    pub forge_subjects: Vec<(u64, u32, u64)>,
    pub forge_before: Vec<Option<ForgeDocument>>,
    pub forge_after: Vec<Option<ForgeDocument>>,
}

/// Last answers and retained rows at a quiescent boundary.
pub struct Snapshot {
    rows: BTreeMap<Key, Record>,
    wall: skein_lib::Wall,
    pub queries: Vec<(u64, Read)>,
    pub answers: Vec<Answer>,
}
impl Snapshot {
    #[must_use]
    pub fn new(rows: &BTreeMap<Key, Record>, wall: skein_lib::Wall) -> Self {
        let mut snapshot = Self::reads_only(rows, wall);
        snapshot.rows = rows.clone();
        snapshot
    }

    /// Capture public queries without copying durable rows. This snapshot is for
    /// answer comparisons only; row-comparing oracles use `new`.
    #[must_use]
    pub fn reads_only(rows: &BTreeMap<Key, Record>, wall: skein_lib::Wall) -> Self {
        let sessions: Vec<_> = rows
            .values()
            .filter_map(|row| {
                if let Record::Core(core::Record::People(people::Stored::SignIn { number, expires, .. })) = row {
                    (*expires > wall).then_some(*number)
                } else {
                    None
                }
            })
            .collect();
        let mut queries = Vec::new();
        for sign_in in sessions {
            queries.push((sign_in, Read::Frame));
            for task in crate::store::task_rows(rows) {
                queries.push((sign_in, Read::Task { task: task.number }));
                queries.push((sign_in, Read::Result { task: task.number }));
            }
        }
        Self { rows: BTreeMap::new(), wall, queries, answers: Vec::new() }
    }

    pub fn compare_rows(&self, rows: &BTreeMap<Key, Record>) -> Result<(), String> {
        // Walk both sorted maps once. Report a difference in the saved keys
        // before reporting a newly present key, as in the restart oracle.
        let mut afters = rows.iter().peekable();
        let mut extra = None;
        for (key, before) in &self.rows {
            while let Some((key, row)) = afters.next_if(|(after_key, _)| *after_key < key) {
                if extra.is_none()
                    && let Some(row) = retained(key, row, self.wall)
                {
                    extra = Some((key, row));
                }
            }
            let after = afters.next_if(|(after_key, _)| *after_key == key).map(|(_, row)| row);
            if after == Some(before) {
                continue;
            }
            let before = retained(key, before, self.wall);
            let after = after.and_then(|row| retained(key, row, self.wall));
            if before != after {
                return Err(row_difference(key, before.as_deref(), after.as_deref()));
            }
        }
        if let Some((key, after)) =
            extra.or_else(|| afters.find_map(|(key, row)| retained(key, row, self.wall).map(|row| (key, row))))
        {
            return Err(row_difference(key, None, Some(after.as_ref())));
        }
        Ok(())
    }

    pub fn compare_answer(&self, index: usize, answer: Answer) -> Result<(), String> {
        let before = &self.answers[index];
        // Exact equality also implies equality under the pair's durable projection.
        if *before == answer {
            return Ok(());
        }
        let mut before = before.clone();
        let mut after = answer;
        durable::answers(&mut before, &mut after);
        if before == after {
            return Ok(());
        }
        let (sign_in, read) = &self.queries[index];
        Err(format!(
            "assertion `left == right` failed: restart oracle document {read:?}, sign-in {sign_in}, field {}\n  left: {before:?}\n right: {after:?}",
            difference(&format!("{before:#?}"), &format!("{after:#?}"))
        ))
    }
}

fn row_difference(key: &Key, before: Option<&Record>, after: Option<&Record>) -> String {
    format!(
        "assertion `left == right` failed: restart oracle row {key:?}, field {}\n  left: {before:?}\n right: {after:?}",
        difference(&format!("{before:#?}"), &format!("{after:#?}"))
    )
}

// Only the inventory's rebuilt rows need an owned projection.
fn retained<'a>(key: &Key, row: &'a Record, wall: skein_lib::Wall) -> Option<Cow<'a, Record>> {
    if matches!(row, Record::Core(core::Record::People(people::Stored::SignIn { expires, .. })) if *expires <= wall) {
        return None;
    }
    if rebuilt(key).is_none() {
        return Some(Cow::Borrowed(row));
    }
    let mut row = row.clone();
    if let Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))) = &mut row {
        header.commits = 0;
    }
    if let Record::Forge { row } = &mut row {
        if let temper_engine_domain_forge::Stored::Client(skein_forge_client::Stored::Repository(record)) = row.as_mut()
        {
            record.clock = skein_lib::Time::ZERO;
            record.mark = None;
        }
        if let temper_engine_domain_forge::Stored::Client(skein_forge_client::Stored::Live(record)) = row.as_mut() {
            record.listed = skein_lib::Time::ZERO;
            if let Key::Forge(temper_engine_domain_forge::Key::Client(skein_forge_client::Key::Live(resource))) = key
                && resource == &record.watch.resource
                && empty_repository_live(record)
            {
                return None;
            }
        }
    }
    Some(Cow::Owned(row))
}

fn empty_repository_live(record: &skein_forge_client::LiveRecord) -> bool {
    record.watch.resource.what == skein_forge_client::What::Repository
        && !record.watch.participating
        && record.position
            == (skein_forge_client::Position { at: skein_lib::Time::ZERO, comment: 0, review: 0, review_page: 1 })
        && record.listed == skein_lib::Time::ZERO
        && record.comment_after == 0
        && record.comment_page == 1
        && record.comment_scanning
        && record.comment_repair
        && record.review_page == 1
        && record.comments.is_empty()
        && record.reviews.is_empty()
        && record.cached.item.is_none()
        && record.cached.pull.is_none()
        && record.cached.tip.is_none()
        && record.pushed.is_none()
        && record.echoes.is_empty()
}

impl Driver {
    /// The restart oracle (jig jig/docs/design/domain/engine/world.md 1; temper testing/scenarios.md 1.2): settle, snapshot,
    /// restart cold over the same store, read back, compare through `durable`.
    /// Panics naming the first difference (document or row, field, key).
    pub fn close(&mut self) {
        self.close_erasing(None).unwrap_or_else(|finding| panic!("{finding}"));
    }

    /// A deliberately broken fixture: `close` does nothing.
    pub fn broken(&mut self, reason: &str) {
        assert!(!reason.is_empty(), "a broken fixture names its reason");
        self.broken = Fixture::Broken;
        self.ends.no_opening(self.env.now);
    }

    /// Fixture fault at the snapshot/restart boundary, returning the oracle's first difference.
    pub fn close_erasing(&mut self, erase: Option<Key>) -> Result<(), String> {
        let result = if self.store_finding.is_some() {
            self.broken = Fixture::Closed;
            self.check_store()
        } else {
            match self.broken {
                Fixture::Checked => {
                    // A finding ends this attempt too; Drop must not repeat a failed readback.
                    self.broken = Fixture::Closed;
                    self.restart_oracle(erase)
                }
                Fixture::Broken | Fixture::Closed => self.ends.close(self.env.now, &self.store),
                Fixture::Transferred => Ok(()),
            }
        };
        let kept = self.store_checks.finish(&self.restart_config);
        result.and(kept)
    }

    fn restart_oracle(&mut self, erase: Option<Key>) -> Result<(), String> {
        let mut observation = self.close_observation.take();
        self.oracle_settle();
        self.ends.close(self.env.now, &self.store)?;
        assert!(self.store.idle() && self.store.held.is_empty(), "restart oracle store settled");
        let forges = self.root.forges();
        let mut snapshot = Snapshot::new(&self.store.rows, self.env.wall);
        for (sign_in, read) in &snapshot.queries {
            snapshot.answers.push(self.oracle_read(*sign_in, read));
        }
        let subjects = if self.forge.is_some() { forge_subjects(&self.store.rows) } else { Vec::new() };
        let mut documents = Vec::new();
        for (sign_in, project, subject) in &subjects {
            documents.push(self.oracle_document(*sign_in, *project, *subject));
        }
        self.oracle_settle();
        if let Some(key) = erase {
            self.store.seed(Box::new([]), Box::new([key]));
        }
        self.store.crash(0);
        // Check the crashed inventory before startup can mask a missing writer row.
        snapshot.compare_rows(&self.store.rows)?;
        self.root =
            engine::Domain::new(config::copy(&self.restart_config), &self.env.limits).expect("restart configuration");
        self.out.clear();
        self.events.clear();
        self.events.push_back(engine::Event::Restart { pending: self.store.pending_generation });
        self.period_sent = true;
        self.ends.restarted(self.env.now);
        self.oracle_settle();
        snapshot.compare_rows(&self.store.rows)?;
        compare_forges(forges, self.root.forges())?;
        for (index, (sign_in, read)) in snapshot.queries.iter().enumerate() {
            let answer = self.oracle_read(*sign_in, read);
            if let Some(observation) = &mut observation {
                observation.after.push(answer.clone());
            }
            snapshot.compare_answer(index, answer)?;
        }
        for ((sign_in, project, subject), before) in subjects.iter().zip(documents) {
            let after = self.oracle_document(*sign_in, *project, *subject);
            if let Some(observation) = &mut observation {
                observation.forge_before.push(before.clone());
                observation.forge_after.push(after.clone());
            }
            compare_forge(*project, *subject, before, after)?;
        }
        self.oracle_settle();
        self.ends.close(self.env.now, &self.store)?;
        self.check_store()?;
        if let Some(observation) = &mut observation {
            observation.queries = snapshot.queries;
            observation.before = snapshot.answers;
            observation.forge_subjects = subjects;
        }
        self.close_observation = observation;
        Ok(())
    }

    fn oracle_settle(&mut self) {
        self.protocol.defer_render = false;
        for (to, name) in std::mem::take(&mut self.protocol.pending_renders) {
            self.events.push_back(engine::Event::Worker(engine::WorkerInput::RenderedHostCall {
                to,
                answer: core::SettledAnswer::Host { error: false, body: name },
            }));
        }
        let mut quiet = 0;
        for _ in 0..200 {
            while let Some(number) = self.store.release_held() {
                self.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
            }
            let commit = self.store.applied;
            if self.events.is_empty()
                && self.out.is_empty()
                && self.store.idle()
                && self.forge.as_ref().is_none_or(crate::forge_stand::ForgeStand::quiescent)
                && self.store.has_due_change()
                && let Some(due) = self.root.next_deadline(&self.env)
                && due <= self.env.now.saturating_add(skein_lib::Duration::from_secs(1))
            {
                self.advance_to(self.env.now.max(due));
                engine::fire(&mut self.root, &self.env);
                engine::release(&mut self.root, &self.env, &mut self.out);
                self.collect();
            }
            self.advance(true);
            if self.idle_since(commit) {
                quiet += 1;
                if quiet == 2 {
                    self.check_quiet();
                    return;
                }
            } else {
                quiet = 0;
            }
        }
        panic!("restart oracle direct world did not settle: {:?}", self.root);
    }

    fn oracle_document(
        &mut self,
        sign_in: u64,
        project: u32,
        subject: u64,
    ) -> Option<temper_engine_domain_forge::Document> {
        let watcher = Token::new(0x4e34);
        self.delivered.clear();
        self.send(engine::Event::Party(engine::PartyInput::Watch {
            watcher,
            sign_in,
            subject: core::documents::Watched::Application {
                scope: people::ApplicationScope::Project(project),
                connector: self.restart_config.core.connectors[0],
                subject: Token::new(subject),
            },
            window: core::documents::Window { rows: self.env.limits.documents.rows, after: None },
        }));
        self.oracle_settle();
        let document = self.delivered.iter().rev().find_map(|delivery| match delivery {
            Delivery::ForgeDocument { watcher: got, document } if *got == watcher => Some(*document.clone()),
            Delivery::Panel { watcher: got, panel } if *got == watcher => {
                Some(temper_engine_domain_forge::Document::Change(*panel.clone()))
            }
            Delivery::Push(_)
            | Delivery::Start(_)
            | Delivery::ProjectMade { .. }
            | Delivery::Bound { .. }
            | Delivery::ProjectReleased { .. }
            | Delivery::MoveStanding { .. }
            | Delivery::FindingsOwned { .. }
            | Delivery::Providers(..)
            | Delivery::WorkerFigures { .. }
            | Delivery::PlaygroundNote { .. }
            | Delivery::Panel { .. }
            | Delivery::ForgeDocument { .. }
            | Delivery::Trials { .. }
            | Delivery::Ways { .. }
            | Delivery::Approvals { .. }
            | Delivery::Forges { .. }
            | Delivery::SourceSignedOut { .. }
            | Delivery::Application { .. }
            | Delivery::Host(_)
            | Delivery::AssignedJob { .. }
            | Delivery::Link { .. }
            | Delivery::ForgeCommitted { .. }
            | Delivery::TakeForgeAdoption { .. }
            | Delivery::ForgeCall { .. }
            | Delivery::Procedure { .. }
            | Delivery::Relay { .. }
            | Delivery::Fleet(_)
            | Delivery::Assigned { .. }
            | Delivery::WebReply { .. }
            | Delivery::CarriedUnused { .. }
            | Delivery::ForgeRead { .. }
            | Delivery::Read { .. }
            | Delivery::PartLoad { .. }
            | Delivery::EngineStore(_)
            | Delivery::Public { .. }
            | Delivery::PublicWait { .. }
            | Delivery::Document { .. }
            | Delivery::WatchEnded { .. }
            | Delivery::Restarting { .. }
            | Delivery::LimitSettingsCommitted { .. }
            | Delivery::Drained
            | Delivery::Refuse { .. }
            | Delivery::PartBusy { .. }
            | Delivery::TurnBusy { .. }
            | Delivery::Load { .. }
            | Delivery::Result { .. } => None,
        });
        self.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher }));
        self.oracle_settle();
        document
    }

    fn oracle_read(&mut self, sign_in: u64, read: &Read) -> Answer {
        self.serial += 1;
        let token = Token::new(self.serial);
        let reply_to = ReplyTo::new(token);
        self.send(engine::Event::Party(engine::PartyInput::Read { reply_to, sign_in, read: Box::new(read.clone()) }));
        for _ in 0..200 {
            if let Some(index) = self
                .delivered
                .iter()
                .position(|delivery| matches!(delivery, Delivery::Read { reply, .. } if *reply == ReplyTo::new(token)))
            {
                let Delivery::Read { answer, .. } = self.delivered.remove(index) else { unreachable!() };
                return answer;
            }
            self.advance(true);
        }
        panic!("restart oracle read {read:?}, sign-in {sign_in} did not answer");
    }
}

/// Transfer a settled story's live root to the shared oracle, then return its cold root.
pub fn close_world(
    root: &mut engine::Domain,
    env: &skein_lib::Env<engine::Limits>,
    store: &mut crate::store::Store,
    protocol: &mut crate::protocol::Protocol,
    config: engine::Config,
) -> Result<(), String> {
    let mut driver = Driver::cold(store.take_for_drop(), config, &env.limits);
    std::mem::swap(&mut driver.root, root);
    std::mem::swap(&mut driver.protocol, protocol);
    driver.env = env.with(env.limits);
    driver.period_sent = true;
    // The borrowed root already opened in its owning world's driver.
    driver.ends.no_opening(env.now);
    driver.events.clear();
    let finding = driver.close_erasing(None);
    std::mem::swap(&mut driver.root, root);
    std::mem::swap(&mut driver.store, store);
    std::mem::swap(&mut driver.protocol, protocol);
    driver.broken = Fixture::Broken;
    finding
}

/// Public forge subjects are discovered from retained owners, including ended tasks.
/// Historical Issue rows never require a live goal link (survey-1, 8, R17).
#[must_use]
pub fn forge_subjects(rows: &BTreeMap<Key, Record>) -> Vec<(u64, u32, u64)> {
    let mut subjects = std::collections::BTreeSet::new();
    let mut projects = std::collections::BTreeSet::new();
    for task in crate::store::task_rows(rows) {
        subjects.insert((task.project, task.number));
        projects.insert(task.project);
    }
    let mut sessions = Vec::new();
    for row in rows.values() {
        match row {
            Record::Forge { row } => {
                if let temper_engine_domain_forge::Stored::Repository(repository) = row.as_ref() {
                    projects.insert(repository.project);
                }
            }
            Record::Core(core::Record::People(people::Stored::SignIn { number, .. })) => sessions.push(*number),
            Record::Core(_) => {}
        }
    }
    for project in projects {
        for ended in [false, true] {
            subjects.insert((project, temper_engine_domain_forge::landings_subject(project, ended)));
        }
    }
    let mut queries = Vec::with_capacity(sessions.len() * subjects.len());
    for sign_in in sessions {
        queries.extend(subjects.iter().map(|(project, subject)| (sign_in, *project, *subject)));
    }
    queries
}

/// Equality is typed; pretty Debug is used only to label the first differing field.
#[must_use]
pub fn difference(before: &str, after: &str) -> String {
    let mut field = "presence/variant";
    for (left, right) in before.lines().zip(after.lines()) {
        if let Some((name, _)) = left.trim().split_once(':') {
            field = name;
        }
        if left != right {
            return field.into();
        }
    }
    field.into()
}

pub fn compare_forge(
    project: u32,
    subject: u64,
    before: Option<temper_engine_domain_forge::Document>,
    after: Option<temper_engine_domain_forge::Document>,
) -> Result<(), String> {
    let before = durable::forge(before);
    let after = durable::forge(after);
    if before != after {
        return Err(format!(
            "assertion `left == right` failed: restart oracle forge document project {project}, key {subject}, field {}\n  left: {before:?}\n right: {after:?}",
            difference(&format!("{before:#?}"), &format!("{after:#?}"))
        ));
    }
    Ok(())
}

pub fn compare_forges(
    before: temper_engine_domain_forge::Forges,
    after: temper_engine_domain_forge::Forges,
) -> Result<(), String> {
    let before = durable::forges(before);
    let after = durable::forges(after);
    if before != after {
        return Err(format!(
            "assertion `left == right` failed: restart oracle forge document forges, field {}\n  left: {before:?}\n right: {after:?}",
            difference(&format!("{before:#?}"), &format!("{after:#?}"))
        ));
    }
    Ok(())
}

impl Drop for Driver {
    fn drop(&mut self) {
        let result = if std::thread::panicking() { Ok(()) } else { self.close_erasing(None) };
        if !matches!(self.broken, Fixture::Transferred) {
            eprintln!("{}", self.store_check_counts().report());
        }
        result.unwrap_or_else(|finding| panic!("{finding}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_domain_tasks as tasks;

    #[test]
    fn closing_twice_keeps_the_first_cold_readback() {
        let mut driver = Driver::new(crate::store::Store::new());
        driver.close();
        let rows = driver.store.rows.clone();
        let commits = driver.store.applied;
        driver.close();
        assert_eq!(driver.store.rows, rows);
        assert_eq!(driver.store.applied, commits, "one cold restart per story");
        assert!(matches!(driver.broken, Fixture::Closed));
    }

    #[test]
    fn a_changed_answer_names_its_query_and_sign_in() {
        let before = Answer::None(core::reads::Why::Unknown);
        let mut snapshot = Snapshot::new(&BTreeMap::new(), skein_lib::Wall::EPOCH);
        snapshot.queries.push((42, Read::Result { task: 7 }));
        snapshot.answers.push(before.clone());
        assert_eq!(snapshot.compare_answer(0, before), Ok(()));
        let finding = snapshot.compare_answer(0, Answer::None(core::reads::Why::Gone)).expect_err("changed readback");
        assert!(finding.contains("restart oracle document Result { task: 7 }, sign-in 42, field"), "{finding}");
        assert!(finding.contains("Unknown") && finding.contains("Gone"), "{finding}");
    }

    #[test]
    fn task_answers_compare_only_durable_wait_projections() {
        use jig_domain_core::documents::{Document, PhaseLine, TaskRunWait, TaskSourceState, TaskWaitWhy};
        use jig_domain_core::{AtBound, Bound, Nearest, Wait, WaitKind, WaitObject};
        use skein_lib::Wall;

        let (mut driver, assignment) = Driver::chat();
        let sign_in = driver.session();
        let task = assignment.task;
        let read = Read::Task { task };
        let Answer::Document(document) = driver.oracle_read(sign_in, &read) else { panic!("task answer") };
        let Document::Task(mut cold) = *document else { panic!("task document") };
        cold.chip.phase = PhaseLine::Active(tasks::Active::Due);
        assert!(cold.wait.is_none() && cold.nearest.is_none() && cold.waits.is_empty());
        let answer = |row| Answer::Document(Box::new(Document::Task(row)));
        let at = Wall::from_nanos(100);
        let then = AtBound::Hold;
        let run_wait = TaskRunWait::Account {
            account: Some(1),
            source: Some(1),
            why: TaskWaitWhy::Source(TaskSourceState::Limited { until: None }),
        };
        let account = Wait {
            task,
            kind: WaitKind::Account,
            object: WaitObject::Run(run_wait),
            since: Wall::EPOCH,
            bound: Bound::At { at, then },
            hears: None,
        };
        let mut waiting = cold.clone();
        waiting.run_wait = Some(run_wait);
        waiting.wait = Some(account.clone());
        waiting.nearest = Some(Nearest { at, then, kind: account.kind });
        waiting.waits.push(account.clone()).expect("account wait line");
        waiting.wait_count = 1;
        let mut snapshot = Snapshot::new(&BTreeMap::new(), Wall::EPOCH);
        snapshot.queries.push((sign_in, read));
        snapshot.answers.push(answer(waiting));
        assert_eq!(snapshot.compare_answer(0, answer(cold.clone())), Ok(()));

        let dependency = Wait { kind: WaitKind::Dependency, object: WaitObject::Task(task + 1), ..account };
        cold.wait = Some(dependency.clone());
        cold.nearest = Some(Nearest { at, then, kind: dependency.kind });
        cold.waits.push(dependency).expect("dependency wait line");
        cold.wait_count = 1;
        snapshot.answers[0] = answer(cold.clone());
        for field in ["wait", "nearest"] {
            let mut changed = cold.clone();
            if field == "wait" {
                changed.wait = None;
            } else {
                changed.nearest = None;
            }
            let finding = snapshot.compare_answer(0, answer(changed)).expect_err("durable projection changed");
            assert!(finding.contains(&format!("field {field}\n")), "{finding}");
        }
    }

    #[test]
    fn task_answers_discard_rebuilt_procedure_waits_but_keep_dependencies() {
        use jig_domain_core::documents::{Document, PhaseLine, WaitsOn};
        use jig_domain_core::{AtBound, Bound, Nearest, Unbounded, Wait, WaitKind, WaitObject};
        use skein_lib::{List, Wall};

        let (mut driver, assignment) = Driver::chat();
        let sign_in = driver.session();
        let task = assignment.task;
        let read = Read::Task { task };
        let Answer::Document(document) = driver.oracle_read(sign_in, &read) else { panic!("task answer") };
        let Document::Task(mut cold) = *document else { panic!("task document") };
        cold.chip.phase = PhaseLine::Active(tasks::Active::Idle);
        assert!(cold.wait.is_none() && cold.nearest.is_none() && cold.waits.is_empty());
        let answer = |row| Answer::Document(Box::new(Document::Task(row)));

        let at = Wall::from_nanos(100);
        let room = Wait {
            task,
            kind: WaitKind::Room,
            object: WaitObject::Procedure {
                step: temper_client_protocol_wire::waits::CHANGE_PRODUCING,
                things: List::with_capacity(0),
            },
            since: Wall::from_nanos(20),
            bound: Bound::At { at, then: AtBound::Hold },
            hears: None,
        };
        let mut waiting = cold.clone();
        waiting.wait = Some(room.clone());
        waiting.nearest = Some(Nearest { at, then: AtBound::Hold, kind: WaitKind::Room });
        waiting.waits.push(room.clone()).expect("room wait line");
        waiting.wait_count = 1;
        let mut snapshot = Snapshot::new(&BTreeMap::new(), Wall::EPOCH);
        snapshot.queries.push((sign_in, read));
        snapshot.answers.push(answer(waiting.clone()));
        assert_eq!(snapshot.compare_answer(0, answer(cold.clone())), Ok(()));

        let news = Wait {
            kind: WaitKind::News,
            object: WaitObject::News { subscriptions: List::with_capacity(0), timer: None },
            since: Wall::from_nanos(30),
            ..room
        };
        cold.wait = Some(news.clone());
        cold.nearest = Some(Nearest { at, then: AtBound::Hold, kind: WaitKind::News });
        cold.waits_on = WaitsOn::News;
        cold.waits.push(news.clone()).expect("news fallback line");
        cold.wait_count = 1;
        assert_eq!(snapshot.compare_answer(0, answer(cold.clone())), Ok(()));

        let dependency = Wait {
            kind: WaitKind::Dependency,
            object: WaitObject::Task(task + 1),
            since: Wall::from_nanos(10),
            bound: Bound::None(Unbounded::NoDeadline),
            ..news
        };
        for row in [&mut waiting, &mut cold] {
            row.waits_on = WaitsOn::Dependencies { pending: List::from_boxed(Box::new([task + 1])), count: 1 };
            row.wait = Some(dependency.clone());
            let report = row.waits.get_mut(0).expect("room or fallback line");
            let report = std::mem::replace(report, dependency.clone());
            row.waits.push(report).expect("report after dependency");
            row.wait_count = 2;
        }
        snapshot.answers[0] = answer(waiting);
        assert_eq!(snapshot.compare_answer(0, answer(cold.clone())), Ok(()));
        cold.waits.get_mut(0).expect("retained dependency line").since = Wall::from_nanos(11);
        let finding = snapshot.compare_answer(0, answer(cold)).expect_err("dependency wait line changed");
        assert!(
            finding.contains(&format!(
                "restart oracle document Task {{ task: {task} }}, sign-in {sign_in}, field since\n"
            )),
            "{finding}"
        );
    }

    #[test]
    fn landings_documents_compare_retained_times_not_the_read_clock() {
        let before = forge::LandingsView {
            now: skein_lib::Wall::from_nanos(7),
            turn_since: Some(skein_lib::Wall::from_nanos(3)),
            project: 1,
            turn: Some(61),
            landings: Box::new([]),
        };
        let mut after = before.clone();
        after.now = skein_lib::Wall::from_nanos(11);
        assert_eq!(
            compare_forge(
                1,
                2,
                Some(forge::Document::Landings(before.clone())),
                Some(forge::Document::Landings(after.clone()))
            ),
            Ok(())
        );
        after.turn_since = None;
        assert!(
            compare_forge(1, 2, Some(forge::Document::Landings(before)), Some(forge::Document::Landings(after)))
                .is_err(),
            "the turn's stored start still compares"
        );
    }

    #[test]
    fn a_missing_forge_document_names_its_project_and_key() {
        let before = Some(forge::Document::Dismissals(Box::new([])));
        assert_eq!(compare_forge(42, 7, before.clone(), before.clone()), Ok(()));
        let finding = compare_forge(42, 7, before, None).expect_err("lost forge document");
        assert!(finding.contains("restart oracle forge document project 42, key 7, field"), "{finding}");
        assert!(finding.contains("Dismissals") && finding.contains("None"), "{finding}");
    }

    #[test]
    fn a_missing_forge_names_the_forges_document() {
        let mut driver = Driver::new(crate::store::Store::new());
        driver.settle();
        let before = driver.root.forges();
        assert!(!before.rows.is_empty(), "fixture configures a forge");
        assert_eq!(compare_forges(before.clone(), before.clone()), Ok(()));
        let finding =
            compare_forges(before, forge::Forges { full: Box::new([]), rows: Box::new([]) }).expect_err("lost forge");
        assert!(finding.contains("restart oracle forge document forges, field"), "{finding}");
        assert!(finding.contains("left: Forges") && finding.contains("right: Forges"), "{finding}");
    }

    use skein_forge_client as client;
    use skein_lib::Time;
    use temper_engine_domain_forge as forge;

    fn empty_live() -> client::LiveRecord {
        client::LiveRecord {
            watch: client::Watch {
                resource: client::Resource {
                    repository: skein_forge::api::Repository { forge: 1, repository: 2 },
                    what: client::What::Repository,
                },
                participating: false,
            },
            position: client::Position { at: Time::ZERO, comment: 0, review: 0, review_page: 1 },
            listed: Time::from_nanos(100),
            comment_after: 0,
            comment_page: 1,
            comment_scanning: true,
            comment_repair: true,
            review_page: 1,
            comments: Box::new([]),
            reviews: Box::new([]),
            cached: Box::new(client::Cached { item: None, pull: None, tip: None }),
            pushed: None,
            echoes: Box::new([]),
        }
    }

    fn live_row(record: client::LiveRecord) -> (Key, Record) {
        let key = Key::Forge(forge::Key::Client(client::Key::Live(record.watch.resource.clone())));
        (key, Record::Forge { row: Box::new(forge::Stored::Client(client::Stored::Live(record))) })
    }

    #[test]
    fn an_empty_repository_live_row_is_only_startup_bookkeeping() {
        let (key, row) = live_row(empty_live());
        assert!(retained(&key, &row, skein_lib::Wall::EPOCH).is_none());
        let rows = BTreeMap::from([(key, row)]);
        assert_eq!(Snapshot::new(&BTreeMap::new(), skein_lib::Wall::EPOCH).compare_rows(&rows), Ok(()));
        assert_eq!(Snapshot::new(&rows, skein_lib::Wall::EPOCH).compare_rows(&BTreeMap::new()), Ok(()));
    }

    #[test]
    fn live_delivery_positions_and_cached_content_are_retained() {
        type Change = fn(&mut client::LiveRecord);
        let cases: &[(&str, Change)] = &[
            ("position.at", |row| row.position.at = Time::from_nanos(1)),
            ("position.comment", |row| row.position.comment = 1),
            ("position.review", |row| row.position.review = 1),
            ("position.review_page", |row| row.position.review_page = 2),
            ("comment_after", |row| row.comment_after = 1),
            ("comment_page", |row| row.comment_page = 2),
            ("comment_scanning", |row| row.comment_scanning = false),
            ("comment_repair", |row| row.comment_repair = false),
            ("review_page", |row| row.review_page = 2),
            ("comments", |row| row.comments = Box::new([client::Delivery { id: 1, revision: 1 }])),
            ("reviews", |row| row.reviews = Box::new([client::Delivery { id: 1, revision: 1 }])),
            ("cached.item", |row| {
                row.cached.item = Some(skein_forge::api::Summary {
                    labels: Box::new([]),
                    number: 1,
                    kind: skein_forge::api::Kind::Issue,
                    state: skein_forge::api::State::Open,
                    author: 7,
                    key: None,
                    title: b"cached".as_slice().into(),
                    body: Box::new([]),
                    updated: Time::ZERO,
                });
            }),
            ("cached.pull", |row| {
                row.cached.pull = Some(skein_forge::api::Pull {
                    reviewers: Box::new([]),
                    number: 1,
                    state: skein_forge::api::State::Open,
                    head: b"topic".as_slice().into(),
                    base: b"main".as_slice().into(),
                    commit: [1; 32],
                    base_commit: None,
                    merged: None,
                    mergeable: true,
                    ci: skein_forge::api::Ci::None,
                    queued: None,
                    dequeued: None,
                    size: None,
                });
            }),
            ("cached.tip", |row| row.cached.tip = Some([1; 32])),
            ("pushed", |row| row.pushed = Some([1; 32])),
            ("echoes", |row| row.echoes = Box::new([client::Echo::Comment(1)])),
            ("participating", |row| row.watch.participating = true),
            ("another resource", |row| row.watch.resource.what = client::What::Branch(b"main".as_slice().into())),
        ];
        for (name, change) in cases {
            let mut row = empty_live();
            change(&mut row);
            let mut projected = row.clone();
            projected.listed = Time::ZERO;
            let (key, record) = live_row(row);
            let (_, expected) = live_row(projected);
            assert_eq!(retained(&key, &record, skein_lib::Wall::EPOCH).as_deref(), Some(&expected), "{name}");
        }
        let (mut key, record) = live_row(empty_live());
        if let Key::Forge(forge::Key::Client(client::Key::Live(resource))) = &mut key {
            resource.repository.repository = 3;
        }
        assert!(retained(&key, &record, skein_lib::Wall::EPOCH).is_some(), "a mismatched key is not bookkeeping");
    }

    #[test]
    fn dropping_a_live_delivery_position_is_still_a_writer_fault() {
        let mut row = empty_live();
        row.position.comment = 1;
        let (key, record) = live_row(row);
        let label = format!("restart oracle row {key:?}");
        let rows = BTreeMap::from([(key, record)]);
        let finding = Snapshot::new(&rows, skein_lib::Wall::EPOCH)
            .compare_rows(&BTreeMap::new())
            .expect_err("lost delivery position");
        assert!(finding.contains(&label), "{finding}");
    }

    #[test]
    fn inventing_live_cached_content_is_still_a_writer_fault() {
        let mut row = empty_live();
        row.cached.tip = Some([1; 32]);
        let (key, record) = live_row(row);
        let label = format!("restart oracle row {key:?}");
        let rows = BTreeMap::from([(key, record)]);
        let finding = Snapshot::new(&BTreeMap::new(), skein_lib::Wall::EPOCH)
            .compare_rows(&rows)
            .expect_err("invented cached content");
        assert!(finding.contains(&label), "{finding}");
    }
}
