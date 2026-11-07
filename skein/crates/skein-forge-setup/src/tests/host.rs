use super::fixtures;
use crate::{
    call::{Auth, Call},
    host::{Answer, Event, Forge, Host, Kind, Limits, Need, Plan, Request, Settings, Stacked, Transport, Unusable},
    item::Item,
    reconcile::{Failure, Outcome, Stage, Why},
};
use skein_lib::{CryptoRng, Duration, Env, List, Queue, Time, Token, Wall};

#[test]
fn incompatible_calls_and_authentication_are_refused_unexpected() {
    skein_rows::check(
        [
            (
                "a_github_call_on_a_forgejo_forge_is_refused_unexpected: token",
                (
                    Call::GitHub(skein_forge::github::request::Request::App),
                    Auth::Token(fixtures::secret(b"forgejo-token")),
                ),
            ),
            (
                "a_github_call_on_a_forgejo_forge_is_refused_unexpected: app",
                (
                    Call::GitHub(skein_forge::github::request::Request::App),
                    Auth::App { id: 7, key: fixtures::secret(b"app-key") },
                ),
            ),
            (
                "a_github_call_on_a_forgejo_forge_is_refused_unexpected: bearer",
                (
                    Call::GitHub(skein_forge::github::request::Request::App),
                    Auth::Bearer(fixtures::secret(b"installation-token")),
                ),
            ),
            (
                "a_github_call_on_a_forgejo_forge_is_refused_unexpected: none",
                (Call::GitHub(skein_forge::github::request::Request::App), Auth::None),
            ),
            (
                "a_forgejo_call_on_an_app_or_bearer_or_no_auth_is_refused_unexpected: app",
                (
                    Call::Forgejo(skein_forge::forgejo::request::Request::User),
                    Auth::App { id: 7, key: fixtures::secret(b"app-key") },
                ),
            ),
            (
                "a_forgejo_call_on_an_app_or_bearer_or_no_auth_is_refused_unexpected: bearer",
                (
                    Call::Forgejo(skein_forge::forgejo::request::Request::User),
                    Auth::Bearer(fixtures::secret(b"installation-token")),
                ),
            ),
            (
                "a_forgejo_call_on_an_app_or_bearer_or_no_auth_is_refused_unexpected: none",
                (Call::Forgejo(skein_forge::forgejo::request::Request::User), Auth::None),
            ),
        ],
        |(call, auth)| assert_eq!(crate::host::run::forgejo_pair(call, auth), Err(Why::Unexpected)),
    );
}

