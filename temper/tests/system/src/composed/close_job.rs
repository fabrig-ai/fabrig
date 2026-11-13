//! A close change's job through both roots (temper/docs/design/domain/worker/jobs.md;
//! temper/docs/design/protocol/tooling.md, 4.2).
use super::{Duration, Script, Token, World, call, engine, host, link, tasks, worker};
use jig_domain_link::JobBound;
use temper_fake_checkout::git::Remote;
use temper_worker_domain::tooling;

const REMOTE: &[u8] = b"org/repo";
const COMMAND: &[u8] = b"world close command\n";
const REQUEST: &[u8] = br#"{"version":1,"goal":1,"changes":[{"task":9,"landed":true}]}"#;
const STEPS: u32 = 10000;

/// Whether the command stages the status file (temper/docs/design/protocol/tooling.md, 4.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Settled,
    Unchanged,
}

pub(super) struct Story {
    row: Row,
    job: Option<u64>,
    hello: Vec<JobBound>,
    pushes: u32,
    tip: Option<u64>,
    spawn: Option<Spawn>,
}

#[derive(Debug)]
struct Spawn {
    program: Vec<u8>,
    args: Vec<Vec<u8>>,
    request: Vec<u8>,
    directory: Vec<u8>,
}

/// The durable change and the worker's delivery, read from the store and fake forge.
#[derive(Debug)]
pub struct CloseJobReport {
    pub hello: Vec<JobBound>,
    pub tip: u64,
    pub head: Option<u64>,
    pub parents: Vec<u64>,
    pub message: Vec<u8>,
    pub tree: temper_fake_checkout::git::Tree,
    pub pushes: u32,
    pub opened: bool,
    pub no_change: bool,
    pub job_result: tasks::TaskResult,
    pub program: Vec<u8>,
    pub args: Vec<Vec<u8>>,
    pub request: Vec<u8>,
    pub directory: Vec<u8>,
}

pub(super) fn worker_kinds(declared: &[JobBound], limits: &mut worker::Limits) -> Box<[tooling::Kind]> {
    let kinds: Box<[_]> = temper_worker_service::worker_settings::job_kinds()
        .into_vec()
        .into_iter()
        .filter(|kind| declared.contains(&kind.bound))
        .collect();
    for kind in &kinds {
        limits.tooling.file_bytes = limits.tooling.file_bytes.max(kind.bound.spec).max(kind.bound.result);
    }
    kinds
}

fn config(seed: u64) -> engine::Config {
    let mut config = super::agreement::config(seed);
    let mut jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    jig.core.tasks = temper_engine_domain_world::forge_routes::box_limits().tasks;
    config.core.jobs = temper_engine_service::limits::jobs(&jig);
    assert!(config.core.jobs.contains(&temper_engine_service::limits::close_bound()));
    let kinds = &mut config.core.defaults.rules.ceiling.delegation.kinds;
    let mut extended = kinds.to_vec();
    let close = jig_domain_authority::Executor::Job(temper_engine_domain_forge::CLOSE_KIND);
    if !extended.contains(&close) {
        extended.push(close);
    }
    *kinds = extended.into_boxed_slice();
    for project in &mut config.core.seed_projects {
        project.policy.policy.ceiling.delegation.kinds.clone_from(kinds);
        for role in &mut project.policy.policy.roles {
            role.authority.delegation.kinds.clone_from(kinds);
        }
    }
    let command = Some(b".temper/close".as_slice().into());
    if let Some(landing) = config.landings.get_mut(&1) {
        landing.close_job = command;
    } else {
        config
            .landings
            .insert(
                1,
                engine::ProjectLanding {
                    gate_retries: 2,
                    batch: 1,
                    combine_main: engine::Combine::Merge,
                    combine_other: engine::Combine::Squash,
                    gate_timeout: skein_lib::Duration::from_millis(3_600_000),
                    landing_attempts: engine::LANDING_ATTEMPTS,
                    check_timeout: None,
                    by: engine::Way::Queue,
                    close_job: command,
                    merge_job: None,
                    repositories: Box::new([]),
                    after: engine::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 },
                },
            )
            .expect("project landing room");
    }
    config
}

