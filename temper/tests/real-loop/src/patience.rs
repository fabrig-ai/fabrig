//! Temper-only detail at the world's patience boundary
//! (skein/docs/foundation/testing-strategy.md, 2.8 and 2.10).
use crate::{
    calls::Caller,
    deadline::Deadline,
    world::{Kind, Proc},
};
use skein_lib::{Duration, Time};
use skein_shell::Host;
use skein_world::Expired;
use std::fmt::Write;

#[derive(Debug)]
pub(crate) struct Patience {
    after: Duration,
    deadline: Option<Deadline>,
    figures: Option<String>,
    report: Option<String>,
    deferred: bool,
    #[cfg(test)]
    forced: bool,
}

enum Verdict {
    Starved,
    Failed(String),
}

fn verdict(expired: Expired) -> Verdict {
    match expired {
        Expired::Starved(_) => Verdict::Starved,
        Expired::Failed(figures) => Verdict::Failed(figures.to_string()),
    }
}

impl Patience {
    pub(crate) fn new(patience: Duration) -> Self {
        let margin = Duration::from_secs(1);
        let after = patience.as_nanos().checked_sub(margin.as_nanos()).expect("patience exceeds diagnostic margin");
        assert!(after > 0, "a positive diagnostic window");
        Self {
            after: Duration::from_nanos(after),
            deadline: None,
            figures: None,
            report: None,
            deferred: false,
            #[cfg(test)]
            forced: false,
        }
    }
    #[cfg(test)]
    pub(crate) fn scripted(patience: Duration, start: Time) -> Self {
        let mut guard = Self::new(patience);
        assert!(!guard.due_with(start, Deadline::scripted));
        guard
    }

    /// The first act arms the real-clock deadline after all fixture admission.
    pub(crate) fn due(&mut self, now: Time) -> bool {
        self.due_with(now, Deadline::arm)
    }