#[test]
fn a_forgejo_call_on_a_token_passes_through_forgejo_pair() {
    let request = skein_forge::forgejo::request::Request::User;
    let token = fixtures::secret(b"forgejo-token");
    assert_eq!(
        crate::host::run::forgejo_pair(request.clone().into(), Auth::Token(token.clone())),
        Ok((request, token))
    );
}
pub(crate) fn limits() -> Limits {
    Limits {
        forges: 2,
        runs: 2,
        states: 8,
        token_bytes: 64,
        head_bytes: 4096,
        headers: 32,
        body_bytes_in: 65536,
        depth: 16,
        chunk: 1024,
        string_bytes: 512,
        body_bytes: 16384,
        inner: 32,
        target_bytes: 4096,
        tls: skein_tls::client::Limits { read: 16384, send: 16384, records: skein_tls::client::MAX_RECORD },
        io: skein_io::Limits { sockets: 8, refusals: 2, receive: 4096, sends: 4, ..skein_io::Limits::TEST },
    }
}
fn forge(number: u16) -> Forge {
    let mut given = fixtures::given();
    given.forge = number;
    Forge {
        kind: Kind::Forgejo(given),
        address: skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into()),
        transport: Transport::Plaintext { name: None },
    }
}
fn forgejo_mut(settings: &mut Settings) -> &mut crate::given::Given {
    match &mut settings.forges.get_mut(0).expect("one forge").kind {
        Kind::Forgejo(given) => given,
        Kind::GitHub(_) => panic!("Forgejo fixture"),
    }
}
pub(crate) fn settings(numbers: &[u16]) -> Settings {
    let mut forges = List::with_capacity(u32::try_from(numbers.len()).expect("small table"));
    for number in numbers {
        forges.push(forge(*number)).expect("forge count");
    }
    Settings { forges, user_agent: b"kit/0.0.0".as_slice().into() }
}
pub(crate) fn crypto() -> CryptoRng {
    CryptoRng::new(&fixtures::secret(&[42; 32]), 0).expect("32-byte seed")
}
struct World {
    host: Host,
    env: Env<Limits>,
    up: Queue<Event>,
    io: Queue<skein_io::Request>,
}
impl World {
    fn new() -> Self {
        Self::with(limits(), settings(&[1]))
    }
    fn with(limits: Limits, settings: Settings) -> Self {
        Self {
            host: Host::new(&limits, settings, crypto()).expect("host fixtures"),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            up: Queue::uncapped(),
            io: Queue::uncapped(),
        }
    }
    fn down(&mut self, request: Request) {
        self.host.down(&self.env, request, &mut self.up, &mut self.io);
    }
    fn plan(&mut self, run: u64, plan: Plan) {
        self.down(Request::Run { run: Token::new(run), plan });
    }
    fn run(&mut self, run: u64, forge: u16) {
        self.down(Request::Run { run: Token::new(run), plan: Plan::Reconcile { forge, adopted: Box::new([]) } });
    }
    fn answer(&mut self, run: u64, answer: Answer) {
        self.down(Request::Answer { run: Token::new(run), answer });
    }
    fn event(&mut self, event: skein_io::Event) {
        self.host.up(&self.env, event, &mut self.up, &mut self.io);
    }
    fn fire(&mut self, millis: u64) {
        self.env.now = Time::ZERO.saturating_add(Duration::from_millis(millis));
        self.host.fire(&self.env, &mut self.up, &mut self.io);
    }
    fn read_admin(&mut self) {
        self.run(1, 1);
        assert_eq!(self.up.pop(), Some(Event::Need { run: Token::new(1), need: Need::Read(Item::AdminToken) }));
    }
    fn send(&mut self) {
        self.read_admin();
        self.answer(1, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
    }
    fn connect(&mut self) -> Token {
        let Some(skein_io::Request::Connect { owner, addr }) = self.io.pop() else {
            panic!("Connect");
        };
        assert_eq!(addr, skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into()));
        assert_eq!(self.io.len(), 0);
        owner
    }
    fn refused(&mut self) {
        let owner = self.connect();
        self.event(skein_io::Event::Connecting {
            owner,
            socket: Token::new(owner.raw().checked_add(100).expect("small connection count")),
        });
        self.event(skein_io::Event::Failed { owner, error: skein_io::Error::Refused });
        // io closes a failed connect without waiting for this request.
        let _close = self.io.pop();
        self.event(skein_io::Event::Closed { owner });
        assert_eq!(self.up.len(), 0);
    }
}
fn stop(run: u64, forge: u16, why: Why) -> Event {
    Event::Stop {
        run: Token::new(run),
        failure: Failure { forge, step: Stage::Admin, subject: b"kit-admin".as_slice().into(), why },
    }
}
#[test]
fn a_runs_given_takes_the_plans_adopted_and_the_rest_from_the_forge() {
    let mut forge = fixtures::given();
    forge.adopted = Box::new([99]);
    forge.orgs = Box::new([b"acme".as_slice().into()]);
    forge.org_owner = Some(b"ada".as_slice().into());
    forge.listener = Some(([127, 0, 0, 1], 9898).into());
    let plan = [7, 3];
    let given = crate::host::run::given_for(&forge, &plan);
    let mut expected = forge.clone();
    expected.adopted = plan.into();
    assert_eq!(given, expected);
    assert_eq!(forge.adopted.as_ref(), &[99]);
    assert!(crate::host::run::given_for(&forge, &[]).adopted.is_empty());
}
#[test]
#[expect(clippy::too_many_lines, reason = "the labelled cases keep every input and expectation of one rule together")]
fn unusable_host_settings_name_their_refusal() {
    use crate::given::{Place, Refused};
    use skein_forge::forgejo::limits::Problem;
    let mut cases = List::with_capacity(24);
    let mut l = limits();
    l.io.sockets = 1 << 29_u32;
    cases
        .push(("the_host_refuses_unusable_settings_naming_each: io sockets", (l, settings(&[1]), Unusable::Io)))
        .unwrap();
    let mut l = limits();
    l.tls.read = 0;
    cases
        .push(("the_host_refuses_unusable_settings_naming_each: tls intake", (l, settings(&[1]), Unusable::Tls)))
        .unwrap();
    let mut l = limits();
    l.io.intake = 0;
    cases
        .push(("the_host_refuses_unusable_settings_naming_each: io intake", (l, settings(&[1]), Unusable::Io)))
        .unwrap();
    let mut l = limits();
    l.runs = 0;
    cases
        .push(("the_host_refuses_unusable_settings_naming_each: zero runs", (l, settings(&[1]), Unusable::Zero)))
        .unwrap();
    let mut l = limits();
    l.token_bytes = 63;
    cases
        .push((
            "the_host_refuses_unusable_settings_naming_each: token bytes",
            (l, settings(&[1]), Unusable::TokenBytes { token_bytes: 63 }),
        ))
        .unwrap();
    let mut l = limits();
    l.depth = 7;
    cases
        .push((
            "the_host_refuses_unusable_settings_naming_each: document depth",
            (l, settings(&[1]), Unusable::Documents(Problem::Depth)),
        ))
        .unwrap();
    let mut l = limits();
    l.io.intake = 16;
    cases
        .push((
            "the_host_refuses_unusable_settings_naming_each: stack intake",
            (l, settings(&[1]), Unusable::Stack { stacked: Stacked::HttpOverIo, demand: 4096, cap: 16 }),
        ))
        .unwrap();
    cases
        .push((
            "the_host_refuses_unusable_settings_naming_each: forge count",
            (limits(), settings(&[1, 2, 3]), Unusable::Forges { forges: 2 }),
        ))
        .unwrap();
    cases
        .push((
            "the_host_refuses_unusable_settings_naming_each: duplicate forge",
            (limits(), settings(&[1, 1]), Unusable::Duplicate { forge: 1 }),
        ))
        .unwrap();
    let mut s = settings(&[1]);
    s.forges.get_mut(0).unwrap().address = skein_io::Addr::Ip(([192, 0, 2, 1], 3000).into());
    cases
        .push((
            "the_host_refuses_unusable_settings_naming_each: off loopback",
            (limits(), s, Unusable::Address { forge: 1 }),
        ))
        .unwrap();
    let mut s = settings(&[1]);
    s.forges.get_mut(0).unwrap().transport = Transport::Plaintext { name: Some(b"".as_slice().into()) };
    cases
        .push((
            "the_host_refuses_unusable_settings_naming_each: empty plaintext name",
            (limits(), s, Unusable::PlaintextName { forge: 1 }),
        ))
        .unwrap();
    let mut s = settings(&[1]);
    s.user_agent = b"kit\r\n".as_slice().into();
    cases
        .push((
            "the_host_refuses_unusable_settings_naming_each: user agent",
            (limits(), s, Unusable::Request { forge: 1 }),
        ))
        .unwrap();
    let mut s = settings(&[1]);
    forgejo_mut(&mut s).bot_login = b"".as_slice().into();
    cases
        .push((
            "the_host_refuses_unusable_settings_naming_each: bot login",
            (limits(), s, Unusable::Given { forge: 1, refused: Refused { place: Place::BotLogin, other: None } }),
        ))
        .unwrap();
    cases
        .push(("settings_with_too_many_adopted_repositories_are_refused", {
            let mut s = settings(&[1]);
            let given = forgejo_mut(&mut s);
            given.limits.repositories = 1;
            given.adopted = Box::new([1, 2]);
            (limits(), s, Unusable::Given { forge: 1, refused: Refused { place: Place::Repositories, other: None } })
        }))
        .unwrap();
    cases
        .push(("a_forge_whose_page_could_pass_the_listing_bound_is_unusable", {
            let l = limits();
            let mut s = settings(&[1]);
            forgejo_mut(&mut s).limits.reachable = l.inner + 1;
            (l, s, Unusable::Reachable { forge: 1, reachable: 33, inner: 32 })
        }))
        .unwrap();
    cases
        .push(("a_zero_page_bound_is_refused_by_the_given_first", {
            let mut s = settings(&[1]);
            forgejo_mut(&mut s).limits.reachable = 0;
            (limits(), s, Unusable::Given { forge: 1, refused: Refused { place: Place::ReachableLimit, other: None } })
        }))
        .unwrap();
    cases
        .push(("a_github_given_that_is_refused_is_named", {
            let mut s = github_settings();
            github_mut(&mut s).limits.reachable = 0;
            (
                limits(),
                s,
                Unusable::GitHubGiven {
                    forge: 4,
                    refused: crate::github::given::Refused {
                        place: crate::github::given::Place::Reachable,
                        other: None,
                    },
                },
            )
        }))
        .unwrap();
    cases
        .push(("a_github_forge_off_loopback_is_refused_as_a_forgejo_is", {
            let mut s = github_settings();
            s.forges.get_mut(1).expect("second forge").address = skein_io::Addr::Ip(([192, 0, 2, 1], 3001).into());
            (limits(), s, Unusable::Address { forge: 4 })
        }))
        .unwrap();
    cases
        .push(("forges_counts_both_kinds", {
            let mut l = limits();
            l.forges = 1;
            (l, github_settings(), Unusable::Forges { forges: 1 })
        }))
        .unwrap();
    cases
        .push(("a_forge_number_used_by_both_kinds_is_a_duplicate: github second", {
            let mut s = github_settings();
            github_mut(&mut s).forge = 1;
            (limits(), s, Unusable::Duplicate { forge: 1 })
        }))
        .unwrap();
    cases
        .push(("a_forge_number_used_by_both_kinds_is_a_duplicate: github first", {
            let mut s = github_settings();
            let mut forges = List::with_capacity(2);
            let mut github = github_forge();
            github.kind = Kind::GitHub(crate::github::given::Given { forge: 1, ..super::github_fixtures::given() });
            forges.push(github).expect("first forge");
            forges.push(forge(1)).expect("second forge");
            s.forges = forges;
            (limits(), s, Unusable::Duplicate { forge: 1 })
        }))
        .unwrap();
    cases
        .push(("the_host_refuses_invalid_settings: duplicate", {
            (limits(), settings(&[1, 1]), Unusable::Duplicate { forge: 1 })
        }))
        .unwrap();
    cases
        .push(("the_host_refuses_invalid_settings: empty user agent", {
            let mut s = settings(&[1]);
            s.user_agent = Box::new([]);
            (limits(), s, Unusable::Request { forge: 1 })
        }))
        .unwrap();
    cases
        .push(("the_host_refuses_invalid_settings: github given", {
            let mut s = github_settings();
            github_mut(&mut s).limits.reachable = 0;
            (
                limits(),
                s,
                Unusable::GitHubGiven {
                    forge: 4,
                    refused: crate::github::given::Refused {
                        place: crate::github::given::Place::Reachable,
                        other: None,
                    },
                },
            )
        }))
        .unwrap();
    skein_rows::check(cases.into_boxed(), |(l, s, expected)| {
        assert_eq!(Host::new(&l, s, crypto()).err(), Some(expected));
    });
}
#[test]
fn missing_or_invalid_admin_items_stop_before_connecting() {
    skein_rows::check(
        [
            (
                "a_run_without_an_admin_token_stops_before_any_connection",
                (Plan::Reconcile { forge: 1, adopted: Box::new([]) }, None),
            ),
            (
                "an_admin_token_that_breaks_its_item_is_absent",
                (Plan::Reconcile { forge: 1, adopted: Box::new([]) }, Some(fixtures::secret(b"invalid token"))),
            ),
            ("an_adopt_run_asks_for_the_admin_token_and_a_missing_one_stops_naming_the_admin", (adopt(), None)),
        ],
        |(plan, token)| {
            let mut w = World::new();
            w.plan(1, plan);
            read(&mut w, Item::AdminToken);
            assert_eq!(w.io.len(), 0);
            w.answer(1, Answer::Read(token));
            assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Missing)));
            assert_eq!(w.io.len(), 0);
            assert_eq!(w.host.next_deadline(), None);
        },
    );
}
#[test]
fn a_second_run_for_a_forge_with_one_live_stops_busy() {
    let mut w = World::new();
    w.read_admin();
    w.run(2, 1);
    assert_eq!(w.up.pop(), Some(stop(2, 1, Why::Busy)));
    w.answer(1, Answer::Read(None));
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Missing)));
    w.run(3, 1);
    assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(3), need: Need::Read(Item::AdminToken) }));
}
struct CapacityCase {
    limits: Limits,
    settings: Settings,
    incumbents: Box<[Plan]>,
    candidate: Plan,
    expected: Event,
}
#[test]
fn host_run_capacity_counts_writing_and_reachable_runs() {
    skein_rows::check(
        [
            (
                "a_run_past_runs_stops_busy",
                CapacityCase {
                    limits: Limits { runs: 1, ..limits() },
                    settings: settings(&[1, 2]),
                    incumbents: Box::new([Plan::Reconcile { forge: 1, adopted: Box::new([]) }]),
                    candidate: Plan::Reconcile { forge: 2, adopted: Box::new([]) },
                    expected: stop(2, 2, Why::Busy),
                },
            ),
            (
                "runs_counts_reachable_and_writing_runs_together: third picker",
                CapacityCase {
                    limits: Limits { forges: 3, ..limits() },
                    settings: settings(&[1, 2, 3]),
                    incumbents: Box::new([Plan::Reconcile { forge: 1, adopted: Box::new([]) }, reachable(2)]),
                    candidate: reachable(3),
                    expected: stop(3, 3, Why::Busy),
                },
            ),
            (
                "runs_counts_reachable_and_writing_runs_together: third writer",
                CapacityCase {
                    limits: Limits { forges: 3, ..limits() },
                    settings: settings(&[1, 2, 3]),
                    incumbents: Box::new([Plan::Reconcile { forge: 1, adopted: Box::new([]) }, reachable(2)]),
                    candidate: Plan::Reconcile { forge: 3, adopted: Box::new([]) },
                    expected: stop(3, 3, Why::Busy),
                },
            ),
            (
                "runs_counts_reachable_and_writing_runs_together: writer first",
                CapacityCase {
                    limits: Limits { runs: 1, ..limits() },
                    settings: settings(&[1]),
                    incumbents: Box::new([Plan::Reconcile { forge: 1, adopted: Box::new([]) }]),
                    candidate: reachable(1),
                    expected: stop(2, 1, Why::Busy),
                },
            ),
            (
                "runs_counts_reachable_and_writing_runs_together: picker first",
                CapacityCase {
                    limits: Limits { runs: 1, ..limits() },
                    settings: settings(&[1]),
                    incumbents: Box::new([reachable(1)]),
                    candidate: Plan::Reconcile { forge: 1, adopted: Box::new([]) },
                    expected: stop(2, 1, Why::Busy),
                },
            ),
        ],
        |case| {
            let mut w = World::with(case.limits, case.settings);
            let mut run = 1;
            for plan in case.incumbents {
                w.plan(run, plan);
                assert_eq!(w.up.pop(), Some(owner_read(run)));
                run += 1;
            }
            w.plan(run, case.candidate);
            assert_eq!(w.up.pop(), Some(case.expected));
        },
    );
}
#[test]
fn an_answer_with_no_need_waiting_is_ignored() {
    let mut w = World::new();
    w.answer(99, Answer::Written);
    assert_eq!(w.up.len(), 0);
    assert_eq!(w.io.len(), 0);
    w.send();
    let _owner = w.connect();
    w.answer(1, Answer::Written);
    assert_eq!(w.up.len(), 0);
    assert_eq!(w.io.len(), 0);
}
#[test]
fn owner_waits_stop_down_at_their_deadline() {
    skein_rows::check(
        [
            (
                "a_run_waiting_on_the_owner_at_its_deadline_stops_down_at_its_step",
                (Plan::Reconcile { forge: 1, adopted: Box::new([]) }, Item::AdminToken, stop(1, 1, Why::Down)),
            ),
            (
                "a_check_run_waiting_on_the_owner_at_its_deadline_stops_down_naming_its_token",
                (check(), Item::ApiToken, token_stop(Why::Down)),
            ),
            (
                "an_adopt_run_waiting_on_the_owner_at_its_deadline_stops_down_at_its_step",
                (adopt(), Item::AdminToken, stop(1, 1, Why::Down)),
            ),
        ],
        |(plan, item, expected)| {
            let mut w = World::new();
            w.plan(1, plan);
            read(&mut w, item);
            assert_eq!(w.host.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(120))));
            w.fire(120_000);
            assert_eq!(w.up.pop(), Some(expected));
            w.answer(1, Answer::Read(None));
            assert_eq!(w.up.len(), 0);
            assert_eq!(w.host.next_deadline(), None);
        },
    );
}
#[test]
fn the_first_try_waits_double_from_backoff_to_backoff_max() {
    let mut w = World::new();
    w.send();
    for at in [0, 100, 300, 700, 1500, 3100, 6300, 11300, 16300] {
        if at != 0 {
            w.fire(at - 1);
            assert_eq!(w.io.len(), 0);
            w.fire(at);
        }
        w.refused();
    }
}
#[test]
fn the_deadline_cuts_the_last_wait_and_stops_the_run_down() {
    let mut s = settings(&[1]);
    let given = forgejo_mut(&mut s);
    given.limits.deadline = Duration::from_secs(1);
    given.limits.backoff_max = Duration::from_millis(400);
    let mut w = World::with(limits(), s);
    w.send();
    for at in [0, 100, 300, 700] {
        w.fire(at);
        w.refused();
    }
    assert_eq!(w.host.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(1))));
    w.fire(1000);
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Down)));
    assert_eq!(w.io.len(), 0);
    assert_eq!(w.host.next_deadline(), None);
}
#[test]
fn close_says_closed_only_once_io_has_closed_every_connection() {
    let mut w = World::new();
    w.send();
    let owner = w.connect();
    w.down(Request::Close);
    assert_eq!(w.up.len(), 0);
    assert_eq!(w.io.len(), 0);
    let socket = Token::new(100);
    w.event(skein_io::Event::Connecting { owner, socket });
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: socket }));
    assert_eq!(w.up.len(), 0);
    w.event(skein_io::Event::Closed { owner });
    assert_eq!(w.up.pop(), Some(Event::Closed));
    w.event(skein_io::Event::Closed { owner });
    w.run(2, 1);
    assert_eq!(w.up.len(), 0);
    assert_eq!(w.io.len(), 0);
    assert_eq!(w.host.next_deadline(), None);
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the redaction test inspects the host's Debug rendering")]
fn the_hosts_debug_shows_no_token_in_flight() {
    let mut w = World::new();
    w.send();
    let rendered = format!("{:?}", w.host);
    for bytes in rendered.as_bytes().windows(fixtures::ADMIN.len()) {
        assert_ne!(bytes, fixtures::ADMIN, "credentials stay out of Debug");
    }
}

