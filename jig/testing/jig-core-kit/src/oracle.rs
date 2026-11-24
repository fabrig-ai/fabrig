//! The cold restart oracle (`jig/docs/design/domain/engine/restart.md` and `jig/docs/design/domain/engine/world.md`, section 1).
use crate::{reads, world::World};
use jig_domain_accounts as accounts;
use jig_domain_core::{self as core, documents::Document};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_protocol_store::keys;
use jig_test_domain as root;
use std::collections::{BTreeMap, BTreeSet};

type Answers = BTreeMap<(u64, Option<u64>), core::reads::Answer>;

/// The documents compared by the closing oracle, keyed by sign-in and optional task.
#[derive(Debug, PartialEq, Eq)]
pub struct Documents {
    pub before: Answers,
    pub after: Answers,
}

pub(crate) fn configuration(source: &root::Config) -> root::Config {
    let core = &source.core;
    let settings = &core.settings;
    root::Config {
        first: source.first.clone(),
        second: source.second.clone(),
        core: core::Config {
            fixed_providers: core.fixed_providers.clone(),
            parts: core.parts.clone(),
            jobs: core.jobs.clone(),
            kept_starts: core.kept_starts.clone(),

            deployment: core.deployment,
            seed: core.seed,
            workers_in_process: core.workers_in_process,
            hub_turns: core.hub_turns,
            owners: core.owners.clone(),
            grants: core.grants.clone(),
            kinds: core.kinds.clone(),
            defaults: core::settings::Defaults {
                limits: core.defaults.limits.clone(),
                providers: core.defaults.providers.clone(),
                workers: core.defaults.workers.clone(),
                models: core.defaults.models.clone(),
                charters: core.defaults.charters.clone(),
                templates: core.defaults.templates.clone(),
                policies: core.defaults.policies.clone(),
                rules: core.defaults.rules.clone(),
            },
            connectors: core.connectors.clone(),
            seed_projects: core
                .seed_projects
                .iter()
                .map(|project| core::SeedProject {
                    number: project.number,
                    name: project.name.clone(),
                    policy: project.policy.clone(),
                })
                .collect(),
            settings: core::Settings {
                private_tools: settings.private_tools.clone(),
                endpoints: settings.endpoints.clone(),
                deployment_provider: settings.deployment_provider,
                models: settings.models.clone(),
                resume_bytes: settings.resume_bytes,
                tail_bytes: settings.tail_bytes,
                tools: settings.tools,
                llm_providers: settings.llm_providers.clone(),
                key_bytes: settings.key_bytes,
                redirect_bytes: settings.redirect_bytes,
            },
        },
    }
}

impl Drop for World {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.close();
        }
        self.report_store_checks();
    }
}

impl World {
    /// Declare this world a deliberately broken fixture: `close` does nothing.
    pub fn broken(&mut self) {
        self.broken = true;
    }

    /// The restart oracle (domain/engine/world.md 1): settle, snapshot what the parties were last
    /// answered and the store's retained rows, restart cold, read back, and compare
    /// through `durable`. Panics naming the first difference (document, field, key).
    pub fn close(&mut self) {
        if let Err(message) = self.finish() {
            panic!("{message}");
        }
    }

    /// Close the restart oracle, then rehearse the kept commit. Return the first finding.
    pub fn finish(&mut self) -> Result<(), String> {
        let oracle = self.close_oracle();
        let check = self.finish_store_check();
        self.report_store_checks();
        oracle.and(check)
    }

    fn report_store_checks(&mut self) {
        if !self.store_checks_reported {
            eprintln!("{}", self.store_checks.report());
            self.store_checks_reported = true;
        }
    }