    fn due_with(&mut self, now: Time, arm: fn(&'static str, Duration, Time) -> Deadline) -> bool {
        if self.deferred {
            return false;
        }
        #[cfg(test)]
        if self.forced {
            return self.handle(Verdict::Failed("forced failed deadline figures".into()));
        }
        let Some(deadline) = &self.deadline else {
            self.deadline = Some(arm("the temper judge's verdict", self.after, now));
            return false;
        };
        let Some(expired) = deadline.expiration(now) else {
            return false;
        };
        self.handle(verdict(expired))
    }
    #[cfg(test)]
    pub(crate) fn force_failed(&mut self) {
        self.forced = true;
    }
    fn handle(&mut self, verdict: Verdict) -> bool {
        match verdict {
            Verdict::Starved => {
                self.defer();
                false
            }
            Verdict::Failed(figures) => {
                self.figures = Some(figures);
                true
            }
        }
    }
    /// The world owns process teardown and the final starvation verdict.
    pub(crate) fn defer(&mut self) {
        self.deferred = true;
    }
    pub(crate) fn deadline(&self) -> Option<Time> {
        if self.deferred { None } else { self.deadline.as_ref().map(Deadline::until) }
    }
    pub(crate) fn capture(&mut self, mut report: String) {
        writeln!(report, "{}", self.figures.as_ref().expect("failed deadline figures")).expect("string writer");
        self.report = Some(report);
    }
    pub(crate) fn overdue(&self) -> Option<String> {
        self.report.clone()
    }
}

/// Recorded beside the readiness check that the host already makes, once per loop pass.
#[derive(Debug, Default)]
pub(crate) struct PendingStreak(u64);
impl PendingStreak {
    pub(crate) fn observe(&mut self, pending: bool) {
        self.0 = if pending { self.0.saturating_add(1) } else { 0 };
    }
    pub(crate) const fn passes(&self) -> u64 {
        self.0
    }
}

/// Skein asks overdue only on an idle pass. A busy spin must ask it from act as well.
pub(crate) fn check_overdue(now: Time, report: Option<String>) -> Result<(), String> {
    if let Some(why) = report {
        return Err(format!("at {now:?}, the referee failed the real loop before patience:\n{why}"));
    }
    Ok(())
}

pub(crate) fn header(id: usize, name: &str, host: &dyn Host, streak: u64, now: Time, out: &mut String) {
    let pending = host.work_pending(now);
    let streak = if pending { streak } else { 0 };
    writeln!(out, "proc slot={id} {name}: work_pending={pending} pending_passes={streak} next_deadline={:?} next_policy_deadline={:?} empty={} exit={:?}", host.next_deadline(), host.next_policy_deadline(), host.is_empty(), host.exit()).expect("string writer");
}

pub(crate) fn llm(id: usize, peer: &mut skein_fake_peers::llm::Peer, streak: u64, now: Time, out: &mut String) {
    header(id, "LLM", peer, streak, now, out);
    let outstanding = peer.observations().iter().fold(0_i64, |pending, event| {
        use skein_fake_peers::llm::Observation;
        pending + i64::from(matches!(event, Observation::Query { .. }))
            - i64::from(matches!(event, Observation::Answered { .. }))
    });
    // These peer getters return the queues directly; unlike the service adapter they drain nothing.
    let completions = peer.completions().len();
    let submissions = peer.submissions().len();
    writeln!(out, "LLM queues completions={completions} submissions={submissions} observed_unanswered={outstanding}")
        .expect("string writer");
}

pub(crate) fn caller(id: usize, caller: &Caller, now: Time, out: &mut String) {
    header(id, "caller", caller, 0, now, out);
    out.push_str(&caller.describe_pending());
}

/// Formatting belongs to the std harness; the service only returns closed data.
fn service_pending(service: &crate::service::Driven, now: Time, out: &mut String) {
    writeln!(out, "service {:#?}", service.pending(now)).expect("string writer");
}

/// Constructed only at overdue, including the caller's bounded terminal transcript metadata.
pub(crate) fn processes(procs: &mut [Proc], now: Time, out: &mut String) {
    let typist = procs.iter().find_map(|proc| match &proc.kind {
        Kind::Caller(caller) => Some(caller.terminal_pending()),
        Kind::Llm(_)
        | Kind::Forge(_)
        | Kind::Issuer(_)
        | Kind::Service(_)
        | Kind::Ctl(_)
        | Kind::Terminal(_)
        | Kind::Agent(_)
        | Kind::Git(_)
        | Kind::GitRoute(_)
        | Kind::Claude(_)
        | Kind::Relay(_) => None,
    });
    for (id, proc) in procs.iter_mut().enumerate() {
        describe(id, proc, now, out);
        if matches!(&proc.kind, Kind::Terminal(_)) {
            writeln!(out, "terminal typist {typist:#?}").expect("string writer");
        }
    }
}

pub(crate) fn describe(id: usize, proc: &mut Proc, now: Time, out: &mut String) {
    match &mut proc.kind {
        Kind::Llm(peer) => {
            llm(id, &mut peer.peer, peer.pending_streak.passes(), now, out);
            writeln!(out, "LLM hold release={:?} released={:?}", peer.release, peer.released).expect("string writer");
        }
        Kind::Service(host) => {
            header(id, "service", host.as_ref(), host.pending_streak.passes(), now, out);
            if let Some(service) = &host.service {
                service_pending(service, now, out);
            } else {
                out.push_str("service awaiting preparation\n");
            }
        }
        Kind::Caller(host) => caller(id, host, now, out),
        Kind::Forge(host) => {
            header(id, "forge", &host.peer, host.pending_streak.passes(), now, out);
            let completions = host.peer.completions().len();
            let submissions = host.peer.submissions().len();
            writeln!(
                out,
                "forge queues completions={completions} submissions={submissions} blob_records={:?}",
                host.peer.blob_records().map(skein_lib::Queue::len)
            )
            .expect("string writer");
        }
        Kind::Ctl(recorded) => {
            header(id, "ctl", recorded.as_ref(), 0, now, out);
            writeln!(out, "ctl call={} setup={}", recorded.call, recorded.setup).expect("string writer");
        }
        Kind::Agent(host) => header(id, "smith agent", host.as_ref(), 0, now, out),
        Kind::Claude(host) => header(id, "fake Claude", host.as_ref(), 0, now, out),
        Kind::Relay(host) => header(id, "smith relay", host.as_ref(), 0, now, out),
        Kind::Git(host) => header(id, "hosted git", host.as_ref(), 0, now, out),
        Kind::GitRoute(host) => header(id, "git route", host.as_ref(), 0, now, out),
        Kind::Issuer(host) => header(id, "issuer", host.as_ref(), 0, now, out),
        Kind::Terminal(host) => {
            header(id, "terminal", host.as_ref(), 0, now, out);
            writeln!(out, "terminal {:#?}", host.pending()).expect("string writer");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Deadline, Patience, Verdict, check_overdue, verdict};
    use skein_io::kernel::Fd;
    use skein_lib::{Duration, Time, Wall};
    use skein_shell::Host;
    use skein_world::{Expired, Figures};

    const ARM_AT: Time = Time::from_nanos(42);

    #[test]
    fn the_caller_snapshot_names_the_current_wait_without_advancing_it() {
        let mut caller = crate::calls::Caller::new(
            vec![crate::calls::call(
                crate::calls::D1,
                crate::argv!["wait", "task", crate::calls::number(3), "--until", "ended"],
            )],
            std::path::PathBuf::new(),
            Fd::new(0),
        );
        caller.released = true;
        caller.iterate(Time::from_nanos(1), Wall::EPOCH);
        let queued = caller.submissions().len();
        let first = caller.describe_pending();
        assert_eq!(first, caller.describe_pending());
        assert_eq!(caller.submissions().len(), queued);
        assert_eq!(caller.calls.len(), 1);
        assert_eq!(caller.calls[0].tries, 1);
        assert!(first.contains("step=0 call=Some(0)"), "{first}");
        assert!(first.contains("argv=[\"wait\", \"task\", \"3\", \"--until\", \"ended\"]"), "{first}");
        assert!(first.contains("waits_for=ctl response and exit: wait task 3 --until ended"), "{first}");
    }

    fn armed_guard() -> Patience {
        Patience::scripted(Duration::from_secs(20), ARM_AT)
    }

    #[test]
    fn a_starved_loop_leaves_both_deadline_moments_to_the_world() {
        let mut guard = armed_guard();
        let early = guard.deadline().expect("armed deadline");
        assert!(!guard.handle(Verdict::Starved));
        for now in [early, early.saturating_add(Duration::from_secs(1))] {
            assert!(!guard.due_with(now, Deadline::scripted), "the world's patience owns the starved verdict");
            assert_eq!(guard.deadline(), None, "the early deadline cannot keep waking the loop");
            assert_eq!(guard.overdue(), None);
            check_overdue(now, guard.overdue()).expect("starved loop has no finding");
        }
    }

    #[test]
    fn a_failed_verdict_keeps_the_early_diagnostic_with_its_report_and_figures() {
        let mut guard = armed_guard();
        let early = guard.deadline().expect("armed deadline");
        let figures = Figures {
            what: "the temper judge's verdict",
            patience: Duration::from_secs(19),
            elapsed: Duration::from_secs(20),
            thread_waited: Some(Duration::from_nanos(7)),
            thread_cpu: Some(Duration::from_nanos(11)),
            tree_waited: None,
        };
        let expected = figures.to_string();
        assert!(guard.handle(verdict(Expired::Failed(figures))));
        guard.capture("held service component".into());
        assert_eq!(guard.deadline(), Some(early));
        let message = check_overdue(early, guard.overdue()).expect_err("a failed verdict still fails before patience");
        assert!(message.contains("the referee failed the real loop before patience"), "{message}");
        assert!(message.contains("held service component"), "{message}");
        assert!(message.contains(&expected), "{message}");
    }

    fn held_startup() -> crate::service::ServiceHost {
        use temper_playground::documents;
        use temper_service::startup;

        let configuration = documents::world_document();
        let files = documents::files();
        let agent = documents::agent();
        let seeds = documents::seeds(7).service();
        let mut service = startup::checked_start_with_limits(
            &documents::document(),
            &jig_protocol_config::Directories::working(),
            &files,
            Some(&agent),
            &seeds,
            documents::forges(),
            None,
            &documents::given(&configuration),
            Some(b"/usr/bin/git"),
            &configuration.jig,
            &configuration.limits,
        )
        .and_then(|checked| {
            startup::from_checked(
                checked,
                &files,
                Some(&agent),
                &seeds,
                None,
                Some(temper_service::WorkerRuntime {
                    wires: None,
                    root: Fd::new(2),
                    path: b"/worker".as_slice().into(),
                    agent: documents::pinned_agent(),
                }),
                None,
                None,
                Box::new([]),
            )
        })
        .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&startup::lines(&why))));
        // The store owns this scripted descriptor but has not been stepped.
        service.adopt_store(Fd::new(9), 0).expect("store root");
        assert!(service.submissions().is_empty());
        crate::service::ServiceHost::held_startup(configuration, service)
    }

    #[test]
    fn a_busy_startup_snapshot_dispatches_the_early_finding() {
        use crate::world::{
            Kind, Proc,
            diagnostics::{Context, Judge},
        };

        let mut host = held_startup();
        let service = host.service.as_ref().expect("prepared service");
        let first = format!("{:?}", service.pending(ARM_AT));
        assert_eq!(first, format!("{:?}", service.pending(ARM_AT)), "snapshots do not advance startup");
        assert!(host.work_pending(ARM_AT));
        let patience = Duration::from_millis(1010);
        let mut judge = Judge::new(Patience::scripted(patience, ARM_AT));

        // Sample readiness, but hold the service before it can submit its first store operation.
        host.pending_streak.observe(host.work_pending(ARM_AT));
        let mut procs = [Proc::new(Kind::Service(Box::new(host)))];
        assert!(!judge.act(ARM_AT, &mut procs, Context::default()));
        judge.watch(&procs, procs.iter().filter_map(Proc::observed_engine)).expect("no early finding yet");

        let elapsed = Duration::from_millis(11);
        let now = ARM_AT.saturating_add(elapsed);
        assert!(now < ARM_AT.saturating_add(patience));
        assert!(procs[0].work_pending(now), "the world would skip its idle overdue check");
        let Kind::Service(host) = &mut procs[0].kind else { panic!("held service") };
        host.pending_streak.observe(host.work_pending(now));
        assert!(judge.act(now, &mut procs, Context::default()));
        let message = judge
            .watch(&procs, procs.iter().filter_map(Proc::observed_engine))
            .expect_err("act dispatches the finding even while busy");
        for expected in [
            "the referee failed the real loop before patience",
            "engine: Some(\n        Engine {\n            work_pending: true",
            "store: true",
            "settled_revision:",
            "work_pending=true pending_passes=2 ",
            "the temper judge's verdict: patience 10000000 ns; elapsed 11000000 ns; thread runnable wait UNAVAILABLE (ns)",
        ] {
            assert!(message.contains(expected), "missing {expected}: {message}");
        }
    }

    #[test]
    fn a_held_terminal_snapshot_is_bounded_nonsecret_and_does_not_load_a_machine() {
        use crate::world::{Kind, Proc};
        let printed = b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let host = crate::terminal::compose(
            std::path::Path::new("/private-directory-must-not-appear"),
            temper_box_shell::open::target(printed).expect("synthetic link"),
            Fd::new(3),
            Fd::new(0),
            Fd::new(1),
            Fd::new(2),
            false,
        );
        let caller = crate::calls::Caller::new(Vec::new(), std::path::PathBuf::new(), Fd::new(4));
        let mut procs = [Proc::new(Kind::Caller(Box::new(caller))), Proc::new(Kind::Terminal(Box::new(host)))];
        let mut first = String::new();
        super::processes(&mut procs, Time::ZERO, &mut first);
        let mut second = String::new();
        super::processes(&mut procs, Time::ZERO, &mut second);
        assert_eq!(first, second, "snapshot observation changes no client or typist state");
        for expected in [
            "proc slot=1 terminal:",
            "loaded: false",
            "held: 0",
            "active: 0",
            "queued: 0",
            "attempts: 0",
            "deferred_reads: 0",
            "chat: None",
            "view_revision: None",
            "builds: 0",
            "terminal typist Some",
            "commands: 0",
            "pages: 0",
            "recent: []",
        ] {
            assert!(first.contains(expected), "missing {expected}: {first}");
        }
        for secret in ["private-directory-must-not-appear", "127.0.0.1", "AAAAAAAA", "Token("] {
            assert!(!first.contains(secret), "diagnostic leaked {secret}: {first}");
        }
        assert!(first.len() < 4096, "an unloaded snapshot has a fixed small envelope");
        assert!(procs[1].submissions().is_empty(), "no descriptor was driven or loaded");
    }

    #[test]
    fn the_guard_derives_its_wake_deadline_from_the_supplied_patience() {
        #[cfg(target_os = "linux")]
        assert_eq!(crate::PATIENCE, Duration::from_secs(20));
        for supplied in [Duration::from_secs(3), crate::PATIENCE, Duration::from_secs(40)] {
            let mut guard = Patience::new(supplied);
            assert_eq!(guard.after.saturating_add(Duration::from_secs(1)), supplied);
            assert_eq!(guard.deadline(), None);
            assert!(!guard.due_with(ARM_AT, Deadline::scripted));
            let patience = Duration::from_nanos(supplied.as_nanos().checked_sub(1_000_000_000).expect("margin"));
            let deadline = guard.deadline().expect("armed deadline");
            assert!(deadline >= ARM_AT.saturating_add(patience));
            assert!(deadline <= ARM_AT.saturating_add(patience));
            assert!(guard.overdue().is_none(), "no message or snapshot before it fires");
        }
    }
}
