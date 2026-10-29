//! The fake's hosted file operations against the real kernel.
use temper_fake_git_hosted::fixtures;
use temper_real_loop_world::git::Disk;
use temper_worker_protocol_git::invocation::{self, Command};

// A read left due before the hosted git process starts owns this reply.
const OLDER_READ: skein_lib::Token = skein_lib::Token::new(42);
// Only the older read remains after hosted git consumes its own reply.
const PENDING_READ_REPLIES: u32 = 1;

fn an_older_read_is_due() -> fixtures::Scenario {
    use skein_fake_forge_domain::{self as domain, api};
    use skein_lib::{Duration, Env, ReplyTo, Time, Wall};
    let scenario = fixtures::scenario();
    {
        let mut forge = scenario.transport.forge.borrow_mut();
        let mut config = forge.config;
        config.limits.calls = 2;
        let mut state = domain::Domain::new(&config, 1);
        domain::repository(
            &mut state,
            &config,
            api::Setup {
                id: 2,
                name: b"acme/app".as_slice().into(),
                default: b"main".as_slice().into(),
                tree: Box::new([api::File {
                    path: b"conflict.txt".as_slice().into(),
                    content: b"base\n".as_slice().into(),
                }]),
                labels: Box::new([]),
                workflows: Box::new([]),
                checks: api::Checks {
                    cues: Box::new([]),
                    contexts: Box::new([]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
                protection: None,
                hooked: false,
            },
        );
        domain::grant(&mut state, b"acme/app", 1, api::Permission::Write);
        *forge = skein_fake_peers::forge::Forge::new(state, config);
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config };
        let skein_fake_peers::forge::Forge { domain, pending, .. } = &mut *forge;
        domain::step(
            domain,
            &env,
            domain::Event::Call {
                reply_to: ReplyTo::new(OLDER_READ),
                user: 1,
                repository: b"acme/app".as_slice().into(),
                op: api::Op::Read(api::Read::Locate),
            },
            pending,
        );
        assert!(pending.is_empty(), "the older read is submitted but not fired");
        assert_eq!(domain.next_deadline(), Some(Time::ZERO));
        forge.now = Time::from_nanos(1);
    }
    scenario
}

/// Hosted git consumes its own reply after an older due call, retaining that call's reply once.
#[test]
fn hosted_git_fires_past_an_older_due_read_and_keeps_its_reply_once() {
    use skein_fake_forge_domain as domain;
    let mut disk = Disk::new(an_older_read_is_due);
    let result = disk.run(
        &Command::Fetch {
            remote: b"http://127.0.0.1:3000/acme/app.git".as_slice().into(),
            source: invocation::Source::Default,
        },
        b"",
    );
    assert_eq!(result.exit, 0);
    let forge = disk.forge();
    assert!(forge.now > skein_lib::Time::ZERO, "git's call follows the older read");
    assert_eq!(forge.pending.len(), PENDING_READ_REPLIES, "git consumes its reply and retains only the older read's");
    let domain::Request::Reply { to, result } = forge.pending.iter().next().expect("older read reply") else {
        panic!("the pending request is the older read's reply");
    };
    assert_eq!(to, &skein_lib::ReplyTo::new(OLDER_READ));
    assert!(result.is_ok());
    assert_eq!(forge.domain.next_deadline(), None, "neither call is waiting for a timer");
}
