//! Scripted files, executable checks and their outstanding terminal rights.
//! Contract: smith/docs/design/domain/run.md, sections 8.1 and 13.

use std::collections::{BTreeMap, BTreeSet};

use skein_lib::{Rng, Time, Token};
use skein_world::domain::Key;
use smith_domain_run as run;

use super::{Checkouts, Delivery, World};

#[derive(Default)]
pub(super) struct Io {
    pub(super) files: BTreeMap<(Token, Vec<u8>), Vec<u8>>,
    pub(super) executables: BTreeSet<(Token, Vec<u8>)>,
    pub(super) failures: BTreeMap<Token, u64>,
    pub(super) looks: BTreeSet<Token>,
    pub(super) checking: BTreeMap<Token, Token>,
    pub(super) checks: BTreeMap<Token, Key>,
    pub(super) passed: BTreeMap<Token, BTreeSet<Token>>,
}

impl Io {
    fn read(&self, at: &run::Place, max: u32) -> run::Read {
        match self.files.get(&(at.root, at.path.to_vec())) {
            // io reads text, cut where a character ends.
            Some(content) => match std::str::from_utf8(content) {
                Ok(text) => {
                    let mut cut = text.len().min(usize::try_from(max).expect("a u32 fits"));
                    while !text.is_char_boundary(cut) {
                        cut -= 1;
                    }
                    run::Read::Text { text: text.as_bytes()[..cut].into(), whole: cut == text.len() }
                }
                Err(_) => run::Read::NotText,
            },
            None => run::Read::Missing,
        }
    }

    pub(super) fn complete(&mut self, event: &run::Event) -> bool {
        match event {
            run::Event::Read { owner, .. } | run::Event::Probed { owner, .. } => self.looks.remove(owner),
            run::Event::Checked { owner, ran } => {
                self.checks.remove(owner);
                let root = self.checking.remove(owner);
                if let Some(root) = root
                    && matches!(ran.exit, run::Exit::Code { code: 0 })
                {
                    self.passed.entry(*owner).or_default().insert(root);
                }
                root.is_some()
            }
            run::Event::Aborted { owner } => {
                self.checks.remove(owner);
                self.checking.remove(owner).is_some()
            }
            run::Event::HostReturned { .. }
            | run::Event::Message { .. }
            | run::Event::Acknowledge { .. }
            | run::Event::Turn { .. }
            | run::Event::Start { .. }
            | run::Event::Cancel { .. }
            | run::Event::JobsCancelled { .. }
            | run::Event::Started { .. }
            | run::Event::Yielded { .. }
            | run::Event::Priced { .. }
            | run::Event::Used { .. }
            | run::Event::Ended { .. }
            | run::Event::Delegated { .. }
            | run::Event::Withdraw { .. }
            | run::Event::Delivered { .. } => unreachable!("io answers reads, probes and checks"),
        }
    }

    /// When io answers a read or a probe of `owner`'s due by `deadline`, or
    /// `None` if it fails or runs out of time.
    pub(super) fn result(
        &mut self,
        now: Time,
        settings: Checkouts,
        rng: &mut Rng,
        owner: Token,
        deadline: Time,
    ) -> Option<Time> {
        assert!(self.looks.insert(owner), "a run has one look in flight at a time");
        let at = now.saturating_add(settings.io.draw(rng));
        let failed = rng.chance(settings.io_failures);
        (!failed && at <= deadline).then_some(at)
    }

    /// Lays out what each repository of a checkout the host made holds, as
    /// io finds it.
    pub(super) fn checkout(&mut self, settings: Checkouts, rng: &mut Rng, directories: &[run::Directory]) {
        for &run::Directory { root, .. } in directories {
            if rng.chance(settings.guides) {
                let len = usize::try_from(rng.between(1, u64::from(settings.guide_max))).expect("small");
                // Text with characters of more than one byte, so a cut may
                // fall inside one; or, now and then, not text at all.
                let mut guide = String::new();
                for c in "Keep changes small — and run the tests, où qu'ils soient. ".chars().cycle() {
                    if guide.len() + c.len_utf8() > len {
                        break;
                    }
                    guide.push(c);
                }
                let mut guide = guide.into_bytes();
                if rng.chance(settings.not_text) {
                    guide.insert(0, 0xff);
                }
                self.files.insert((root, b"AGENTS.md".to_vec()), guide);
            }
            if rng.chance(settings.checks) {
                self.executables.insert((root, b".temper/pre-pr".to_vec()));
                self.failures.insert(root, rng.between(0, u64::from(settings.check_failures)));
            }
        }
    }
}

