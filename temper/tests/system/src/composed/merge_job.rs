//! The engine's merge job crosses its worker and returns to the change
//! (temper/docs/design/testing/README.md, 2.1; domain/forge/change/repairs.md).
use super::{Duration, Script, Token, VecDeque, World, call, engine, host, link, tasks, worker};
use jig_domain_link::JobBound;
use temper_fake_checkout::git::Remote;
use temper_worker_domain::tooling;

const STEPS: u32 = 10000;
const REMOTE: &[u8] = b"org/repo";
const COMMAND: &[u8] = b"world merge command\n";
const LEFT: &[u8] = br#"{"settled":[],"left":["FILE"]}"#;

/// One row of the merge job's story (temper/docs/design/testing/README.md, 2.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    Settled,
    Left,
    Clean,
    OtherBranch,
    PastTheBound,
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
    pub conflicts: Vec<Vec<u8>>,
    pub staged: Vec<u8>,
    pub conflict_bound: u32,
    pub catch_up: Option<[u8; 32]>,
    pub head_tree: temper_fake_checkout::git::Tree,
    pub base_tree: temper_fake_checkout::git::Tree,
    pub merge_head: Option<Vec<u8>>,
    pub cached_repository: Option<Vec<u8>>,
    pub base_present_before: Option<bool>,
    pub base_fetch_repository: Option<Vec<u8>>,
    pub base_present_at_fetch: Option<bool>,
}

pub(super) struct Story {
    pub(super) row: Row,
    pub(super) held: Option<engine::Link>,
    pub(super) pending: Option<(Token, engine::Assignment)>,
    pub(super) base: Option<u64>,
    pub(super) hello: Vec<JobBound>,
    pub(super) clean_answered: bool,
    declared: Vec<JobBound>,
    job: Option<u64>,
    spawn: Option<Spawn>,
    cached_repository: Option<Vec<u8>>,
    base_present_before: Option<bool>,
    base_fetch_repository: Option<Vec<u8>>,
    base_present_at_fetch: Option<bool>,
}

