//! Rehearse the forge's half of Link over a private store image at a committed batch
//! (temper/docs/design/testing/store-check.md, 2–3).
use super::{Duration, Env, Map, Queue, REPO, StoreChecks, World, configured_forge, tasks, top};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_domain_forge_tiers as tiers;

pub(super) struct Kept {
    stored: Map<top::Key, top::Stored>,
    // Link reads task presence only; the scripted tasks can be rebuilt from their numbers.
    task_numbers: Box<[u64]>,
    config: RestoreConfig,
}

pub(super) struct RestoreConfig {
    env: Env<top::Limits>,
    engine_seed: u64,
    writer: Option<skein_forge_client::Writer>,
    landing: Option<(u32, change::Way, Box<[top::RepositoryGates]>)>,
    landing_policy: landing::Project,
    landing_batch: u16,
}

impl World {
    /// Start a new store-check run, discarding any previously kept batch.
    pub fn store_checks(&mut self, checks: StoreChecks) {
        self.store_checks = checks;
        self.kept_store = None;
        self.store_finished = false;
    }

    #[must_use]
    pub fn store_report(&self) -> String {
        self.store_checks.report()
    }

    #[must_use]
    pub fn store_counts(&self) -> (u64, u64) {
        (self.store_checks.commits(), self.store_checks.checked())
    }

    /// A scripted writer's output batch, including deliberate writer defects.
    pub fn commit(&mut self, requests: Box<[top::Request]>) -> Result<(), String> {
        assert!(self.top_out.is_empty(), "previous output batch drained");
        for request in requests {
            self.top_out.push(request);
        }
        self.try_take_top()
    }

    pub(super) fn restore_config(&self) -> RestoreConfig {
        RestoreConfig {
            env: self.env.with(self.env.limits),
            engine_seed: self.engine_seed,
            writer: self.writer,
            landing: self.landing_config.clone(),
            landing_policy: self.landing_policy,
            landing_batch: self.landing_batch,
        }
    }

    pub(super) fn check_commit(&mut self) -> Result<(), String> {
        assert!(!self.store_finished, "start a new store-check run to count more commits");
        self.commits = self.commits.checked_add(1).expect("world commit count fits u64");
        if self.store_checks.due(self.commits) {
            self.restore_config().check(&self.stored, &self.tasks, self.commits)?;
        } else if self.store_checks.keeps() {
            self.kept_store = Some(Kept {
                stored: self.stored.clone(),
                task_numbers: self.tasks.view_tasks().iter().map(|task| task.number).collect(),
                config: self.restore_config(),
            });
        }
        Ok(())
    }

    /// End this run's store checks, reporting a kept batch's refusal if any.
    pub fn finish(&mut self) -> Result<(), String> {
        if self.store_finished {
            return Ok(());
        }
        self.store_finished = true;
        let result = self.check_kept();
        eprintln!("{}", self.store_report());
        result
    }

    fn check_kept(&mut self) -> Result<(), String> {
        let Some(commit) = self.store_checks.end() else { return Ok(()) };
        let kept = self.kept_store.take().expect("selected commit has a store image");
        let mut tasks = tasks::domain(self.seed);
        for number in kept.task_numbers {
            tasks::hold(&mut tasks, kept.config.env.now, kept.config.env.wall, number);
        }
        kept.config.check(&kept.stored, &tasks, commit)
    }
}

impl RestoreConfig {
    pub(super) fn restored(&self, stored: &Map<top::Key, top::Stored>) -> (top::Domain, Queue<top::Request>) {
        let mut restored = top::Domain::new(
            &self.env.limits,
            self.engine_seed,
            top::Config { push: None, namespace: Box::from(&b"world"[..]), writers: self.writer.into_iter().collect() },
            &[top::GitIdentity { forge: REPO.forge, credential: 1 }],
            &[configured_forge()],
            false,
        )
        .expect("same configured connector");
        if let Some((project, way, gates)) = &self.landing {
            assert!(restored.project_landing(
                *project,
                *way,
                gates.clone(),
                self.landing_policy,
                top::AfterLanding {
                    wait: change::Wait::Cleared,
                    verdict: tiers::Settings { retries: 2, suspects: tiers::Suspects::Bisect },
                    bound: Duration::from_millis(7_200_000),
                },
            ));
            assert!(restored.project_batch(*project, self.landing_batch));
        }
        let mut output = Queue::uncapped();
        restored.restore_begin(self.env.wall);
        for (_, record) in stored {
            top::step(&mut restored, &self.env, top::Event::Restore { record: record.clone() }, &mut output);
        }
        top::step(
            &mut restored,
            &self.env,
            top::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
            &mut output,
        );
        (restored, output)
    }

    fn check(
        &self,
        stored: &Map<top::Key, top::Stored>,
        tasks: &jig_domain_tasks::Domain,
        commit: u64,
    ) -> Result<(), String> {
        let (mut restored, mut output) = self.restored(stored);
        output.clear();
        StoreChecks::rehearse(|| restored.start_links(tasks, &self.env, false, &mut output))
            .map_err(|refusal| jig_conformance::store_check::finding(commit, "Link", &format!("{refusal:?}")))
    }
}

impl Drop for World {
    fn drop(&mut self) {
        if self.store_finished {
            return;
        }
        if std::thread::panicking() {
            eprintln!("{}", self.store_report());
        } else if let Err(finding) = self.finish() {
            panic!("{finding}");
        }
    }
}
