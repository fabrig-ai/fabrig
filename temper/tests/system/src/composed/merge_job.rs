//! The engine's merge job crosses its worker and returns to the change
//! (temper/docs/design/domain/testing.md, 2.1; domain/forge.md, 8.4).
use super::{Duration, Script, Token, VecDeque, World, call, engine, host, link, tasks, worker};
use jig_domain_link::JobBound;
use temper_fake_checkout::git::Remote;
use temper_worker_domain::tooling;

const STEPS: u32 = 10000;
const REMOTE: &[u8] = b"org/repo";
const COMMAND: &[u8] = b"world merge command\n";
const SETTLED: &[u8] = br#"{"settled":["FILE"]}"#;
const LEFT: &[u8] = br#"{"settled":[],"left":["FILE"]}"#;

/// One row of the merge job's story (temper/docs/design/domain/testing.md, 2.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    Settled,
    Left,
}

/// What a row's run showed, read from the store, the fake forge and the fake disk.
#[derive(Debug)]
pub struct MergeJobReport {
    pub job: tasks::End,
    pub failures: u32,
    pub branch: u64,
    pub parents: Vec<u64>,
    pub program: Vec<u8>,
    pub args: Vec<Vec<u8>>,
    pub request: Vec<u8>,
    pub command_found: bool,
    pub resolution: Option<Vec<u8>>,
    pub hello: Vec<JobBound>,
    pub declared: Vec<JobBound>,
    pub head: u64,
    pub base: u64,
    pub directory: Vec<u8>,
    pub tree: temper_fake_checkout::git::Tree,
    pub continued: Option<[u8; 32]>,
    pub resolution_base: Option<[u8; 32]>,
    pub marked: bool,
    pub merge_head: Option<Vec<u8>>,
}

pub(super) struct Story {
    row: Row,
    pub(super) held: Option<engine::Link>,
    pub(super) pending: Option<(Token, engine::Assignment)>,
    pub(super) base: Option<u64>,
    pub(super) hello: Vec<JobBound>,
    declared: Vec<JobBound>,
    job: Option<u64>,
    spawn: Option<Spawn>,
}

struct Spawn {
    program: Vec<u8>,
    args: Vec<Vec<u8>>,
    request: Vec<u8>,
    directory: Vec<u8>,
    command_found: bool,
    marked: bool,
    merge_head: Option<Vec<u8>>,
}

#[derive(Default)]
struct NextStep {
    continued: Option<[u8; 32]>,
    resolution: Option<Resolution>,
}

struct Resolution {
    base: [u8; 32],
    words: Vec<u8>,
}

fn config(seed: u64) -> engine::Config {
    let mut config = super::agreement::config(seed);
    let mut jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    jig.core.tasks = temper_engine_domain_world::forge_routes::box_limits().tasks;
    config.core.jobs = temper_service::limits::merge_jobs(&jig);
    assert_eq!(config.core.jobs.as_ref(), [temper_service::limits::merge_bound()]);
    let kinds = &mut config.core.defaults.rules.ceiling.delegation.kinds;
    let mut extended = kinds.to_vec();
    if !extended.contains(&jig_domain_authority::Executor::Job(3)) {
        extended.push(jig_domain_authority::Executor::Job(3));
    }
    *kinds = extended.into_boxed_slice();
    for project in &mut config.core.seed_projects {
        project.policy.policy.ceiling.delegation.kinds.clone_from(kinds);
        for role in &mut project.policy.policy.roles {
            role.authority.delegation.kinds.clone_from(kinds);
        }
    }
    let mut landing = engine::ProjectLanding {
        merge_job: Some(b".temper/merge".as_slice().into()),
        by: engine::Way::Queue,
        repositories: Box::new([]),
        after: engine::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 },
    };
    if let Some(existing) = config.landings.get_mut(&1) {
        existing.merge_job = landing.merge_job.take();
    } else {
        config.landings.insert(1, landing).expect("project landing room");
    }
    // The changed base takes the update route before the forge's merge.
    let rule = engine::LandingRule {
        connector: 1,
        kind: 4,
        enforce: true,
        ci: true,
        up_to_date: true,
        pattern: jig_domain_authority::Pattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: jig_domain_authority::Last::Open(Box::default()),
        },
        gates: Box::new([]),
        approvals: Box::new([]),
    };
    config.kinds_stock = engine::landing_stock(&[rule], &skein_lib::Map::with_capacity(0), 2, 4);
    config
}

