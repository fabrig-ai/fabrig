use crate::translate;
use skein_fake_forge_domain::{self as forge, api as raw};
use skein_lib::{Duration, Env, List, Map, Queue, ReplyTo, Time, Token, Wall};
use temper_engine_domain_forge::{self as top, history};

pub const REPO: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 2 };
const CLIENT: skein_forge_client::Limits = skein_forge_client::Limits {
    pending: 4,
    calls: 2,
    rate: 100,
    reserve: 0,
    window: Duration::from_secs(10),
    op_bytes: 512,
    answer_bytes: 4096,
    rows: 8,
    inbox: 16,
    read_attempts: 3,
    backoff: Duration::from_secs(1),
    backoff_max: Duration::from_secs(4),
    entries: 8,
    write_attempts: 3,
    lifetime: Duration::from_secs(10),
    clock_margin: Duration::from_secs(1),
    resources: 8,
    repositories: 2,
    poll: Duration::from_secs(5),
    poll_max: Duration::from_secs(30),
    hinted: Duration::from_secs(1),
    slow: Duration::from_secs(40),
    settle: Duration::from_secs(30),
    fallback: Duration::ZERO,
    facts: 16,
};
pub const LIMITS: top::Limits = top::Limits {
    forges: 4,
    repositories: 2,
    tasks: 8,
    holds: 8,
    subscriptions: 8,
    entries: 8,
    named_calls: 16,
    changes: 8,
    issues: 8,
    resources_per_task: 4,
    paths_per_subscription: 4,
    authority_segments: 8,
    name_bytes: 128,
    output: 80,
    facts: 16,
    adoptions: 2,
    makings: 2,
    applications: 4,
    bindings: 4,
    collaborators: 8,
    landings: 8,
    landing_members: 3,
    landings_kept: 16,
    changes_rows: 32,
    changes_landed: 8,
    gates: 2,
    receipts_per_gate: 4,
    ignored_paths: 4,
    key_entries: 16,
    key_path_bytes: 1024,
    trees_in_flight: 2,
    tiers: temper_engine_domain_forge::TierLimits { tiers: 2, failing_tests: 16, test_id_bytes: 256 },
    judge_projects: 2,
    decided_per_project: 64,
    findings_per_project: 256,
    judge_criteria: 8,
    landing_rules: 4,
    follows: 2,
    dependencies: 4,
    follow_paths: 8,
    follow_queue: 16,
    follow_merge: 8,
    follow_commits: 8,
    follow_files: 32,
    follow_message_bytes: 512,
    follow_skips: 8,
    follow_reason_bytes: 256,
    roles: 4,
    setting_proposals: 4,
    setting_history: 4,
    brief_sections: 32,
    brief_bytes: 4096,
    client: CLIENT,
    issue_policy: temper_engine_domain_forge_issues::Limits {
        plan_items: 8,
        milestones: 8,
        title_bytes: 128,
        body_bytes: 512,
        comment_bytes: 256,
        interval: Duration::from_secs(5),
    },
    change_policy: temper_engine_domain_forge_change::Limits {
        reruns: 1,
        repairs: 3,
        resolutions: 3,
        updates: 3,
        questions: 4,
        tasks: 6,
        stall: Duration::from_secs(30),
        gates: 8,
        clean_heads: 8,
    },
    queue_window: Duration::from_secs(30),
    sitting: temper_engine_domain_forge::SITTING,
};

#[derive(Debug)]
struct Pending {
    repository: Box<[u8]>,
    client: Token,
    op: skein_forge::api::Op,
    generation: u64,
}

mod follows;

struct ReadScript {
    ancestry: Option<Result<skein_forge::api::Answer, skein_forge::api::Error>>,
    comparisons_too_large: bool,
    files_excess: Option<skein_forge::api::Excess>,
    files_page: Option<u32>,
}

/// Scripted root and forge protocol surrounding the connector top.
pub struct World {
    pub(crate) top: top::Domain,
    env: Env<top::Limits>,
    forge: forge::Domain,
    forge_env: Env<forge::Config>,
    to_forge: Queue<forge::Event>,
    forge_out: Queue<forge::Request>,
    to_top: Queue<(u64, top::Event)>,
    top_out: Queue<top::Request>,
    pending: Map<Token, Pending>,
    stored: Map<top::Key, top::Stored>,
    seen: List<top::Request>,
    serial: u64,
    generation: u64,
    seed: u64,
    writer: Option<skein_forge_client::Writer>,
    writer_saves: u32,
    writes: u32,
    lose_writes: bool,
    lose_rerun_answer: bool,
    unavailable: Option<(Wall, Wall)>,
    calls: u32,
    call_ops: Vec<skein_forge::api::Op>,
    hints: Vec<skein_forge::api::Hint>,
    signatures: bool,
    pull_snapshot: Option<skein_forge::api::Pull>,
    held_compares: Option<Vec<(u64, top::Event)>>,
    read_script: ReadScript,
    landing_policy: temper_engine_domain_forge_landing::Project,
    landing_batch: u16,
    landing_config: Option<(u32, temper_engine_domain_forge_change::Way, Box<[top::RepositoryGates]>)>,
}

impl World {
    #[must_use]
    pub fn tiered(seed: u64, workflow: raw::Workflow) -> World {
        Self::configured(
            seed,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            6,
            false,
            Some(workflow),
        )
    }
    #[must_use]
    pub fn turn() -> World {
        Self::configured(
            11,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            8,
            true,
            None,
        )
    }
    #[must_use]
    pub fn trains() -> World {
        Self::configured(
            11,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            8,
            true,
            None,
        )
    }
    #[must_use]
    pub fn batch_turn(seed: u64, workflow: Option<raw::Workflow>) -> World {
        let mut config = fake_config();
        config.limits.files = 8;
        Self::configured_repositories(
            seed,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            12,
            true,
            workflow,
            config,
            None,
            4,
        )
    }
    pub fn project_landing(
        &mut self,
        project: u32,
        way: temper_engine_domain_forge_change::Way,
        gates: Box<[top::RepositoryGates]>,
    ) -> bool {
        let accepted = self.top.project_landing(
            project,
            way,
            gates.clone(),
            self.landing_policy,
            top::AfterLanding {
                verdict: temper_engine_domain_forge_tiers::Settings {
                    retries: 2,
                    suspects: temper_engine_domain_forge_tiers::Suspects::Bisect,
                },
                bound: Duration::from_millis(7_200_000),
            },
        );
        if accepted {
            self.landing_config = Some((project, way, gates));
            top::resume(&mut self.top, &self.env, &mut self.top_out);
            self.take_top();
        }
        accepted
    }
    pub fn batch(&mut self, batch: u16) {
        let project = self.landing_config.as_ref().expect("landing configured").0;
        assert!(self.top.project_batch(project, batch));
        self.landing_batch = batch;
    }
    pub fn gate_policy(&mut self, settings: temper_engine_domain_forge_landing::Project) {
        self.landing_policy = settings;
        let (project, way, gates) = self.landing_config.clone().expect("landing configured");
        assert!(self.project_landing(project, way, gates));
    }
    pub fn runner_setup_faults(&mut self, faults: u32) {
        self.forge_env.limits.setup_faults = faults;
    }
    #[must_use]
    pub fn hold(&self, name: &top::Name) -> Option<&top::Hold> {
        self.top.hold(name)
    }
    #[must_use]
    pub fn unit_words(&self, changes: &[u64]) -> Box<[u8]> {
        self.top.unit_words(changes)
    }
    #[must_use]
    pub fn first_waiting_change(&self) -> Option<u64> {
        self.top.first_waiting(REPO, b"main", &[], self.env.wall)
    }
    #[must_use]
    pub fn judge_landing_evidence(&self, task: u64, evidence: &top::ChangeEvidence) -> Option<top::JudgeVerdict> {
        self.top.judge_landing(self.landing(task).landing.project, 5, 0, task, evidence.head?, evidence, &[])
    }
    #[must_use]
    pub fn has_receipt_at(&self, candidate: skein_forge::api::Commit) -> bool {
        self.stored
            .iter()
            .any(|(_, row)| matches!(row, top::Stored::Receipt { receipt, .. } if receipt.candidate == candidate))
    }
    #[must_use]
    pub fn landing_news_words(&self, task: u64, member: u16, step: top::LandingNews) -> Option<(u64, Box<[u8]>)> {
        self.top.landing_news_words(task, member, step)
    }
    #[must_use]
    pub fn landing_member_leave_words(&self, task: u64, member: u16) -> Option<Box<[u8]>> {
        self.top.landing_leave_words(task, member)
    }
    #[must_use]
    pub fn landing_leave_words(&self, task: u64) -> Option<Box<[u8]>> {
        self.top.landing_leave_words(task, 0)
    }
    #[must_use]
    pub fn change(&self, task: u64) -> &top::ChangeRow {
        self.top.change(task).expect("fixture change")
    }
    /// The whole document requested by the root for this watch subject.
    #[must_use]
    pub fn document(&self, project: u32, subject: u64) -> Option<top::Document> {
        self.top.document(
            subject,
            jig_domain_people::ApplicationScope::Project(project),
            0,
            self.env.wall,
            &self.env.limits,
        )
    }
    #[must_use]
    pub fn landing_hold_words(&self, task: u64) -> Option<Box<[u8]>> {
        self.top.landing_hold_words(task)
    }
    #[must_use]
    pub fn landing(&self, task: u64) -> &top::LandingRow {
        self.top.landing(task).expect("fixture landing")
    }
    #[must_use]
    pub fn inspect(&self, read: &raw::Read) -> raw::Answer {
        self.forge.inspect(&self.forge_env.limits, b"org/repo", read).expect("fixture read")
    }
    #[must_use]
    pub fn new(seed: u64) -> World {
        Self::with_facts(seed, LIMITS.facts)
    }