#[test]
fn a_connect_without_a_socket_retries_from_its_closed_time() {
    let mut w = World::new();
    w.send();
    let owner = w.connect();
    w.event(skein_io::Event::Failed { owner, error: skein_io::Error::Busy });
    assert_eq!(w.io.len(), 0);
    w.fire(200);
    assert_eq!(w.io.len(), 0);
    w.event(skein_io::Event::Closed { owner });
    assert_eq!(w.host.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_millis(300))));
    w.fire(299);
    assert_eq!(w.io.len(), 0);
    w.fire(300);
    let next = w.connect();
    assert_ne!(owner, next);
    assert_eq!(w.up.len(), 0);
}
#[test]
fn a_wrong_answer_kind_stops_the_waiting_plan_unexpected() {
    let mut w = World::new();
    w.read_admin();
    w.answer(1, Answer::Written);
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Unexpected)));
    assert_eq!(w.io.len(), 0);
}
#[test]
fn idle_shutdown_closes_once_and_ignores_later_requests() {
    skein_rows::check(
        [
            ("an_empty_host_closes_once_and_ignores_later_requests", Request::Close),
            ("abort_when_idle_is_closed_at_once", Request::Abort),
        ],
        |shutdown| {
            let mut w = World::new();
            w.down(shutdown);
            assert_eq!(w.up.pop(), Some(Event::Closed));
            w.down(Request::Close);
            w.down(Request::Abort);
            w.run(1, 1);
            w.answer(1, Answer::Written);
            w.fire(120_000);
            assert_eq!(w.up.len(), 0);
            assert_eq!(w.io.len(), 0);
            assert_eq!(w.host.next_deadline(), None);
        },
    );
}
#[test]
fn one_timer_turn_stops_every_run_whose_deadline_passed() {
    let mut l = limits();
    l.runs = 10;
    l.forges = 10;
    let mut w = World::with(l, settings(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
    for number in 1_u16..=10 {
        w.run(u64::from(number), number);
        assert_eq!(
            w.up.pop(),
            Some(Event::Need { run: Token::new(u64::from(number)), need: Need::Read(Item::AdminToken) })
        );
    }
    w.fire(120_000);
    for run in 1_u64..=10 {
        assert_eq!(w.up.pop(), Some(stop(run, u16::try_from(run).unwrap(), Why::Down)));
    }
    assert!(w.up.is_empty());
    assert_eq!(w.host.next_deadline(), None);
    assert_eq!(w.io.len(), 0);
}

fn check() -> Plan {
    Plan::Check { forge: 1 }
}
fn take_over(token: &[u8]) -> Plan {
    Plan::TakeOver { forge: 1, admin_token: fixtures::secret(token) }
}
fn adopt() -> Plan {
    Plan::Adopt { forge: 1, repository: crate::host::Repository::Id(core::num::NonZeroU64::new(7).unwrap()) }
}
fn token_stop(why: Why) -> Event {
    Event::Stop {
        run: Token::new(1),
        failure: Failure { forge: 1, step: Stage::Token, subject: b"api".as_slice().into(), why },
    }
}
fn read(w: &mut World, item: Item) {
    assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(1), need: Need::Read(item) }));
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
}
#[test]
fn a_check_run_asks_for_the_api_token_and_a_missing_one_stops_naming_api() {
    let mut w = World::new();
    w.plan(1, check());
    read(&mut w, Item::ApiToken);
    w.answer(1, Answer::Read(None));
    assert_eq!(w.up.pop(), Some(token_stop(Why::Missing)));
    w.plan(2, check());
    assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(2), need: Need::Read(Item::ApiToken) }));
}
fn silent_deadline(w: &mut World, expected: Event) {
    let owner = w.connect();
    let socket = Token::new(100);
    w.event(skein_io::Event::Connecting { owner, socket });
    assert!(w.io.is_empty());
    w.fire(120_000);
    assert_eq!(w.up.pop(), Some(expected));
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: socket }));
    assert!(w.io.is_empty());
    assert!(w.up.is_empty());
    w.event(skein_io::Event::Closed { owner });
    assert_eq!(w.host.next_deadline(), None);
}
#[test]
fn unanswered_connections_stop_down_naming_the_active_step() {
    skein_rows::check(
        [
            (
                "a_check_whose_forge_never_answers_stops_down_naming_the_token_in_flight",
                (check(), Box::from([(Item::ApiToken, fixtures::secret(b"api-token"))]), token_stop(Why::Down)),
            ),
            (
                "a_take_over_run_whose_forge_never_answers_stops_down_at_its_step",
                (take_over(fixtures::TAKEN), Box::<[_]>::from([]), stop(1, 1, Why::Down)),
            ),
        ],
        |(plan, answers, expected)| {
            let mut w = World::new();
            w.plan(1, plan);
            for (item, token) in answers {
                read(&mut w, item);
                w.answer(1, Answer::Read(Some(token)));
            }
            silent_deadline(&mut w, expected);
        },
    );
}
#[test]
fn valid_handed_tokens_connect_without_asking_the_owner() {
    skein_rows::check(
        [
            ("a_take_over_run_sends_the_handed_token_and_asks_the_owner_for_nothing", &[b't'; 40][..]),
            ("a_take_over_token_of_seven_bytes_connects", b"1234567".as_slice()),
        ],
        |token| {
            let mut w = World::new();
            w.plan(1, take_over(token));
            assert!(w.up.is_empty());
            w.connect();
        },
    );
}
#[test]
fn a_take_over_token_that_is_empty_or_breaks_its_item_stops_missing_at_once() {
    for token in [b"".as_slice(), b"has a space", &[b't'; 65]] {
        let mut w = World::new();
        w.plan(1, take_over(token));
        assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Missing)));
        assert!(w.io.is_empty());
        assert_eq!(w.host.next_deadline(), None);
        w.fire(1000);
        w.run(2, 1);
        assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(2), need: Need::Read(Item::AdminToken) }));
        assert_eq!(w.host.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(121))));
    }
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the redaction test inspects request and host Debug rendering")]
fn no_taken_token_is_in_a_requests_or_a_hosts_debug() {
    let mut w = World::new();
    let request = Request::Run { run: Token::new(1), plan: take_over(fixtures::TAKEN) };
    let rendered = format!("{request:?}");
    w.down(request);
    for debug in [rendered, format!("{:?}", w.host)] {
        for fragment in [fixtures::TAKEN, fixtures::TAKEN_SUFFIX] {
            for bytes in debug.as_bytes().windows(fragment.len()) {
                assert_ne!(bytes, fragment);
            }
        }
    }
}
fn plans() -> [Plan; 5] {
    [
        Plan::Reconcile { forge: 1, adopted: Box::new([]) },
        check(),
        take_over(fixtures::TAKEN),
        adopt(),
        Plan::Create {
            forge: 1,
            org: b"first".as_slice().into(),
            repository: b"starter".as_slice().into(),
            files: Box::new([]),
        },
    ]
}
#[test]
fn a_second_run_of_any_plan_for_a_forge_with_one_live_stops_busy() {
    for first in plans() {
        for second in plans() {
            let mut w = World::new();
            w.plan(1, first.clone());
            let connection = match first {
                Plan::Reconcile { .. }
                | Plan::Check { .. }
                | Plan::Adopt { .. }
                | Plan::Create { .. }
                | Plan::Copy { .. } => {
                    assert!(w.up.pop().is_some());
                    None
                }
                Plan::TakeOver { .. } => Some(w.connect()),
                Plan::Reachable { .. }
                | Plan::Manifest { .. }
                | Plan::Convert { .. }
                | Plan::Paste { .. }
                | Plan::Install { .. }
                | Plan::Accept { .. } => {
                    unreachable!("Forgejo writing plan table")
                }
            };
            w.plan(2, second);
            assert_eq!(w.up.pop(), Some(stop(2, 1, Why::Busy)));
            assert!(w.up.is_empty());
            if let Some(owner) = connection {
                use skein_lib::stream::{Down, Up};
                let socket = Token::new(100);
                w.event(skein_io::Event::Connecting { owner, socket });
                w.event(skein_io::Event::Connected { owner });
                let Some(skein_io::Request::Stream { down: Down::Demand { .. }, .. }) = w.io.pop() else {
                    panic!("stream demand");
                };
                w.event(skein_io::Event::Stream { owner, up: Up::Room });
                let Some(skein_io::Request::Stream { down: Down::Send(_), .. }) = w.io.pop() else {
                    panic!("request head");
                };
                for line in [b"HTTP/1.1 401 Unauthorized\r\n".as_slice(), b"Content-Length: 0\r\n", b"\r\n"] {
                    let Some(skein_io::Request::Stream { down: Down::Demand { .. }, .. }) = w.io.pop() else {
                        panic!("stream demand");
                    };
                    w.event(skein_io::Event::Stream { owner, up: Up::Bytes(line.into()) });
                }
                assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Refused(401))));
            } else {
                w.answer(1, Answer::Read(None));
                let expected = match first {
                    Plan::Check { .. } => token_stop(Why::Missing),
                    Plan::Reconcile { .. }
                    | Plan::Adopt { .. }
                    | Plan::TakeOver { .. }
                    | Plan::Create { .. }
                    | Plan::Copy { .. } => stop(1, 1, Why::Missing),
                    Plan::Reachable { .. }
                    | Plan::Manifest { .. }
                    | Plan::Convert { .. }
                    | Plan::Paste { .. }
                    | Plan::Install { .. }
                    | Plan::Accept { .. } => {
                        unreachable!("Forgejo writing plan table")
                    }
                };
                assert_eq!(w.up.pop(), Some(expected));
            }
        }
    }
}