impl World {
    /// Make a close change directly with bytes parameter 17, into the goal's branch.
    #[must_use]
    pub fn close_job(seed: u64, row: Row) -> Self {
        let goal = Script {
            cue: b"Help the person with their work.".as_slice().into(),
            turns: Box::new([
                call(b"change", br#"{"words":"Settle goal 1's status.","landing":"goal"}"#),
                call(b"wait", b"{}"),
            ]),
        };
        let mut world = Self::configured_with(seed, goal.clone(), true, config);
        world.goal_branch = true;
        world.scripts.push_back(goal);
        world.forge.advance(REMOTE, b"main", b".temper/close", COMMAND);
        world.forge.advance(REMOTE, b"main", b"status.md", b"in flight\n");
        world.close_job = Some(Story { row, job: None, hello: Vec::new(), pushes: 0, tip: None, spawn: None });
        world
    }

    // Supply the direct procedure delegation, including its close request parameter.
    pub(super) fn decode_close_call(
        &mut self,
        to: skein_lib::ReplyTo,
        call: &jig_domain_fleet::Call,
    ) -> engine::WorkerInput {
        if self.close_job.is_none() || call.tool.as_ref() != b"change" {
            return super::review::decode(to, call);
        }
        let mut body = link::decode(call);
        let goal = self.assignment();
        let key = engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(goal.task)));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(live)))) =
            self.store.rows.get(&key)
        else {
            panic!("goal task live")
        };
        let mut delegate =
            temper_engine_domain_world::fixtures::report_delegate(b"Settle goal 1's status.", Box::new([]));
        delegate.executor = tasks::Executor::Procedure { connector: 1, code: 2 };
        delegate.contract = tasks::Contract::Change { connector: 1, kind: 1, words: 256 };
        delegate.authority = live.authority.clone();
        delegate.authority.delegation.depth = delegate.authority.delegation.depth.saturating_sub(1);
        delegate.authority.delegation.tasks = 6;
        delegate.authority.budget.spend = 40;
        let branch = format!("temper/{0}/g{0}", goal.task).into_bytes();
        let tip = self.forge.branch(REMOTE, &branch).expect("goal branch exists before close");
        delegate.spec.parameters = Box::new([
            tasks::Parameter::Resource {
                name: 1,
                connector: 1,
                resource: temper_engine_domain_forge::tag(
                    temper_engine_forge_world::REPO.forge,
                    temper_engine_forge_world::REPO.repository,
                ),
            },
            tasks::Parameter::Bytes { name: 2, value: branch.into() },
            tasks::Parameter::Bytes { name: 17, value: REQUEST.into() },
        ]);
        self.close_job.as_mut().expect("close story").tip = Some(tip);
        body.tool = engine::Tool::Delegate { batch: Box::new([delegate]) };
        engine::WorkerInput::DecodedHostCall { to, body }
    }

    /// Stop once the changed branch opens, or the unchanged close ends durably.
    pub fn run_close_job(&mut self) -> CloseJobReport {
        for _ in 0..STEPS {
            self.tick();
            if let Some(change) = self.close_change()
                && (change.pull.is_some() || self.close_ended_without_change(change.task))
                && self.close_job_result().is_some()
            {
                break;
            }
        }
        let change = self.close_change().unwrap_or_else(|| panic!("close change made: {:?}", self.observed));
        let opened = change.pull.is_some();
        let no_change = self.close_ended_without_change(change.task);
        let head = self.forge.branch(REMOTE, &change.branch);
        let parents = head
            .into_iter()
            .flat_map(|head| [self.forge.parent(head), self.forge.merge_parent(head)].into_iter().flatten())
            .collect();
        let tree = head.map(|head| self.forge.tree(head)).unwrap_or_default();
        let story = self.close_job.as_ref().expect("close story");
        let spawn = story.spawn.as_ref().unwrap_or_else(|| panic!("close command spawned: {change:?}"));
        CloseJobReport {
            hello: story.hello.clone(),
            tip: story.tip.expect("goal tip"),
            head,
            parents,
            tree,
            message: head.map(|head| self.forge.message(head)).unwrap_or_default(),
            pushes: story.pushes,
            opened,
            no_change,
            job_result: self.close_job_result().unwrap_or_else(|| panic!("job finished: {change:?}")),
            program: spawn.program.clone(),
            args: spawn.args.clone(),
            request: spawn.request.clone(),
            directory: spawn.directory.clone(),
        }
    }

    fn close_change(&self) -> Option<&temper_engine_domain_forge::ChangeRow> {
        self.store.rows.values().find_map(|row| match row {
            engine::Record::Forge { row } => match row.as_ref() {
                temper_engine_domain_forge::Stored::Change(row) if row.close.is_some() => Some(row.as_ref()),
                _ => None,
            },
            engine::Record::Core(_) => None,
        })
    }

    fn close_ended_without_change(&self, task: u64) -> bool {
        let key = engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
            self.store.rows.get(&key)
        else {
            return false;
        };
        matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::NoChange { words })) if words.as_ref() == b"Nothing to settle.")
    }

    fn close_job_result(&self) -> Option<tasks::TaskResult> {
        let task = self.close_job.as_ref()?.job?;
        [tasks::Key::Ended(task), tasks::Key::Live(task)].into_iter().find_map(|key| {
            let key = engine::Key::Core(jig_domain_core::Key::Tasks(key));
            match self.store.rows.get(&key)? {
                engine::Record::Core(jig_domain_core::Record::Tasks(
                    tasks::Stored::Ended(row) | tasks::Stored::Live(row),
                )) => {
                    if let tasks::Phase::Ended(tasks::Ending::Done(result)) = &row.phase {
                        Some(result.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
    }

    pub(super) fn close_hello(&mut self, hello: &jig_domain_link::Hello) {
        if let Some(story) = &mut self.close_job {
            story.hello = hello.jobs.to_vec();
        }
    }

    pub(super) fn assign_story_job(
        &mut self,
        channel: Token,
        job: host::JobAssignment,
        workspace: Option<&engine::ForgeWorkspace>,
    ) {
        let Some(story) = &mut self.close_job else { return self.assigned_job(channel, job, workspace) };
        assert_eq!(channel, super::CHANNEL);
        assert_eq!(job.kind.0, temper_engine_domain_forge::CLOSE_KIND);
        assert_eq!(job.deadline, Duration::from_secs(600));
        story.job = Some(job.run.raw());
        let own = temper_service_core::limits::profile(temper_service_core::limits::Profile::Small);
        let limits = temper_worker_service::worker_settings::payload_limits(
            &temper_worker_service::worker_settings::Limits::shared(own),
        )
        .expect("worker payload limits");
        self.worker_events.push_back(link::job(job, workspace, &limits));
    }

    pub(super) fn observe_close_io(&mut self, op: &worker::checkout::git::Op) {
        if let Some(story) = &mut self.close_job
            && matches!(op, worker::checkout::git::Op::Push { .. })
        {
            story.pushes += 1;
        }
    }

    pub(super) fn story_job_operation(&mut self, owner: Token, op: tooling::Request) {
        if self.close_job.is_none() {
            return self.job_operation(owner, op);
        }
        if let tooling::Request::Spawn { owner: inner, program, args, dir, .. } = op {
            assert_eq!(owner, inner);
            self.play_close_command(owner, &program, &args, &dir);
        } else {
            self.job_operation(owner, op);
        }
    }

    fn play_close_command(&mut self, owner: Token, program: &[u8], args: &[Box<[u8]>], directory: &[u8]) {
        assert_eq!(self.disk.content(&disk_path(program)), Some(COMMAND), "command read from goal's tree");
        let dir = disk_path(directory);
        let repo = [dir.as_slice(), b"/repo"].concat();
        assert_eq!(self.disk.content(&[repo.as_slice(), b"/status.md"].concat()), Some(b"in flight\n".as_slice()));
        assert!(!self.disk.exists(&[repo.as_slice(), b"/.git/MERGE_HEAD"].concat()));
        let request = self.disk.content(&disk_path(&args[2])).expect("request written").to_vec();
        let story = self.close_job.as_mut().expect("close story");
        let result = if story.row == Row::Settled {
            self.disk.write(&[repo.as_slice(), b"/status.md"].concat(), b"settled\n");
            self.disk.write(&[repo.as_slice(), b"/.git/index"].concat(), b"status.md\0settled\n");
            br#"{"settled":["status.md"]}"#.as_slice()
        } else {
            br#"{"settled":[]}"#.as_slice()
        };
        self.disk.write(&[dir.as_slice(), b"/result.json"].concat(), result);
        story.spawn = Some(Spawn {
            program: program.to_vec(),
            args: args.iter().map(|arg| arg.to_vec()).collect(),
            request,
            directory: directory.to_vec(),
        });
        for event in [
            tooling::Event::Spawned { owner },
            tooling::Event::Exited { owner, exit: tooling::Exit::Code(0), tail: Box::new([]) },
            tooling::Event::Gone { owner },
        ] {
            self.worker_events.push_back(worker::Event::Job { owner, event });
        }
    }
}

fn disk_path(path: &[u8]) -> Vec<u8> {
    [b"ws/".as_slice(), path.strip_prefix(b"workspaces/").expect("job workspace path")].concat()
}
