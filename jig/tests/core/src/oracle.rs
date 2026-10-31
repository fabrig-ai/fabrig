//! The cold restart oracle (`jig/docs/design/domain/engine.md`, sections 6 and 15).
use crate::{effects::World, reads};
use jig_domain_accounts as accounts;
use jig_domain_core::{self as core, documents::Document};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_protocol_store::keys;
use jig_test_domain as root;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn configuration(source: &root::Config) -> root::Config {
    let core = &source.core;
    let settings = &core.settings;
    let mut policies = skein_lib::Map::with_capacity(core.defaults.policies.capacity());
    for (key, value) in &core.defaults.policies {
        policies.insert(*key, value.clone()).expect("configuration policies fit");
    }
    root::Config {
        first: source.first.clone(),
        second: source.second.clone(),
        core: core::Config {
            worker_names: core.worker_names.clone(),
            deployment: core.deployment,
            seed: core.seed,
            workers_in_process: core.workers_in_process,
            hub_turns: core.hub_turns,
            owners: core.owners.clone(),
            grants: core.grants.clone(),
            defaults: core::settings::Defaults {
                charters: core.defaults.charters.clone(),
                templates: core.defaults.templates.clone(),
                policies,
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
    }
}

impl World {
    /// Transfer the started application to another scripted driver.
    pub(crate) fn into_domain_store(mut self) -> (root::Domain, crate::Store) {
        self.oracle_closed = true;
        let config = configuration(&self.restart_config.0);
        let limits = self.restart_config.1;
        let store = crate::Store::starting(&limits, &config.core.settings.models);
        let domain = root::Domain::new(config, &limits);
        (std::mem::replace(&mut self.domain, domain), std::mem::replace(&mut self.store, store))
    }

    /// Declare this world a deliberately broken fixture: `close` does nothing.
    pub fn broken(&mut self) {
        self.broken = true;
    }

    /// The restart oracle (engine.md 15): settle, snapshot what the parties were last
    /// answered and the store's retained rows, restart cold, read back, and compare
    /// through `durable`. Panics naming the first difference (document, field, key).
    pub fn close(&mut self) {
        if self.broken || self.oracle_closed {
            return;
        }
        self.oracle_closed = true;
        // Oracle transport does not advance the scenario's clocks or age its deadlines.
        let (now, wall) = self.clocks();
        self.set_clock(now, wall);
        self.settle_oracle();
        let inventory = self.store.retained();
        let records = self.store.readback();
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
        let before = self.oracle_documents(&parties, &tasks);
        self.restart_with(configuration(&self.restart_config.0));
        self.settle_oracle();
        assert!(self.domain.ready(), "restart oracle: decisions did not open");
        let after = self.oracle_documents(&parties, &tasks);
        for (key, expected) in &before {
            let found = after.get(key).expect("same oracle read set");
            compare_answer(key, expected, found);
        }
        let restored = self.store.retained();
        let recovered = self.store.readback();
        let mut expected_header = deployment(&records);
        let found_header = deployment(&recovered);
        expected_header.commits = found_header.commits;
        assert_eq!(expected_header, found_header, "restart oracle deployment field at key Deployment");
        for key in inventory.keys().chain(restored.keys()).collect::<BTreeSet<_>>() {
            let exception = rebuilt(key);
            let header = keys::decode(key) == Some(core::Key::Core(core::CoreKey::Deployment));
            let wires = keys::decode_own(key) == Some(keys::Own::Wires);
            let expired = exception.is_some()
                && !restored.contains_key(key)
                && keys::decode(key).is_some_and(|key| self.expired(&records, &key));
            let refreshed_move = keys::decode(key).is_some_and(|key| {
                let key = root::Key::Core(key);
                refreshed_move(records.get(&key), recovered.get(&key))
            });
            if !(refreshed_move || exception.is_some() && (header || wires || expired)) {
                assert_eq!(
                    inventory.get(key),
                    restored.get(key),
                    "restart oracle retained key {:?} ({:?}); rebuild {:?}",
                    key,
                    keys::decode(key),
                    exception
                );
            }
        }
        for (key, row) in &records {
            if matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Turn(_)))) {
                assert_eq!(Some(row), recovered.get(key), "restart oracle transcript key {key:?}");
            }
        }
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

    fn settle_oracle(&mut self) {
        for _ in 0..300 {
            self.drain();
            self.release_commits();
            assert!(!self.stopped, "restart oracle: stopped fixture must declare broken()");
            if self.domain.quiescent()
                && self.store.pending.is_empty()
                && self.store.held.is_empty()
                && !self.store.loading()
            {
                // retained() also checks the component's private in-flight and reply queues.
                return;
            }
        }
        panic!("restart oracle: held delivery at store readback boundary");
    }

    fn oracle_documents(
        &mut self,
        parties: &BTreeSet<u64>,
        tasks: &BTreeSet<u64>,
    ) -> BTreeMap<(u64, Option<u64>), core::reads::Answer> {
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
                let mut answer = reads::latest(self).clone();
                if let core::reads::Answer::Document(document) = &mut answer {
                    **document = durable(*document.clone());
                }
                found.insert((*party, task), answer);
            }
        }
        self.reads = reads;
        self.client_peer = client;
        self.sign_in = sign_in;
        found
    }
}

