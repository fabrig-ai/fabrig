//! A kept restart word holds claims, not channels (domain/engine/fleet.md).
use jig_domain_fleet::{self as fleet, Event, Request};
use jig_domain_link::{Hello, Hosting, Phase};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};

const RUN: Token = Token::new(1);
const ATTEMPT: Token = Token::new(11);
const CHANNEL: Token = Token::new(101);

struct World {
    domain: fleet::Domain,
    env: Env<fleet::Limits>,
    out: Queue<Request>,
    heard: Vec<Request>,
}
impl World {
    fn new() -> Self {
        let limits = fleet::Limits { grace: Duration::from_secs(360), ..jig_fleet_world::LIMITS };
        let mut world = Self {
            domain: jig_fleet_world::fixtures::domain(&limits, jig_fleet_world::fixtures::needs()),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::uncapped(),
            heard: Vec::new(),
        };
        world.step(Event::Restarting { wait: Duration::from_secs(1200) });
        world
    }
    fn drain(&mut self) {
        while let Some(request) = self.out.pop() {
            self.heard.push(request);
        }
        self.domain.reclaim();
    }
    fn step(&mut self, event: Event) {
        fleet::step(&mut self.domain, &self.env, event, &mut self.out);
        self.drain();
    }
    fn at(&mut self, seconds: u64) {
        self.env.now = Time::ZERO.saturating_add(Duration::from_secs(seconds));
        while self.domain.is_due(self.env.now) {
            fleet::fire(&mut self.domain, &self.env, &mut self.out);
        }
        while self.domain.is_ready() {
            fleet::resume(&mut self.domain, &self.env, true, &mut self.out);
        }
        self.drain();
    }
    fn adopt(&mut self, run: Token, attempt: Token) {
        self.step(Event::Adopt {
            reply_to: ReplyTo::new(attempt),
            run,
            attempt,
            kept: 0,
            kind: fleet::HostKind::Worker,
            worked: true,
            holds: Box::new([]),
            grants: Box::new([]),
        });
    }
    fn hello(&mut self, hosting: &[(Token, Token)]) {
        self.step(Event::Hello {
            channel: CHANNEL,
            worker: 0,
            hello: Hello {
                reads_usage: true,
                slots: 3,
                stop_bound: Duration::from_secs(300),
                workstreams: Box::new([]),
                jobs: Box::new([]),
                hosting: hosting.iter().map(|&(run, attempt)| Hosting { run, attempt, phase: Phase::Active }).collect(),
                bounds: jig_fleet_world::fixtures::bounds(),
            },
        });
    }
    fn logins(&mut self) {
        self.step(Event::Logins { channel: CHANNEL, logins: Box::new([]) });
    }
    fn losses(&self) -> usize {
        self.heard.iter().filter(|r| matches!(r, Request::Lost { .. })).count()
    }
    fn discharges(&self) -> usize {
        self.heard.iter().filter(|r| matches!(r, Request::Discharged)).count()
    }
}

#[test]
fn adopted_and_crossed_claims_wait_for_the_word_plus_grace_from_adoption() {
    for crossed in [false, true] {
        let mut world = World::new();
        world.at(7);
        if crossed {
            world.step(Event::Cross { reply_to: ReplyTo::new(ATTEMPT), run: RUN, attempt: ATTEMPT, worked: true });
        } else {
            world.adopt(RUN, ATTEMPT);
        }
        world.step(Event::Loaded);
        world.at(367);
        assert_eq!(world.losses(), 0);
        world.at(1566);
        assert_eq!((world.losses(), world.discharges()), (0, 0));
        world.at(1567);
        assert_eq!((world.losses(), world.discharges()), (1, 1));
        world.step(Event::Loaded);
        world.at(2000);
        assert_eq!(world.discharges(), 1, "the last hold discharges once");
    }
}

#[test]
fn hello_and_loss_before_logins_keep_the_later_of_hold_and_loss_grace() {
    for loss in [500, 1500] {
        let mut world = World::new();
        world.adopt(RUN, ATTEMPT);
        world.step(Event::Loaded);
        world.at(loss);
        world.hello(&[(RUN, ATTEMPT)]);
        assert!(world.heard.iter().any(|r| matches!(r, Request::Placed { .. })));
        assert_eq!(world.discharges(), 0, "a hello does not prove the engine was heard");
        world.step(Event::Lost { channel: CHANNEL });
        let end = 1560.max(loss + 360);
        world.at(end - 1);
        assert_eq!(world.losses(), 0);
        world.at(end);
        assert_eq!((world.losses(), world.discharges()), (1, 1));
    }
}

#[test]
fn logins_clears_only_its_channels_holds_and_later_loss_gets_grace_alone() {
    let mut world = World::new();
    world.adopt(RUN, ATTEMPT);
    world.adopt(Token::new(2), Token::new(22));
    world.step(Event::Loaded);
    world.at(500);
    world.hello(&[(RUN, ATTEMPT)]);
    world.logins();
    assert_eq!(world.discharges(), 0, "the unlisted claim remains held");
    world.step(Event::Lost { channel: CHANNEL });
    world.at(859);
    assert_eq!(world.losses(), 0);
    world.at(860);
    assert_eq!((world.losses(), world.discharges()), (1, 0), "heard claim has grace alone");
    world.at(1560);
    assert_eq!((world.losses(), world.discharges()), (2, 1));
}

#[test]
fn a_replacement_is_not_placed_while_the_old_claim_is_held() {
    let mut world = World::new();
    world.adopt(RUN, ATTEMPT);
    world.step(Event::Loaded);
    world.hello(&[]);
    world.logins();
    world.step(Event::Start {
        reply_to: ReplyTo::new(Token::new(12)),
        run: RUN,
        attempt: Token::new(12),
        workstream: 1,
        kinds: fleet::Kinds::Workers,
        version: fleet::Version { charter: 0, revision: 0 },
        placement: None,
        assignment: fleet::Assignment { turns: Token::new(0), answered: Token::new(0) },
        grants: Box::new([]),
    });
    world.at(1559);
    assert!(!world.heard.iter().any(|r| matches!(r, Request::Assign { .. })));
    world.at(1560);
    assert!(world.heard.iter().any(|r| matches!(r, Request::Assign { attempt, .. } if attempt.raw() == 12)));
    assert_eq!(world.discharges(), 1);
}

#[test]
fn a_channel_already_heard_before_adoption_needs_no_hold() {
    let mut world = World::new();
    world.hello(&[(RUN, ATTEMPT)]);
    world.logins();
    world.adopt(RUN, ATTEMPT);
    assert_eq!(world.discharges(), 0, "discharge waits for Loaded");
    world.step(Event::Loaded);
    assert_eq!(world.discharges(), 1);
    world.step(Event::Lost { channel: CHANNEL });
    world.at(360);
    assert_eq!(world.losses(), 1);
}