    fn close_oracle(&mut self) -> Result<(), String> {
        if self.broken || self.oracle_closed {
            return Ok(());
        }
        self.oracle_closed = true;
        self.manual = false;
        self.walking = None;
        // Oracle transport does not advance the scenario's clocks or age its deadlines.
        let (now, wall) = self.clocks();
        self.set_clock(now, wall);
        self.settle_oracle()?;
        let inventory = self.store.retained().map_err(|finding| format!("restart oracle readback: {finding:?}"))?;
        let records =
            self.store.readback_of(&inventory).map_err(|finding| format!("restart oracle readback: {finding:?}"))?;
        let forgotten = self.forgotten_due(&records);
        let (parties, tasks) = self.oracle_read_set(&records);
        let mut before = self.oracle_documents(&parties, &tasks);
        self.restart_configured(configuration(&self.restart_config.0))?;
        self.settle_oracle()?;
        if !self.domain.ready() {
            return Err("restart oracle: decisions did not open".into());
        }
        let mut after = self.oracle_documents(&parties, &tasks);
        for (key, expected) in &mut before {
            let found = after.get_mut(key).expect("same oracle read set");
            durable_answers(expected, found);
            compare_answer(key, expected, found, &forgotten)?;
        }
        let restored = self.store.retained().map_err(|finding| format!("restart oracle readback: {finding:?}"))?;
        let recovered =
            self.store.readback_of(&restored).map_err(|finding| format!("restart oracle readback: {finding:?}"))?;
        let mut expected_header = deployment(&records)?;
        let found_header = deployment(&recovered)?;
        expected_header.commits = found_header.commits;
        // A cold start discharges the restart word once none of its claims
        // remains held (domain/engine/restart.md, Held across starts).
        if found_header.restart_wait_ms.is_none() {
            expected_header.restart_wait_ms = None;
        }
        if expected_header != found_header {
            return Err(format!(
                "restart oracle deployment field at key Deployment: expected {expected_header:?}, found {found_header:?}"
            ));
        }
        for key in inventory.keys().chain(restored.keys()).collect::<BTreeSet<_>>() {
            let exception = rebuilt(key);
            let header = keys::decode(key) == Some(core::Key::Core(core::CoreKey::Deployment));
            let wires = keys::decode_own(key) == Some(keys::Own::Wires);
            let expired = exception.is_some()
                && !restored.contains_key(key)
                && keys::decode(key).is_some_and(|key| self.expired(&records, &key));
            let forgotten_erasure = inventory.contains_key(key)
                && !restored.contains_key(key)
                && forgotten_remains(key, &forgotten, &records);
            let refreshed_move = keys::decode(key).is_some_and(|key| {
                let key = root::Key::Core(key);
                refreshed_move(records.get(&key), recovered.get(&key))
            });
            // Dropped transcripts leave attempt places for bounded later commits (transcripts.md, 5).
            let orphan = match keys::decode_own(key) {
                Some(keys::Own::Attempt { task, .. }) => {
                    let owner =
                        keys::encode_own(&keys::Own::Transcript { task }, u32::MAX).expect("fixed transcript key");
                    !inventory.contains_key(&owner) && !restored.contains_key(key)
                }
                Some(keys::Own::LogAttempt { task, .. }) => {
                    let owner = keys::encode_own(&keys::Own::TurnLog { task }, u32::MAX).expect("fixed turn-log key");
                    !inventory.contains_key(&owner) && !restored.contains_key(key)
                }
                _ => false,
            };
            if !(orphan || forgotten_erasure || refreshed_move || exception.is_some() && (header || wires || expired))
                && inventory.get(key) != restored.get(key)
            {
                return Err(format!(
                    "restart oracle retained key {:?} ({:?}); rebuild {:?}: expected {:?}, found {:?}",
                    key,
                    keys::decode(key),
                    exception,
                    inventory.get(key),
                    restored.get(key)
                ));
            }
        }
        for (key, row) in &records {
            let forgotten_erasure = matches!(key,
                root::Key::Core(core::Key::Core(core::CoreKey::Turn { task, .. })) if forgotten.contains(task))
                && !recovered.contains_key(key);
            if matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Turn(_))))
                && Some(row) != recovered.get(key)
                && !forgotten_erasure
            {
                return Err(format!(
                    "restart oracle transcript key {key:?}: expected {row:?}, found {:?}",
                    recovered.get(key)
                ));
            }
        }
        self.closed_documents = Some(Documents { before, after });
        Ok(())
    }

    fn oracle_read_set(&self, records: &BTreeMap<root::Key, root::Record>) -> (BTreeSet<u64>, BTreeSet<u64>) {
        let mut parties: BTreeSet<_> = records
            .values()
            .filter_map(|row| {
                if let root::Record::Core(core::Record::People(people::Stored::SignIn { number, .. })) = row {
                    Some(*number)
                } else {
                    None
                }
            })
            .collect();
        let mut tasks: BTreeSet<_> = records
            .values()
            .filter_map(|row| {
                if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row))) =
                    row
                {
                    Some(row.number)
                } else {
                    None
                }
            })
            .collect();
        parties.extend(self.sign_in);
        parties.extend(self.people_answers.iter().filter_map(|reply| {
            if let people::Reply::SignedIn { sign_in, .. } = reply { Some(*sign_in) } else { None }
        }));
        if let Some(peers) = &self.peers {
            parties.extend(peers.parties.iter().filter_map(|party| party.sign_in));
        }
        tasks.extend(self.assignments.iter().map(|(task, _)| *task));
        for reply in &self.people_answers {
            if let people::Reply::Outcome(people::Outcome::Created { first, last, .. }) = reply {
                tasks.extend(*first..=*last);
            }
        }
        for (_, _, answer) in &self.answers {
            if let core::CallPart::Delegated(made) = answer {
                tasks.extend(made.iter().map(|made| made.task));
            }
        }
        if let Some(peers) = &self.peers {
            tasks.extend(peers.parties.iter().filter_map(|party| party.last_task));
        }
        (parties, tasks)
    }

    /// A restart runs the due Forgotten pass (jig/docs/design/protocol/store/retention.md, 6.4).
    fn forgotten_due(&self, records: &BTreeMap<root::Key, root::Record>) -> BTreeSet<u64> {
        let wall = self.clocks().1;
        records
            .values()
            .filter_map(|row| {
                if let root::Record::Core(core::Record::Tasks(tasks::Stored::Retention(row))) = row
                    && row.stage == tasks::RetentionStage::Forgotten
                    && row.since <= wall
                {
                    Some(row.number)
                } else {
                    None
                }
            })
            .collect()
    }

    fn expired(&self, records: &BTreeMap<root::Key, root::Record>, key: &core::Key) -> bool {
        let Some(root::Record::Core(core::Record::People(row))) = records.get(&root::Key::Core(key.clone())) else {
            return false;
        };
        let expires = if let people::Stored::SignIn { expires, .. } = row {
            *expires
        } else if let people::Stored::Answer { at, .. } | people::Stored::Acted { at, .. } = row {
            skein_lib::Wall::from_nanos(
                at.as_nanos().saturating_add(self.restart_config.1.core.people.request_retention.as_nanos()),
            )
        } else {
            return false;
        };
        expires <= self.clocks().1
    }

    fn settle_oracle(&mut self) -> Result<(), String> {
        for _ in 0..300 {
            self.try_drain()?;
            self.try_release_commits()?;
            if self.stopped {
                return Err(match self.domain.start_failure() {
                    Some((phase, failure)) => format!("restart oracle: start refused at {phase:?}: {failure:?}"),
                    None => "restart oracle: stopped fixture must declare broken()".into(),
                });
            }
            if self.domain.quiescent()
                && self.store.pending.is_empty()
                && self.store.held.is_empty()
                && !self.store.loading()
            {
                // retained() also checks the component's private in-flight and reply queues.
                return Ok(());
            }
        }
        Err("restart oracle: held delivery at store readback boundary".into())
    }

    fn oracle_documents(&mut self, parties: &BTreeSet<u64>, tasks: &BTreeSet<u64>) -> Answers {
        let reads = self.reads.take();
        let client = self.client_peer.take();
        let sign_in = self.sign_in;
        self.reads = Some(reads::Parties::new(self.store.applied));
        let mut found = BTreeMap::new();
        for party in parties {
            self.sign_in = Some(*party);
            for task in std::iter::once(None).chain(tasks.iter().copied().map(Some)) {
                let read = task.map_or(core::reads::Read::Frame, |task| core::reads::Read::Task { task });
                reads::ask(self, 0, read);
                found.insert((*party, task), reads::latest(self).clone());
            }
        }
        self.reads = reads;
        self.client_peer = client;
        self.sign_in = sign_in;
        found
    }
}

