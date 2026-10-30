//! Parent drains and accounts for its agent's held entries (protocol/limits.md 3.8).
use super::*;
use crate::tests::{drain_requests, host_limits, saved_window_turn, spawned_chat_with_saved_hello};
use smith_host_domain::{self as host, process};

fn start_without_workspace(domain: &Domain) -> host::Start {
    let mut charter = charter(&domain.config);
    charter.conventions = None;
    charter.llm.account = None;
    domain
        .agent
        .prepare(RunStart {
            live_text: false,
            messages: Box::default(),
            identity: agent::run::Identity::new([1; 16]),
            activation: 1,
            charter,
            workspace: None,
            transcript: None,
            answered: Box::default(),
            grants: Box::default(),
        })
        .expect("bounded start")
}

#[test]
fn one_pass_drains_every_inline_lifecycle_fact() {
    const ATTEMPTS: u32 = 40;
    let limits = crate::tests::limits();
    let mut domain = Domain::new(crate::tests::config(), &limits, 11).expect("inline parent");
    for _ in 0..ATTEMPTS {
        let start = start_without_workspace(&domain);
        command(&mut domain.agent, Time::ZERO, Wall::EPOCH, parent::Event::Spawn { client: Token::new(1), start });
    }
    let Agent::Inline { domain: child, .. } = &domain.agent else { panic!("inline child") };
    assert_eq!(child.pending_slot_facts(), ATTEMPTS, "one opened slot and each remaining spawn refused");
    drain_agent_content(&mut domain);
    let Agent::Inline { domain: child, .. } = &mut domain.agent else { panic!("inline child") };
    assert_eq!(child.pending_slot_facts(), 0);
    assert!(child.pop_slot_fact().is_none());
    assert!(child.backlog() > 0, "the service still owns the native facts");
}

#[test]
fn one_pass_drains_spawned_facts_beyond_the_allowance() {
    let mut limits = crate::tests::limits();
    let child_limits = host::Limits { facts: 1, ..host_limits() };
    limits.agent = AgentLimits::Spawned(child_limits);
    let mut domain = Domain::new(crate::tests::config(), &limits, 11).expect("spawned parent");
    for _ in 0_u32..4 {
        let start = start_without_workspace(&domain);
        command(&mut domain.agent, Time::ZERO, Wall::EPOCH, parent::Event::Spawn { client: Token::new(1), start });
    }
    assert!(spawned_backlog(&domain) > child_limits.facts);
    drain_agent_content(&mut domain);
    assert_eq!(spawned_backlog(&domain), 0);
    let Agent::Spawned { domain: child, .. } = &mut domain.agent else { panic!("spawned child") };
    assert!(child.pop_fact().is_none());
}

#[test]
fn one_content_drain_reclaims_an_ended_inline_slot_with_zero_or_several_records() {
    use smith_inline_agent::{Below, Lower};
    for records in [0, 3] {
        let limits = crate::tests::limits();
        let mut domain = Domain::new(crate::tests::config(), &limits, 11).expect("inline parent");
        let start = start_without_workspace(&domain);
        command(&mut domain.agent, Time::ZERO, Wall::EPOCH, parent::Event::Spawn { client: Token::new(1), start });
        let mut completion = None;
        while let Some(output) = domain.agent.pop() {
            if let Output::Lower { agent, request: Lower::Complete { owner, .. } } = output {
                completion = Some((agent, owner));
            }
        }
        let (handle, owner) = completion.expect("provider request");
        if records > 0 {
            domain.agent.below(
                Time::ZERO,
                Wall::EPOCH,
                handle,
                Below::Completed {
                    owner,
                    completion: agent::llm::Completion {
                        reasoning_dropped: Box::default(),
                        content: Box::new([
                            agent::llm::Said::Text { text: b"one".as_slice().into(), replay: None },
                            agent::llm::Said::Text { text: b"two".as_slice().into(), replay: None },
                        ]),
                        stop: agent::llm::Stop::EndTurn,
                        usage: agent::llm::Usage::ZERO,
                    },
                },
            );
        }
        command(&mut domain.agent, Time::ZERO, Wall::EPOCH, parent::Event::Drop { agent: handle });
        while let Some(output) = domain.agent.pop() {
            if let Output::Lower { agent, request: Lower::Cancel { owner } } = output {
                domain.agent.below(Time::ZERO, Wall::EPOCH, agent, Below::Cancelled { owner });
            }
        }
        while domain.pop_run_fact().is_some() {}
        let Agent::Inline { domain: child, .. } = &domain.agent else { panic!("inline child") };
        assert_eq!(child.pending_content(Token::new(1)), records);
        domain.reclaim();
        let Agent::Inline { domain: child, .. } = &domain.agent else { panic!("inline child") };
        assert_eq!(child.hosted(), 1, "content has not had its empty probe");
        drain_agent_content(&mut domain);
        let Agent::Inline { domain: child, .. } = &domain.agent else { panic!("inline child") };
        assert_eq!(child.pending_content(Token::new(1)), 0);
        assert_eq!(child.pending_slot_facts(), 0);
        domain.reclaim();
        let Agent::Inline { domain: child, .. } = &domain.agent else { panic!("inline child") };
        assert_eq!(child.hosted(), 0, "one drain permits terminal-slot reclaim");
    }
}

