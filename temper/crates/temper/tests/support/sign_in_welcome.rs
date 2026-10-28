//! The welcome's departing tab, fake issuer, come-back page and durable account.
use super::{Process, person};
use skein_fake_peers::oauth;
use skein_io::kernel::{Exit, Fd, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, end_to_end::Streams, real};
use std::path::Path;

// No model call or forge work: two local service starts and one fake grant.
const WITHIN: Duration = Duration::from_secs(20);
const READY: &[u8] = b"/welcome?at=ready";
// configuration's issuer queues exactly one authorization-code grant.
const GRANTS: usize = 1;

struct Judge {
    authorization: Vec<u8>,
    return_root: Option<Fd>,
    visited: bool,
    stopped: bool,
    settled: bool,
    deadline: Time,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let [Process::Binary(binary), Process::Person(person), Process::Issuer(issuer), Process::Visitor(visitor)] =
            procs
        else {
            panic!("welcome's service, person, issuer and visitor");
        };
        if let Some(to) = person.leave() {
            let query = to.strip_prefix(self.authorization.as_slice()).expect("Leave names the fake issuer");
            assert!(query.starts_with(b"?"), "Leave names the authorization endpoint");
            visitor.hand(&to);
            self.visited = true;
        }
        if self.return_root.is_some()
            && person.client.as_ref().is_some_and(Host::is_empty)
            && visitor.is_empty()
            && let Some((status, location)) = visitor.last_redirect()
        {
            assert_eq!(status, 303, "the engine listener redirects the tab");
            let link = super::printed_link(person.stdout()).expect("printed client link");
            let origin_end = link
                .strip_prefix(b"http://")
                .expect("local client link")
                .iter()
                .position(|byte| *byte == b'/')
                .expect("client link path")
                + b"http://".len();
            let origin = &link[..origin_end];
            let expected = [origin, READY].concat();
            assert_eq!(location, expected, "the return names the clients' origin and welcome Ready");
            let path = location.strip_prefix(origin).expect("checked client origin");
            person.come_back(self.return_root.take().expect("come-back root"), path);
        }
        if !self.stopped && person.finished && person.client.as_ref().is_some_and(Host::is_empty) {
            binary.signal(Signal::Terminate);
            issuer.shutdown();
            self.stopped = true;
        }
    }
    fn observe(&mut self, now: Time, procs: &[Process]) {
        let [Process::Binary(binary), Process::Person(person), Process::Issuer(issuer), Process::Visitor(visitor)] =
            procs
        else {
            panic!("welcome's processes");
        };
        assert!(person.failed.is_none(), "{}", person.failed.as_deref().unwrap_or("person running"));
        assert!(
            self.stopped || binary.exit_status().is_none(),
            "serve exited early: {}",
            String::from_utf8_lossy(binary.stderr())
        );
        self.settled = self.stopped && procs.iter().all(Host::is_empty);
        assert!(
            now < self.deadline || self.settled,
            "welcome sign-in stalled at step {}; visited {}; returned {}; redirect {:?}; authorize {}; posts {}; page {:?}",
            person.steps_seen(),
            self.visited,
            self.return_root.is_none(),
            visitor.last_redirect().map(|(status, _)| status),
            issuer.observations().iter().filter(|event| matches!(event, oauth::Observation::Authorize { .. })).count(),
            issuer.observations().iter().filter(|event| matches!(event, oauth::Observation::Post { .. })).count(),
            person.client.as_ref().map(|client| client
                .tree()
                .nodes()
                .iter()
                .filter(|node| matches!(node.role, skein_ui::Role::Heading { .. } | skein_ui::Role::Status))
                .filter_map(|node| node.name.as_deref())
                .map(String::from_utf8_lossy)
                .collect::<Vec<_>>())
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| "welcome sign-in did not settle".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}

pub fn run(directory: &Path, issuer: oauth::Peer) {
    let configuration: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("temper.json")).expect("configuration"))
            .expect("configuration JSON");
    let authorization = configuration["engine"]["llm"]["providers"][0]["sources"]["sign_in"]["authorization"]
        .as_str()
        .expect("fake authorization endpoint")
        .as_bytes()
        .to_vec();
    let clock = Clock::new();
    let mut binary = super::sign_in::start(directory);
    let streams = binary.take_streams().expect("serve pipes");
    let mut fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let root = skein_shell::open_root(directory).expect("client parent root");
    let return_root = skein_shell::open_root(directory).expect("come-back parent root");
    fds.extend([root, return_root]);
    let mut world = real::World::new(Judge {
        authorization,
        return_root: Some(return_root),
        visited: false,
        stopped: false,
        settled: false,
        deadline: clock.now().now.saturating_add(WITHIN),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || {
        Process::Person(Box::new(person::Person::departing(root, input, output, person::welcome_sign_in())))
    });
    world.spawn(|| Process::Issuer(Box::new(issuer)));
    world.spawn(|| Process::Visitor(Box::default()));
    let outcome = world.run(&clock, WITHIN);
    let [Process::Binary(binary), Process::Person(person), Process::Issuer(issuer), Process::Visitor(_)] =
        outcome.procs.as_slice()
    else {
        panic!("welcome's processes");
    };
    assert_eq!(
        issuer.observations().iter().filter(|event| matches!(event, oauth::Observation::Authorize { .. })).count(),
        GRANTS,
        "one authorization visit"
    );
    assert_eq!(
        issuer.observations().iter().filter(|event| matches!(event, oauth::Observation::Post { .. })).count(),
        GRANTS,
        "one code exchange"
    );
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "a healthy stop before restart");
    assert!(outcome.procs.iter().all(Host::is_empty), "all welcome sign-in io settled");
    let tree = person.client.as_ref().expect("come-back client").tree();
    let account = tree
        .nodes()
        .iter()
        .filter(|node| node.role == skein_ui::Role::Paragraph)
        .find_map(|node| node.name.as_deref()?.strip_prefix(b"Model: ChatGPT"))
        .expect("Ready named the made ChatGPT account");
    // Ready names the account's heading, as Accounts draws it, then " · MODEL".
    let (account, _) = std::str::from_utf8(account)
        .expect("Ready account and model are UTF-8")
        .rsplit_once(" · ")
        .expect("Ready names the selected model after the account");
    let heading = format!("ChatGPT{account}");
    let restarted = super::run(directory, person::kept_chatgpt_account(heading.as_bytes()), None, WITHIN);
    assert!(restarted.finished, "the same account is signed in after restart");
}