fn deployment(records: &BTreeMap<root::Key, root::Record>) -> Result<core::Deployment, String> {
    let Some(root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(row)))) =
        records.get(&root::Key::Core(core::Key::Core(core::CoreKey::Deployment)))
    else {
        return Err("restart oracle deployment header".into());
    };
    Ok(*row)
}

// accounts.md 6: restart cancels an in-flight move; the first refreshed grant clears its Moving hold.
fn refreshed_move(before: Option<&root::Record>, after: Option<&root::Record>) -> bool {
    let (
        Some(root::Record::Core(core::Record::Accounts(before))),
        Some(root::Record::Core(core::Record::Accounts(after))),
    ) = (before, after)
    else {
        return false;
    };
    let mut expected = before.clone();
    if let accounts::Stored::Source { kind: accounts::Kind::SignIn, held, .. } = &mut expected
        && *held == Some(accounts::Held::Moving)
    {
        *held = None;
        return &expected == after;
    }
    false
}

/// Only a due task's remains may disappear (protocol/store/retention.md, 6.4).
fn forgotten_remains(key: &[u8], forgotten: &BTreeSet<u64>, records: &BTreeMap<root::Key, root::Record>) -> bool {
    // Log extents carry a log number, not a domain key (protocol/store/logs.md, 1).
    if let Some(suffix) = key.strip_prefix(&jig_protocol_store::logs::EXTENTS) {
        return <[u8; 8]>::try_from(suffix).is_ok_and(|bytes| {
            let log = u64::from_be_bytes(bytes);
            forgotten.iter().any(|task| {
                jig_protocol_store::logs::history(*task) == Some(log)
                    || jig_protocol_store::logs::turns(*task) == Some(log)
            })
        });
    }
    let task = match keys::decode(key) {
        Some(
            core::Key::Tasks(
                tasks::Key::Summary(task)
                | tasks::Key::Retention { stage: tasks::RetentionStage::Forgotten, number: task, .. }
                | tasks::Key::History { task, .. }
                | tasks::Key::Activity { task, .. }
                | tasks::Key::Attempt { task, .. }
                | tasks::Key::Hold { task, .. }
                | tasks::Key::Delegate { task, .. }
                | tasks::Key::EndedDelegate { task, .. }
                | tasks::Key::EndedByParty { task, .. }
                | tasks::Key::EndedByProject { task, .. },
            )
            | core::Key::People(people::Key::EndedRead { task, .. } | people::Key::ReadConversation { task, .. })
            | core::Key::Core(
                core::CoreKey::Terminal { task, .. }
                | core::CoreKey::Turn { task, .. }
                | core::CoreKey::ProposalDecision { proposer: people::Proposer::Task(task), .. }
                | core::CoreKey::EscalationDecision { task, .. },
            ),
        ) => Some(task),
        Some(core::Key::Tasks(tasks::Key::Update { task, .. })) => Some(task.value()),
        Some(key @ core::Key::Tasks(tasks::Key::Result { .. })) => {
            // The result's key has no task number; its retained row identifies its owner.
            if let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Result(row)))) =
                records.get(&root::Key::Core(key))
            {
                Some(row.task)
            } else {
                None
            }
        }
        _ => match keys::decode_own(key) {
            Some(
                keys::Own::Transcript { task }
                | keys::Own::Attempt { task, .. }
                | keys::Own::History { task }
                | keys::Own::TurnLog { task }
                | keys::Own::LogAttempt { task, .. },
            ) => Some(task),
            _ => None,
        },
    };
    task.is_some_and(|task| forgotten.contains(&task))
}

