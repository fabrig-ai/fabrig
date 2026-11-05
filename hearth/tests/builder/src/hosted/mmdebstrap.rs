//! Apt's pure resolver and printer, with its lists written through the kernel.
use super::{Hosted, Job, Tool};
use crate::fixtures::{Scenario, Shared};
use hearth_fake_git::face::{Read, Want};
use hearth_fake_mmdebstrap::{
    apt,
    face::{self, Outcome, Printed, Simulate},
};
use skein_io::kernel::Spawn;
use skein_lib::Duration;
use skein_world::Inherited;
use std::collections::{BTreeMap, VecDeque};
#[derive(Debug)]
pub struct Mmdebstrap {
    simulate: Simulate,
    printed: Option<Printed>,
    per_package: Duration,
}
impl Mmdebstrap {
    #[must_use]
    pub fn host(spawn: &Spawn, inherited: &Inherited, scenario: &Scenario<'_>, shared: Shared, seed: u64) -> Hosted {
        let simulate = face::check(&spawn.args, &spawn.env);
        let jobs = VecDeque::from([
            Job::Verify(spawn.program.clone()),
            Job::Read([simulate.scratch.as_ref(), b"/sources.list"].concat().into()),
        ]);
        let delay = scenario.pace.mmdebstrap(&mut super::rng(seed, spawn), 0);
        Hosted::new(
            Tool::Mmdebstrap(Self { simulate, printed: None, per_package: scenario.pace.per_package }),
            shared,
            inherited,
            jobs,
            delay,
            scenario.fault,
        )
    }
    pub(super) fn prepare(
        &mut self,
        shared: &Shared,
        files: &mut BTreeMap<Box<[u8]>, Read>,
        jobs: &mut VecDeque<Job>,
    ) -> Duration {
        let sources_path = [self.simulate.scratch.as_ref(), b"/sources.list"].concat();
        let Read::Contents { bytes, .. } = files.remove(sources_path.as_slice()).expect("sources read") else {
            panic!("sources readable")
        };
        let sources = face::sources(&bytes);
        let (lists, outcome) = {
            let fake = shared.borrow();
            apt::pass(fake.behind.machine().reach(self.simulate.proxy), &self.simulate, &sources)
        };
        let fetched = lists.iter().map(|(_, bytes)| u64::try_from(bytes.len()).expect("list length")).sum::<u64>();
        shared.borrow_mut().served_bytes += fetched;
        let target = [self.simulate.scratch.as_ref(), b"/root"].concat();
        for suffix in [
            b"".as_slice(),
            b"/var",
            b"/var/lib",
            b"/var/lib/apt",
            b"/var/lib/apt/lists",
            b"/var/lib/apt/lists/partial",
        ] {
            jobs.push_back(Job::Make([target.as_slice(), suffix].concat().into()));
        }
        let directory = [target.as_slice(), b"/var/lib/apt/lists/"].concat();
        jobs.push_back(Job::Write([directory.as_slice(), b"lock"].concat().into(), Box::default()));
        if !matches!(outcome, Outcome::Resolved(_)) {
            jobs.push_back(Job::Make([directory.as_slice(), b"auxfiles"].concat().into()));
        }
        for (name, bytes) in lists {
            jobs.push_back(Job::Write([directory.as_slice(), &name].concat().into(), bytes));
        }
        let count = match &outcome {
            Outcome::Resolved(packages) => packages.len(),
            Outcome::Unreachable | Outcome::NotFound { .. } | Outcome::Failed { .. } => 0,
        };
        let mut printed = face::print(&self.simulate, &sources, &outcome);
        printed.stderr = [format!("Fetched {fetched} B in 0s\n").as_bytes(), &printed.stderr].concat().into();
        self.printed = Some(printed);
        Duration::from_nanos(self.per_package.as_nanos().saturating_mul(u64::try_from(count).expect("package count")))
    }
    pub(super) fn next(&mut self) -> Want {
        let printed = self.printed.as_mut().expect("prepared mmdebstrap");
        if !printed.stdout.is_empty() {
            return Want::Stdout(std::mem::take(&mut printed.stdout));
        }
        if !printed.stderr.is_empty() {
            return Want::Stderr(std::mem::take(&mut printed.stderr));
        }
        Want::Exit(printed.exit)
    }
}