struct Spawn {
    program: Vec<u8>,
    args: Vec<Vec<u8>>,
    request: Vec<u8>,
    directory: Vec<u8>,
    command_found: bool,
    marked: bool,
    conflicts: Vec<Vec<u8>>,
    staged: Vec<u8>,
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
        gate_retries: 2,
        gate_timeout: skein_lib::Duration::from_millis(3_600_000),
        check_timeout: None,
        close_job: None,
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
        let declared = config(seed).core.jobs.to_vec();
        let mut world = Self::configured_with(seed, goal.clone(), true, config);
        world.goal_branch = true;
        world.forge.advance(REMOTE, b"main", b".temper/merge", COMMAND);
        let count = if row == Row::PastTheBound { world.worker_env.limits.checkout.conflicts + 1 } else { 1 };
        let files: Vec<_> = (0..count)
            .map(|index| if index == 0 { b"FILE".to_vec() } else { format!("FILE{index}").into_bytes() })
            .collect();
        world.scripts = VecDeque::from([goal, producer(&files, "head\n"), producer(&files, "base\n")]);
        world.merge_job = Some(Story {
            row,
            held: None,
            pending: None,
            base: None,
            hello: Vec::new(),
            clean_answered: false,
            declared,
            job: None,
            spawn: None,
            cached_repository: None,
            base_present_before: None,
            base_fetch_repository: None,
            base_present_at_fetch: None,
        });
        world
    }

    /// Run until the job has ended durably and the change takes its next step.
    pub fn run_merge_job(&mut self) -> MergeJobReport {
        let (branch, head, base) = self.reach_merge_job();
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
            if let Some((ending, _)) = self.job_outcome(job)
                && (matches!(ending, tasks::End::Failed(_))
                    || self.next_merge_step().is_some()
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
        let spawn =
            story.spawn.as_ref().unwrap_or_else(|| panic!("worker spawned the command: {job:?}, failures {failures}"));
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
            conflicts: spawn.conflicts.clone(),
            staged: spawn.staged.clone(),
            conflict_bound: self.worker_env.limits.checkout.conflicts,
            catch_up: self.merge_change().catch_up,
            head_tree: self.forge.tree(head),
            base_tree: self.forge.tree(base),
            merge_head: spawn.merge_head.clone(),
            cached_repository: story.cached_repository.clone(),
            base_present_before: story.base_present_before,
            base_fetch_repository: story.base_fetch_repository.clone(),
            base_present_at_fetch: story.base_present_at_fetch,
        };
        report.branch = self
            .forge
            .branch(REMOTE, &branch)
            .unwrap_or_else(|| panic!("change branch remains: {:?}, failures {}", report.job, report.failures));
        report.parents =
            [self.forge.parent(report.branch), self.forge.merge_parent(report.branch)].into_iter().flatten().collect();
        report.tree = self.forge.tree(report.branch);
        report
    }

    fn reach_merge_job(&mut self) -> (Box<[u8]>, u64, u64) {
        if self.merge_story().row == Row::Clean {
            for _ in 0..STEPS {
                self.tick();
                if self.merge_story().job.is_some() {
                    break;
                }
            }
            let change = self.merge_change();
            (
                change.branch.clone(),
                self.forge.branch(REMOTE, &change.base).expect("goal tip"),
                self.merge_story().base.expect("main advanced"),
            )
        } else {
            // Hold the first producer's answer while the independent change
            // moves its landing branch on the released worker.
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
            if self.merge_story().row == Row::OtherBranch {
                let suffix = format!("/.git/objects/{head}");
                let repository = self
                    .disk
                    .tree(b"ws")
                    .keys()
                    .find_map(|path| {
                        path.strip_suffix(suffix.as_bytes()).map(|prefix| [b"ws/".as_slice(), prefix].concat())
                    })
                    .expect("producer's cached clone holds its delivery");
                self.merge_job.as_mut().expect("merge story").cached_repository = Some(repository);
            }
            let base = self.advance_goal_for_producer(&landing);
            let story = self.merge_job.as_mut().expect("merge story");
            story.base = Some(base);
            let answer = story.held.take().expect("held answer");
            self.up(answer);
            (branch, head, base)
        }
    }

    pub(super) fn prepare_clean_goal(&mut self, task: u64) {
        if task != 1 || self.merge_story().row != Row::Clean || self.merge_story().base.is_some() {
            return;
        }
        // The goal branch exists before main moves, so its catch-up names
        // a commit outside that branch's history.
        let base = self.forge.advance(REMOTE, b"main", b"MAIN", b"main work\n");
        self.merge_job.as_mut().expect("merge story").base = Some(base);
        let text =
            temper_engine_domain_forge::commit_text(&temper_worker_checkout_world::translate::commit(base).raw());
        let text = core::str::from_utf8(&text).expect("commit text");
        let input = format!(r#"{{"words":"Take main's changes","landing":"goal","merge":"{text}"}}"#);
        self.scripts = VecDeque::from([
            Script {
                cue: b"Help the person with their work.".as_slice().into(),
                turns: Box::new([
                    call(b"change", br#"{"words":"Write the goal's FILE","landing":"goal"}"#),
                    call(b"wait", b"{}"),
                    call(b"change", input.as_bytes()),
                    call(b"wait", b"{}"),
                ]),
            },
            producer(&[b"FILE".to_vec()], "goal\n"),
        ]);
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
                    temper_engine_domain_forge::Stored::Change(row) => {
                        let target =
                            if self.merge_story().row == Row::Clean { row.catch_up.is_some() } else { row.task == 2 };
                        target.then_some(row)
                    }
                    _ => None,
                },
                engine::Record::Core(_) => None,
            })
            .expect("merge change")
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
        let story = self.merge_job.as_mut().expect("merge story");
        story.job = Some(job.run.raw());
        if let Some(repository) = &story.cached_repository {
            let base = story.base.expect("base made after producer's clone");
            story.base_present_before =
                Some(self.disk.exists(&[repository.as_slice(), format!("/.git/objects/{base}").as_bytes()].concat()));
        }
        self.worker_events.push_back(link::job(job, workspace, &limits));
    }

    pub(super) fn observe_merge_fetch(&mut self, op: &worker::checkout::git::Op) {
        use worker::checkout::git::{Op, Want};
        let Some(story) = &mut self.merge_job else { return };
        if story.job.is_none() {
            return;
        }
        if let Op::Fetch { at, want: Want::Commit { commit }, .. } = op {
            let base = story.base.expect("job's base");
            if temper_worker_checkout_world::translate::fake(*commit) == base {
                let repository = temper_worker_checkout_world::translate::path(at);
                story.base_present_at_fetch = Some(
                    self.disk.exists(&[repository.as_slice(), format!("/.git/objects/{base}").as_bytes()].concat()),
                );
                story.base_fetch_repository = Some(repository);
            }
        }
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
        // The fake's temper-conflicts entries stand for the unmerged index,
        // independent of the bounded list returned to the checkout domain.
        let conflicts = self.disk.tree(&[repository.as_slice(), b"/.git/temper-conflicts"].concat());
        let unmerged: Vec<_> = conflicts.values().cloned().collect();
        let marked = !unmerged.is_empty()
            && unmerged.iter().all(|path| {
                self.disk.content(&[repository.as_slice(), b"/", path].concat()).is_some_and(|bytes| {
                    bytes.starts_with(b"<<<<<<<")
                        && bytes.windows(b"head\n".len()).any(|part| part == b"head\n")
                        && bytes.windows(b"base\n".len()).any(|part| part == b"base\n")
                })
            });
        let merge_head = self.disk.content(&[repository.as_slice(), b"/.git/MERGE_HEAD"].concat()).map(<[u8]>::to_vec);
        let request = args.get(2).and_then(|path| self.disk.content(&disk_path(path))).unwrap_or_default().to_vec();
        let story = self.merge_job.as_mut().expect("merge story");
        let mut staged = Vec::new();
        if found && story.row != Row::Left {
            for (entry, path) in &conflicts {
                self.disk.write(&[repository.as_slice(), b"/", path].concat(), b"settled\n");
                staged.extend_from_slice(path);
                staged.extend_from_slice(b"\0settled\n");
                self.disk.remove(&[repository.as_slice(), b"/.git/temper-conflicts/", entry].concat());
            }
            if !staged.is_empty() {
                self.disk.write(&[repository.as_slice(), b"/.git/index"].concat(), &staged);
            }
        }
        story.spawn = Some(Spawn {
            program: program.to_vec(),
            args: args.iter().map(|arg| arg.to_vec()).collect(),
            request,
            directory: directory.to_vec(),
            command_found: found,
            marked,
            conflicts: unmerged.clone(),
            staged: self.disk.content(&[repository.as_slice(), b"/.git/index"].concat()).unwrap_or_default().to_vec(),
            merge_head,
        });
        if found {
            let paths: Vec<_> =
                unmerged.iter().map(|path| core::str::from_utf8(path).expect("conflict path")).collect();
            let result = if story.row == Row::Left {
                LEFT.to_vec()
            } else {
                format!("{{\"settled\":{}}}", serde_json::to_string(&paths).expect("settled paths JSON")).into_bytes()
            };
            self.disk.write(&[dir.as_slice(), b"/result.json"].concat(), &result);
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

fn producer(files: &[Vec<u8>], content: &str) -> Script {
    let mut turns: Vec<_> = files
        .iter()
        .map(|path| {
            call(
                b"write",
                &serde_json::to_vec(&serde_json::json!({
                    "path": core::str::from_utf8(path).expect("file path"), "content": content,
                }))
                .expect("write JSON"),
            )
        })
        .collect();
    turns.push(call(b"finish", br#"{"title":"Edit files","body":"Keep the edit."}"#));
    Script { cue: b"Read only what the change needs".as_slice().into(), turns: turns.into_boxed_slice() }
}