    #[must_use]
    pub fn with_facts(seed: u64, facts: u32) -> World {
        Self::configured(
            seed,
            facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            4,
            false,
            None,
        )
    }

    #[must_use]
    pub fn workspace_item(&self, seed: top::items::Seed) -> Option<top::items::Item> {
        self.top.workspace_item(seed)
    }
    pub fn pull_brief(&mut self, task: u64, head: skein_forge::api::Commit) -> Box<[u8]> {
        let row = self.top.change(task).expect("change");
        let section = Token::new(901);
        let (number, _, _, _) = row.review_target().expect("review target");
        self.event(top::Event::PlanBrief {
            section,
            partner: None,
            source: top::BriefSource::Pull {
                item: top::BriefItem { repository: row.repository.repository, number },
                head: top::BriefCommit(head),
            },
        });
        self.event(top::Event::GatherPlanned {
            task,
            section,
            parts: 2,
            bytes: 2048,
            ci_budget: 2048,
            deadline: self.env.now.saturating_add(Duration::from_secs(30)),
        });
        self.run_for(1);
        self.top.take_brief(section).expect("gathered pull brief")
    }
    /// Gather the repair's pinned CI section through the forge client.
    pub fn ci_brief(&mut self, task: u64, head: skein_forge::api::Commit) -> Box<[u8]> {
        let row = self.top.change(task).expect("change");
        let section = Token::new(900);
        self.event(top::Event::PlanBrief {
            section,
            partner: None,
            source: top::BriefSource::Ci {
                item: top::BriefItem { repository: row.repository.repository, number: row.pull.expect("pull") },
                head: top::BriefCommit(head),
            },
        });
        self.event(top::Event::GatherPlanned {
            task,
            section,
            parts: 2,
            bytes: 2048,
            ci_budget: 2048,
            deadline: self.env.now.saturating_add(Duration::from_secs(30)),
        });
        self.run_for(1);
        self.top.take_brief(section).expect("gathered CI brief")
    }

    /// Script the runner's failure independently from its check's outcome.
    #[must_use]
    pub fn runner_failure(seed: u64, passes: u32, fault: raw::Failing) -> World {
        let mut world = Self::configured_repositories(
            seed,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            4,
            false,
            None,
            fake_config(),
            Some(passes),
            LIMITS.landing_members,
        );
        forge::set_failing(&mut world.forge, b"org/repo", b"build", fault);
        world
    }

    /// The forge makes the next re-run but its transport loses the answer.
    pub fn lose_rerun_answer(&mut self) {
        self.lose_rerun_answer = true;
    }

    /// Branch rules from the independent fake, with signatures supplied by the layer.
    #[must_use]
    pub fn with_protection(seed: u64, protection: raw::Protection, signatures: bool) -> World {
        Self::configured(
            seed,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            Some(protection),
            signatures,
            4,
            false,
            None,
        )
    }

    #[must_use]
    pub fn with_branches(seed: u64, branches: u32) -> World {
        Self::configured(
            seed,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            branches,
            false,
            None,
        )
    }

    /// Keep client admission at eight rows while the fake can answer larger comparisons.
    #[must_use]
    pub fn with_comparison_room(seed: u64) -> World {
        let mut config = fake_config();
        config.limits.files = 20;
        config.limits.commits = 64;
        Self::configured_repositories(
            seed,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            6,
            false,
            None,
            config,
            None,
            LIMITS.landing_members,
        )
    }

    /// Give the fake room for a goal while independently bounding client admission.
    #[must_use]
    pub fn with_goal_room(seed: u64, rows: u32, answer_bytes: u32) -> World {
        let mut world = Self::with_comparison_room(seed);
        world.forge_env.limits.limits.files = 40;
        world.forge_env.limits.limits.content_bytes = 4096;
        world.env.limits.client.rows = rows;
        world.env.limits.client.answer_bytes = answer_bytes;
        world
    }

    /// Read the full goal comparison and check the client's row and byte admission.
    #[must_use]
    pub fn comparison_fits(&self, base: u64, head: u64) -> bool {
        let read = skein_forge::api::Read::Compare { before: translate::commit(base), after: translate::commit(head) };
        let answer = self.inspect(&raw::Read::Compare { base, head, page: 1, limit: 40 });
        let answer = translate::answer_as(&skein_forge::api::Op::Read(read), answer, &self.env.limits.client, 1, false)
            .expect("comparison projection");
        skein_forge_client::measure::answer_rows(&answer, self.env.limits.client.rows)
            && skein_forge::measure::answer(&answer).expect("bounded comparison")
                <= u64::from(self.env.limits.client.answer_bytes)
    }

    /// Script each ancestry range's terminal result until the script is cleared.
    pub fn script_ancestry(&mut self, result: Option<Result<skein_forge::api::Answer, skein_forge::api::Error>>) {
        self.read_script.ancestry = result;
    }

    /// Script a dialect's measured refusal of pull-file sides.
    pub fn reject_file_sides(&mut self, excess: skein_forge::api::Excess) {
        self.read_script.files_excess = Some(excess);
        self.read_script.files_page = None;
    }

    /// Refuse only the seeded page, allowing earlier pages to reach the fake.
    pub fn reject_file_page(&mut self, page: u32, excess: skein_forge::api::Excess) {
        self.read_script.files_excess = Some(excess);
        self.read_script.files_page = Some(page);
    }

    /// Refuse full comparisons at the protocol body bound; ancestry reads still reach the fake.
    pub fn reject_comparisons(&mut self) {
        self.read_script.comparisons_too_large = true;
    }

    pub fn preload_branches(&mut self, count: u32) {
        for number in 0..count {
            self.external(
                raw::Op::Git(raw::Git::Create {
                    branch: format!("foreign/{number}").into_bytes().into_boxed_slice(),
                    commit: 1,
                }),
                2,
            );
        }
    }

    pub fn signatures(&mut self, required: bool) {
        self.signatures = required;
    }

