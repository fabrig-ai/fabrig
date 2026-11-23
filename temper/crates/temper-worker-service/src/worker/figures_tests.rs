//! Two checkouts and a real link over scripted IO (worker-restarts.md, 9, test 7).
#![expect(
    clippy::disallowed_methods,
    reason = "ordinary worker integration tests inspect link frames, Git spawns and diagnostics"
)]
use super::*;
use crate::tests::{
    figures::{scalar, set_limit, stock_figures, stock_worker},
    fixtures::{
        self,
        worker_peers::{AgentPeer, LinkPeer, LinkRecord, socket_stand_with_limits},
    },
};
use crate::worker_alone::Note;
use jig_protocol_engine_workers as engine;
use jig_protocol_worker_wire::Figures;
use skein_io::kernel;
use skein_lib::Duration;
use temper_worker_protocol_git_world::io::Invocation;

const TIMEOUT: &str = "checkout.remote_timeout_ms";
const STDOUT: &str = "git_component.stdout_bytes";
const STEPS: u16 = 1000;

struct Story {
    stand: fixtures::WorkerStand,
    link: LinkPeer,
    settings: Agent,
    agent: Option<AgentPeer>,
    base: Figures,
}
impl Story {
    fn open() -> Self {
        let base = stock_figures();
        let (mut part, settings) = stock_worker(base.clone());
        crate::startup::worker::check_worker_part(&mut part, &fixtures::files(), &settings).unwrap();
        part.worker.git = fixtures::worker_settings().git;
        let mut stand = socket_stand_with_limits(&part.worker, &settings, &part.limits);
        stand.worker.built_from(base.clone());
        let mut link = LinkPeer::with_limits(&part.worker, &settings, &part.limits);
        link.component.down(link.now, engine::Event::Figures { figures: base.clone() }, &mut link.out);
        link.open(&mut stand.worker);
        stand.worker.notes().clear();
        stand.worker.link(link.now, fixtures::application_value(fixtures::IDENTITY, 1, b"test-token"));
        Self { stand, link, settings, agent: None, base }
    }
    fn assign(&mut self, run: u64, remote: &[u8]) {
        let mut assignment = fixtures::worker_assignment();
        assignment.assignment.run = Token::new(run);
        assignment.assignment.identity = [u8::try_from(run).unwrap(); 16];
        assignment.assignment.workspace.as_mut().unwrap().workstream = run;
        let mut charter = fixtures::charter();
        let endpoint = self.settings.endpoints.entries().get(0).expect("scripted endpoint");
        charter.llm.dialect = endpoint.dialect;
        charter.llm.endpoint = smith_domain_run::charter::Endpoint(endpoint.number);
        assignment.assignment.charter =
            smith_protocol_channel::encode_charter(&charter, &self.settings.limits.charter, &self.settings.endpoints)
                .unwrap();
        let mut items = fixtures::one_repository();
        items.repositories[0].remote = remote.into();
        self.stand.worker.offer(root::Event::Assign { assignment, items: Ok(items) });
    }
    fn figures(&mut self, figures: Figures) -> Note {
        self.link.component.down(self.link.now, engine::Event::Figures { figures }, &mut self.link.out);
        for _ in 0..STEPS {
            self.link.exchange(&mut self.stand.worker);
            while let Some(note) = self.stand.worker.notes().pop() {
                if let Note::FiguresTaken { .. } | Note::FiguresRefused { .. } = note {
                    return note;
                }
                if let Note::FiguresWaiting { .. } = note {
                    panic!("no deferred changes: {note:?}");
                }
            }
        }
        panic!("the worker hears the figures frame");
    }
    fn cycle(&mut self) {
        let worker = &mut self.stand.worker;
        worker.iterate(self.link.now, Wall::EPOCH);
        if let Some(peer) = &mut self.agent {
            peer.now = self.link.now;
            peer.settle(worker);
            assert!(peer.events.is_empty(), "no agent drain or cancellation: {:?}", peer.events.pop());
        }
        for _ in 0_u8..16 {
            let _request = self.link.exchange(worker);
        }
        assert_eq!(self.link.records.as_slice(), [LinkRecord::Hello], "no lost link, new hello or run ending");
        assert!(!worker.stopping());
    }
    fn hold(&mut self, command: &[u8]) -> (Token, kernel::Spawn) {
        for _ in 0..STEPS {
            self.cycle();
            for _ in 0..self.stand.worker.io().len() {
                let request = self.stand.worker.io().pop().unwrap();
                if let io::Request::Spawn { spawn, .. } = &request
                    && spawn.args.iter().any(|arg| arg.as_ref() == command)
                {
                    let io::Request::Spawn { owner, spawn } = request else { unreachable!("matched spawn") };
                    return (owner, spawn);
                }
                self.stand.worker.io().push(request);
            }
            self.stand.drain();
            self.stand.input();
        }
        panic!("Git spawns {command:?}");
    }
    fn answer(&mut self, (owner, spawn): (Token, kernel::Spawn), invocation: Invocation) {
        self.stand.below.script.invocations.push_back(invocation);
        self.stand.below.out.io.push(io::Request::Spawn { owner, spawn });
    }
    fn admit_first_agent(&mut self) {
        self.stand
            .below
            .tree
            .insert(b"workspaces/0/app/.git".to_vec(), temper_worker_protocol_git_world::io::Node::Directory);
        for capture in [
            "object-format",
            "init",
            "remote-add",
            "init",
            "fetch-branch",
            "update-ref",
            "fetch-missing-branch",
            "update-ref",
            "counts-absent",
            "counts-delete",
            "checkout-1",
            "checkout-2",
            "object-format",
        ] {
            let mut invocation = Invocation::captured(capture);
            if capture == "fetch-branch" {
                invocation.transcript.stdout = b"* 0000000000000000000000000000000000000000 0000000000000000000000000000000000000001 refs/temper/0/fetched\n".as_slice().into();
            }
            self.stand.below.script.invocations.push_back(invocation);
        }
        self.stand.below.script.invocations.back_mut().unwrap().transcript.stdout =
            b"0000000000000000000000000000000000000001\n".as_slice().into();
        for _ in 0..STEPS {
            self.cycle();
            if let Some((owner, _)) = self.stand.worker.processes.iter().next() {
                let owner = *owner;
                self.agent = Some(AgentPeer::launch(&mut self.stand.worker, &self.settings, owner, true));
                return;
            }
            self.stand.drain();
            self.stand.input();
        }
        panic!("run A prepares and starts its agent");
    }
    fn advance(&mut self, seconds: u64) {
        for _ in 0..seconds {
            self.link.now = self.link.now.saturating_add(Duration::from_secs(1));
            self.link.component.fire(self.link.now, &mut self.link.out);
            self.cycle();
            self.stand.drain();
            self.stand.input();
        }
    }
    fn deadline(&self, from: Time, seconds: u64) {
        assert_eq!(
            self.stand.worker.git.component.next_deadline(),
            Some(from.saturating_add(Duration::from_secs(seconds)))
        );
    }
}