impl World {
    /// A project naming `.temper/merge`, both roots declaring production's merge kind.
    #[must_use]
    pub fn merge_job(seed: u64, row: Row) -> Self {
        let goal = Script {
            cue: b"Help the person with their work.".as_slice().into(),
            turns: Box::new([
                call(b"change", br#"{"changes":[{"words":"Write the first FILE","landing":"goal"},{"words":"Advance the goal FILE","landing":"goal"}]}"#),
                call(b"wait", b"{}"),
            ]),
        };
        let producer = |content: &str| Script {
            cue: b"Read only what the change needs".as_slice().into(),
            turns: Box::new([
                call(
                    b"write",
                    &serde_json::to_vec(&serde_json::json!({"path":"FILE","content":content})).expect("write JSON"),
                ),
                call(b"finish", br#"{"title":"Edit FILE","body":"Keep the edit."}"#),
            ]),
        };
        let declared = config(seed).core.jobs.to_vec();
        let mut world = Self::configured_with(seed, goal.clone(), true, config);
        world.goal_branch = true;
        world.forge.advance(REMOTE, b"main", b".temper/merge", COMMAND);
        world.scripts = VecDeque::from([goal, producer("head\n"), producer("base\n")]);
        world.merge_job = Some(Story {
            row,
            held: None,
            pending: None,
            base: None,
            hello: Vec::new(),
            declared,
            job: None,
            spawn: None,
        });
        world
    }

    /// Run until the job has ended durably and the change takes its next step.
    pub fn run_merge_job(&mut self) -> MergeJobReport {
        // The channel holds the first producer's answer, while the released
        // worker serves the independent change that moves its landing branch.
        for _ in 0..STEPS {
            self.tick();
            if self.merge_story().held.is_some() {
                break;
            }
        }
        assert!(self.merge_story().held.is_some(), "first producer finished");
        if let Some((channel, assignment)) = self.merge_job.as_mut().expect("merge story").pending.take() {
            self.assigned(channel, assignment);
        }
        let change = self.merge_change();
        let branch = change.branch.clone();
        let landing = change.base.clone();
        let head = self.forge.branch(REMOTE, &branch).expect("producer delivered");
        let base = self.advance_goal_for_producer(&landing);
        let story = self.merge_job.as_mut().expect("merge story");
        story.base = Some(base);
        let answer = story.held.take().expect("held answer");
        self.up(answer);
        for _ in 0..STEPS {
            if self.process.answered.is_some()
                && self.root_events.is_empty()
                && self.worker_events.is_empty()
                && !self.committing()
            {
                self.env.now = self.env.now.saturating_add(Duration::from_secs(1));
            }
            self.tick();
            let Some(job) = self.merge_story().job else { continue };
            if self.job_outcome(job).is_some()
                && (self.next_merge_step().is_some()
                    || self.merge_story().spawn.as_ref().is_some_and(|spawn| !spawn.command_found))
            {
                break;
            }
        }
        let story = self.merge_story();
        let job = story.job.expect("merge job placed on its worker");
        let (job, failures) = self.job_outcome(job).expect("job outcome kept durably");
        let next = if matches!(job, tasks::End::Finished { .. }) {
            self.next_merge_step().expect("change went on after the job")
        } else {
            NextStep::default()
        };
        let spawn = story.spawn.as_ref().expect("worker spawned the command");
        let hello = story.hello.clone();
        let declared = story.declared.clone();
        let mut report = MergeJobReport {
            job,
            failures,
            branch: head,
            parents: Vec::new(),
            program: spawn.program.clone(),
            args: spawn.args.clone(),
            request: spawn.request.clone(),
            command_found: spawn.command_found,
            resolution: next.resolution.as_ref().map(|resolution| resolution.words.clone()),
            hello,
            declared,
            head,
            base,
            directory: spawn.directory.clone(),
            tree: temper_fake_checkout::git::Tree::new(),
            continued: next.continued,
            resolution_base: next.resolution.map(|resolution| resolution.base),
            marked: spawn.marked,
            merge_head: spawn.merge_head.clone(),
        };
        report.branch = self.forge.branch(REMOTE, &branch).expect("change branch remains");
        report.parents =
            [self.forge.parent(report.branch), self.forge.merge_parent(report.branch)].into_iter().flatten().collect();
        report.tree = self.forge.tree(report.branch);
        report
    }

    fn merge_story(&self) -> &Story {
        self.merge_job.as_ref().expect("merge story")
    }

    fn merge_change(&self) -> &temper_engine_domain_forge::ChangeRow {
        self.store
            .rows
            .values()
            .find_map(|row| match row {
                engine::Record::Forge { row } => match row.as_ref() {
                    temper_engine_domain_forge::Stored::Change(row) if row.task == 2 => Some(row),
                    _ => None,
                },
                engine::Record::Core(_) => None,
            })
            .expect("first change")
    }

    fn job_outcome(&self, task: u64) -> Option<(tasks::End, u32)> {
        let row = [tasks::Key::Ended(task), tasks::Key::Live(task)].into_iter().find_map(|key| {
            let key = engine::Key::Core(jig_domain_core::Key::Tasks(key));
            match self.store.rows.get(&key)? {
                engine::Record::Core(jig_domain_core::Record::Tasks(
                    tasks::Stored::Ended(row) | tasks::Stored::Live(row),
                )) => Some(row),
                _ => None,
            }
        })?;
        let classes = [
            tasks::Class::Transient,
            tasks::Class::Permanent,
            tasks::Class::Run,
            tasks::Class::Agent,
            tasks::Class::Lost,
            tasks::Class::Invalid,
        ];
        let failures = classes.iter().map(|class| row.tries.of(*class)).sum();
        if let tasks::Phase::Ended(tasks::Ending::Done(result)) = &row.phase {
            Some((tasks::End::Finished { result: result.clone(), cancel_delegates: false }, failures))
        } else {
            let class = classes.into_iter().find(|class| row.tries.of(*class) != 0)?;
            Some((tasks::End::Failed(class), failures))
        }
    }

    fn next_merge_step(&self) -> Option<NextStep> {
        let change = self.merge_change();
        if let Some((task, temper_engine_domain_forge_change::Delegate::Resolve { base })) = change.delegate {
            let key = engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)));
            if let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
                self.store.rows.get(&key)
            {
                return Some(NextStep {
                    continued: None,
                    resolution: Some(Resolution { base, words: row.spec.words.to_vec() }),
                });
            }
        }
        if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }) {
            return Some(NextStep { continued: change.result.head, resolution: None });
        }
        None
    }

    pub(super) fn assigned_job(
        &mut self,
        channel: Token,
        job: host::JobAssignment,
        workspace: Option<&engine::ForgeWorkspace>,
    ) {
        assert_eq!(channel, super::CHANNEL);
        assert_eq!(job.kind.0, 3);
        assert_eq!(job.deadline, Duration::from_secs(600));
        let mut own = temper_service::limits::profile(temper_service::limits::Profile::Small);
        let checkout = &self.worker_env.limits.checkout;
        own.checkout.repositories = checkout.repositories;
        own.checkout.name_bytes = checkout.name_bytes;
        own.checkout.conflicts = checkout.conflicts;
        own.checkout.path_bytes = checkout.path_bytes;
        let limits = temper_service::worker_settings::payload_limits(&own).expect("worker payload limits");
        self.merge_job.as_mut().expect("merge story").job = Some(job.run.raw());
        self.worker_events.push_back(link::job(job, workspace, &limits));
    }

    pub(super) fn job_operation(&mut self, owner: Token, op: tooling::Request) {
        let event = match op {
            tooling::Request::Write { owner: inner, path, bytes } => {
                assert_eq!(owner, inner);
                self.disk.write(&disk_path(&path), &bytes);
                tooling::Event::Written { owner, written: true }
            }
            tooling::Request::Spawn { owner: inner, program, args, dir, .. } => {
                assert_eq!(owner, inner);
                self.play_merge_command(owner, &program, &args, &dir);
                return;
            }
            tooling::Request::Read { owner: inner, path, most } => {
                assert_eq!(owner, inner);
                let bytes = self
                    .disk
                    .content(&disk_path(&path))
                    .filter(|bytes| bytes.len() <= usize::try_from(most).expect("result bound"))
                    .map(Into::into);
                tooling::Event::Read { owner, bytes }
            }
            tooling::Request::Kill { owner: inner } => {
                assert_eq!(owner, inner);
                self.worker_events.push_back(worker::Event::Job {
                    owner,
                    event: tooling::Event::Exited { owner, exit: tooling::Exit::Signal, tail: Box::new([]) },
                });
                tooling::Event::Gone { owner }
            }
            other => panic!("root routes job IO only: {other:?}"),
        };
        self.worker_events.push_back(worker::Event::Job { owner, event });
    }

    fn play_merge_command(&mut self, owner: Token, program: &[u8], args: &[Box<[u8]>], directory: &[u8]) {
        let found = self.disk.content(&disk_path(program)) == Some(COMMAND);
        let dir = disk_path(directory);
        let repository = [dir.as_slice(), b"/repo"].concat();
        let file = [repository.as_slice(), b"/FILE"].concat();
        let unmerged = self.disk.tree(&[repository.as_slice(), b"/.git/temper-conflicts"].concat());
        let marked = unmerged.values().any(|path| path.as_slice() == b"FILE")
            && self.disk.content(&file).is_some_and(|bytes| {
                bytes.starts_with(b"<<<<<<<")
                    && bytes.windows(b"head\n".len()).any(|part| part == b"head\n")
                    && bytes.windows(b"base\n".len()).any(|part| part == b"base\n")
            });
        let merge_head = self.disk.content(&[repository.as_slice(), b"/.git/MERGE_HEAD"].concat()).map(<[u8]>::to_vec);
        let request = args.get(2).and_then(|path| self.disk.content(&disk_path(path))).unwrap_or_default().to_vec();
        let story = self.merge_job.as_mut().expect("merge story");
        story.spawn = Some(Spawn {
            program: program.to_vec(),
            args: args.iter().map(|arg| arg.to_vec()).collect(),
            request,
            directory: directory.to_vec(),
            command_found: found,
            marked,
            merge_head,
        });
        if found {
            if story.row == Row::Settled {
                self.disk.write(&file, b"settled\n");
                self.disk.write(&[repository.as_slice(), b"/.git/index"].concat(), b"FILE\0settled\n");
            }
            self.disk.write(
                &[dir.as_slice(), b"/result.json"].concat(),
                if story.row == Row::Left { LEFT } else { SETTLED },
            );
            self.worker_events.push_back(worker::Event::Job { owner, event: tooling::Event::Spawned { owner } });
        }
        let exit =
            if found { tooling::Exit::Code(if story.row == Row::Left { 3 } else { 0 }) } else { tooling::Exit::Signal };
        self.worker_events
            .push_back(worker::Event::Job { owner, event: tooling::Event::Exited { owner, exit, tail: Box::new([]) } });
        self.worker_events.push_back(worker::Event::Job { owner, event: tooling::Event::Gone { owner } });
    }
}

/// The job IO face and the fake git face spell the same workspace directory differently.
fn disk_path(path: &[u8]) -> Vec<u8> {
    [b"ws/".as_slice(), path.strip_prefix(b"workspaces/").expect("job workspace path")].concat()
}
