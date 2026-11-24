//! Drives the same harness turn on the ring and in a world (skein/docs/design/ui/web/testing.md, 2).
use super::{
    Account, BrowserEnd, BrowserFace, Complete, Duration, Harness, Lent, Outcome, POLL, Play, Queue, ROOT, Request,
    Runtime, SERVER_ROOT, Scripted, Starved, Submit, Time, Wall, bounds, capacity, kernel, peers, person, pidfd_pid,
};
use skein_shell::{Clock, Config, Kernel, Wait, open_root};
use skein_world::{Host, ring::State};
use std::{os::unix::ffi::OsStrExt, path::Path};

const PEER_STARTUP: Duration = Duration::from_secs(36);

pub(super) enum BrowserStart<'a> {
    Chromium { profile: &'a Path, root: kernel::Fd, browser_root: kernel::Fd },
    Scripted(Scripted),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Lending,
    Opening,
    Playing,
}

/// One browser harness run. The test keeps its scripts and peers after it ends.
pub struct Hosted<'a, 'p, 'l> {
    setup: &'a mut Harness,
    people: &'a mut [&'p mut dyn Play],
    peers: peers::Peers<'a, 'l>,
    runtime: Runtime,
    operations: State,
    limit: u32,
    startup: Option<Account>,
    horizon: Option<Time>,
    next_poll: Time,
    classified: bool,
    faces: Vec<BrowserFace<'static>>,
    done: Vec<bool>,
    stage: Stage,
    opening_page: bool,
    ending: bool,
}