    #[expect(clippy::too_many_arguments, reason = "one shared fake fixture with independent scenario switches")]
    fn configured(
        seed: u64,
        facts: u32,
        hooked: bool,
        writer: Option<skein_forge_client::Writer>,
        protection: Option<raw::Protection>,
        signatures: bool,
        branches: u32,
        turn: bool,
        tier: Option<raw::Workflow>,
    ) -> World {
        Self::configured_repositories(
            seed,
            facts,
            hooked,
            writer,
            protection,
            signatures,
            branches,
            turn,
            tier,
            fake_config(),
            None,
            LIMITS.landing_members,
        )
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the public world adds repository room to the shared scenario switches"
    )]
    #[expect(clippy::too_many_lines, reason = "one shared fixture constructs its independent bounded neighbours")]
    fn configured_repositories(
        seed: u64,
        facts: u32,
        hooked: bool,
        writer: Option<skein_forge_client::Writer>,
        protection: Option<raw::Protection>,
        signatures: bool,
        branches: u32,
        turn: bool,
        tier: Option<raw::Workflow>,
        config: forge::Config,
        passes: Option<u32>,
        members: u32,
    ) -> World {
        let tiered = tier.is_some();
        let limits = top::Limits {
            landings: if turn { 2 } else { LIMITS.landings },
            landing_members: members,
            facts,
            client: skein_forge_client::Limits {
                facts,
                op_bytes: if turn { 2048 } else { CLIENT.op_bytes },
                rate: if members > LIMITS.landing_members { 1000 } else { CLIENT.rate },
                ..CLIENT
            },
            ..LIMITS
        };
        let config = scenario_config(config, branches, turn, tiered, protection.as_ref());
        let mut fake = forge::Domain::new(&config, seed);
        forge::repository(
            &mut fake,
            &config,
            raw::Setup {
                workflows: workflows(turn, tier),
                id: 2,
                name: Box::from(&b"org/repo"[..]),
                default: Box::from(&b"main"[..]),
                tree: Box::new([]),
                labels: Box::new([]),
                checks: raw::Checks {
                    cues: Box::new([]),
                    contexts: if turn { Box::new([]) } else { Box::new([Box::from(&b"build"[..])]) },
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: passes.unwrap_or(1000),
                    reruns: 0,
                    cue: None,
                },
                protection,
                hooked,
            },
        );
        if turn {
            forge::manual_merges(
                &mut fake,
                b"org/repo",
                raw::ManualMerges { allow_manual_merge: true, autodetect_manual_merge: true },
            );
        }
        forge::grant(&mut fake, b"org/repo", 1, raw::Permission::Admin);
        forge::grant(&mut fake, b"org/repo", 2, raw::Permission::Write);
        let top = top::Domain::new(
            &limits,
            seed,
            temper_engine_domain_forge::Config {
                push: None,
                namespace: Box::from(&b"world"[..]),
                writers: writer.into_iter().collect(),
            },
            &[top::GitIdentity { forge: REPO.forge, credential: 1 }],
            &[configured_forge()],
            false,
        )
        .expect("valid connector configuration");
        let now = Time::from_nanos(100_000_000_000);
        let mut world = World {
            top,
            env: Env { now, wall: Wall::from_nanos(now.as_nanos()), limits },
            forge: fake,
            forge_env: Env { now, wall: Wall::EPOCH, limits: config },
            to_forge: Queue::with_capacity(32),
            forge_out: Queue::uncapped(),
            to_top: Queue::with_capacity(32),
            top_out: Queue::uncapped(),
            pending: Map::with_capacity(32),
            stored: Map::with_capacity(128),
            seen: List::with_capacity(256),
            serial: 0,
            generation: 1,
            seed,
            writer,
            writer_saves: 0,
            writes: 0,
            lose_writes: false,
            lose_rerun_answer: false,
            unavailable: None,
            calls: 0,
            call_ops: Vec::new(),
            hints: Vec::new(),
            signatures,
            pull_snapshot: None,
            held_compares: None,
            read_script: ReadScript {
                ancestry: None,
                comparisons_too_large: false,
                files_excess: None,
                files_page: None,
            },
            landing_policy: temper_engine_domain_forge_landing::Project::DEFAULT,
            landing_batch: 1,
            landing_config: None,
        };
        world.top.restore_begin(world.env.wall);
        world.event(top::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        world.connected();
        world
    }

    /// The fake routing reports access again after each start
    /// (temper/docs/design/domain/forge/adoption.md, 1).
    fn connected(&mut self) {
        if let Some(writer) = self.writer {
            self.event(top::Event::Connection {
                forge: writer.forge,
                state: top::Connection::Connected { id: writer.author, login: b"bot".as_slice().into() },
            });
        }
    }

    /// Construct and restart without any configured writer.
    #[must_use]
    pub fn writerless(seed: u64) -> World {
        Self::configured(seed, LIMITS.facts, false, None, None, false, 4, false, None)
    }
    /// Restore a durable tier snapshot through the connector's restart.
    pub fn restore_tier(&mut self, row: top::TierRow) {
        self.stored
            .insert(top::Key::Tier { repository: row.repository, tier: row.tier.clone() }, top::Stored::Tier(row))
            .expect("fixture tier room");
        self.restart();
    }

    /// Bound tier news and client answers independently.
    pub fn tier_log_room(&mut self, brief_bytes: u32, answer_bytes: u32) {
        self.env.limits.brief_bytes = brief_bytes;
        self.env.limits.client.answer_bytes = answer_bytes;
    }

    pub fn tier_failures(&mut self, failing_tests: u32) {
        self.env.limits.tiers.failing_tests = failing_tests;
    }
    pub fn runner_lost(&mut self, lost: u32) {
        self.forge_env.limits.lost = lost;
    }
    #[must_use]
    pub fn has_branch(&self, branch: &[u8]) -> bool {
        self.forge.branch(b"org/repo", branch).is_some()
    }
    pub fn give_writer(&mut self, writer: skein_forge_client::Writer) -> bool {
        let accepted = top::writer(&mut self.top, &self.env, writer, &mut self.top_out);
        self.take_top();
        accepted
    }
    #[must_use]
    pub fn writer_saves(&self) -> u32 {
        self.writer_saves
    }

    /// As `new`, hearing the repository's hooks.
    #[must_use]
    pub fn hooked(seed: u64) -> World {
        Self::configured(
            seed,
            LIMITS.facts,
            true,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            4,
            false,
            None,
        )
    }

    #[must_use]
    pub fn hints(&self) -> &[skein_forge::api::Hint] {
        &self.hints
    }

    #[must_use]
    pub fn wall(&self) -> Wall {
        self.env.wall
    }

    pub fn run_to(&mut self, until: Wall) {
        let end = self
            .env
            .now
            .saturating_add(Duration::from_nanos(until.as_nanos().saturating_sub(self.env.wall.as_nanos())));
        for _ in 0..20_000 {
            if !self.iterate(end) {
                return;
            }
        }
        panic!("world did not reach horizon");
    }

    pub fn issue(&mut self, body: &[u8]) -> u64 {
        let raw::Answer::Created(number) = self.external(
            raw::Op::Write(raw::Write::CreateIssue {
                title: Box::from(&b"An issue"[..]),
                body: Box::from(body),
                labels: Box::new([]),
            }),
            2,
        ) else {
            panic!("issue creation")
        };
        number
    }

    /// Read exactly the public tasks the scripted core hands to the forge.
    #[must_use]
    pub fn public(
        &self,
        project: u32,
        kind: u16,
        task: Option<u64>,
        public: &[top::PublicTask],
    ) -> Option<top::PublicMade> {
        self.top.public(
            &top::PublicAsk {
                project,
                kind,
                version: temper_client_protocol_wire::PUBLIC_VERSION,
                task,
                public,
                now: self.env.wall,
            },
            &self.env.limits,
        )
    }
    /// A turn world whose independent runner follows the supplied workflow.
    #[must_use]
    pub fn public_turn(seed: u64, workflow: raw::Workflow) -> World {
        let mut world = Self::configured_repositories(
            seed,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            4,
            true,
            Some(workflow.clone()),
            repository_config(2),
            None,
            LIMITS.landing_members,
        );
        forge::repository(
            &mut world.forge,
            &world.forge_env.limits,
            raw::Setup {
                id: 3,
                name: Box::from(b"org/second".as_slice()),
                default: Box::from(b"main".as_slice()),
                tree: Box::new([]),
                labels: Box::new([]),
                workflows: Box::new([workflow]),
                protection: None,
                hooked: false,
                checks: raw::Checks {
                    cues: Box::new([]),
                    contexts: Box::new([]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
            },
        );
        forge::grant(&mut world.forge, b"org/second", 1, raw::Permission::Admin);
        forge::grant(&mut world.forge, b"org/second", 2, raw::Permission::Write);
        world
    }
    /// Prepare a second member through the independent repository's git and pull API.
    pub fn public_second(&mut self, task: u64, branch: &[u8]) {
        self.event(top::Event::Application {
            role: None,
            carried: None,
            request: Token::new(101),
            token: Token::new(101),
            retention: Duration::from_secs(60),
            ask: top::Ask::Adopt(top::Adoption {
                project: 5,
                home: false,
                forge: REPO.forge,
                host: Box::from(b"forge.example".as_slice()),
                owner: Box::from(b"org".as_slice()),
                name: Box::from(b"second".as_slice()),
                prefix: Box::from(b"temper/".as_slice()),
                role: top::Role::Owned,
                landing: Box::from(b"main".as_slice()),
                ci: true,
                checks: Box::new([]),
            }),
        });
        self.run_for(1);
        let main = self.forge.branch(b"org/second", b"main").expect("second main");
        self.external_repository(
            b"org/second",
            raw::Op::Git(raw::Git::Create { branch: Box::from(branch), commit: main }),
            2,
        );
        forge::advance(&mut self.forge, &self.forge_env, b"org/second", branch, b"file", b"change", 1)
            .expect("second change");
        let raw::Answer::Created(pull) = self.external_repository(
            b"org/second",
            raw::Op::Write(raw::Write::OpenPull {
                opens_empty: false,
                title: Box::from(b"secret-public-title".as_slice()),
                body: Box::from(b"secret-public-body".as_slice()),
                head: Box::from(branch),
                base: Box::from(b"main".as_slice()),
            }),
            1,
        ) else {
            panic!("second pull")
        };
        let head = translate::commit(self.forge.branch(b"org/second", branch).expect("second head"));
        let mut row = crate::changes::change_row();
        row.task = task;
        row.change.task = task;
        row.repository.repository = 3;
        row.branch = Box::from(branch);
        row.pull = Some(pull);
        row.change.last_head = Some(head);
        row.change.state = temper_engine_domain_forge_change::State::Ready { head, ready_since: self.env.wall };
        row.change.way = temper_engine_domain_forge_change::Way::Turn;
        crate::public::mark(&mut row);
        self.event(top::Event::Change { row });
    }
    pub fn check_pull(&mut self, number: u64, span: Duration) {
        forge::check(&mut self.forge, &self.forge_env, b"org/repo", number, span);
    }

    fn hear(&mut self, hook: &forge::Request) {
        let repository = match hook {
            forge::Request::Hook { repository, .. } if repository.as_ref() == b"org/second" => {
                skein_forge::api::Repository { forge: REPO.forge, repository: 3 }
            }
            forge::Request::Hook { .. } | forge::Request::Reply { .. } => REPO,
        };
        if let Some(hint) = translate::hint(repository, hook) {
            self.to_top.push((self.generation, top::Event::Hint { hint: hint.clone() }));
            self.hints.push(hint);
        }
    }

    pub fn event(&mut self, event: top::Event) {
        top::step(&mut self.top, &self.env, event, &mut self.top_out);
        self.take_top();
        self.top.reclaim();
    }

    /// Set the project's CI rule for a landing branch through durable settings.
    pub fn landing_ci(&mut self, branch: &[u8], required: bool) {
        let item = top::settings::Asked {
            kind: top::settings::Kind::ProjectRules.number(),
            scope: 5,
            number: 1,
            edit: top::settings::Edit::Set(top::settings::SizedValue {
                size: 64,
                value: top::settings::Value::Rule(top::settings::LandingRule {
                    sets: Box::new([]),
                    pattern: top::GatePattern {
                        segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"]
                            .map(Box::from)
                            .into(),
                        last: top::GateLast::Exact(branch.into()),
                    },
                    enforce: true,
                    ci: required,
                    up_to_date: false,
                    gates: Box::new([]),
                    approvals: Box::new([]),
                }),
            }),
        };
        self.top.keep_settings(&self.env, 1, None, &[item], &mut self.top_out);
        self.take_top();
        self.run_for(1);
    }

    /// Keep the scripted root's mapped reviews before it sends `ReviewsChanged`.
    pub fn keep_review_facts(&mut self, task: u64, facts: top::ReviewFacts) -> bool {
        self.top.keep_review_facts(self.wall(), task, facts)
    }

    /// Commit the forge's settings and drive the resulting live-change wakes.
    pub fn keep_settings(&mut self, revision: u64, items: &[top::settings::Asked]) {
        self.top.keep_settings(&self.env, revision, None, items, &mut self.top_out);
        top::fire(&mut self.top, &self.env, &mut self.top_out);
        self.take_top();
        self.run_for(1);
    }

    pub fn adopt(&mut self) {
        self.adopt_as(top::Role::Owned);
    }

    pub fn adopt_as(&mut self, role: top::Role) {
        self.event(top::Event::Application {
            role: None,
            carried: None,
            request: Token::new(100),
            token: Token::new(100),
            retention: Duration::from_secs(60),
            ask: top::Ask::Adopt(top::Adoption {
                project: 5,
                home: role != top::Role::Context,
                forge: REPO.forge,
                host: Box::from(&b"forge.example"[..]),
                owner: Box::from(&b"org"[..]),
                name: Box::from(&b"repo"[..]),
                prefix: Box::from(&b"temper/"[..]),
                role,
                landing: Box::from(&b"main"[..]),
                ci: true,
                checks: Box::new([]),
            }),
        });
        self.run_for(1);
    }

    #[must_use]
    pub fn seen(&self) -> &[top::Request] {
        self.seen.as_slice()
    }
    pub fn take_seen(&mut self) -> Box<[top::Request]> {
        let taken = core::mem::replace(&mut self.seen, List::with_capacity(256));
        taken.into_boxed()
    }
    #[must_use]
    pub fn writes(&self) -> u32 {
        self.writes
    }
    #[must_use]
    pub fn calls(&self) -> u32 {
        self.calls
    }

    /// Requests sent to the independent forge, in submission order.
    pub fn take_calls(&mut self) -> Vec<skein_forge::api::Op> {
        core::mem::take(&mut self.call_ops)
    }
    #[must_use]
    pub fn stored(&self) -> &Map<top::Key, top::Stored> {
        &self.stored
    }
    /// A change's indexed events across projects, in event-number order
    /// (temper/docs/design/domain/landing-history.md, 4.1).
    #[must_use]
    pub fn history(&self, change: u64) -> Vec<history::Event> {
        let mut events = Vec::new();
        for (_, record) in &self.stored {
            if let top::Stored::HistoryChange(index) = record
                && index.change == change
                && let Some(top::Stored::HistoryEvent(event)) =
                    self.stored.get(&top::Key::HistoryEvent { project: index.project, number: index.number })
            {
                events.push(event.clone());
            }
        }
        events.sort_by_key(|event| event.number);
        events
    }

    /// A landing's indexed events across projects, in event-number order.
    #[must_use]
    pub fn landing_history(&self, landing: u64) -> Vec<history::Event> {
        let mut events = Vec::new();
        for (_, record) in &self.stored {
            if let top::Stored::HistoryLanding(index) = record
                && index.landing == landing
                && let Some(top::Stored::HistoryEvent(event)) =
                    self.stored.get(&top::Key::HistoryEvent { project: index.project, number: index.number })
            {
                events.push(event.clone());
            }
        }
        events.sort_by_key(|event| event.number);
        events
    }

    /// Write a fixture event through the domain and keep its store requests.
    pub fn write_event(&mut self, change: u64, step: history::Step) -> Option<u64> {
        let number = self.top.write_event(self.env.wall, change, step, &mut self.top_out);
        self.take_top();
        number
    }

    /// Erase a fixture event while retaining its index rows.
    pub fn erase_history_event(&mut self, project: u32, number: u64) {
        self.stored.remove(&top::Key::HistoryEvent { project, number }).expect("fixture event exists");
    }

    pub fn fallback(&mut self, interval: Duration) {
        self.env.limits.client.fallback = interval;
    }

    pub fn slow_calls(&mut self, delay: Duration) {
        self.call_latencies(delay, delay);
    }

    /// Each fake call draws its latency from this seeded range.
    pub fn call_latencies(&mut self, min: Duration, max: Duration) {
        self.forge_env.limits.latency_min = min;
        self.forge_env.limits.latency_max = max;
    }

    #[must_use]
    pub fn now(&self) -> Time {
        self.env.now
    }

    /// One world step, with no time advance beyond the given horizon.
    pub fn step_until(&mut self, end: Time) -> bool {
        self.iterate(end)
    }
    /// Stop with the forge's reply still pending delivery to the connector.
    pub fn run_until_posted(&mut self) {
        let end = self.env.now.saturating_add(Duration::from_secs(5));
        for _ in 0..20_000 {
            if self.to_top.iter().any(|(_, event)| {
                matches!(
                    event,
                    top::Event::Client(skein_forge_client::Event::Answered {
                        result: Ok(skein_forge::api::Answer::Commented(_)),
                        ..
                    })
                )
            }) {
                return;
            }
            assert!(self.iterate(end), "post must reach the forge before the horizon");
        }
        panic!("post did not settle at the forge");
    }

    /// Stop before the fake processes the named read, after the connector submits it.
    pub fn run_until_read(&mut self, read: &skein_forge::api::Read) {
        let end = self.env.now.saturating_add(Duration::from_secs(5));
        for _ in 0..20_000 {
            if self.call_ops.contains(&skein_forge::api::Op::Read(read.clone())) {
                return;
            }
            assert!(self.iterate(end), "read must be submitted before the horizon");
        }
        panic!("read was not submitted");
    }

    pub fn observations(&mut self) -> Vec<forge::Observation> {
        let mut records = Vec::new();
        while let Some(record) = self.forge.pop_observation() {
            records.push(record);
        }
        assert_eq!(self.forge.observations_lost(), 0);
        records
    }

    pub fn ended_landing_fixture(&mut self, task: u64) {
        let mut row = self.landing(task).clone();
        row.landing.phase = temper_engine_domain_forge_landing::Phase::Landed;
        self.stored.insert(top::Key::Landing(task), top::Stored::Landing(row)).expect("existing landing");
        self.restart();
    }
    pub fn restart(&mut self) {
        self.generation += 1;
        self.top = top::Domain::new(
            &self.env.limits,
            self.seed,
            temper_engine_domain_forge::Config {
                push: None,
                namespace: Box::from(&b"world"[..]),
                writers: self.writer.into_iter().collect(),
            },
            &[top::GitIdentity { forge: REPO.forge, credential: 1 }],
            &[top::Configured {
                number: REPO.forge,
                kind: top::Kind::Forgejo,
                mode: top::Mode::Connected,
                admin_connect: false,
                bot: Box::new([]),
                owned: None,
                git: Box::new([]),
                web: None,
                read_only: false,
                asked: false,
            }],
            false,
        )
        .expect("same configured connector");
        if let Some((project, way, gates)) = &self.landing_config {
            assert!(self.top.project_landing(
                *project,
                *way,
                gates.clone(),
                self.landing_policy,
                top::AfterLanding {
                    verdict: temper_engine_domain_forge_tiers::Settings {
                        retries: 2,
                        suspects: temper_engine_domain_forge_tiers::Suspects::Bisect,
                    },
                    bound: Duration::from_millis(7_200_000),
                },
            ));
            assert!(self.top.project_batch(*project, self.landing_batch));
        }
        let mut rows = List::with_capacity(self.stored.len());
        for (_, record) in &self.stored {
            rows.push(record.clone()).expect("store snapshot capacity");
        }
        self.top.restore_begin(self.env.wall);
        for record in rows.into_boxed() {
            self.event(top::Event::Restore { record });
        }
        self.event(top::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        self.connected();
    }

    /// Script a worker's pushed branch through the independent fake git.
    pub fn permission(&mut self, permission: raw::Permission) {
        forge::grant(&mut self.forge, b"org/repo", 1, permission);
    }

    pub fn produce(&mut self, branch: &[u8]) {
        self.produce_file(branch, b"file", b"change");
    }
    pub fn produce_on(&mut self, branch: &[u8], base: &[u8]) {
        self.external(raw::Op::Git(raw::Git::Create { branch: branch.into(), commit: self.branch(base) }), 2);
        self.push_file(branch, b"file", b"change");
    }
    pub fn produce_file(&mut self, branch: &[u8], path: &[u8], content: &[u8]) {
        self.external(raw::Op::Git(raw::Git::Create { branch: Box::from(branch), commit: 1 }), 2);
        self.push_file(branch, path, content);
    }
    pub fn preload_history(&mut self, count: u32) {
        for number in 0..count {
            self.external(
                raw::Op::Write(raw::Write::CreateIssue {
                    title: Box::from(&b"old"[..]),
                    body: Box::from(number.to_be_bytes()),
                    labels: Box::new([]),
                }),
                2,
            );
        }
    }
    pub fn push(&mut self, branch: &[u8], content: &[u8]) {
        self.push_file(branch, b"file", content);
    }
    /// Push a complete tree as one commit, including an unchanged tree for history bounds.
    pub fn push_tree(&mut self, branch: &[u8], files: Box<[raw::File]>) {
        let parent = self.branch(branch);
        let commit = forge::commit_tree(&mut self.forge, &self.forge_env.limits, parent, None, files, b"landing")
            .expect("fixture tree fits");
        self.external(raw::Op::Git(raw::Git::Push { branch: Box::from(branch), commit, expected: Some(parent) }), 1);
    }

    pub fn push_file(&mut self, branch: &[u8], path: &[u8], content: &[u8]) {
        self.push_as(branch, path, content, 1);
    }
    pub fn push_as(&mut self, branch: &[u8], path: &[u8], content: &[u8], actor: u64) {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: self.forge_env.limits.clone() };
        forge::advance(&mut self.forge, &env, b"org/repo", branch, path, content, actor).expect("scripted push");
    }
    pub fn status(&mut self, branch: &[u8], check: raw::Check) {
        self.external(
            raw::Op::Write(raw::Write::Status {
                commit: self.branch(branch),
                context: Box::from(&b"build"[..]),
                state: check,
                description: Box::new([]),
                link: Box::new([]),
            }),
            1,
        );
    }
    pub fn open_pull(&mut self, head: &[u8], base: &[u8]) -> u64 {
        let raw::Answer::Created(number) = self.external(
            raw::Op::Write(raw::Write::OpenPull {
                opens_empty: false,
                title: Box::from(&b"Change"[..]),
                body: Box::from(&b"Description"[..]),
                head: Box::from(head),
                base: Box::from(base),
            }),
            2,
        ) else {
            panic!("fixture pull opened")
        };
        number
    }
    pub fn close(&mut self, number: u64) {
        self.external(raw::Op::Write(raw::Write::Close { number }), 2);
    }

    pub fn delete_branch(&mut self, branch: &[u8]) {
        self.external(raw::Op::Write(raw::Write::DeleteBranch { branch: branch.into() }), 2);
    }

    pub fn merge(&mut self, number: u64, branch: &[u8]) -> skein_forge::api::Commit {
        let head = self.branch(branch);
        let raw::Answer::Merged(merge) =
            self.external(raw::Op::Write(raw::Write::Merge { number, head, how: raw::MergeHow::Style }), 1)
        else {
            panic!("fixture pull merged")
        };
        translate::commit(merge)
    }

    /// Merge at the forge, retaining its preceding pull snapshot for the
    /// connector's next fresh-read chain, whose base read sees the merge.
    pub fn merge_before_pull_read(&mut self, number: u64) -> skein_forge::api::Commit {
        let op = skein_forge::api::Op::Read(skein_forge::api::Read::Pull { number, ci: false });
        let answer = self.external(raw::Op::Read(raw::Read::Pull { number }), 1);
        let skein_forge::api::Answer::Pull(snapshot) =
            translate::answer_as(&op, answer, &self.env.limits.client, 1, self.signatures).expect("pull projection")
        else {
            panic!("fixture pull read")
        };
        let head = self.branch(&snapshot.head);
        let raw::Answer::Merged(merge) = self.external(
            raw::Op::Write(raw::Write::Merge { number, head, how: skein_fake_forge_domain::api::MergeHow::Style }),
            1,
        ) else {
            panic!("fixture pull merged")
        };
        self.pull_snapshot = Some(snapshot);
        translate::commit(merge)
    }
    pub fn resolve(&mut self, branch: &[u8], content: &[u8]) {
        let head = self.branch(branch);
        let base = self.branch(b"main");
        let merged = forge::merge_commit(
            &mut self.forge,
            &self.forge_env.limits,
            head,
            base,
            Box::new([raw::File { path: Box::from(&b"file"[..]), content: Box::from(content) }]),
            b"Resolve conflict",
        )
        .expect("fixture merge commit");
        assert_eq!(
            self.external(
                raw::Op::Git(raw::Git::Push { branch: Box::from(branch), commit: merged, expected: Some(head) }),
                1,
            ),
            raw::Answer::Pushed(raw::Pushed::Pushed)
        );
    }
    /// The scripted worker delivers a clean prepare on a new landing branch.
    pub fn prepare_landing(&mut self, task: u64, squash: bool) -> u64 {
        let positions: Vec<_> = (0..self.landing(task).landing.members.len()).collect();
        self.prepare_subset(task, squash, &positions)
    }
    pub fn prepare_subset(&mut self, task: u64, squash: bool, positions: &[usize]) -> u64 {
        let row = self.landing(task).clone();
        let mut base = row
            .landing
            .members
            .iter()
            .filter(|m| m.left.is_none() && m.landed.is_none())
            .find_map(|m| m.base)
            .map(translate::number)
            .expect("round base");
        for (index, member) in row.landing.members.iter().enumerate() {
            if member.left.is_some() || member.landed.is_some() {
                continue;
            }
            if let Some(prefix) = member.candidate {
                base = translate::number(prefix);
                continue;
            }
            if !positions.contains(&index) {
                continue;
            }
            let head = translate::number(member.head.expect("round head"));
            let files = crate::landings::stack_files(&self.forge, base, head, self.forge_env.limits.limits.commits);
            let brings = row.landing.round.upstream.is_some_and(|upstream| {
                let raw::Answer::Commits { commits } =
                    self.inspect(&raw::Read::Commits { before: base, after: translate::number(upstream) })
                else {
                    panic!("upstream commits")
                };
                commits.iter().any(|commit| self.forge.is_ancestor(commit.id, head))
            });
            let squash = squash && !brings;
            let message = if squash {
                format!(
                    "candidate\n\nHead: {}",
                    String::from_utf8(top::commit_text(&translate::commit(head)).into_vec()).expect("hex")
                )
            } else {
                "Prepared merge".to_owned()
            };
            let candidate = forge::commit_tree(
                &mut self.forge,
                &self.forge_env.limits,
                base,
                if squash { None } else { Some(head) },
                files,
                message.as_bytes(),
            )
            .expect("prepare commit");
            let provider = skein_forge::api::Repository {
                forge: member.repository.forge,
                repository: member.repository.repository,
            };
            let adopted = self.top.repository(provider).expect("train repository");
            let repository = [&adopted.owner[..], b"/", &adopted.name[..]].concat();
            let answer = self.external_repository(
                &repository,
                raw::Op::Git(raw::Git::PushNew { branch: member.branch.clone(), commit: candidate }),
                1,
            );
            assert!(
                matches!(answer, raw::Answer::PushNew { existing: None }),
                "seed {}: prepare member {index} onto {base} in {:?}; answer {answer:?}",
                self.seed,
                row.landing
            );
            base = candidate;
        }
        base
    }
    #[must_use]
    pub fn branch(&self, branch: &[u8]) -> u64 {
        self.forge.branch(b"org/repo", branch).expect("fixture branch")
    }

    pub fn external(&mut self, op: raw::Op, user: u64) -> raw::Answer {
        self.external_repository(b"org/repo", op, user)
    }
    fn external_repository(&mut self, repository: &[u8], op: raw::Op, user: u64) -> raw::Answer {
        let mut config = self.forge_env.limits.clone();
        config.latency_min = Duration::ZERO;
        config.latency_max = Duration::ZERO;
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config };
        self.serial += 1;
        let token = Token::new(self.serial);
        forge::step(
            &mut self.forge,
            &env,
            forge::Event::Call { reply_to: ReplyTo::new(token), user, repository: Box::from(repository), op },
            &mut self.forge_out,
        );
        let mut answer = None;
        for _ in 0..8 {
            for _ in 0..self.forge_out.len() {
                match self.forge_out.pop().expect("fake output") {
                    forge::Request::Reply { result, .. } => answer = Some(result.expect("fixture succeeds")),
                    hook @ forge::Request::Hook { .. } => self.hear(&hook),
                }
            }
            if !self.forge.is_due(env.now) {
                break;
            }
            forge::fire(&mut self.forge, &env, &mut self.forge_out);
        }
        self.forge.reclaim();
        answer.expect("fixture answered")
    }

    pub fn run_for(&mut self, seconds: u64) {
        let end = self.env.now.saturating_add(Duration::from_secs(seconds));
        for _ in 0..20_000 {
            if !self.iterate(end) {
                return;
            }
        }
        panic!("world did not reach horizon");
    }

    fn iterate(&mut self, end: Time) -> bool {
        if let Some((generation, event)) = self.to_top.pop() {
            if generation == self.generation {
                self.event(event);
            }
        } else if let Some(event) = self.to_forge.pop() {
            forge::step(&mut self.forge, &self.forge_env, event, &mut self.forge_out);
            self.take_forge();
        } else if self.top.is_ready() {
            top::resume(&mut self.top, &self.env, &mut self.top_out);
            self.take_top();
            self.top.reclaim();
        } else if self.forge.is_due(self.env.now) {
            forge::fire(&mut self.forge, &self.forge_env, &mut self.forge_out);
            self.take_forge();
        } else if self.top.next_deadline().is_some_and(|time| time <= self.env.now) {
            top::fire(&mut self.top, &self.env, &mut self.top_out);
            self.take_top();
            self.top.reclaim();
        } else {
            if self.env.now >= end {
                return false;
            }
            let next = self.top.next_deadline().unwrap_or(end).min(self.forge.next_deadline().unwrap_or(end)).min(end);
            self.env.now = next;
            self.env.wall = Wall::from_nanos(next.as_nanos());
            self.forge_env.now = next;
        }
        self.forge.reclaim();
        true
    }

    #[expect(clippy::too_many_lines, reason = "one exhaustive connector output dispatcher")]
    fn take_top(&mut self) {
        let mut calls = List::with_capacity(self.top_out.len());
        let mut restart = None;
        for _ in 0..self.top_out.len() {
            let request = self.top_out.pop().expect("counted top output");
            match request {
                top::Request::AdoptAccess { forge, request, .. } => {
                    top::step(
                        &mut self.top,
                        &self.env,
                        top::Event::AccessAnswered { forge, request, answer: Ok(()) },
                        &mut self.top_out,
                    );
                }
                top::Request::ApplicationPart { request, .. } => {
                    top::step(&mut self.top, &self.env, top::Event::Adopt { request }, &mut self.top_out);
                }
                top::Request::FollowRun { .. }
                | top::Request::CancelFollow { .. }
                | top::Request::MakingCopied { .. }
                | top::Request::MakingAct { .. }
                | top::Request::ApplicationDone { .. }
                | top::Request::MarkedForges
                | top::Request::Marked { .. } => {}
                top::Request::ApplicationAgain { .. } => panic!("application capacity available"),
                top::Request::Adopted { reply_to, result } => {
                    let outcome = match &result {
                        Ok(adopted) => jig_domain_people::Outcome::Adopted { project: adopted.repository.project },
                        Err(_) => jig_domain_people::Outcome::Refused(jig_domain_people::Refusal::Unknown),
                    };
                    top::step(
                        &mut self.top,
                        &self.env,
                        top::Event::ApplicationAnswered { request: reply_to, outcome },
                        &mut self.top_out,
                    );
                    self.seen.push(top::Request::Adopted { reply_to, result }).expect("world output capacity");
                }
                top::Request::BriefClient { event } => {
                    top::step(&mut self.top, &self.env, top::Event::Client(event), &mut self.top_out);
                }
                top::Request::ProjectionEffect { row, entry, .. } => {
                    let mut kept = Queue::uncapped();
                    top::step(&mut self.top, &self.env, top::Event::KeepProjection { row, entry }, &mut kept);
                    for _ in 0..kept.len() {
                        let top::Request::Save { record, .. } = kept.pop().expect("projection keep count") else {
                            panic!("admitted projection saves only")
                        };
                        self.stored.insert(key(&record), record).expect("world store capacity");
                    }
                }
                top::Request::Save { record, release, .. } => {
                    if matches!(record, top::Stored::Writer(_)) {
                        self.writer_saves += 1;
                    }
                    let key = key(&record);
                    self.stored.insert(key, record).expect("world store capacity");
                    if let Some(entry) = release {
                        self.to_top.push((self.generation, top::Event::Committed { entry }));
                    }
                }
                top::Request::Erase { key } => {
                    self.stored.remove(&key).expect("erased row exists");
                }
                top::Request::Call { call, repository, op } => {
                    calls.push((call, repository, op)).expect("bounded calls");
                }
                top::Request::Settled { unfinished } => {
                    self.seen.push(top::Request::Settled { unfinished }).expect("world output capacity");
                }
                top::Request::RestartDone { stage } => {
                    restart = match stage {
                        top::RestartStage::Restored => Some(top::Event::ReadAfresh),
                        top::RestartStage::ReadAfresh => Some(top::Event::SettleOutbox),
                        top::RestartStage::Settled => None,
                    };
                    self.seen.push(top::Request::RestartDone { stage }).expect("world output capacity");
                }
                other @ (top::Request::Bound { .. }
                | top::Request::PlaygroundActs { .. }
                | top::Request::ResetAct { .. }
                | top::Request::LinkNoted { .. }
                | top::Request::ForgeKept(_)
                | top::Request::ForgeGone { .. }
                | top::Request::BoxGroup { .. }
                | top::Request::BoxRemove { .. }
                | top::Request::SetUp { .. }
                | top::Request::Connect { .. }
                | top::Request::Replace { .. }
                | top::Request::Reachable { .. }
                | top::Request::Install { .. }
                | top::Request::Accept { .. }
                | top::Request::Manifest { .. }
                | top::Request::CarriedUnused { .. }
                | top::Request::EraseCarried { .. }
                | top::Request::Resource { .. }
                | top::Request::Taken { .. }
                | top::Request::Refused { .. }
                | top::Request::ResourceRefused { .. }
                | top::Request::HeadRefused { .. }
                | top::Request::LostReadFailed { .. }
                | top::Request::MakeFailed { .. }
                | top::Request::Outcome { .. }
                | top::Request::ProjectionOutcome { .. }
                | top::Request::NotMade { .. }
                | top::Request::ContinueRelease { .. }
                | top::Request::ContinueGoalBranch { .. }
                | top::Request::EffectsSettled { .. }
                | top::Request::Released { .. }
                | top::Request::ProjectReleased { .. }
                | top::Request::Retained { .. }
                | top::Request::ReleaseFailed { .. }
                | top::Request::StepDependents { .. }
                | top::Request::CloserChecked { .. }
                | top::Request::WakeProcedure { .. }
                | top::Request::ProjectAfter { .. }
                | top::Request::ProjectionSettled { .. }
                | top::Request::GoalUntopic { .. }
                | top::Request::ProjectionFailed { .. }
                | top::Request::LandingNews { .. }
                | top::Request::LandingDecision { .. }
                | top::Request::Decided { .. }
                | top::Request::StatusRun { .. }
                | top::Request::TierRun { .. }
                | top::Request::TierDecision { .. }
                | top::Request::FileRedFinding { .. }
                | top::Request::ChangeDecision { .. }
                | top::Request::Reviewed { .. }
                | top::Request::News { .. }
                | top::Request::Read { .. }
                | top::Request::BriefReady { .. }
                | top::Request::BriefReadRefused { .. }
                | top::Request::BriefSized { .. }
                | top::Request::BriefTaken { .. }
                | top::Request::Drift { .. }) => {
                    self.seen.push(other).expect("world output capacity");
                }
            }
        }
        for (call, repository, op) in calls.into_boxed() {
            self.submit(call, repository, op);
        }
        if let Some(event) = restart {
            self.event(event);
        }
    }

    fn submit(&mut self, call: Token, repository: skein_forge::api::Repository, op: skein_forge::api::Op) {
        assert_eq!(repository.forge, REPO.forge);
        assert!(repository.repository == REPO.repository || repository.repository == 0 || repository.repository == 3);
        self.calls += 1;
        self.call_ops.push(op.clone());
        if let Some((from, until)) = self.unavailable
            && self.env.wall >= from
            && self.env.wall < until
        {
            self.to_top.push((
                self.generation,
                top::Event::Client(skein_forge_client::Event::Answered {
                    free: 0,
                    rate: None,
                    call,
                    cost: 1,
                    result: Err(skein_forge::api::Error::Unavailable),
                }),
            ));
            return;
        }
        if let Some(result) = self.scripted_read(&op) {
            self.to_top.push((
                self.generation,
                top::Event::Client(skein_forge_client::Event::Answered { free: 0, rate: None, call, cost: 1, result }),
            ));
            return;
        }
        if let skein_forge::api::Op::Write(_) = op {
            self.writes += 1;
            if self.lose_writes {
                self.to_top.push((
                    self.generation,
                    top::Event::Client(skein_forge_client::Event::Answered {
                        free: 0,
                        rate: None,
                        call,
                        cost: 1,
                        result: Err(skein_forge::api::Error::Timeout),
                    }),
                ));
                return;
            }
        }
        self.serial += 1;
        let token = Token::new(self.serial);
        let raw = translate::op(&op, &self.env.limits.client);
        let name = translate::locate_name(&op).unwrap_or_else(|| {
            Box::from(if repository.repository == 3 { b"org/second".as_slice() } else { b"org/repo".as_slice() })
        });
        self.pending
            .insert(token, Pending { client: call, op, generation: self.generation, repository: name.clone() })
            .expect("world call slots");
        self.to_forge.push(forge::Event::Call { reply_to: ReplyTo::new(token), user: 1, repository: name, op: raw });
    }

    fn scripted_read(
        &self,
        op: &skein_forge::api::Op,
    ) -> Option<Result<skein_forge::api::Answer, skein_forge::api::Error>> {
        match op {
            skein_forge::api::Op::Read(skein_forge::api::Read::PullFiles { page, .. })
                if self.read_script.files_excess.is_some()
                    && self.read_script.files_page.is_none_or(|refused| refused == *page) =>
            {
                Some(Err(skein_forge::api::Error::TooLarge(self.read_script.files_excess.expect("scripted refusal"))))
            }
            skein_forge::api::Op::Read(skein_forge::api::Read::Compare { .. })
                if self.read_script.comparisons_too_large =>
            {
                Some(Err(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed())))
            }
            skein_forge::api::Op::Read(skein_forge::api::Read::Commits { .. }) => self.read_script.ancestry.clone(),
            _ => None,
        }
    }

    fn take_forge(&mut self) {
        for _ in 0..self.forge_out.len() {
            match self.forge_out.pop().expect("counted fake output") {
                forge::Request::Reply { to, result } => {
                    let pending = self.pending.remove(&to.into_token()).expect("pending call");
                    let result = match result {
                        Ok(answer) => {
                            let mut answer =
                                translate::answer_as(&pending.op, answer, &self.env.limits.client, 1, self.signatures);
                            if let Ok(skein_forge::api::Answer::Run(run)) = &mut answer {
                                let raw::Answer::Jobs(jobs) = self
                                    .forge
                                    .inspect(
                                        &self.forge_env.limits,
                                        &pending.repository,
                                        &raw::Read::Jobs { run: run.id },
                                    )
                                    .expect("run jobs")
                                else {
                                    panic!("jobs answer")
                                };
                                run.jobs = translate::run_jobs(jobs);
                            }
                            if matches!(
                                &pending.op,
                                skein_forge::api::Op::Read(
                                    skein_forge::api::Read::PullFor { .. } | skein_forge::api::Read::Pull { .. }
                                )
                            ) && let Ok(skein_forge::api::Answer::Pull(pull)) = &mut answer
                                && let Some(snapshot) = self.pull_snapshot.take()
                            {
                                *pull = snapshot;
                            }
                            answer
                        }
                        Err(raw::Error::TooLarge)
                            if matches!(
                                &pending.op,
                                skein_forge::api::Op::Read(
                                    skein_forge::api::Read::PullFiles { .. } | skein_forge::api::Read::Compare { .. }
                                )
                            ) =>
                        {
                            let limit = u64::from(self.env.limits.client.answer_bytes);
                            Err(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::at_least(
                                skein_forge::api::Bound::AnswerBytes,
                                limit,
                                limit.saturating_add(1),
                            )))
                        }
                        Err(error) => {
                            Err(translate::error(error, forge::time(&self.forge_env.limits, self.forge_env.now)))
                        }
                    };
                    let result = if self.lose_rerun_answer
                        && matches!(&pending.op, skein_forge::api::Op::Write(skein_forge::api::Write::Rerun { .. }))
                    {
                        self.lose_rerun_answer = false;
                        Err(skein_forge::api::Error::Timeout)
                    } else {
                        result
                    };
                    let event = (
                        pending.generation,
                        top::Event::Client(skein_forge_client::Event::Answered {
                            free: 0,
                            rate: None,
                            call: pending.client,
                            cost: 1,
                            result,
                        }),
                    );
                    if matches!(pending.op, skein_forge::api::Op::Read(skein_forge::api::Read::Compare { .. }))
                        && let Some(held) = &mut self.held_compares
                    {
                        held.push(event);
                    } else {
                        self.to_top.push(event);
                    }
                }
                hook @ forge::Request::Hook { .. } => self.hear(&hook),
            }
        }
    }

    /// Every connector call is unavailable in this interval; external changes
    /// still reach the fake, without inventing a delivery (domain/forge/keeping-up.md, 5).
    pub fn unavailable(&mut self, from: Wall, until: Wall) {
        assert!(until > from);
        self.unavailable = Some((from, until));
    }

    /// Answer writes as timed out, then apply their copies after this delay.
    /// Updates may arrive beyond their lifetime (domain/forge/README.md, 6).
    pub fn land_writes_late(&mut self, after: Duration) {
        self.forge_env.limits.landing = 1000;
        self.forge_env.limits.land_min = after;
        self.forge_env.limits.land_max = after;
    }

    /// Lose writes in transit, with no copy reaching the forge.
    pub fn lose_writes(&mut self) {
        self.lose_writes = true;
    }

    /// Answer subsequent writes normally; already sent copies keep their timers.
    pub fn answer_writes(&mut self) {
        self.lose_writes = false;
        self.forge_env.limits.landing = 0;
    }
}