fn reachable(forge: u16) -> Plan {
    Plan::Reachable { forge, installation: None, page: core::num::NonZeroU32::new(1).unwrap() }
}
fn owner_read(run: u64) -> Event {
    Event::Need { run: Token::new(run), need: Need::Read(Item::AdminToken) }
}
fn reachable_stop(run: u64, why: Why) -> Event {
    Event::Stop {
        run: Token::new(run),
        failure: Failure { forge: 1, step: Stage::Token, subject: b"api".as_slice().into(), why },
    }
}
fn paired() -> World {
    let mut w = World::new();
    w.run(1, 1);
    w.plan(2, reachable(1));
    w.up.clear();
    w
}
#[test]
fn a_page_bound_equal_to_the_listing_bound_is_usable() {
    let l = limits();
    let mut s = settings(&[1]);
    forgejo_mut(&mut s).limits.reachable = l.inner;
    Host::new(&l, s, crypto()).expect("a picker page at the listing bound is usable");
}
#[test]
fn a_reachable_run_beside_a_writing_run_of_its_forge_is_live_with_it() {
    let mut w = World::new();
    w.read_admin();
    w.plan(2, reachable(1));
    assert_eq!(w.up.pop(), Some(owner_read(2)));
    w.answer(2, Answer::Read(None));
    assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(2), need: Need::Read(Item::ApiToken) }));
    w.answer(2, Answer::Read(None));
    assert_eq!(w.up.pop(), Some(reachable_stop(2, Why::Missing)));
    w.answer(1, Answer::Read(None));
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Missing)));
    assert!(w.up.is_empty());
    assert_eq!(w.host.next_deadline(), None);
}
#[test]
fn a_second_reachable_run_for_a_forge_stops_busy_naming_the_admin() {
    let mut w = World::new();
    w.plan(2, reachable(1));
    w.up.pop();
    w.plan(3, reachable(1));
    assert_eq!(w.up.pop(), Some(stop(3, 1, Why::Busy)));
    w.answer(2, Answer::Read(None));
    assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(2), need: Need::Read(Item::ApiToken) }));
    w.answer(2, Answer::Read(None));
    assert_eq!(w.up.pop(), Some(reachable_stop(2, Why::Missing)));
    w.answer(3, Answer::Read(None));
    assert!(w.up.is_empty());
}
#[test]
fn existing_writers_make_another_writing_plan_stop_busy() {
    skein_rows::check(
        [
            (
                "a_second_writing_run_beside_a_reachable_and_a_writing_one_stops_busy: reconcile",
                (
                    Box::from([Plan::Reconcile { forge: 1, adopted: Box::new([]) }, reachable(1)]),
                    Plan::Reconcile { forge: 1, adopted: Box::new([]) },
                ),
            ),
            (
                "a_second_writing_run_beside_a_reachable_and_a_writing_one_stops_busy: check",
                (Box::from([Plan::Reconcile { forge: 1, adopted: Box::new([]) }, reachable(1)]), check()),
            ),
            (
                "a_second_writing_run_beside_a_reachable_and_a_writing_one_stops_busy: take over",
                (
                    Box::from([Plan::Reconcile { forge: 1, adopted: Box::new([]) }, reachable(1)]),
                    take_over(fixtures::TAKEN),
                ),
            ),
            (
                "a_second_writing_run_beside_a_reachable_and_a_writing_one_stops_busy: adopt",
                (Box::from([Plan::Reconcile { forge: 1, adopted: Box::new([]) }, reachable(1)]), adopt()),
            ),
            (
                "a_second_writing_run_beside_a_reachable_and_a_writing_one_stops_busy: create",
                (
                    Box::from([Plan::Reconcile { forge: 1, adopted: Box::new([]) }, reachable(1)]),
                    Plan::Create {
                        forge: 1,
                        org: b"first".as_slice().into(),
                        repository: b"starter".as_slice().into(),
                        files: Box::new([]),
                    },
                ),
            ),
            (
                "copy_takes_the_forgejo_writing_slot",
                (Box::<[_]>::from([Plan::Reconcile { forge: 1, adopted: Box::new([]) }]), copy()),
            ),
        ],
        |(incumbents, candidate)| {
            let mut w = World::new();
            let mut run = 1;
            for plan in incumbents {
                w.plan(run, plan);
                w.up.clear();
                run += 1;
            }
            w.plan(run, candidate);
            assert_eq!(w.up.pop(), Some(stop(run, 1, Why::Busy)));
            assert!(w.up.is_empty());
        },
    );
}
#[test]
fn a_reachable_run_for_another_forge_is_live_beside_a_reachable_run() {
    let mut w = World::with(limits(), settings(&[1, 2]));
    for (run, forge) in [(1, 1), (2, 2)] {
        w.plan(run, reachable(forge));
        assert_eq!(w.up.pop(), Some(owner_read(run)));
    }
    for run in [2, 1] {
        w.answer(run, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
        w.connect();
    }
    assert!(w.up.is_empty());
}
#[test]
fn a_reachable_run_asks_the_owner_for_the_admin_token_and_nothing_else() {
    let mut w = World::new();
    w.plan(1, reachable(1));
    read(&mut w, Item::AdminToken);
    w.answer(1, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
    w.connect();
    assert!(w.up.is_empty());
}
#[test]
fn a_reachable_run_with_neither_token_stops_naming_the_api_token() {
    let mut w = World::new();
    w.plan(2, reachable(1));
    w.up.pop();
    w.answer(2, Answer::Read(None));
    assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(2), need: Need::Read(Item::ApiToken) }));
    assert!(w.host.next_deadline().is_some());
    assert!(w.io.is_empty());
    w.answer(2, Answer::Read(None));
    assert_eq!(w.up.pop(), Some(reachable_stop(2, Why::Missing)));
    assert!(w.io.is_empty());
    assert_eq!(w.host.next_deadline(), None);
    w.plan(3, reachable(1));
    assert_eq!(w.up.pop(), Some(owner_read(3)));
}
#[test]
fn a_reachable_plan_for_an_installation_on_forgejo_stops_at_once_and_is_never_live() {
    let mut w = World::new();
    w.read_admin();
    let deadline = w.host.next_deadline();
    w.fire(50_000);
    w.plan(2, Plan::Reachable { forge: 1, installation: Some(7), page: core::num::NonZeroU32::new(1).unwrap() });
    assert_eq!(
        w.up.pop(),
        Some(Event::Stop {
            run: Token::new(2),
            failure: Failure {
                forge: 1,
                step: Stage::Installation,
                subject: b"7".as_slice().into(),
                why: Why::Unexpected
            },
        })
    );
    assert_eq!(w.host.next_deadline(), deadline);
    assert!(w.io.is_empty());
    w.plan(3, reachable(1));
    assert_eq!(w.up.pop(), Some(owner_read(3)));
}
#[test]
fn a_reachable_run_at_its_deadline_stops_down_at_its_step() {
    let mut w = World::new();
    w.plan(1, reachable(1));
    read(&mut w, Item::AdminToken);
    w.fire(120_000);
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Down)));
    assert_eq!(w.host.next_deadline(), None);
    let mut w = World::new();
    w.plan(1, reachable(1));
    read(&mut w, Item::AdminToken);
    w.answer(1, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
    silent_deadline(&mut w, stop(1, 1, Why::Down));
}
#[test]
fn each_run_keeps_its_own_deadline_when_two_are_live() {
    let mut w = World::new();
    w.read_admin();
    w.fire(50_000);
    w.plan(2, reachable(1));
    assert_eq!(w.up.pop(), Some(owner_read(2)));
    assert_eq!(w.host.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(120))));
    w.fire(120_000);
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Down)));
    assert!(w.up.is_empty());
    assert_eq!(w.host.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(170))));
    w.fire(170_000);
    assert_eq!(w.up.pop(), Some(stop(2, 1, Why::Down)));
    assert_eq!(w.host.next_deadline(), None);
}
#[test]
fn close_ends_both_runs_unheard_and_says_closed_once_io_has_closed_both_connections() {
    let mut w = paired();
    let mut connections = List::with_capacity(2);
    for run in [1, 2] {
        w.answer(run, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
        let owner = w.connect();
        connections.push(owner).unwrap();
    }
    w.down(Request::Close);
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    for owner in &connections {
        w.event(skein_io::Event::Connecting { owner: *owner, socket: *owner });
        assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: *owner }));
    }
    w.event(skein_io::Event::Closed { owner: connections.as_slice()[0] });
    assert!(w.up.is_empty());
    w.event(skein_io::Event::Closed { owner: connections.as_slice()[1] });
    assert_eq!(w.up.pop(), Some(Event::Closed));
    w.fire(120_000);
    w.event(skein_io::Event::Closed { owner: connections.as_slice()[0] });
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    assert_eq!(w.host.next_deadline(), None);
}
#[test]
fn a_halted_reachable_stops_at_each_phase_with_the_why_given() {
    use crate::host::run::Machine;
    for (machine, stage, subject) in super::reachable::phase_cases().into_iter().skip(1) {
        let mut machine = Machine::Reachable(machine);
        let mut out = Queue::uncapped();
        machine.halt(Why::Down, &mut out);
        assert_eq!(out.pop(), Some(fixtures::named_failure(stage, subject, Why::Down)));
        assert!(out.is_empty());
    }
}
#[test]
fn a_halt_of_an_ended_reachable_pushes_nothing() {
    use crate::{host::run::Machine, reconcile};
    for event in [reconcile::Event::Reply(reconcile::Reply::Rows(Box::new([]))), reconcile::Event::Written] {
        let (machine, _, _) = super::reachable::phase_cases().into_iter().last().unwrap();
        let mut machine = Machine::Reachable(machine);
        let mut out = Queue::uncapped();
        machine.step(event, &mut out);
        out.pop();
        machine.halt(Why::Down, &mut out);
        assert!(out.is_empty());
    }
}