fn deployment(records: &BTreeMap<root::Key, root::Record>) -> core::Deployment {
    let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(row))) =
        &records[&root::Key::Core(core::Key::Core(core::CoreKey::Deployment))]
    else {
        panic!("restart oracle deployment header")
    };
    *row
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

/// Every exception names its rebuilding step; other keys are durable.
fn rebuilt(key: &[u8]) -> Option<&'static str> {
    if keys::decode_own(key) == Some(keys::Own::Wires) {
        return Some("engine.md 6, Switch commits this start's wires");
    }
    if matches!(
        keys::decode(key),
        Some(core::Key::People(people::Key::SignIn(_) | people::Key::Answer(_) | people::Key::Acted(_)))
    ) {
        return Some("engine.md 6 step 1, the sweep erases expired people's rows");
    }
    if matches!(keys::decode(key), Some(core::Key::Core(core::CoreKey::Settings(_)))) {
        // close supplies the same defaults. Step 2 therefore makes no release
        // revision, and the comparison below still requires byte equality.
        return Some("engine.md 6 step 2, a release's settings revision only when defaults change");
    }
    if keys::decode(key) == Some(core::Key::Core(core::CoreKey::Deployment)) {
        Some("engine.md 6, Switch advances the commit position")
    } else {
        None
    }
}

/// Fields rebuilt by the restart script; fields in doubt stay durable.
fn durable(mut document: Document) -> Document {
    match &mut document {
        // Engine 6 step 1 restores the accounts, not the last sign-in's runtime end.
        Document::Accounts(row) => row.last_end = None,
        // Engine 6 step 3 adopts runs afresh; the dead host's receipt is not retained.
        Document::Task(row) => row.chip.details.observation = None,
        Document::Frame(_)
        | Document::Chips(_)
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
    document
}

fn compare_answer(key: &(u64, Option<u64>), expected: &core::reads::Answer, found: &core::reads::Answer) {
    let document = key.1.map_or_else(|| "Frame".to_owned(), |task| format!("Task {task}"));
    macro_rules! fields {
        ($left:expr, $right:expr; $($field:ident),+ $(,)?) => {$(
            assert_eq!($left.$field, $right.$field,
                "restart oracle document {} field {} (sign-in {})", document, stringify!($field), key.0);
        )+};
    }
    if let (core::reads::Answer::Document(left), core::reads::Answer::Document(right)) = (expected, found) {
        if let (Document::Task(left), Document::Task(right)) = (&**left, &**right) {
            fields!(left, right;
                settings,
                waits_on,
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
            );
        }
    }
    assert_eq!(expected, found, "restart oracle document {document} field answer (sign-in {})", key.0);
}

#[cfg(test)]
mod tests {
    use super::{accounts, core, refreshed_move, root};

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