impl World {
    /// The run's reads, probes and aborts, carried out the way io would.
    pub(super) fn io_request(&mut self, request: run::Request) {
        match request {
            run::Request::Read { owner, at, max, deadline } => {
                let (at_time, read) =
                    match self.io.result(self.now, self.settings.checkout, &mut self.rng, owner, deadline) {
                        Some(at_time) => {
                            let read = self.io.read(&at, max);
                            (at_time, read)
                        }
                        None => (deadline, run::Read::Failed),
                    };
                self.stats.reads += 1;
                self.schedule(at_time, Delivery::Io(run::Event::Read { owner, read }));
            }
            run::Request::Probe { owner, at, deadline } => {
                let (at_time, executable) = if let Some(at_time) =
                    self.io.result(self.now, self.settings.checkout, &mut self.rng, owner, deadline)
                {
                    (at_time, self.io.executables.contains(&(at.root, at.path.to_vec())))
                } else {
                    // The run cannot know of checks io failed to find.
                    self.views.get_mut(&owner).expect("a run looks once admitted").checks.remove(&at.root);
                    (deadline, false)
                };
                self.stats.probes += 1;
                self.schedule(at_time, Delivery::Io(run::Event::Probed { owner, executable }));
            }
            run::Request::Abort { owner } => {
                self.stats.aborts += 1;
                // A check whose result is on its way already won the race.
                if let Some(key) = self.io.checks.get(&owner).copied()
                    && !self.rng.chance(self.settings.races)
                {
                    self.wire.withdraw(key).expect("a check in flight has its result on the wire");
                    self.io.checks.remove(&owner);
                    let at = self.now.saturating_add(self.draw(self.settings.checkout.io));
                    self.schedule(at, Delivery::Io(run::Event::Aborted { owner }));
                }
            }
            run::Request::HostCall { .. }
            | run::Request::WithdrawHost { .. }
            | run::Request::Turn { .. }
            | run::Request::MessageRefused { .. }
            | run::Request::Waiting { .. }
            | run::Request::Admitted { .. }
            | run::Request::Answer { .. }
            | run::Request::Open { .. }
            | run::Request::Say { .. }
            | run::Request::CancelJobs { .. }
            | run::Request::Yield { .. }
            | run::Request::Close { .. }
            | run::Request::Return { .. }
            | run::Request::Check { .. }
            | run::Request::Checking { .. }
            | run::Request::ChecksEnded { .. }
            | run::Request::CompletionReady { .. }
            | run::Request::Deliver { .. } => unreachable!("not one of io's requests"),
        }
    }

    /// Runs the checks at `program` as io would: they fail as many times as
    /// their repository was given, then pass, unless they outlast `deadline`.
    pub(super) fn check(&mut self, owner: Token, program: &run::Place, deadline: Time, tail: u32) {
        assert!(self.io.checking.insert(owner, program.root).is_none(), "a call has one check in flight at a time");
        assert!(self.io.executables.contains(&(program.root, program.path.to_vec())), "checks are run where found");
        self.stats.checks += 1;
        let at = self.now.saturating_add(self.draw(self.settings.checkout.check));
        let failures = self.io.failures.entry(program.root).or_insert(0);
        let (at, exit, written) = if at > deadline {
            (deadline, run::Exit::TimedOut, 100)
        } else if *failures > 0 {
            *failures -= 1;
            (at, run::Exit::Code { code: 1 }, self.rng.between(0, u64::from(tail) * 3))
        } else {
            (at, run::Exit::Code { code: 0 }, 20)
        };
        // io keeps the tail of what the checks wrote.
        let kept = written.min(u64::from(tail));
        let output =
            b"test parse_tabs ... FAILED\n".iter().copied().cycle().take(usize::try_from(kept).expect("small"));
        let ran = run::Ran { exit, output: output.collect(), cut: written - kept };
        let key = self.schedule(at, Delivery::Io(run::Event::Checked { owner, ran }));
        self.io.checks.insert(owner, key);
    }
}