/// Every exception names its rebuilding step; other keys are durable.
fn rebuilt(key: &[u8]) -> Option<&'static str> {
    if keys::decode_own(key) == Some(keys::Own::Wires) {
        return Some("domain/engine/restart.md, Switch commits this start's wires");
    }
    if matches!(
        keys::decode(key),
        Some(core::Key::People(people::Key::SignIn(_) | people::Key::Answer(_) | people::Key::Acted(_)))
    ) {
        return Some("domain/engine/restart.md step 1, the sweep erases expired people's rows");
    }
    if matches!(keys::decode(key), Some(core::Key::Core(core::CoreKey::Settings(_)))) {
        // close supplies the same defaults. Step 2 therefore makes no release
        // revision, and the comparison below still requires byte equality.
        return Some("domain/engine/restart.md step 2, a release's settings revision only when defaults change");
    }
    if keys::decode(key) == Some(core::Key::Core(core::CoreKey::Deployment)) {
        Some("domain/engine/restart.md, Switch advances the commit position")
    } else {
        None
    }
}

fn durable_answers(before: &mut core::reads::Answer, after: &mut core::reads::Answer) {
    use core::reads::Answer;
    if let (Answer::Document(left), Answer::Document(right)) = (&mut *before, &mut *after)
        && let (Document::Task(left), Document::Task(right)) = (left.as_mut(), right.as_mut())
    {
        let (left_wait, left_bound) = durable_waits(left);
        let (right_wait, right_bound) = durable_waits(right);
        // A discarded winner can expose a different wait on the other side. The
        // retained wait lines still compare; only the affected projection is masked.
        if left_wait || right_wait {
            left.wait = None;
            right.wait = None;
        }
        if left_bound || right_bound {
            left.nearest = None;
            right.nearest = None;
        }
    }
    for answer in [before, after] {
        if let Answer::Document(document) = answer {
            durable(document);
        }
    }
}