const ADMIN_BODY: &[u8] = br#"{"id":1,"login":"kit-admin","is_admin":true,"full_name":""}"#;
const BOT_BODY: &[u8] = br#"{"id":42,"login":"kit","is_admin":false,"full_name":"kit:bot"}"#;
const SETTINGS_BODY: &[u8] = br#"{"max_response_items":4,"default_paging_num":30}"#;
impl World {
    fn reply(&mut self, body: &[u8]) {
        self.reply_status(200, body);
    }
    #[expect(clippy::disallowed_macros, reason = "the HTTP fixture frames its status and body's length")]
    fn reply_status(&mut self, status: u16, body: &[u8]) -> u32 {
        use skein_lib::{
            Intake,
            stream::{Down, Up},
        };
        let owner = self.connect();
        self.event(skein_io::Event::Connecting { owner, socket: owner });
        self.event(skein_io::Event::Connected { owner });
        let mut intake = Intake::with_capacity(limits().body_bytes_in);
        let head = format!("HTTP/1.1 {status} Reply\r\nContent-Length: {}\r\n\r\n", body.len());
        intake.append(head.as_bytes()).unwrap();
        intake.append(body).unwrap();
        for _ in 0_u32..4096 {
            match self.io.pop() {
                Some(skein_io::Request::Stream { down: Down::Demand { read, room }, .. }) => {
                    if room > 0 {
                        self.event(skein_io::Event::Stream { owner, up: Up::Room });
                    } else if let Some(bytes) = intake.meet(read) {
                        self.event(skein_io::Event::Stream { owner, up: Up::Bytes(bytes) });
                    }
                }
                Some(skein_io::Request::Stream { down: Down::Send(_) | Down::Finish, .. }) => (),
                Some(skein_io::Request::Close { entity }) => {
                    assert_eq!(entity, owner);
                    self.event(skein_io::Event::Closed { owner });
                    return intake.len();
                }
                Some(other) => panic!("unexpected io: {other:?}"),
                None => self.host.fire(&self.env, &mut self.up, &mut self.io),
            }
        }
        panic!("HTTP reply did not settle");
    }
    fn finish_reachable(&mut self) {
        self.answer(2, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
        self.reply(ADMIN_BODY);
        self.reply(SETTINGS_BODY);
        self.reply(br#"{"ok":true,"data":[]}"#);
    }
    fn finish_reconcile(&mut self) {
        self.answer(1, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
        self.reply(ADMIN_BODY);
        self.reply(BOT_BODY);
        self.up.pop();
        for _ in [Item::ApiToken, Item::GitToken] {
            self.answer(1, Answer::Read(Some(fixtures::secret(b"bot-token"))));
            self.reply(BOT_BODY);
            self.up.pop();
        }
        self.answer(1, Answer::Read(Some(fixtures::secret(&42_u64.to_le_bytes()))));
    }
}
#[test]
fn a_finished_run_frees_its_own_kind_only() {
    let mut w = paired();
    w.finish_reachable();
    assert_eq!(
        w.up.pop(),
        Some(Event::Done { run: Token::new(2), outcome: Outcome::Reachable { rows: Box::new([]), more: false } })
    );
    w.plan(3, reachable(1));
    assert_eq!(w.up.pop(), Some(owner_read(3)));
    w.run(4, 1);
    assert_eq!(w.up.pop(), Some(stop(4, 1, Why::Busy)));
    let mut w = paired();
    w.finish_reconcile();
    assert_eq!(w.up.pop(), Some(Event::Done { run: Token::new(1), outcome: Outcome::Nothing }));
    w.run(3, 1);
    assert_eq!(w.up.pop(), Some(owner_read(3)));
    w.plan(4, reachable(1));
    assert_eq!(w.up.pop(), Some(stop(4, 1, Why::Busy)));
}

fn github_forge() -> Forge {
    Forge {
        kind: Kind::GitHub(super::github_fixtures::given()),
        address: skein_io::Addr::Ip(([127, 0, 0, 1], 3001).into()),
        transport: Transport::Plaintext { name: None },
    }
}
pub(crate) fn github_settings() -> Settings {
    let mut settings = settings(&[1, 4]);
    *settings.forges.get_mut(1).expect("second forge") = github_forge();
    settings
}
fn github_mut(settings: &mut Settings) -> &mut crate::github::given::Given {
    match &mut settings.forges.get_mut(1).expect("second forge").kind {
        Kind::Forgejo(_) => panic!("GitHub fixture"),
        Kind::GitHub(given) => given,
    }
}
#[test]
fn the_host_admits_a_github_forge_beside_a_forgejo_forge() {
    let settings = github_settings();
    let forgejo = settings.forges.get(0).expect("first forge");
    assert_eq!(forgejo.number(), 1);
    assert!(forgejo.forgejo().is_some());
    assert!(forgejo.github().is_none());
    let github = settings.forges.get(1).expect("second forge");
    assert_eq!(github.number(), 4);
    assert!(github.forgejo().is_none());
    assert_eq!(github.github(), Some(&super::github_fixtures::given()));
    Host::new(&limits(), settings, crypto()).expect("both kinds admitted");
}
fn github_stop(run: u64, name: &[u8], why: Why) -> Event {
    Event::Stop { run: Token::new(run), failure: Failure { forge: 4, step: Stage::App, subject: name.into(), why } }
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the failure's Debug must redact the handed token")]
fn a_forgejo_plan_on_a_github_forge_stops_at_once_unexpected() {
    let token = b"handed-token-that-must-stay-secret";
    for (plan, name) in [
        (Plan::Check { forge: 4 }, b"check".as_slice()),
        (Plan::TakeOver { forge: 4, admin_token: fixtures::secret(token) }, b"take-over"),
        (
            Plan::Adopt {
                forge: 4,
                repository: crate::host::Repository::Id(core::num::NonZeroU64::new(1).expect("repository")),
            },
            b"adopt",
        ),
    ] {
        let mut limits = limits();
        limits.runs = 1;
        let mut w = World::with(limits, github_settings());
        // Reusing the owner's run token proves that the refused run is never live.
        for _ in 0..2u8 {
            w.plan(1, plan.clone());
            let event = w.up.pop().expect("immediate Stop");
            assert_eq!(event, github_stop(1, name, Why::Unexpected));
            let rendered = format!("{event:?}");
            for bytes in rendered.as_bytes().windows(token.len()) {
                assert_ne!(bytes, token, "credentials stay out of Debug");
            }
            assert_eq!(w.up.len(), 0);
            assert_eq!(w.io.len(), 0);
            assert_eq!(w.host.next_deadline(), None);
        }
        // A writing plan on the other forge can consume the host's only run slot.
        w.run(2, 1);
        assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(2), need: Need::Read(Item::AdminToken) }));
    }
}
#[test]
#[expect(clippy::disallowed_macros, reason = "a Busy failure's Debug must redact the handed token")]
fn a_busy_github_forge_names_the_plan_at_the_app() {
    let forgejo = forge(1);
    let github = github_forge();
    for (plan, name) in [
        (Plan::Reconcile { forge: 4, adopted: Box::new([]) }, b"reconcile".as_slice()),
        (Plan::Check { forge: 4 }, b"check"),
        (Plan::TakeOver { forge: 4, admin_token: fixtures::secret(b"handed-token") }, b"take-over"),
        (
            Plan::Adopt {
                forge: 4,
                repository: crate::host::Repository::Id(core::num::NonZeroU64::new(1).expect("repository")),
            },
            b"adopt",
        ),
        (
            Plan::Reachable {
                forge: 4,
                installation: Some(1),
                page: core::num::NonZeroU32::new(1).expect("first page"),
            },
            b"reachable",
        ),
    ] {
        let failure = crate::host::about(&github, &plan, Why::Busy);
        assert_eq!(failure, Failure { forge: 4, step: Stage::App, subject: name.into(), why: Why::Busy });
        let rendered = format!("{failure:?}");
        for bytes in rendered.as_bytes().windows(b"handed-token".len()) {
            assert_ne!(bytes, b"handed-token", "credentials stay out of Debug");
        }
        assert_eq!(
            crate::host::about(&forgejo, &plan, Why::Busy),
            Failure { forge: 1, step: Stage::Admin, subject: b"kit-admin".as_slice().into(), why: Why::Busy }
        );
    }
}
fn github_reachable(installation: Option<u64>) -> Plan {
    Plan::Reachable { forge: 4, installation, page: core::num::NonZeroU32::new(1).expect("first page") }
}
#[test]
fn a_github_reachable_with_no_installation_stops_at_once() {
    let mut w = World::with(limits(), github_settings());
    w.plan(1, github_reachable(None));
    assert_eq!(
        w.up.pop(),
        Some(Event::Stop {
            run: Token::new(1),
            failure: Failure {
                forge: 4,
                step: Stage::Installation,
                subject: b"none".as_slice().into(),
                why: Why::Unexpected
            },
        })
    );
    assert_eq!(w.up.len(), 0);
    assert_eq!(w.io.len(), 0);
    assert_eq!(w.host.next_deadline(), None);
}
#[test]
fn a_github_reachable_asks_the_owner_for_the_accepted_set_first() {
    let mut w = World::with(limits(), github_settings());
    w.plan(1, github_reachable(Some(2)));
    assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(1), need: Need::Read(Item::Accepted) }));
    assert_eq!(w.up.len(), 0);
    assert_eq!(w.io.len(), 0);
}
#[test]
fn a_github_reachable_is_not_a_writing_run() {
    let mut w = World::with(limits(), github_settings());
    w.run(1, 1);
    assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(1), need: Need::Read(Item::AdminToken) }));
    w.plan(2, github_reachable(Some(2)));
    assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(2), need: Need::Read(Item::Accepted) }));
    assert_eq!(w.up.len(), 0);
    assert_eq!(w.io.len(), 0);
}
#[test]
fn a_github_forges_bounds_come_from_its_given() {
    use crate::item::{self, Bounds};
    let mut given = super::github_fixtures::given();
    given.limits.name_bytes = 3;
    given.limits.key_bytes = 99;
    given.limits.installations = 2;
    let forge = Forge { kind: Kind::GitHub(given), ..github_forge() };
    let bounds = crate::host::bounds(&limits(), &forge);
    assert_eq!(bounds, Bounds { token_bytes: 64, name_bytes: 3, key_bytes: 99, installations: 2 });
    assert!(item::fits(Item::AppSlug, b"kit", &bounds));
    assert!(!item::fits(Item::AppSlug, b"kits", &bounds));
    assert_eq!(
        crate::host::bounds(&limits(), &self::forge(1)),
        Bounds { token_bytes: 64, name_bytes: 0, key_bytes: 0, installations: 0 }
    );
}
#[test]
fn a_github_picker_page_must_fit_the_hosts_listing_bound() {
    let limits = limits();
    for reachable in [limits.inner, limits.inner + 1] {
        let mut settings = github_settings();
        github_mut(&mut settings).limits.reachable = reachable;
        let expected = if reachable > limits.inner {
            Some(Unusable::Reachable { forge: 4, reachable, inner: limits.inner })
        } else {
            None
        };
        assert_eq!(Host::new(&limits, settings, crypto()).err(), expected);
    }
}
#[test]
fn a_wrong_kind_plan_is_unexpected_even_when_the_host_is_full() {
    let mut limits = limits();
    limits.runs = 1;
    let mut w = World::with(limits, github_settings());
    w.read_admin();
    w.plan(2, Plan::Check { forge: 4 });
    assert_eq!(w.up.pop(), Some(github_stop(2, b"check", Why::Unexpected)));
    assert_eq!(w.up.len(), 0);
    assert_eq!(w.io.len(), 0);
}