fn spawned_backlog(domain: &Domain) -> u32 {
    let Agent::Spawned { domain: child, .. } = &domain.agent else { panic!("spawned child") };
    child.backlog()
}

#[test]
fn a_spawned_childs_facts_are_drained_by_the_local_domain() {
    const TURNS: u32 = 8; // The fixture's configured chat budget.
    let (mut domain, env, mut out, owner) = spawned_chat_with_saved_hello();
    assert_eq!(spawned_backlog(&domain), 0, "Started and Admitted were drained");
    for number in 1..=TURNS {
        let mut turn = saved_window_turn();
        turn.sequence = number;
        step(
            &mut domain,
            &env,
            Event::Process(process::Event::Received {
                owner,
                message: host::Up::Turn {
                    turn: host::Turn {
                        opens_window: true,
                        number,
                        read: Some(Token::new(1)),
                        spent: 0,
                        body: host::TurnValue::new(turn, host_limits().turn_bytes).expect("sealed turn"),
                    },
                },
            }),
            &mut out,
        );
        let requests = drain_requests(&mut out);
        let mut saved = false;
        for request in requests {
            if let Request::SaveTurn { number: saved_number, .. } = request {
                saved |= saved_number == number;
            }
        }
        assert!(saved, "the actual turn reaches the local store");
        assert_eq!(spawned_backlog(&domain), 0, "each turn leaves no host facts behind");
        for event in [Event::TurnSaved { number }, Event::Process(process::Event::Sent { owner })] {
            step(&mut domain, &env, event, &mut out);
            drain_requests(&mut out);
            assert_eq!(spawned_backlog(&domain), 0, "settlement leaves no host facts behind");
        }
    }
}

#[test]
fn the_local_backlog_counts_its_agents_held_entries() {
    let (mut domain, env, _, owner) = spawned_chat_with_saved_hello();
    // Stop before routing this child's fault: one host fact remains for the parent.
    domain.agent.process(env.now, env.wall, process::Event::Malformed { owner });
    let Agent::Spawned { out, .. } = &mut domain.agent else { panic!("spawned child") };
    while out.pop().is_some() {}
    assert_eq!(domain.backlog(), 1, "the child's one Faulted fact counts");
    let Agent::Spawned { out, .. } = &mut domain.agent else { panic!("spawned child") };
    out.push(host::Output::Process(process::Request::Read { owner, process: Token::new(3) }));
    assert_eq!(domain.backlog(), 2, "the fact and the unrouted child output each count once");
    let Agent::Spawned { domain: child, .. } = &mut domain.agent else { panic!("spawned child") };
    assert_eq!(child.pop_fact(), Some(host::Fact::Faulted { client: Token::new(1), fault: host::Fault::Rules }));
    assert_eq!(domain.backlog(), 1, "only the child output remains");
    domain.agent.pop().expect("held output");
    assert_eq!(domain.backlog(), 0);
}
