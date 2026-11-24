//! A recorded walk on stock chat, agreement and landing policy (temper/docs/design/testing/tiers.md, 1.1).
use super::{Key, Script, Token, World, agreement, engine, provider::Playback, signed_in_forge_party};
use jig_domain_core::{CoreRecord, Record, documents, reads};
use skein_fake_forge_domain::{Observation, api};
use smith_domain_session::record;
use temper_engine_domain_forge::{Adoption, GitIdentity};
use temper_engine_service_vocabulary::settings;
use temper_fake_checkout::git::Tree;
use temper_worker_checkout_world::forge::Forge;

pub(super) const TREE: Token = Token::new(952);
const RESULT_BYTES: u32 = 512;

/// One seeded forge and the identities and URL by which the roots see it.
#[derive(Debug)]
pub struct WalkRepository {
    pub seed: api::Setup,
    pub adoption: Adoption,
    pub binding: settings::Forge,
    /// The project owner's user number on the fake forge, not the API writer.
    pub person: u64,
}

/// One recorded walk's adoption and the in-flight tree read.
pub(super) struct Party {
    pub(super) adoption: Adoption,
    pub(super) repository: u32,
    pub(super) tree: Option<Box<documents::Document>>,
    pub(super) tree_closed: bool,
}

impl World {
    /// Replay a checked document across the stock chat, agreed goal and reviewed change.
    #[must_use]
    pub fn recorded_walk(
        seed: u64,
        playback: Playback,
        repository: WalkRepository,
        rule: engine::LandingRule,
        request: &[u8],
    ) -> Self {
        let WalkRepository { seed: setup, adoption, binding, person } = repository;
        assert_eq!(adoption.forge, binding.number);
        assert_eq!(adoption.landing, setup.default);
        let party = temper_engine_domain_forge::user_key(binding.number, person);
        let mut config = agreement::config_with_result_bytes(seed, RESULT_BYTES);
        config.core.settings.resume_bytes = 131_072;
        config.kinds_stock = engine::landing_stock(&[rule], &skein_lib::Map::with_capacity(0), 2, 4);
        config.core.owners[0].identity = party.clone();
        config.forge.writers[0] = skein_forge_client::Writer { forge: binding.number, author: binding.user };
        config.identities[0] = GitIdentity { forge: binding.number, credential: binding.identity };
        config.forges[0].number = binding.number;
        config.forges[0].git.clone_from(&binding.git);
        config.forges[0].web.clone_from(&binding.web);
        let mut forge = Forge::new(seed);
        forge.worker_user(binding.user);
        let repository = setup.id;
        let name = setup.name.clone();
        forge.seed_repository(setup);
        forge.grant(&name, binding.user, api::Permission::Admin);
        forge.grant(&name, person, api::Permission::Read);
        let empty = Script { cue: Box::default(), turns: Box::default() };
        let mut world = Self::configured_case(seed, empty, true, config, limits, (forge, binding));
        world.party = signed_in_forge_party(request, party);
        world.repository_name = name;
        world.walk = Some(Party { adoption, repository, tree: None, tree_closed: false });
        world.playback = Some(playback);
        world
    }

    /// The owner's outstanding needs, read through the authenticated root.
    pub fn inbox_needs(&mut self) -> Box<reads::Page> {
        let answer = self.notes_read(reads::Read::Inbox {
            group: reads::Group::Needs,
            filter: reads::Filter { project: None, goal: None },
            from: None,
        });
        let reads::Answer::Page(page) = answer else { panic!("inbox needs: {answer:?}") };
        page
    }

    /// The whole original question on the named chat, through the authenticated root.
    pub fn task_question(&mut self, task: u64, message: u64) -> documents::Question {
        let answer = self.notes_read(reads::Read::Question { task, message });
        let reads::Answer::Document(document) = answer else { panic!("question read: {answer:?}") };
        let documents::Document::Question(question) = *document else { panic!("question read: {document:?}") };
        question
    }