#[test]
fn a_host_with_no_github_forge_checks_what_it_did() {
    const HOST_BYTES: u32 = 14; // 127.0.0.1:3000, from forge().
    const AGENT_BYTES: u32 = 9; // kit/0.0.0, from settings().
    const FRAMING: u32 = 192; // Forgejo's request-head overhead in host::limits.
    let mut l = limits();
    l.depth = 9;
    l.tls.send = 1;
    l.io.output = l.target_bytes + HOST_BYTES + l.token_bytes + AGENT_BYTES + FRAMING;
    Host::new(&l, settings(&[1]), crypto()).unwrap();
}
#[test]
fn a_github_forge_with_documents_that_cannot_hold_the_pin_is_unusable() {
    let mut l = limits();
    l.depth = 9;
    assert_eq!(
        Host::new(&l, github_settings(), crypto()).err(),
        Some(Unusable::GitHubDocuments(skein_forge::github::limits::Problem::Depth))
    );
    // Scalar and body strings have independent bounds.
    l.depth = skein_forge::github::limits::DEPTH;
    l.string_bytes = 27;
    Host::new(&l, github_settings(), crypto()).unwrap();
}

fn tls_settings() -> Settings {
    let root = std::fs::read("../../tests/tls/fixtures/root.der").expect("TLS world's root");
    let trust = skein_tls::Config::from_der(&[root.into_boxed_slice()], &[]).expect("valid root").0;
    let mut settings = settings(&[1]);
    settings.forges.get_mut(0).expect("forge").transport =
        Transport::Tls { server_name: skein_tls::Name::new("skein.test").expect("TLS name"), trust };
    settings
}
#[test]
fn a_tls_forge_of_either_kind_off_loopback_is_usable() {
    for github in [false, true] {
        let mut settings = tls_settings();
        let forge = settings.forges.get_mut(0).expect("forge");
        if github {
            forge.kind = Kind::GitHub(super::github_fixtures::given());
        }
        forge.address = skein_io::Addr::Ip(([192, 0, 2, 1], 3000).into());
        Host::new(&limits(), settings, crypto()).expect("TLS at any IP");
    }
}
#[test]
fn tls_stack_demands_past_the_caps_are_refused() {
    let mut l = limits();
    l.tls.read = 1;
    assert_eq!(
        Host::new(&l, tls_settings(), crypto()).err(),
        Some(Unusable::Stack { stacked: Stacked::HttpOverTls, demand: 4096, cap: 1 })
    );
    let mut l = limits();
    l.io.intake = 8192;
    assert_eq!(
        Host::new(&l, tls_settings(), crypto()).err(),
        Some(Unusable::Stack { stacked: Stacked::TlsOverIo, demand: skein_tls::client::LARGEST_READ, cap: 8192 })
    );
    let mut l = limits();
    l.tls.send = l.io.output;
    assert_eq!(
        Host::new(&l, tls_settings(), crypto()).err(),
        Some(Unusable::Stack {
            stacked: Stacked::TlsOverIo,
            demand: skein_tls::client::largest_room(&l.tls),
            cap: l.io.output
        })
    );
}
#[test]
fn in_process_forges_are_refused_on_both_transports() {
    for mut settings in [settings(&[1]), tls_settings()] {
        settings.forges.get_mut(0).expect("forge").address =
            skein_io::Addr::InProcess(skein_io::Name::new(b"setup").expect("in-process name"));
        assert_eq!(Host::new(&limits(), settings, crypto()).err(), Some(Unusable::Address { forge: 1 }));
    }
}

