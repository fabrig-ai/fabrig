//! Idle usage crosses the core's account, fleet and worker ports without a run.
use super::{CONFIGURED_SOURCE, OTHER_WATCH, OWN_WATCH, program_run, take, watch};
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_host::LinkUp;
use jig_domain_link::{SourceName, UsageOutcome, UsageWindow};
use jig_test_domain as root;
use skein_lib::{Duration, Token, Wall};

#[test]
fn idle_reads_reach_the_link_and_answers_mark_only_changed_facts() {
    const CHANNEL: Token = Token::new(7); // The world's first worker.
    const ACCOUNT: u32 = 0;
    const QUIET: Duration = Duration::from_secs(30 * 60); // accounts.md, 3.7.
    let mut world = program_run(185);
    // Connect the root's link port; the fleet already holds this worker's hello.
    world.send(root::Event::Link {
        channel: CHANNEL,
        link: root::link::Event::Hello {
            worker: 0,
            hello: jig_domain_link::Hello {
                reads_usage: true,
                jobs: Box::new([]),
                bounds: jig_test_profile::fixtures::bounds(),
                stop_bound: Duration::from_millis(100),
                slots: 2,
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        },
    });
    world.client_peer = Some(jig_core_world::client_peer::ClientPeer::default());
    let window = UsageWindow {
        label: b"five_hour".as_slice().into(),
        window: accounts::FactWindow {
            length: Some(Duration::from_secs(18_000)),
            used: Some(1200),
            resets: None,
            state: accounts::WINDOW_OPEN,
        },
    };
    for (outcome, changed) in [
        (UsageOutcome::Read(Box::new([window.clone()])), true),
        (UsageOutcome::Read(Box::new([window.clone()])), false),
        (UsageOutcome::NotSignedIn, false),
    ] {
        let (now, wall) = world.clocks();
        world.set_clock(now.saturating_add(QUIET), Wall::from_nanos(wall.as_nanos() + QUIET.as_nanos()));
        world.send(root::Event::Timer(core::Timer::Account));
        let (channel, read, source) = *world.usage_reads.last().expect("LinkDown::ReadUsage reached the worker");
        assert_eq!((channel, source), (CHANNEL, SourceName::new(b"personal").expect("source name")));
        // The fixture's one-minute sign-in expires before the quiet clock. Sign in anew,
        // then watch, so the answer is isolated from unrelated deadlines without raising a timeout.
        let peer = world.client_peer.take();
        world.send(jig_core_world::fixtures::sign_in(Token::new(101), 0, &7_u64.to_be_bytes(), b"person", b"Person"));
        world.client_peer = peer;
        let signed = world.sign_in.expect("fresh sign-in");
        watch(&mut world, signed, OWN_WATCH, core::documents::Watched::Accounts);
        watch(&mut world, signed, OTHER_WATCH, core::documents::Watched::Frame);
        take(&mut world, OWN_WATCH);
        take(&mut world, OTHER_WATCH);
        world.client_peer.as_mut().expect("peer").delivered.clear();
        let commits = world.commits.len();
        world.send(root::Event::Link { channel, link: root::link::Event::Up(LinkUp::UsageRead { read, outcome }) });
        assert_eq!(world.domain.accounts().account_facts(ACCOUNT).len(), 1, "answer becomes a fact");
        let deliveries = &world.client_peer.as_ref().expect("peer").delivered;
        for watcher in [OWN_WATCH, OTHER_WATCH] {
            let marks = deliveries
                .iter()
                .filter(|(_, delivery)| {
                    matches!(delivery,
                root::Delivery::Core(core::Held::Delivery { watcher: got, .. }) if got.raw() == watcher)
                })
                .count();
            assert_eq!(marks, usize::from(changed), "answer marks watcher {watcher} only when its fact changed");
        }
        assert_eq!(world.commits.len() - commits, usize::from(changed), "equal and failed answers store nothing");
        let facts = world.domain.accounts().account_facts(ACCOUNT);
        assert_eq!(facts.len(), 1);
        assert_eq!(
            (facts[0].kind, facts[0].origin, facts[0].source),
            (accounts::KIND_USAGE, accounts::ORIGIN_USAGE_READ, Some(CONFIGURED_SOURCE))
        );
        for watcher in [OWN_WATCH, OTHER_WATCH] {
            world.send(root::Event::Core(core::Event::Unwatch { watcher: Token::new(watcher) }));
        }
    }
    assert_eq!(world.usage_reads.len(), 3, "each read ends before its next quiet clock");
}