    /// The overview visible to the person while the task is live.
    pub fn task_overview(&mut self, task: u64) -> Box<documents::Task> {
        let answer = self.notes_read(reads::Read::Task { task });
        let reads::Answer::Document(document) = answer else { panic!("task read: {answer:?}") };
        let documents::Document::Task(task) = *document else { panic!("task read: {document:?}") };
        task
    }

    /// The live subtree as seen by the owner's standard tree watch.
    pub fn task_tree(&mut self, task: u64) -> documents::Tree {
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Watch {
            watcher: TREE,
            sign_in: self.party.sign_in.expect("authenticated person"),
            subject: documents::Watched::Tree { task },
            window: documents::Window { rows: 32, after: None },
        }));
        for _ in 0..super::STEPS {
            self.tick();
            let walk = self.walk.as_mut().expect("recorded walk");
            if walk.tree_closed {
                walk.tree_closed = false;
                let document = walk.tree.take().expect("tree document");
                let documents::Document::Tree(tree) = *document else { panic!("task tree: {document:?}") };
                return tree;
            }
        }
        panic!("tree never arrived for task {task}");
    }

    /// The owner's typed activity page, not the live process's provisional turns.
    pub fn task_activity(&mut self, task: u64) -> Box<reads::Page> {
        let answer = self.notes_read(reads::Read::Activity { task, before: None });
        let reads::Answer::Page(page) = answer else { panic!("task activity: {answer:?}") };
        page
    }

    /// Decode committed Smith turns in their durable order.
    #[must_use]
    pub fn committed_turns(&self, task: u64) -> Vec<record::Turn> {
        let endpoints = super::worker_protocol::endpoints_with_dialect(3);
        self.store
            .rows
            .values()
            .filter_map(|row| {
                let engine::Record::Core(Record::Core(CoreRecord::Turn(turn))) = row else { return None };
                (turn.task == task).then(|| {
                    let limits = &smith_transcript::CEILINGS;
                    let saved = smith_transcript::decode_archived(limits, &turn.transcript).expect("committed frame");
                    // The store keys each activation locally; the frame keeps the conversation ordinal.
                    smith_protocol_channel::decode_turn(&turn.transcript, saved.place, limits, &endpoints)
                        .expect("committed Smith turn")
                })
            })
            .collect()
    }

    /// The typed terminal task record retained by the store.
    #[must_use]
    pub fn ended_task(&self, task: u64) -> Option<&jig_domain_tasks::TaskRecord> {
        let key = Key::Core(jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Ended(task)));
        let engine::Record::Core(Record::Tasks(jig_domain_tasks::Stored::Ended(row))) = self.store.rows.get(&key)?
        else {
            panic!("ended task key holds its terminal row")
        };
        Some(row)
    }

    /// Ordered writes from the shared git and forge peer, including CI status and merges.
    #[must_use]
    pub fn forge_observations(&self) -> &[Observation] {
        self.forge.observations()
    }

    /// Current head and whole tree, read without issuing a write or git call.
    #[must_use]
    pub fn branch(&self, branch: &[u8]) -> (u64, Tree) {
        let head = self.forge.branch(&self.repository_name, branch).expect("branch exists");
        let tree = self.forge.branch_tree(&self.repository_name, branch).expect("branch's tree exists");
        (head, tree)
    }

    /// The repository's branch names, in the peer's order.
    #[must_use]
    pub fn branches(&self) -> Vec<Vec<u8>> {
        self.forge.branches(&self.repository_name)
    }
}

fn limits(limits: &mut engine::Limits) {
    // Chat and goal pages may load together while the completed page waits to be reclaimed.
    limits.loads.loads = 3;
    limits.tasks.result_bytes = RESULT_BYTES;
    // Each short conversation fits one store page, even when two tasks prepare together.
    limits.loads.rows = 32;
}
