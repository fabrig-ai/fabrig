//! Cold starts over durable copies (`jig/docs/design/domain/testing-fakes.md`, 1).
use super::{Cut, Env, Queue, Rc, StoreChecks, StoreOut, Up, VecDeque, World, root};
use jig_conformance::store_check::finding;
use jig_test_store_files::StoreImage;

#[derive(Debug)]
pub(super) struct Rehearsal {
    image: StoreImage,
    config: Rc<root::Config>,
    limits: root::Limits,
    startup: Option<fn(&mut root::Domain)>,
    env: Env<root::Limits>,
    fill: u64,
}

impl Rehearsal {
    fn run(self, commit: u64) -> Result<(), String> {
        let mut store = self.image.open();
        let config = crate::oracle::configuration(&self.config);
        let mut domain = root::Domain::new(config, &self.limits);
        if let Some(setup) = self.startup {
            setup(&mut domain);
        }
        let env = self.env;
        StoreChecks::rehearse(|| {
            domain.rehearse();
            let mut events = VecDeque::from([root::Event::StartBegin { pending: store.generation_pending }]);
            let mut out = Queue::with_capacity(128);
            loop {
                if let Some(error) = refusal(&domain, commit) {
                    return Err(error);
                }
                if domain.rehearsed() {
                    return Ok(());
                }
                if !sweep_waits(&domain, events.front())
                    && let Some(event) = events.pop_front()
                {
                    root::step(&mut domain, &env, event);
                }
                root::release(&mut domain, &env, &mut out);
                while let Some(request) = out.pop() {
                    match request {
                        root::Request::Start { phase, census } => store.ask(StoreOut::Start { phase, census }),
                        root::Request::Commit { number, mut writes } => {
                            let mut rows = Vec::new();
                            while let Some(write) = writes.pop() {
                                rows.push(write);
                            }
                            store.submit(number, rows.into_boxed_slice(), 0);
                        }
                        // Rehearsal never serves runtime neighbours or external effects.
                        _ => {}
                    }
                }
                match store.tick(false) {
                    Ok(Some(number)) => events.push_front(root::Event::Committed { number, fill: self.fill }),
                    Ok(None) => {}
                    Err(number) => events.push_back(root::Event::Failed { number }),
                }
                for answer in store.answers() {
                    match answer {
                        Up::Root(event) => events.push_back(event),
                        Up::Ask(_) => {
                            return Err(format!("store check: commit {commit}: unexpected page continuation"));
                        }
                    }
                }
                if events.is_empty() && store.pending.is_empty() && !store.loading() {
                    if let Some(error) = refusal(&domain, commit) {
                        return Err(error);
                    }
                    if domain.rehearsed() {
                        return Ok(());
                    }
                    let phase = domain.start_phase().map_or_else(|| "None".into(), |phase| format!("{phase:?}"));
                    return Err(format!("store check: commit {commit}: start stalled at {phase}"));
                }
            }
        })
    }
}

impl World {
    /// Choose the commits checked by subsequent durable writes, starting a new count.
    pub fn store_checks(&mut self, checks: StoreChecks) {
        self.store_checks = checks;
        self.kept_check = None;
        self.store_checks_reported = false;
    }

    /// Rehearse a cold start without touching the live root, store, clock or peers.
    pub fn check_store(&self) -> Result<(), String> {
        self.rehearsal().run(self.store.applied)
    }

    fn rehearsal(&self) -> Rehearsal {
        Rehearsal {
            image: self.store.image(),
            config: Rc::clone(&self.restart_config.0),
            limits: self.restart_config.1,
            startup: self.startup,
            env: self.env(),
            fill: self.fill,
        }
    }

    pub(super) fn check_durable(&mut self) -> Result<(), String> {
        let number = self.store.applied;
        // A selected durable cut is counted by its child's successful exit instead.
        if self.sweep.contains(&Cut::Durable(number)) {
            return Ok(());
        }
        if self.store_checks.due(number) {
            self.check_store()?;
        } else if self.store_checks.keeps() {
            self.kept_check = Some(self.rehearsal());
        }
        Ok(())
    }

    pub(crate) fn finish_store_check(&mut self) -> Result<(), String> {
        if let Some(commit) = self.store_checks.end() {
            let check = self.kept_check.take().expect("the selected commit's store was kept");
            check.run(commit)?;
        }
        Ok(())
    }

    pub(super) fn check_restarted_child(&mut self) -> Result<(), String> {
        if let Some((commit, true)) = self.restart_check {
            if let Some(error) = refusal(&self.domain, commit) {
                self.restart_check = None;
                self.stopped = true;
                eprintln!("{error}");
                return Err(error);
            }
            if self.domain.ready() {
                self.restart_check = None;
                self.store_checks.restarted(commit);
            }
        }
        Ok(())
    }
}

/// Sweep pages wait for writes from their predecessor to release its continuation.
pub(super) fn sweep_waits(domain: &root::Domain, event: Option<&root::Event>) -> bool {
    domain.start_phase() == Some(super::core::start::Phase::Sweep)
        && !domain.quiescent()
        && event.is_some_and(|event| {
            matches!(event, root::Event::StartRecord(_) | root::Event::StartDone(super::core::start::Phase::Sweep))
        })
}

fn refusal(domain: &root::Domain, commit: u64) -> Option<String> {
    if let Some((phase, failure)) = domain.start_page_failure() {
        return Some(finding(commit, &format!("{phase:?}"), &format!("{failure:?}")));
    }
    domain.start_failure().map(|(phase, failure)| {
        let phase = phase.map_or_else(|| "None".into(), |phase| format!("{phase:?}"));
        finding(commit, &phase, &format!("{failure:?}"))
    })
}
