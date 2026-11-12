//! Cold restart/readback (jig/docs/design/domain/engine/README.md, 9;
//! temper/docs/design/testing/README.md, 2).
pub(crate) enum Fixture {
    Checked,
    Closed,
    Broken,
}

pub mod config;
pub mod durable;

use crate::direct::Driver;
use jig_domain_core::{
    self as core,
    reads::{Answer, Read},
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use std::{borrow::Cow, collections::BTreeMap};
use temper_engine_domain::{self as engine, Delivery, Key, Record};

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
            "keep::listed advances LiveRecord.listed from the startup listing; delivery positions and cached contents remain durable",
        )
    } else {
        None
    }
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
            for row in rows.values() {
                if let Record::Core(core::Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task))) = row {
                    queries.push((sign_in, Read::Task { task: task.number }));
                    queries.push((sign_in, Read::Result { task: task.number }));
                }
            }
        }
        Self { rows: rows.clone(), wall, queries, answers: Vec::new() }
    }

    pub fn compare_rows(&self, rows: &BTreeMap<Key, Record>) {
        for key in self.rows.keys().chain(rows.keys().filter(|key| !self.rows.contains_key(*key))) {
            let before = self.rows.get(key).and_then(|row| retained(key, row, self.wall));
            let after = rows.get(key).and_then(|row| retained(key, row, self.wall));
            if before != after {
                assert_eq!(
                    before,
                    after,
                    "restart oracle row {key:?}, field {}",
                    difference(&format!("{before:#?}"), &format!("{after:#?}"))
                );
            }
        }
    }

    pub fn compare_answer(&self, index: usize, answer: Answer) {
        let (sign_in, read) = &self.queries[index];
        let mut before = durable::answer(self.answers[index].clone());
        let mut after = durable::answer(answer);
        for answer in [&mut before, &mut after] {
            if let Answer::Document(document) = answer
                && let core::documents::Document::Task(task) = document.as_mut()
            {
                // jig/docs/design/client/domain/contract/documents.md 1: an omitted choice derives from current accounts and hosts.
                // compare_rows still compares the stored choice, including every named one.
                task.llm = None;
            }
        }
        if before != after {
            assert_eq!(
                before,
                after,
                "restart oracle document {read:?}, sign-in {sign_in}, field {}",
                difference(&format!("{before:#?}"), &format!("{after:#?}"))
            );
        }
    }
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
        }
    }
    Some(Cow::Owned(row))
}

impl Driver {
    /// The restart oracle (jig jig/docs/design/domain/engine/README.md 9; temper testing/README.md 2): settle, snapshot,
    /// restart cold over the same store, read back, compare through `durable`.
    /// Panics naming the first difference (document or row, field, key).
    pub fn close(&mut self) {
        self.close_erasing(None);
    }

    /// A deliberately broken fixture: `close` does nothing.
    pub fn broken(&mut self, reason: &str) {
        assert!(!reason.is_empty(), "a broken fixture names its reason");
        self.broken = Fixture::Broken;
    }