#[test]
fn a_changed_remote_timeout_is_read_by_the_next_operation_not_the_one_in_flight() {
    let mut story = Story::open();
    story.assign(1, b"https://forge.example/acme/app.git");
    let a = story.hold(b"clone");
    let a_sent = story.link.now;
    story.deadline(a_sent, 60);
    story.assign(2, b"https://forge.example/acme/other.git");
    assert_eq!(story.stand.worker.root.workspaces(), 2, "both runs hold workspaces before the change");
    story.advance(1);

    let mut raised = story.base.clone();
    set_limit(&mut raised, TIMEOUT, 600_000);
    assert_eq!(story.figures(raised.clone()), Note::FiguresTaken { figures: Box::new([scalar(TIMEOUT, 600_000)]) });
    story.deadline(a_sent, 60);
    story.answer(a, Invocation::captured("clone"));
    story.admit_first_agent();
    let b = story.hold(b"clone");
    assert!(b.1.args.iter().any(|arg| arg.as_ref() == b"https://forge.example/acme/other.git"));
    let b_sent = story.link.now;
    story.deadline(b_sent, 600);
    story.advance(61);
    story.deadline(b_sent, 600);
    story.advance(59);
    story.deadline(b_sent, 600);

    set_limit(&mut raised, TIMEOUT, 30_000);
    assert_eq!(story.figures(raised), Note::FiguresTaken { figures: Box::new([scalar(TIMEOUT, 30_000)]) });
    story.deadline(b_sent, 600);
    story.answer(b, Invocation::captured("clone"));
    for capture in ["object-format", "init", "remote-add", "init"] {
        story.stand.below.script.invocations.push_back(Invocation::captured(capture));
    }
    // Run B's first remote operation under the new figures fetches main, before its landing pin.
    let _fetch_main = story.hold(b"fetch");
    story.deadline(story.link.now, 30);
    assert_eq!(story.stand.worker.root.workspaces(), 2);
    assert_eq!(story.stand.below.signals, 0);
}

#[test]
fn a_refused_live_figure_keeps_the_other_figures_and_the_next_deadline() {
    let mut story = Story::open();
    let mut refused = story.base.clone();
    set_limit(&mut refused, TIMEOUT, 600_000);
    set_limit(&mut refused, STDOUT, 1024);
    let git = *story.stand.worker.git.component.limits();
    let Note::FiguresRefused { problem } = story.figures(refused.clone()) else { panic!("the live take is refused") };
    let (mut part, agent) = stock_worker(refused);
    let problems = crate::startup::worker::check_worker_part(&mut part, &fixtures::files(), &agent).unwrap_err();
    let words = crate::startup::lines(&problems);
    let stdout = core::str::from_utf8(&words).unwrap().lines().find(|line| line.starts_with(STDOUT)).unwrap();
    let (_, why) = stdout.split_once(": ").unwrap();
    let wording =
        skein_lib::bytes::concat(&[b"the engine's figures: git_component.stdout_bytes 1024: ", why.as_bytes()]);
    assert_eq!(problem, wording);
    assert_eq!(*story.stand.worker.git.component.limits(), git);
    story.assign(1, b"https://forge.example/acme/app.git");
    let _clone = story.hold(b"clone");
    story.deadline(story.link.now, 60);
}