#[test]
fn tls_handshake_failure_answers_down_and_closes_for_both_dialects() {
    use crate::host::connection::Connection;
    use crate::reconcile::Reply;
    use skein_lib::stream::{Fault, Up};
    let settings = tls_settings();
    let forge = settings.forges.get(0).expect("forge");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let forgejo = Connection::new(
        Token::new(1),
        forge,
        b"kit",
        &skein_forge::forgejo::request::Request::User.into(),
        &fixtures::secret(fixtures::ADMIN),
        &env.limits,
    )
    .expect("Forgejo call");
    let github = Connection::github(
        Token::new(1),
        forge,
        b"kit",
        &skein_forge::github::request::Request::Conversion { code: b"code".as_slice().into() }.into(),
        &Auth::None,
        env.wall,
        &env.limits,
    )
    .expect("GitHub call");
    for mut connection in [forgejo, github] {
        let call = connection.call.as_ref().expect("held call");
        let mut host = None;
        for header in &call.headers {
            if header.name.as_ref() == b"Host" {
                host = Some(header.value.as_ref());
            }
        }
        assert_eq!(host, Some(b"skein.test:3000".as_slice()));
        connection.link.connecting(Token::new(2), &mut Queue::uncapped());
        connection.connect(&env);
        assert!(connection.call.is_some(), "HTTP waits for Ready");
        let mut io = Queue::with_capacity(16);
        for _ in 0_u32..8 {
            if !connection.pump(&env, &mut io) {
                break;
            }
        }
        connection.stream(&env, Up::Failed(Fault::Reset));
        for _ in 0_u32..16 {
            if !connection.pump(&env, &mut io) {
                break;
            }
        }
        assert_eq!(connection.result.take(), Some(Reply::Down));
        assert!(connection.call.is_none());
        let mut closed = false;
        for _ in 0..io.len() {
            if let Some(skein_io::Request::Close { entity }) = io.pop() {
                assert_eq!(entity, Token::new(2));
                closed = true;
            }
        }
        assert!(closed);
    }
}

#[test]
fn an_ended_runs_settling_connection_keeps_its_slot_until_closed() {
    use skein_lib::stream::{Down, Up};
    let mut l = limits();
    l.runs = 1;
    let mut w = World::with(l, settings(&[1, 2]));
    w.send();
    let owner = w.connect();
    w.event(skein_io::Event::Connecting { owner, socket: owner });
    w.event(skein_io::Event::Connected { owner });
    let Some(skein_io::Request::Stream { down: Down::Demand { .. }, .. }) = w.io.pop() else {
        panic!("stream demand");
    };
    w.event(skein_io::Event::Stream { owner, up: Up::Room });
    let Some(skein_io::Request::Stream { down: Down::Send(_), .. }) = w.io.pop() else {
        panic!("request head");
    };
    for line in [b"HTTP/1.1 401 Unauthorized\r\n".as_slice(), b"Content-Length: 0\r\n", b"\r\n"] {
        let Some(skein_io::Request::Stream { down: Down::Demand { .. }, .. }) = w.io.pop() else {
            panic!("stream demand");
        };
        w.event(skein_io::Event::Stream { owner, up: Up::Bytes(line.into()) });
    }
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Refused(401))));
    assert_eq!(w.io.pop(), Some(skein_io::Request::Close { entity: owner }));
    w.run(2, 2);
    assert_eq!(w.up.pop(), Some(stop(2, 2, Why::Busy)));
    assert!(w.io.is_empty());
    w.event(skein_io::Event::Closed { owner });
    w.run(2, 2);
    assert_eq!(w.up.pop(), Some(owner_read(2)));
}
#[test]
fn abort_stops_each_run_down_at_once_and_says_closed_once_io_has_aborted_its_connection() {
    let mut w = paired();
    w.answer(1, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
    let owner = w.connect();
    w.down(Request::Abort);
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Down)), "the run sending, as at its deadline");
    assert_eq!(w.up.pop(), Some(stop(2, 1, Why::Down)), "the run waiting on the owner, as at its deadline");
    assert!(w.up.is_empty(), "closed waits for io");
    assert!(w.io.is_empty(), "io has not named the socket");
    w.event(skein_io::Event::Connecting { owner, socket: owner });
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: owner }));
    assert!(w.up.is_empty());
    w.event(skein_io::Event::Closed { owner });
    assert_eq!(w.up.pop(), Some(Event::Closed));
    w.answer(2, Answer::Read(None));
    w.down(Request::Abort);
    w.fire(120_000);
    assert!(w.up.is_empty(), "nothing after closed");
    assert!(w.io.is_empty());
    assert_eq!(w.host.next_deadline(), None);
}
#[test]
fn abort_after_close_changes_nothing_as_the_close_aborted_every_connection() {
    let mut w = World::new();
    w.send();
    let owner = w.connect();
    w.down(Request::Close);
    w.down(Request::Abort);
    assert!(w.up.is_empty(), "the close ended the run unheard");
    assert!(w.io.is_empty());
    w.event(skein_io::Event::Connecting { owner, socket: owner });
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: owner }));
    assert!(w.io.is_empty(), "one abort");
    w.event(skein_io::Event::Closed { owner });
    assert_eq!(w.up.pop(), Some(Event::Closed));
    assert!(w.up.is_empty());
}
#[test]
fn close_after_abort_changes_nothing() {
    let mut w = World::new();
    w.send();
    let owner = w.connect();
    w.down(Request::Abort);
    assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Down)));
    w.down(Request::Close);
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    w.event(skein_io::Event::Connecting { owner, socket: owner });
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: owner }));
    assert!(w.io.is_empty(), "one abort");
    w.event(skein_io::Event::Closed { owner });
    assert_eq!(w.up.pop(), Some(Event::Closed));
    assert!(w.up.is_empty());
}

#[path = "../host/runtime_tests.rs"]
mod runtime_tests;

fn copy() -> Plan {
    Plan::Copy {
        forge: 1,
        org: b"first".as_slice().into(),
        repository: b"copied".as_slice().into(),
        from: b"https://source.invalid/owner/repo".as_slice().into(),
        files: Box::new([]),
    }
}
#[test]
fn copy_selects_its_own_deadline_from_admission_time() {
    let forge = forge(1);
    assert_eq!(crate::host::run::deadline_why(&copy()), Why::TimedOut);
    assert_eq!(crate::host::run::deadline_why(&check()), Why::Down);
    assert_eq!(
        crate::host::run::deadline(&forge, &copy(), Time::ZERO),
        Time::ZERO.saturating_add(Duration::from_secs(1800))
    );
    assert_eq!(
        crate::host::run::deadline(&forge, &check(), Time::ZERO),
        Time::ZERO.saturating_add(Duration::from_secs(120))
    );
}
#[test]
fn copy_on_github_stops_unexpected_naming_its_plan() {
    let mut github = World::with(limits(), github_settings());
    let mut github_plan = copy();
    if let Plan::Copy { forge, .. } = &mut github_plan {
        *forge = 4;
    }
    github.plan(2, Plan::Install { forge: 4 });
    github.up.clear();
    github.plan(1, github_plan);
    assert_eq!(
        github.up.pop(),
        Some(Event::Stop {
            run: Token::new(1),
            failure: Failure { forge: 4, step: Stage::App, subject: b"copy".as_slice().into(), why: Why::Unexpected }
        })
    );
    assert!(github.up.is_empty());
}
#[test]
fn copy_on_forgejo_reads_its_admin_token() {
    let mut forgejo = World::new();
    forgejo.plan(1, copy());
    assert_eq!(forgejo.up.pop(), Some(owner_read(1)));
    assert!(forgejo.up.is_empty());
    assert!(forgejo.io.is_empty());
}

