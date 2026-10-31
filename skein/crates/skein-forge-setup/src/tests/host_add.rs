//! Runtime admission uses the constructor's forge checks (forge-setup.md, 6.3).
use super::*;

const SECOND: u16 = 42;
const SECOND_ADDRESS: skein_io::Addr =
    skein_io::Addr::Ip(core::net::SocketAddr::V4(core::net::SocketAddrV4::new(core::net::Ipv4Addr::LOCALHOST, 3001)));

fn second() -> Forge {
    let mut forge = forge(SECOND);
    forge.address = SECOND_ADDRESS;
    forge
}

#[test]
fn an_added_forge_runs_its_plans() {
    let mut w = World::new();
    w.down(Request::Add { forge: second() });
    assert_eq!(w.up.pop(), Some(Event::Added { forge: SECOND, refused: None }));
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    for (run, number, address) in
        [(1, 1, skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into())), (2, SECOND, SECOND_ADDRESS)]
    {
        w.run(run, number);
        assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(run), need: Need::Read(Item::AdminToken) }));
        w.answer(run, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
        w.reply_at(ADMIN_BODY, address);
        w.reply_at(BOT_BODY, address);
        for item in [Item::ApiToken, Item::GitToken] {
            assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(run), need: Need::Read(item) }));
            w.answer(run, Answer::Read(Some(fixtures::secret(b"bot-token"))));
            w.reply_at(BOT_BODY, address);
        }
        assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(run), need: Need::Read(Item::User) }));
        w.answer(run, Answer::Read(Some(fixtures::secret(&42_u64.to_le_bytes()))));
        assert_eq!(w.up.pop(), Some(Event::Done { run: Token::new(run), outcome: Outcome::Nothing }));
        assert!(w.up.is_empty());
        assert!(w.io.is_empty());
    }
    assert_eq!(w.host.next_deadline(), None);
}

#[test]
fn an_add_past_the_limit_or_of_a_held_forge_is_refused() {
    let mut w = World::new();
    let mut duplicate = second();
    duplicate.kind = forge(1).kind;
    w.down(Request::Add { forge: duplicate });
    assert_eq!(w.up.pop(), Some(Event::Added { forge: 1, refused: Some(Unusable::Duplicate { forge: 1 }) }));
    w.down(Request::Add { forge: second() });
    assert_eq!(w.up.pop(), Some(Event::Added { forge: SECOND, refused: None }));
    w.down(Request::Add { forge: forge(99) });
    assert_eq!(w.up.pop(), Some(Event::Added { forge: 99, refused: Some(Unusable::Forges { forges: 2 }) }));
    // The refused duplicate cannot replace the first forge's address, and the
    // accepted second forge still has its run slot after the limit refusal.
    w.send();
    w.connect();
    w.run(2, SECOND);
    assert_eq!(w.up.pop(), Some(Event::Need { run: Token::new(2), need: Need::Read(Item::AdminToken) }));
    w.answer(2, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
    let Some(skein_io::Request::Connect { addr, .. }) = w.io.pop() else {
        panic!("second forge remains admitted");
    };
    assert_eq!(addr, SECOND_ADDRESS);
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
}

#[test]
fn an_add_checks_each_forge_as_the_constructor_does() {
    let mut address = second();
    address.address = skein_io::Addr::Ip(([192, 0, 2, 1], 3000).into());
    let mut name = second();
    name.transport = Transport::Plaintext { name: Some(b"bad/name".as_slice().into()) };
    let mut given = fixtures::given();
    given.forge = SECOND;
    given.bot_login = Box::new([]);
    let mut invalid_given = second();
    invalid_given.kind = Kind::Forgejo(given);
    let mut given = fixtures::given();
    given.forge = SECOND;
    given.limits.reachable = limits().inner + 1;
    let mut reachable = second();
    reachable.kind = Kind::Forgejo(given);
    let mut github = github_settings().forges.get(1).expect("GitHub fixture").clone();
    match &mut github.kind {
        Kind::GitHub(given) => given.limits.pages = u32::MAX,
        Kind::Forgejo(_) => panic!("GitHub fixture"),
    }
    for forge in [address, name, invalid_given, reachable, github] {
        let number = forge.number();
        let mut settings = settings(&[1]);
        let mut forges = List::with_capacity(2);
        for held in settings.forges.into_boxed() {
            forges.push(held).unwrap();
        }
        forges.push(forge.clone()).unwrap();
        settings.forges = forges;
        let refused = Host::new(&limits(), settings, crypto()).expect_err("constructor refuses forge");
        let mut w = World::new();
        w.down(Request::Add { forge });
        assert_eq!(w.up.pop(), Some(Event::Added { forge: number, refused: Some(refused) }));
        // A refusal leaves room for a valid forge with the same number.
        w.down(Request::Add { forge: super::forge(number) });
        assert_eq!(w.up.pop(), Some(Event::Added { forge: number, refused: None }));
        assert!(w.up.is_empty());
        assert!(w.io.is_empty());
    }
}

#[test]
fn an_add_after_close_or_abort_is_ignored() {
    for close in [Request::Close, Request::Abort] {
        for connecting in [false, true] {
            let mut w = World::new();
            let owner = if connecting {
                w.send();
                Some(w.connect())
            } else {
                None
            };
            w.down(close.clone());
            if owner.is_none() {
                assert_eq!(w.up.pop(), Some(Event::Closed));
            } else if let Request::Abort = close {
                assert_eq!(w.up.pop(), Some(stop(1, 1, Why::Down)));
            }
            w.down(Request::Add { forge: second() });
            assert!(w.up.is_empty());
            assert!(w.io.is_empty());
            if let Some(owner) = owner {
                w.event(skein_io::Event::Closed { owner });
                assert_eq!(w.up.pop(), Some(Event::Closed));
                w.down(Request::Add { forge: second() });
            }
            assert!(w.up.is_empty());
            assert!(w.io.is_empty());
            assert_eq!(w.host.next_deadline(), None);
        }
    }
}

#[test]
fn an_empty_host_can_add_a_github_forge_and_run_its_plan() {
    let mut w = World::with(limits(), settings(&[]));
    let forge = github_settings().forges.get(1).expect("GitHub fixture").clone();
    let number = forge.number();
    w.down(Request::Add { forge });
    assert_eq!(w.up.pop(), Some(Event::Added { forge: number, refused: None }));
    w.plan(1, Plan::Manifest { forge: number, target: crate::github::manifest::Target::Personal });
    match w.up.pop().expect("manifest finishes") {
        Event::Done { run, outcome: Outcome::Manifest { state, .. } } => {
            assert_eq!(run, Token::new(1));
            assert_eq!(state.len(), 32);
        }
        other
        @ (Event::Added { .. } | Event::Need { .. } | Event::Done { .. } | Event::Stop { .. } | Event::Closed) => {
            panic!("manifest outcome: {other:?}");
        }
    }
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    assert_eq!(w.host.next_deadline(), None);
}
