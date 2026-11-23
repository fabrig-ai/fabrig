//! Proposal, approval and a finite Smith plan across both real roots.

use super::{Queue, ReplyTo, Script, Token, VecDeque, World, authority, call, engine, parties, people, tasks};
use temper_engine_domain_world::ends::{Bound, Expect};

fn planner() -> Script {
    Script {
        cue: b"@goalplan".as_slice().into(),
        turns: Box::new([
            call(b"delegate", br#"{"batch":[{"executor":{"kind":"agent","charter":1},"spec":{"words":"@report Complete the goal's first task"},"contract":{"kind":"report","words":128},"authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":10},"notes":0}}]}"#),
            call(b"wait", b"{}"),
            call(b"finish", br#"{"report":"The goal's first task is complete."}"#),
        ]),
    }
}

fn config(seed: u64) -> engine::Config {
    let mut config = super::config(seed);
    let mut rules = config.core.defaults.rules.clone();
    rules.maximum_run_spend = 10;
    rules.ceiling.tools = authority::Tools(3);
    rules.ceiling.delegation.depth = 2;
    rules.ceiling.delegation.tasks = 4;
    let mut limits = temper_engine_domain_world::walking::limits().authority;
    limits.roles = 3;
    let mut domain = authority::Domain::new(rules, limits).expect("goal authority");
    let mut policy = config.core.seed_projects[0].policy.policy.clone();
    policy.ceiling.tools = authority::Tools(3);
    policy.ceiling.delegation.depth = 2;
    policy.ceiling.delegation.tasks = 4;
    let mut owner = policy.roles[0].clone();
    owner.authority.tools = authority::Tools(3);
    owner.authority.delegation.depth = 2;
    owner.authority.delegation.tasks = 4;
    let mut maintainer = owner.clone();
    maintainer.number = 1;
    let mut member = owner.clone();
    member.number = 2;
    member.authority.budget.spend = 100;
    member.period_spend = 500;
    member.decides = authority::Proposals(0);
    policy.roles = Box::new([owner, maintainer, member]);
    let mut out = Queue::uncapped();
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    config.core.seed_projects[0].policy.policy = domain.policy(1).expect("goal policy").clone();
    config.core.defaults.rules = domain.rules().clone();
    config
}

fn limits(limits: &mut engine::Limits) {
    // One worker slot serializes the planner and its child across durable answers.
    limits.fleet.slots = 1;
    limits.fleet.attempts = 8;
    limits.fleet.calls = 4;
    limits.authority.roles = 3;
    limits.authority.batch = 3;
    limits.tasks.tasks = 5;
    limits.tasks.project_tasks = 5;
    limits.tasks.tree_tasks = 5;
    limits.tasks.delegates = 3;
    limits.tasks.depth = 2;
    limits.tasks.batch = 3;
    limits.tasks.inbox_messages = 7;
    limits.people.people = 4;
    limits.people.sign_ins = 4;
    limits.people.holdings = 4;
    limits.people.requests = 8;
    limits.people.pending = 4;
    limits.people.inbox_tasks = 10;
    limits.people.inbox_recipients = 21;
    limits.brief.briefs = 5;
    limits.brief.sections = 8;
    limits.forge.brief_sections = 80;
    limits.brief.brief_bytes = 1024;
    limits.journal.writes = 4735;
    limits.journal.deliveries = 512;
    limits.journal.held = 1536;
}

fn reply(world: &mut World, token: u64) -> people::Reply {
    for _ in 0..super::STEPS {
        world.tick();
        let replies = world.goal_replies.as_mut().expect("scripted goal party");
        if let Some(at) = replies.iter().position(|(to, _)| to.raw() == token) {
            return replies.remove(at).1;
        }
    }
    panic!("no reply to {token}");
}

fn sign_in(world: &mut World, user: u64) -> (u64, u64) {
    world.root_events.push_back(engine::Event::Party(engine::PartyInput::SignedIn {
        until: None,
        held: None,
        reply_to: ReplyTo::new(Token::new(user)),
        identity: people::Identity {
            key: temper_engine_domain_forge::user_key(1, user),
            login: b"person".as_slice().into(),
            name: b"Person".as_slice().into(),
        },
    }));
    let people::Reply::SignedIn { person, sign_in, .. } = reply(world, user) else { panic!("signed in") };
    (person, sign_in)
}

fn ask(world: &mut World, sign_in: u64, token: u8, ask: people::Ask) -> people::Reply {
    world.root_events.push_back(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::default(),
        reply_to: ReplyTo::new(Token::new(u64::from(token))),
        sign_in,
        key: [token; 16],
        ask,
    }));
    reply(world, u64::from(token))
}