#[test]
#[should_panic(expected = "the owner checks a copy plan before sending it")]
fn the_host_asserts_copy_admission_before_starting() {
    let mut plan = copy();
    if let Plan::Copy { from, .. } = &mut plan {
        *from = b"http://source.invalid/owner/repo".as_slice().into();
    }
    World::new().plan(1, plan);
}

fn repository_stop(why: Why) -> Event {
    Event::Stop {
        run: Token::new(1),
        failure: Failure { forge: 1, step: Stage::Repository, subject: b"first/copied".as_slice().into(), why },
    }
}

impl World {
    fn absent_repository(&mut self, plan: Plan) {
        self.plan(1, plan);
        read(self, Item::AdminToken);
        self.answer(1, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
        self.reply(ADMIN_BODY);
        self.reply(br#"{"id":1,"name":"first"}"#);
        self.reply_status(404, b"");
    }

    fn migrate(&mut self) {
        self.absent_repository(copy());
        assert_eq!(
            self.up.pop(),
            Some(Event::Need {
                run: Token::new(1),
                need: Need::Write { item: Item::Copying, bytes: fixtures::secret(&[1]) },
            })
        );
        self.answer(1, Answer::Written);
        assert!(self.up.is_empty());
    }
}

#[test]
fn a_refused_migrate_says_forgejos_message_before_its_stop() {
    let mut w = World::new();
    w.migrate();
    w.reply_status(422, br#"{"message":"source is private"}"#);
    assert_eq!(w.up.pop(), Some(Event::Said { run: Token::new(1), words: b"source is private".as_slice().into() }));
    assert_eq!(w.up.pop(), Some(repository_stop(Why::Refused(422))));
    assert!(w.up.is_empty());
}

#[test]
fn a_refused_migrate_with_no_message_says_nothing() {
    for body in [b"".as_slice(), b"not JSON", br#"{"url":"help"}"#, br#"{"message":""}"#] {
        let mut w = World::new();
        w.migrate();
        w.reply_status(422, body);
        assert_eq!(w.up.pop(), Some(repository_stop(Why::Refused(422))));
        assert!(w.up.is_empty());
    }
}

#[test]
fn a_refused_migrates_words_are_cut_at_a_character() {
    let cap = usize::try_from(limits().string_bytes).expect("string cap");
    for prefix_length in [cap - 3, cap - 1] {
        let mut w = World::new();
        let mut prefix = skein_lib::bytes::zeroed(prefix_length);
        prefix.fill(b'a');
        let body = skein_lib::bytes::concat(&[br#"{"message":""#, &prefix, "界extra\"}".as_bytes()]);
        w.migrate();
        w.reply_status(422, &body);
        let expected =
            if prefix_length == cap - 3 { skein_lib::bytes::concat(&[&prefix, "界".as_bytes()]) } else { prefix };
        assert_eq!(w.up.pop(), Some(Event::Said { run: Token::new(1), words: expected }));
        assert_eq!(w.up.pop(), Some(repository_stop(Why::Refused(422))));
        assert!(w.up.is_empty());
    }
}

#[test]
fn a_401_says_nothing() {
    let mut w = World::new();
    w.migrate();
    let body = br#"{"message":"a token must stay private"}"#;
    let unread = w.reply_status(401, body);
    assert_eq!(usize::try_from(unread).expect("small body"), body.len(), "a 401's body is not read");
    assert_eq!(w.up.pop(), Some(repository_stop(Why::Refused(401))));
    assert!(w.up.is_empty());
}

#[test]
fn no_other_refusal_says_anything() {
    let mut w = World::new();
    w.absent_repository(Plan::Create {
        forge: 1,
        org: b"first".as_slice().into(),
        repository: b"copied".as_slice().into(),
        files: Box::new([]),
    });
    let body = br#"{"message":"refused creation"}"#;
    let unread = w.reply_status(422, body);
    assert_eq!(usize::try_from(unread).expect("small body"), body.len());
    assert_eq!(w.up.pop(), Some(repository_stop(Why::Refused(422))));
    assert!(w.up.is_empty());
}

#[test]
fn a_copy_past_its_deadline_stops_timed_out() {
    let mut settings = settings(&[1]);
    forgejo_mut(&mut settings).limits.copy_deadline = Duration::from_secs(180);
    let mut w = World::with(limits(), settings);
    w.migrate();
    let owner = w.connect();
    let socket = Token::new(100);
    w.event(skein_io::Event::Connecting { owner, socket });
    w.event(skein_io::Event::Connected { owner });
    w.io.clear();
    w.fire(179_999);
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    w.fire(180_000);
    assert_eq!(w.up.pop(), Some(repository_stop(Why::TimedOut)));
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: socket }));
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    w.event(skein_io::Event::Closed { owner });
    assert!(w.up.is_empty());
    assert_eq!(w.host.next_deadline(), None);
}

#[test]
fn a_removed_forges_copy_stops_down() {
    for request in [Request::Remove { forge: 1 }, Request::Abort] {
        let mut w = World::new();
        w.migrate();
        let owner = w.connect();
        let socket = Token::new(100);
        w.event(skein_io::Event::Connecting { owner, socket });
        w.event(skein_io::Event::Connected { owner });
        w.io.clear();
        w.down(request);
        assert_eq!(w.up.pop(), Some(repository_stop(Why::Down)));
        assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: socket }));
        assert!(w.up.is_empty());
        assert!(w.io.is_empty());
    }
}

#[expect(clippy::disallowed_macros, reason = "the HTTP fixture frames its status and body's length")]
fn refused_connection(status: u16, body: &[u8]) -> (crate::reconcile::Reply, Option<Box<[u8]>>, u32) {
    use skein_forge::forgejo::request::{Repo, Request};
    use skein_lib::{
        Intake,
        stream::{Down, Up},
    };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let mut connection = crate::host::connection::Connection::new(
        Token::new(1),
        &forge(1),
        b"kit",
        &Request::Migrate {
            at: Repo { owner: b"first".as_slice().into(), name: b"copied".as_slice().into() },
            from: b"https://source.invalid/owner/repo".as_slice().into(),
        }
        .into(),
        &fixtures::secret(fixtures::ADMIN),
        &env.limits,
    )
    .expect("migrate connection");
    let mut io = Queue::uncapped();
    connection.link.connecting(Token::new(10), &mut io);
    connection.connect(&env);
    let mut intake = Intake::with_capacity(env.limits.body_bytes_in);
    let head = format!("HTTP/1.1 {status} Reply\r\nContent-Length: {}\r\n\r\n", body.len());
    intake.append(head.as_bytes()).unwrap();
    intake.append(body).unwrap();
    for _ in 0_u32..4096 {
        connection.pump(&env, &mut io);
        if let Some(reply) = connection.result.take() {
            return (reply, connection.words.take(), intake.len());
        }
        match io.pop() {
            Some(skein_io::Request::Stream { down: Down::Demand { read, room }, .. }) => {
                if room > 0 {
                    connection.stream(&env, Up::Room);
                } else if let Some(bytes) = intake.meet(read) {
                    connection.stream(&env, Up::Bytes(bytes));
                }
            }
            Some(skein_io::Request::Stream { down: Down::Send(_) | Down::Finish, .. }) | None => (),
            Some(other) => panic!("unexpected io: {other:?}"),
        }
    }
    panic!("migrate refusal did not settle");
}

#[test]
fn a_migrate_connection_preserves_its_status_with_optional_bounded_words() {
    let cap = usize::try_from(limits().string_bytes).expect("string cap");
    let mut prefix = skein_lib::bytes::zeroed(cap - 1);
    prefix.fill(b'a');
    let long = skein_lib::bytes::concat(&[br#"{"message":""#, &prefix, "界extra\"}".as_bytes()]);
    for (status, body, words) in [
        (422, br#"{"message":"private source"}"#.as_slice(), Some(b"private source".as_slice())),
        (422, b"", None),
        (422, b"not JSON", None),
        (422, br#"{"url":"help"}"#, None),
        (422, br#"{"message":"text"} trailing"#, None),
        (422, long.as_ref(), Some(prefix.as_ref())),
        (401, br#"{"message":"a token must stay private"}"#, None),
    ] {
        let (reply, said, unread) = refused_connection(status, body);
        assert_eq!(reply, crate::reconcile::Reply::Status(status));
        assert_eq!(said.as_deref(), words);
        if status == 401 {
            assert_eq!(usize::try_from(unread).expect("small body"), body.len(), "a 401's body is not read");
        }
    }
}