fn key(row: &top::Stored) -> top::Key {
    match row {
        top::Stored::HistoryProject(row) => top::Key::HistoryProject { project: row.project },
        top::Stored::HistoryEvent(row) => top::Key::HistoryEvent { project: row.project, number: row.number },
        top::Stored::HistoryChange(row) => {
            top::Key::HistoryChange { project: row.project, change: row.change, number: row.number }
        }
        top::Stored::HistoryLanding(row) => {
            top::Key::HistoryLanding { project: row.project, landing: row.landing, number: row.number }
        }
        top::Stored::HistoryLast(row) => {
            top::Key::HistoryLast { project: row.project, number: row.number, change: row.change }
        }
        top::Stored::Landing(row) => top::Key::Landing(row.landing.task),
        top::Stored::Receipt { key, .. } => top::Key::Receipt(key.clone()),
        top::Stored::Call { key, .. } => top::Key::Call(*key),
        top::Stored::Repository(row) => top::Key::Repository(row.provider),
        top::Stored::Hold(row) => top::Key::Hold(row.name.clone()),
        top::Stored::Names { task, .. } => top::Key::Names(*task),
        top::Stored::Subscription(row) => top::Key::Subscription { task: row.task, topic: row.topic.clone() },
        top::Stored::BranchHead(row) => top::Key::BranchHead(row.name.clone()),
        top::Stored::PullState(row) => top::Key::PullState(row.name.clone()),
        top::Stored::Ci(row) => top::Key::Ci { repository: row.repository, head: row.head },
        top::Stored::Landed { commit, .. } => top::Key::Landed(*commit),
        top::Stored::ProposedEffect(row) => top::Key::ProposedEffect(row.number),
        top::Stored::Entry { entry, .. } => top::Key::Entry(entry.number),
        top::Stored::Client(row) => top::Key::Client(match row {
            skein_forge_client::Stored::Live(row) => skein_forge_client::Key::Live(row.watch.resource.clone()),
            skein_forge_client::Stored::Repository(row) => skein_forge_client::Key::Repository(row.repository),
            skein_forge_client::Stored::Writer(_) => {
                unreachable!("the connector keeps the client's writer as Stored::Writer")
            }
        }),
        top::Stored::Closer(row) => top::Key::Closer(row.task),
        top::Stored::Change(row) => top::Key::Change(row.task),
        top::Stored::Issue(row) => top::Key::Issue(row.goal),
        top::Stored::RefusedSetting(key) => top::Key::Setting(*key),
        top::Stored::Setting(row) => top::Key::Setting(row.key),
        top::Stored::Making(row) => top::Key::Making(row.key.clone()),
        top::Stored::Binding { address, .. } => top::Key::Binding(address.clone()),
        top::Stored::Dismissed { party, what } => top::Key::Dismissed { party: *party, what: *what },
        top::Stored::Flight { task, entry } => top::Key::Flight { task: *task, entry: *entry },
        top::Stored::Release(row) => top::Key::Release(row.task),
        top::Stored::Application { token, .. } => top::Key::Application(*token),
        top::Stored::Writer(row) => top::Key::Writer(row.forge),
        top::Stored::Forge(row) => top::Key::Forge(row.number),
        top::Stored::ForgeNumber { .. } => top::Key::ForgeNumber,
        top::Stored::Playground(_) => top::Key::Playground,
        top::Stored::Follow(row) => top::Key::Follow { project: row.project, follow: row.follow },
        top::Stored::Tier(row) => top::Key::Tier { repository: row.repository, tier: row.tier.clone() },
        top::Stored::ProjectFacts(row) => top::Key::ProjectFacts { project: row.project },
        top::Stored::Device(row) => top::Key::Device { party: row.party, number: row.number },
        top::Stored::LandedDraft(row) => top::Key::LandedDraft { repository: row.repository, path: row.path.clone() },
        top::Stored::Away(row) => top::Key::Away { party: row.party, project: row.project },
        top::Stored::Decided(row) => top::Key::Decided { project: row.project, number: row.number },
        top::Stored::Finding(row) => top::Key::Finding { project: row.project, number: row.number },
        top::Stored::Looked(row) => top::Key::Looked { party: row.party, project: row.project },
        top::Stored::LandingLog(row) => top::Key::LandingLog { repository: row.repository },
        top::Stored::Verification(row) => top::Key::Verification { repository: row.repository, commit: row.commit },
    }
}