/// Fields rebuilt by the restart script; fields in doubt stay durable.
fn durable(document: &mut Document) {
    match document {
        // Engine 6 step 1 restores the accounts, not the last sign-in's runtime end.
        Document::Accounts(row) => {
            row.last_end = None;
            row.lendable = false;
            row.lending = None;
            row.lent_end = None;
        }
        Document::Terminal { login } => *login = None,
        // Engine 6 step 3 adopts runs afresh; the dead host's receipt is not retained.
        Document::Task(row) => {
            row.chip.details.observation = None;
            // client/domain/contract/documents.md 1: run_wait and an omitted choice derive from
            // current source and host availability, absent after a cold restart.
            // The retained inventory still compares every stored choice, including named ones.
            row.run_wait = None;
            row.llm = None;
        }
        // Engine 6 step 1 restores accounts without runtime grants; step 3 awaits
        // reconnecting workers' logins. Engine 11.1's offers and inferred previews
        // observe those facts; a template's named choice is a retained setting.
        Document::Frame(frame) => {
            for project in frame.projects.iter_mut() {
                project.offer.clear();
                let mut defaults = skein_lib::List::with_capacity(project.defaults.capacity());
                for (number, choice) in &project.defaults {
                    if project.templates.iter().any(|template| template.number == *number && template.llm.is_some()) {
                        defaults.push((*number, choice.clone())).expect("retained preview subset");
                    }
                }
                project.defaults = defaults;
            }
        }
        Document::Chips(_)
        | Document::Run(_)
        | Document::Tree(_)
        | Document::Lines(_)
        | Document::Proposal(_)
        | Document::Escalation(_)
        | Document::Question(_)
        | Document::PersonTask(_)
        | Document::Result(_)
        | Document::Project(_)
        | Document::Note(_)
        | Document::Inbox(_)
        | Document::Deployment(_)
        | Document::Settings(_) => {}
    }
}