impl<'a, 'p, 'l> Hosted<'a, 'p, 'l> {
    pub(super) fn new(
        setup: &'a mut Harness,
        people: &'a mut [&'p mut dyn Play],
        lent: &'a mut [Lent<'l>],
        browser: BrowserStart<'_>,
    ) -> Self {
        setup.kept.clear();
        let peers = peers::Peers::new(lent);
        let (runtime, local_limit) = Runtime::new(setup, people, browser);
        let mut limits = vec![local_limit, capacity::ROOT_OPERATIONS];
        limits.extend_from_slice(&peers.limits);
        let limit =
            limits.iter().try_fold(0_u32, |total, limit| total.checked_add(*limit)).expect("shared ring capacity fits");
        let done = vec![false; people.len()];
        Self {
            setup,
            people,
            peers,
            runtime,
            operations: State::new(limit, limits),
            limit,
            startup: None,
            horizon: None,
            next_poll: Time::ZERO,
            classified: false,
            faces: Vec::new(),
            done,
            stage: Stage::Lending,
            opening_page: false,
            ending: false,
        }
    }

    /// Adopts the launch root the world supplied to this process. The run closes it.
    #[must_use]
    pub fn at_root(mut self, root: kernel::Fd) -> Self {
        assert!(self.horizon.is_none() && self.runtime.roots[0].is_none(), "one launch root before the first turn");
        self.runtime.roots[0] = Some(root);
        self
    }

    /// Publishes completed keeps and returns the verdict after the world settles the run.
    #[must_use]
    pub fn finish(self) -> Outcome {
        assert!(self.is_empty(), "the world has run the harness to its exit");
        let (outcome, starved) = self.publish();
        assert!(starved.is_none(), "simulated runs do not classify scheduler starvation");
        outcome
    }

    fn publish(self) -> (Outcome, Option<Starved>) {
        let (outcome, kept, starved) = self.runtime.report();
        self.setup.kept = kept;
        (outcome, starved)
    }

    fn reap(&mut self) {
        while let Some(complete) = self.operations.completions().pop() {
            let (host, complete) = self.operations.complete(complete);
            if host >= peers::FIRST
                && let Some(report) = self.peers.completion(host, &complete)
            {
                self.runtime.fail(0, 0, report);
            }
            self.operations.returned(host, complete);
        }
        let held = self.runtime.complete.len();
        self.operations.deliver(0, &mut self.runtime.complete);
        // Measure only real children, and in the order io sees their spawns.
        if self.runtime.native() {
            for complete in self.runtime.complete.iter().skip(usize::try_from(held).expect("queue length fits usize")) {
                if let Ok(kernel::Done::Spawned { pidfd, .. }) = &complete.result {
                    self.runtime.spawned_pids.push_back(pidfd_pid(*pidfd));
                }
            }
        }
        let mut roots = Queue::with_capacity(capacity::ROOT_OPERATIONS);
        self.operations.deliver(1, &mut roots);
        while let Some(complete) = roots.pop() {
            if let Err(error) = complete.result {
                self.runtime.fail(0, 0, format!("root close: {error:?}"));
            }
        }
    }

    fn arm(&mut self, now: Time) {
        assert!(self.runtime.roots[0].is_some(), "the run needs its launch root");
        let native = self.runtime.native();
        self.startup = Some(Account::arm(native, "the lent peers' startup", PEER_STARTUP, now));
        // Reserve io's close window inside the run's horizon.
        let (close_patience, patience) = bounds(self.setup.horizon);
        let close_at = Account::arm(native, "the browser harness run", close_patience, now);
        self.horizon = Some(
            close_at
                .until()
                .saturating_add(Duration::from_nanos(patience.as_nanos().saturating_sub(close_patience.as_nanos()))),
        );
        self.runtime.deadline = Some(close_at);
    }

    fn check_run_deadlines(&mut self, now: Time) {
        let run = &mut self.runtime;
        if self.stage == Stage::Lending
            && run.outcome.is_none()
            && let Some(startup) = &self.startup
            && now >= startup.until()
        {
            let report = format!("lent peer startup deadline: {}", self.peers.waiting());
            run.expire(0, 0, &report, startup.expired(now));
            self.classified = true;
        }
        if now >= run.deadline.as_ref().expect("run deadline armed").until() {
            let report = if self.stage == Stage::Lending {
                format!("lent peer startup deadline: {}", self.peers.waiting())
            } else {
                run.stuck(
                    self.people,
                    &self.faces,
                    &self.done,
                    self.stage == Stage::Playing,
                    self.peers.empty(),
                    self.operations.is_empty(),
                )
            };
            run.close_point(&report, &mut self.classified);
        }
    }

    fn play(&mut self, now: Time) {
        let run = &mut self.runtime;
        if self.stage == Stage::Opening
            && run.outcome.is_none()
            && !run.closing.active()
            && run.ready
            && run.address.is_some()
            && run.origins.iter().all(|pages| pages.addr.is_some())
        {
            self.stage = Stage::Playing;
            self.faces = run.faces(self.setup, self.people.len());
            if self.people.is_empty() {
                run.outcome = Some(Outcome::Done);
            } else {
                run.ask(Request::Person {
                    person: person(0),
                    downloads: self.setup.downloads.as_ref().map(|path| path.as_os_str().as_bytes().into()),
                });
            }
        }
        if self.stage == Stage::Playing && run.outcome.is_none() {
            self.opening_page |= run.open_page(self.setup, &mut self.faces, self.opening_page);
            run.play(self.people, &mut self.faces, &mut self.done, now, self.setup);
        }
    }
}