fn proposed_goal() -> (World, u64) {
    let mut world = World::configured_limits(903, planner(), false, config, limits);
    world.party = parties::Party::new(1, parties::Role::Owner, 2000, Box::default());
    world.goal_replies = Some(Vec::new());
    world.scripts = VecDeque::from([
        planner(),
        Script {
            cue: b"@report".as_slice().into(),
            turns: Box::new([call(b"finish", br#"{"report":"The child is complete."}"#)]),
        },
    ]);
    // Open before offering party requests, as the shell does.
    while !world.opened {
        world.tick();
    }
    let (owner, owner_session) = sign_in(&mut world, 7);
    let (maintainer, maintainer_session) = sign_in(&mut world, 8);
    let (member, member_session) = sign_in(&mut world, 9);
    let roles = ask(
        &mut world,
        owner_session,
        34,
        people::Ask::SetRoles {
            project: 1,
            holdings: Box::new([
                people::Holding { person: owner, role: people::Role::Owner },
                people::Holding { person: maintainer, role: people::Role::Maintainer },
                people::Holding { person: member, role: people::Role::Member },
            ]),
        },
    );
    assert!(matches!(roles, people::Reply::Outcome(_)), "{roles:?}");
    let mut proposal = jig_test_profile::fixtures::propose_goal(temper_engine_domain_world::walking::goal(
        1,
        b"@goalplan Ship the goal",
        people::TaskExecutor::Agent { charter: 1 },
        jig_test_profile::fixtures::Goal { budget: 150, priority: 7, deadline: None },
    ));
    let people::Ask::Propose { action: people::Action::Batch(batch), .. } = &mut proposal else {
        panic!("goal proposal")
    };
    batch[0].authority.tools = 3;
    batch[0].authority.delegation =
        people::Delegation { kinds: Box::new([people::Executor::Charter(1)]), tasks: 3, depth: 1 };
    batch[0].contract = people::Contract::Report { words: 128 };
    batch[0].wake.as_mut().expect("goal wake").results = people::ResultsWake::LastOrFailure;
    let people::Reply::Outcome(people::Outcome::Proposed { proposal, .. }) =
        ask(&mut world, member_session, 35, proposal)
    else {
        panic!("proposed before approval")
    };
    assert!(
        !world
            .store
            .rows
            .values()
            .any(|row| matches!(row, engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(_)))))
    );
    let decided = ask(
        &mut world,
        maintainer_session,
        36,
        people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Party(member),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: None,
        },
    );
    assert!(matches!(decided, people::Reply::Outcome(_)), "{decided:?}");
    let key = engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::DecidedProposal(proposal)));
    assert!(matches!(world.store.rows.get(&key),
        Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::DecidedProposal(row))))
        if row.state == tasks::PersonProposalState::Accepted { by: tasks::Party::Person(maintainer) }));
    let goal = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) => Some(row.number),
            _ => None,
        })
        .expect("accepted live goal");
    (world, goal)
}

#[test]
fn a_composed_smith_goal_is_proposed_accepted_planned_and_done() {
    let (mut world, goal) = proposed_goal();
    // The complete story takes 133 ticks / 133 ms; allow three times its work.
    world.ends.expect(goal, Expect::Ends, Bound { minutes: 1, steps: 400 }).expect("nonzero goal bound");
    world.run();
    let key = engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(goal)));
    assert!(matches!(world.store.rows.get(&key),
        Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))))
        if matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(_)))));
    assert_eq!(world.observed.assignments, 3, "planner, child, resumed planner");
    eprintln!("goal {goal}: {} ns, {} ticks", world.env.now.as_nanos(), world.iteration);
}

#[test]
fn a_composed_goal_past_its_step_bound_is_a_named_finding() {
    let (mut world, goal) = proposed_goal();
    world.ends.expect(goal, Expect::Ends, Bound { minutes: 1, steps: 1 }).expect("nonzero goal bound");
    let finding = world.try_run().expect_err("the live goal cannot end in one tick");
    assert!(finding.contains(&format!("engine task {goal}:")), "{finding}");
    assert!(finding.contains("within 1 minutes and 1 steps"), "{finding}");
    assert_eq!(world.try_run(), Err(finding), "the first finding is retained");
}