/// Keep durable wait episodes and report whether first or nearest projected a
/// volatile one (jig/docs/design/domain/engine/waits.md, 4).
fn durable_waits(row: &mut core::documents::Task) -> (bool, bool) {
    use core::WaitKind;
    use core::documents::{ChipWork, PhaseLine, WaitsOn};
    let volatile: fn(WaitKind) -> bool = match row.chip.phase {
        PhaseLine::Active(tasks::Active::Due) if row.run_wait.is_some() => {
            |kind| matches!(kind, WaitKind::Account | WaitKind::Host)
        }
        PhaseLine::Active(tasks::Active::Claimed { .. } | tasks::Active::Running { .. })
            if row.chip.details.live_delegates == 0
                && row.chip.details.observation.as_ref().is_some_and(|value| value.work == ChipWork::Waiting) =>
        {
            |kind| kind == WaitKind::News
        }
        _ => return (false, false),
    };
    let mut durable = skein_lib::List::with_capacity(row.waits.capacity());
    for wait in &row.waits {
        if !volatile(wait.kind) {
            durable.push(wait.clone()).expect("durable waits are a subset of the document lines");
        }
    }
    row.waits = durable;
    if row.waits_on == WaitsOn::News && volatile(WaitKind::News) {
        row.waits_on = WaitsOn::Nothing;
    }
    // Each branch contributes one volatile episode, even if its line is cut.
    row.wait_count = row.wait_count.saturating_sub(1);
    (row.wait.as_ref().is_some_and(|wait| volatile(wait.kind)), row.nearest.is_some_and(|bound| volatile(bound.kind)))
}

fn procedure_wait(kind: core::WaitKind) -> bool {
    matches!(kind, core::WaitKind::Step | core::WaitKind::Room | core::WaitKind::Turn | core::WaitKind::SignOff)
}

// Reported waits and their news fallback are the last group. If a report is
// visible on either side, the entire durable prefix fits on that side; compare
// that prefix without requiring the same last group after a cold start.
fn without_report(row: &core::documents::Task) -> core::documents::Task {
    let mut row = row.clone();
    let prefix = row.waits.iter().take_while(|wait| !procedure_wait(wait.kind) && wait.kind != core::WaitKind::News);
    row.wait_count = u32::try_from(prefix.count()).expect("bounded wait lines");
    row.waits.truncate(row.wait_count);
    if row.waits_on == core::documents::WaitsOn::News {
        row.waits_on = core::documents::WaitsOn::Nothing;
    }
    row
}