    /// Fixture fault at the snapshot/restart boundary, exercising the same oracle.
    pub fn close_erasing(&mut self, erase: Option<Key>) {
        if !matches!(self.broken, Fixture::Checked) {
            return;
        }
        self.oracle_settle();
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
        snapshot.compare_rows(&self.store.rows);
        self.root =
            engine::Domain::new(config::copy(&self.restart_config), &self.env.limits).expect("restart configuration");
        self.out.clear();
        self.events.clear();
        self.events.push_back(engine::Event::Restart { pending: self.store.pending_generation });
        self.period_sent = true;
        self.oracle_settle();
        snapshot.compare_rows(&self.store.rows);
        compare_forges(forges, self.root.forges());
        for (index, (sign_in, read)) in snapshot.queries.iter().enumerate() {
            let answer = self.oracle_read(*sign_in, read);
            snapshot.compare_answer(index, answer);
        }
        for ((sign_in, project, subject), before) in subjects.iter().zip(documents) {
            let after = self.oracle_document(*sign_in, *project, *subject);
            compare_forge(*project, *subject, before, after);
        }
        self.oracle_settle();
        self.broken = Fixture::Closed;
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
            self.advance(true);
            // Domain::quiescent also rejects future fleet deadlines. This
            // boundary drains immediate commits, deliveries and loads only.
            if self.root.ready()
                && self.root.held_bytes() == 0
                && self.events.is_empty()
                && self.out.is_empty()
                && self.store.idle()
                && self.store.held.is_empty()
                && self.forge.as_ref().is_none_or(crate::forge_stand::ForgeStand::quiescent)
                && self.store.applied == commit
            {
                quiet += 1;
                if quiet == 2 {
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
            | Delivery::NotesLoad { .. }
            | Delivery::ProjectMade { .. }
            | Delivery::Bound { .. }
            | Delivery::ProjectReleased { .. }
            | Delivery::MoveStanding { .. }
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
            | Delivery::ForgeCall { .. }
            | Delivery::Procedure { .. }
            | Delivery::Relay { .. }
            | Delivery::ReadEscalationDecision { .. }
            | Delivery::Fleet(_)
            | Delivery::Assigned { .. }
            | Delivery::WebReply { .. }
            | Delivery::CarriedUnused { .. }
            | Delivery::ForgeRead { .. }
            | Delivery::Read { .. }
            | Delivery::ReadLoad { .. }
            | Delivery::Public { .. }
            | Delivery::PublicWait { .. }
            | Delivery::Document { .. }
            | Delivery::WatchEnded { .. }
            | Delivery::Drained
            | Delivery::Refuse { .. }
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
        self.send(engine::Event::Party(engine::PartyInput::Read { reply_to, sign_in, read: read.clone() }));
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
) {
    let mut driver = Driver::cold(store.take_for_drop(), config, &env.limits);
    std::mem::swap(&mut driver.root, root);
    std::mem::swap(&mut driver.protocol, protocol);
    driver.env = env.with(env.limits);
    driver.period_sent = true;
    driver.events.clear();
    driver.close();
    std::mem::swap(&mut driver.root, root);
    std::mem::swap(&mut driver.store, store);
    std::mem::swap(&mut driver.protocol, protocol);
    driver.broken = Fixture::Broken;
}

/// Public forge subjects are discovered from retained owners, including ended tasks.
/// Historical Issue rows never require a live goal link (survey-1, 8, R17).
#[must_use]
pub fn forge_subjects(rows: &BTreeMap<Key, Record>) -> Vec<(u64, u32, u64)> {
    let mut subjects = std::collections::BTreeSet::new();
    for row in rows.values() {
        if let Record::Core(core::Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task))) = row {
            subjects.insert((task.project, task.number));
            for ended in [false, true] {
                subjects.insert((task.project, temper_engine_domain_forge::landings_subject(task.project, ended)));
            }
        }
        if let Record::Forge { row } = row
            && let temper_engine_domain_forge::Stored::Repository(repository) = row.as_ref()
        {
            for ended in [false, true] {
                subjects.insert((
                    repository.project,
                    temper_engine_domain_forge::landings_subject(repository.project, ended),
                ));
            }
        }
    }
    let mut queries = Vec::new();
    for row in rows.values() {
        if let Record::Core(core::Record::People(people::Stored::SignIn { number, .. })) = row {
            queries.extend(subjects.iter().map(|(project, subject)| (*number, *project, *subject)));
        }
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
) {
    let before = durable::forge(before);
    let after = durable::forge(after);
    if before != after {
        assert_eq!(
            before,
            after,
            "restart oracle forge document project {project}, key {subject}, field {}",
            difference(&format!("{before:#?}"), &format!("{after:#?}"))
        );
    }
}

pub fn compare_forges(before: temper_engine_domain_forge::Forges, after: temper_engine_domain_forge::Forges) {
    let before = durable::forges(before);
    let after = durable::forges(after);
    if before != after {
        assert_eq!(
            before,
            after,
            "restart oracle forge document forges, field {}",
            difference(&format!("{before:#?}"), &format!("{after:#?}"))
        );
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