impl Host for Hosted<'_, '_, '_> {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if self.horizon.is_none() {
            self.arm(now);
        }
        self.reap();
        self.next_poll = now.saturating_add(POLL);
        let run = &mut self.runtime;
        run.benv.now = now;
        run.benv.wall = wall;
        run.ienv.now = now;
        run.ienv.wall = wall;
        let stop_peers = run.closing.active()
            && run.closed == run.spawned
            && run.io.is_empty()
            && run.requests.is_empty()
            && run.origins.iter().all(crate::origins::Pages::is_closed);
        self.peers.drive_with(&mut self.operations, now, wall, stop_peers, State::deliver, submit);
        if self.stage == Stage::Lending && self.peers.ready() && run.outcome.is_none() {
            self.stage = Stage::Opening;
            self.startup = None;
            run.start(self.setup, &self.peers);
        }
        // Observe answers and bad exits before classifying an unanswered wait.
        if run.tick(self.setup, &mut self.faces, self.people, now) {
            self.opening_page = false;
        }
        run.judge_steps(self.setup, self.people, &mut self.faces, &self.done, now);
        self.check_run_deadlines(now);
        self.play(now);
        let run = &mut self.runtime;
        run.capture_or_close(self.opening_page, now);
        self.ending = run.close_window_passed(now) || now >= self.horizon.expect("run deadline armed");
        run.flush(self.stage != Stage::Lending, now, wall);
        while let Some(record) = run.submits.pop() {
            let host = usize::from(record.op == ROOT || record.op == SERVER_ROOT);
            submit(&mut self.operations, host, record);
        }
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        self.operations.completions()
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.operations.submissions()
    }

    fn work_pending(&self, _now: Time) -> bool {
        let run = &self.runtime;
        self.horizon.is_none()
            || run.io.is_ready()
            || !run.complete.is_empty()
            || !run.io_events.is_empty()
            || !run.events.is_empty()
            || !run.down.is_empty()
            || !run.requests.is_empty()
            || matches!(&run.browser, BrowserEnd::Chromium(browser) if browser.work_pending())
            || self.peers.pending()
            || self.operations.has_ready()
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        let run = &self.runtime;
        run.io
            .next_deadline()
            .into_iter()
            .chain(
                // Play scripts expose checks, not all of their wakeups; poll while they run.
                (run.native() || (self.stage == Stage::Playing && run.outcome.is_none())).then_some(self.next_poll),
            )
            .min()
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        let run = &self.runtime;
        let now = run.benv.now;
        let browser = match &run.browser {
            BrowserEnd::Chromium(browser) => browser.next_deadline(),
            BrowserEnd::Scripted(browser) => browser.next_deadline(),
        };
        let commands =
            run.commands.iter().flatten().filter_map(|command| command.wait.as_ref().map(|wait| wait.0.until()));
        let close = run.close_window.as_ref().and_then(|window| window.next(now));
        let capture = (!run.closing.active()).then(|| run.capture.as_ref().map(super::Capture::until)).flatten();
        run.deadline
            .as_ref()
            .map(Account::until)
            .filter(|at| *at > now)
            .into_iter()
            .chain(self.startup.as_ref().map(Account::until).filter(|at| *at > now))
            .chain(browser)
            .chain(close)
            .chain(capture)
            .chain(commands)
            .chain(self.peers.deadline())
            .min()
    }

    fn is_empty(&self) -> bool {
        let run = &self.runtime;
        run.closing.active()
            && run.closed == run.spawned
            && run.origins.iter().all(crate::origins::Pages::is_closed)
            && run.io.is_empty()
            && run.complete.is_empty()
            && run.io_events.is_empty()
            && run.events.is_empty()
            && run.down.is_empty()
            && run.requests.is_empty()
            && run.submits.is_empty()
            && self.operations.is_empty()
            && self.peers.empty()
    }

    fn exit(&self) -> Option<kernel::Exit> {
        self.is_empty()
            .then(|| kernel::Exit::Code(u8::from(matches!(self.runtime.outcome, Some(Outcome::Failed { .. })))))
    }

    fn operations(&self) -> u32 {
        self.limit
    }
}

fn submit(operations: &mut State, host: usize, record: Submit) {
    let record = operations.register(host, record);
    operations.queue(record);
}

pub(super) fn native(setup: &mut Harness, people: &mut [&mut dyn Play], peers: &mut [Lent<'_>]) -> Outcome {
    let profile = skein_scratch::Scratch::new("ui-browser");
    let root = open_root(Path::new("/")).expect("server root");
    let browser_root = open_root(Path::new("/")).expect("child root");
    let mut run =
        Hosted::new(setup, people, peers, BrowserStart::Chromium { profile: profile.path(), root, browser_root });
    let mut kernel =
        Kernel::open(Config { operations: run.operations() }).expect("io_uring is required for the browser suite");
    let clock = Clock::new();
    loop {
        kernel.reap(run.completions());
        let stamp = clock.now();
        run.iterate(stamp.now, stamp.wall);
        if run.is_empty() {
            break;
        }
        let wait = if run.work_pending(stamp.now) || run.ending {
            Wait::No
        } else {
            Wait::Until(run.next_deadline().expect("the real harness polls each turn"))
        };
        let submissions = run.submissions();
        kernel.submit(submissions, wait);
        assert!(submissions.is_empty(), "the ring is provisioned for every host and cleanup");
        if run.ending {
            break;
        }
    }
    let (outcome, starved) = run.publish();
    drop(kernel);
    drop(profile);
    if !matches!(outcome, Outcome::Failed { .. })
        && let Some(starved) = starved
    {
        starved.pass();
    }
    outcome
}