#[expect(clippy::too_many_lines, reason = "field-level diagnostics enumerate both document contracts")]
fn compare_answer(
    key: &(u64, Option<u64>),
    expected: &core::reads::Answer,
    found: &core::reads::Answer,
    forgotten: &BTreeSet<u64>,
) -> Result<(), String> {
    if key.1.is_some_and(|task| forgotten.contains(&task))
        && *expected == core::reads::Answer::None(core::reads::Why::Gone)
        && *found == core::reads::Answer::None(core::reads::Why::Unknown)
    {
        return Ok(());
    }
    let document = key.1.map_or_else(|| "Frame".to_owned(), |task| format!("Task {task}"));
    macro_rules! fields {
        ($left:expr, $right:expr; $($field:ident),+ $(,)?) => {$(
            if $left.$field != $right.$field {
                return Err(format!(
                    "restart oracle document {} field {} (sign-in {}): expected {:?}, found {:?}",
                    document, stringify!($field), key.0, $left.$field, $right.$field
                ));
            }
        )+};
    }
    if let (core::reads::Answer::Document(left), core::reads::Answer::Document(right)) = (expected, found) {
        if let (Document::Task(left), Document::Task(right)) = (&**left, &**right) {
            let durable;
            let (left, right) = if left.waits.iter().chain(&right.waits).any(|wait| procedure_wait(wait.kind)) {
                durable = (without_report(left), without_report(right));
                (&durable.0, &durable.1)
            } else {
                (left.as_ref(), right.as_ref())
            };
            fields!(left, right;
                waits_on,
                wait,
                nearest,
                waits,
                wait_count,
                settings,
                label,
                flags,
                last_activity,
                chip,
                acts,
                project,
                goal,
                priority,
                executor,
                requester,
                lineage,
                delegates,
                changed,
                spec_revision,
                person_revision,
                taken_by,
                activity,
                spec,
                words,
                parameters,
                instructions,
                llm,
                kept_account,
                run_wait,
                resources,
                inputs,
                dependencies,
                dependency_facts,
                references,
                subscriptions,
                wake,
                contract,
                result,
                budget,
                authority,
                hold,
                proposals,
                cut,
            );
            if left == right {
                return Ok(());
            }
        }
        if let (Document::Frame(left), Document::Frame(right)) = (&**left, &**right) {
            fields!(left, right;
                party,
                kind,
                login,
                name,
                deployment_role,
                projects,
                needs,
                need_counts,
                updates,
                results_seen,
                retention,
                cut,
                providers,
                providers_cut,
                key_bytes,
                redirect_bytes,
                words_bytes,
            );
        }
    }
    if expected != found {
        return Err(format!(
            "restart oracle document {document} field answer (sign-in {}): expected {expected:?}, found {found:?}",
            key.0
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{World, accounts, core, refreshed_move, root};
    use crate::world::Setup;

    #[test]
    fn closing_restart_names_the_refused_phase_record_and_link() {
        let mut world = World::new(Setup { seed: 2901, ..Setup::default() });
        let mut row = world
            .store
            .rows
            .values()
            .find_map(|record| {
                if let root::Record::Core(core::Record::Tasks(super::tasks::Stored::Live(row))) = record {
                    Some(row.clone())
                } else {
                    None
                }
            })
            .expect("caller's live task");
        row.project = 99;
        // The oracle, not the transaction observer, diagnoses this broken writer link.
        world.observer.record();
        world.seed(Box::new([root::Record::Core(core::Record::Tasks(super::tasks::Stored::Live(row)))]));
        assert!(world.domain.ready());
        let message = world.finish().expect_err("the closing restart refuses the missing project");
        assert_eq!(
            message,
            "restart oracle: start refused at Some(Link): Link { record: Tasks(Live(1)), linked: Project(99) }"
        );
        assert!(!world.domain.ready());
    }

    fn source(kind: accounts::Kind, held: Option<accounts::Held>, place: u32) -> root::Record {
        root::Record::Core(core::Record::Accounts(accounts::Stored::Source {
            account: 3,
            number: 2,
            place,
            kind,
            name: Box::new([]),
            held,
        }))
    }

    #[test]
    fn only_a_signed_in_moves_cleared_hold_is_rebuilt() {
        let moving = source(accounts::Kind::SignIn, Some(accounts::Held::Moving), 0);
        let granted = source(accounts::Kind::SignIn, None, 0);
        assert!(refreshed_move(Some(&moving), Some(&granted)));
        for held in [None, Some(accounts::Held::Refused), Some(accounts::Held::Limited { reset: None })] {
            let before = source(accounts::Kind::SignIn, held, 0);
            assert!(!refreshed_move(Some(&before), Some(&granted)));
            let after = source(accounts::Kind::SignIn, held, 0);
            assert_eq!(refreshed_move(Some(&moving), Some(&after)), held.is_none());
        }
        let key = source(accounts::Kind::Key, Some(accounts::Held::Moving), 0);
        let key_granted = source(accounts::Kind::Key, None, 0);
        assert!(!refreshed_move(Some(&key), Some(&key_granted)));
    }

    #[test]
    fn a_cancelled_move_keeps_its_source_and_other_fields() {
        let moving = source(accounts::Kind::SignIn, Some(accounts::Held::Moving), 0);
        let reordered = source(accounts::Kind::SignIn, None, 1);
        assert!(!refreshed_move(Some(&moving), Some(&reordered)));
        assert!(!refreshed_move(Some(&moving), None));
        assert!(!refreshed_move(None, Some(&reordered)));
    }
}