fn scenario_config(
    mut config: forge::Config,
    branches: u32,
    turn: bool,
    tiered: bool,
    protection: Option<&raw::Protection>,
) -> forge::Config {
    config.limits.branches = branches;
    if tiered {
        config.limits.statuses = 16;
        config.limits.content_bytes = 512;
        config.limits.commits = 32;
    }
    config.limits.workflows = u32::from(turn || tiered);
    config.limits.workflow_runs = if tiered {
        16
    } else if turn {
        4
    } else {
        0
    };
    config.limits.bypassing_users =
        protection.map_or(0, |rule| u32::try_from(rule.bypassing_users.len()).expect("fixture bypass list"));
    config
}

fn repository_config(repositories: u32) -> forge::Config {
    let mut config = fake_config();
    config.limits.repositories = repositories;
    config
}

/// Bounded Forgejo fixture shared by the connector and root worlds.
#[must_use]
pub fn fake_config() -> forge::Config {
    forge::Config {
        setup_faults: 0,
        lost: 0,
        limits: forge::Limits {
            workflows: 0,
            workflow_runs: 0,
            repositories: 1,
            users: 3,
            labels: 2,
            items: 64,
            comments: 32,
            dependencies: 2,
            reviews: 8,
            branches: 4,
            commits: 16,
            files: 4,
            statuses: 4,
            contexts: 4,
            bypassing_users: 0,
            pages: 1,
            name_bytes: 64,
            title_bytes: 64,
            body_bytes: 1024,
            content_bytes: 128,
            page_size: 2,
            calls: 4,
            hooks: 8,
            observations: 64,
        },
        latency_min: Duration::from_millis(10),
        latency_max: Duration::from_millis(50),
        settle_min: Duration::from_secs(2),
        settle_max: Duration::from_secs(2),
        late: 0,
        late_min: Duration::from_secs(1),
        late_max: Duration::from_secs(3),
        unavailable: 0,
        timeouts: 0,
        landing: 0,
        land_min: Duration::from_secs(1),
        land_max: Duration::from_secs(3),
        rate_limit: 0,
        rate_window: Duration::from_secs(10),
        ci: 3,
        made_checks: forge::Config::test().made_checks,
        hook_min: Duration::ZERO,
        hook_max: Duration::ZERO,
        hooks_late: 0,
        hooks_lost: 0,
        resolution: Duration::from_secs(1),
        skew: forge::Skew::Ahead(Duration::from_secs(20)),
        status_updates: false,
        edit_updates: false,
    }
}

fn configured_forge() -> top::Configured {
    top::Configured {
        number: REPO.forge,
        kind: top::Kind::Forgejo,
        mode: top::Mode::Connected,
        admin_connect: false,
        bot: Box::new([]),
        owned: None,
        git: Box::new([]),
        web: None,
        read_only: false,
        asked: false,
    }
}

fn workflows(turn: bool, tier: Option<raw::Workflow>) -> Box<[raw::Workflow]> {
    if let Some(tier) = tier {
        Box::new([tier])
    } else if turn {
        Box::new([raw::Workflow {
            fails: Box::new([]),
            flaky: Box::new([]),
            report: None,
            select: None,
            file: Box::from(b"gate.yml".as_slice()),
            jobs: Box::new([Box::from(b"test".as_slice())]),
            latency_min: Duration::from_secs(2),
            latency_max: Duration::from_secs(2),
            silent: 0,
            passes: 1000,
            cue: None,
        }])
    } else {
        Box::new([])
    }
}

#[cfg(test)]
mod records {
    use super::*;

    #[test]
    fn records_survive_a_restart_with_their_numbers() {
        let mut world = World::new(71);
        let mut expected = Vec::new();
        for number in [3, 2] {
            let record = top::ForgeRecord {
                person: None,
                git_program: None,
                number,
                kind: top::Kind::Forgejo,
                mode: top::Mode::Managed,
                state: top::ForgeState::Kept,
                place: top::Place {
                    api: top::Address { host: b"forge.example".as_slice().into(), port: 443, tls: None },
                    git: b"https://forge.example".as_slice().into(),
                    web: None,
                },
                read_only: false,
                bot: top::Bot { login: b"temper-bot".as_slice().into(), name: Box::new([]), email: Box::new([]) },
                asked: false,
                first: Some(top::FirstIntent::Empty(top::EmptyIntent {
                    playground: false,
                    org: b"org".as_slice().into(),
                    repository: b"repo".as_slice().into(),
                    label: b"Work".as_slice().into(),
                })),
            };
            world
                .stored
                .insert(top::Key::Forge(number), top::Stored::Forge(record.clone()))
                .expect("fixture store room");
            expected.push(record);
        }
        world.stored.insert(top::Key::ForgeNumber, top::Stored::ForgeNumber { last: 9 }).expect("fixture number room");
        for _ in 0..2 {
            world.take_seen();
            world.restart();
            for record in &expected {
                assert_eq!(world.top.record(record.number), Some(record));
                assert!(world.seen().contains(&top::Request::ForgeKept(record.clone())));
            }
            let listed: Vec<_> = world.top.forges().rows.iter().map(|row| row.number).collect();
            assert_eq!(listed, [1, 2, 3]);
            assert_eq!(world.stored().get(&top::Key::ForgeNumber), Some(&top::Stored::ForgeNumber { last: 9 }));
        }
    }
}
