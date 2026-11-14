//! The shared parent referee rejects lost rights and duplicate terminals.
use skein_lib::Token;
use smith_agent_world::Job;
use smith_host_domain::{self as host, End, RunResult, parent};
use smith_host_world::{
    Seen,
    inline::{self, World},
};

fn spawn(world: &mut World, client: u64, job: Job) {
    world.event(parent::Event::Spawn { client: Token::new(client), start: inline::start(client, job) });
}

#[test]
fn the_common_parent_referee_rejects_a_gone_with_an_open_inline_completion() {
    let mut seen = Seen::default();
    let finding = seen.observe_parent(
        parent::Request::Gone { client: Token::new(1), end: End::Settled, detail: Box::default() },
        true,
        false,
    );
    assert_eq!(finding, Err("Gone retains every lower terminal".into()));
    assert!(seen.gone.is_none());
    assert!(seen.gone_detail.is_none());
}

#[test]
fn the_common_parent_referee_rejects_two_run_answers() {
    let mut seen = Seen::default();
    seen.observe_parent(
        parent::Request::Answered {
            client: Token::new(1),
            answer: host::Answer { read: None, turns: 0, spent: 0, result: RunResult::Parked },
            detail: Box::default(),
        },
        true,
        true,
    )
    .expect("first answer");
    seen.messages.push(Token::new(2));
    seen.sent_messages.insert(Token::new(2));
    let finding = seen.observe_parent(
        parent::Request::Answered {
            client: Token::new(1),
            answer: host::Answer { read: Some(Token::new(2)), turns: 1, spent: 1, result: RunResult::Parked },
            detail: Box::default(),
        },
        true,
        true,
    );
    assert_eq!(finding, Err("one run answer".into()));
    assert_eq!(seen.answer.expect("first answer retained").spent, 0);
    assert_eq!(seen.read, None);
    assert_eq!(seen.messages, [Token::new(2)]);
}

#[test]
fn the_common_parent_referee_rejects_a_gone_with_an_unanswered_parent_call() {
    let mut seen = Seen::default();
    seen.calls.insert(Token::new(2));
    let finding = seen.observe_parent(
        parent::Request::Gone { client: Token::new(1), end: End::Settled, detail: Box::default() },
        true,
        true,
    );
    assert_eq!(finding, Err("Gone retains every parent call".into()));
    assert!(seen.calls.contains(&Token::new(2)));
    assert!(seen.gone.is_none());
}

#[test]
fn the_common_parent_referee_rejects_two_answers_to_one_inline_call() {
    let mut world = World::new(319, inline::limits());
    world.auto_answer = false;
    spawn(&mut world, 1, Job::HostTools);
    for _ in 0..100 {
        if !world.seen[&Token::new(1)].calls.is_empty() {
            break;
        }
        world.at(world.next_deadline().expect("provider alarm"));
    }
    let agent = world.handle(Token::new(1));
    let call = *world.seen[&Token::new(1)].calls.first().expect("relayed call");
    world.event(parent::Event::Answer {
        agent,
        call,
        reply: host::Reply::Host { error: false, body: b"host completed".as_slice().into() },
    });
    let seen = world.seen.get_mut(&Token::new(1)).expect("Started slot");
    assert!(!seen.calls.contains(&call));
    assert_eq!(seen.observe_answer(call), Err("one parent terminal".into()));
    assert!(!seen.calls.contains(&call));
}

#[test]
fn the_common_parent_referee_rejects_a_gone_with_a_seen_machine_right() {
    let mut world = World::new(84, inline::limits());
    world.auto_machine = false;
    world.event(parent::Event::Spawn { client: Token::new(1), start: inline::workspace_start(84, Job::Reporting) });
    assert_eq!(world.pending_machine.len(), 1);
    let seen = world.seen.get_mut(&Token::new(1)).expect("Started slot");
    let finding = seen.observe_parent(
        parent::Request::Gone { client: Token::new(1), end: End::Settled, detail: Box::default() },
        true,
        false,
    );
    assert_eq!(finding, Err("Gone retains every lower terminal".into()));
    assert!(seen.gone.is_none());
    assert_eq!(world.pending_machine.len(), 1);
}

#[test]
fn the_common_parent_referee_rejects_a_duplicate_empty_unread_report() {
    let mut seen = Seen::default();
    seen.observe_parent(parent::Request::Unread { client: Token::new(1), names: Box::default() }, true, true)
        .expect("first empty report");
    let finding =
        seen.observe_parent(parent::Request::Unread { client: Token::new(1), names: Box::default() }, true, true);
    assert_eq!(finding, Err("one unread report".into()));
    assert_eq!(seen.unread, Some(Vec::new()));
}

#[test]
fn the_common_parent_referee_rejects_a_report_that_loses_a_queued_name() {
    let mut seen = Seen::default();
    seen.messages.push(Token::new(2));
    let finding =
        seen.observe_parent(parent::Request::Unread { client: Token::new(1), names: Box::default() }, true, true);
    assert!(
        finding.expect_err("queued name missing").contains("unread names every remaining accepted message in order")
    );
    assert_eq!(seen.messages, [Token::new(2)]);
    assert!(seen.unread.is_none());
}

#[test]
fn the_common_parent_referee_rejects_refusal_after_reporting_the_same_name() {
    let mut seen = Seen::default();
    seen.messages.push(Token::new(2));
    seen.observe_parent(
        parent::Request::Unread { client: Token::new(1), names: Box::from([Token::new(2)]) },
        true,
        true,
    )
    .expect("one unread message");
    let finding = seen.observe_parent(
        parent::Request::MessageRefused {
            client: Token::new(1),
            name: Token::new(2),
            reason: host::MessageRefusal::Ending,
        },
        true,
        true,
    );
    assert_eq!(finding, Err("one message terminal".into()));
    assert_eq!(seen.refused_messages.len(), 0);
    assert_eq!(seen.unread, Some(vec![Token::new(2)]));
}

#[test]
fn the_common_parent_referee_rejects_a_fence_covering_an_unissued_name() {
    let mut seen = Seen::default();
    seen.messages.extend([Token::new(2), Token::new(3)]);
    seen.sent_messages.insert(Token::new(2));
    let finding =
        seen.observe_parent(parent::Request::Waiting { client: Token::new(1), read: Some(Token::new(3)) }, true, true);
    assert_eq!(finding, Err("a covering fence follows message issuance".into()));
    assert_eq!(seen.messages, [Token::new(2), Token::new(3)]);
    assert!(seen.sent_messages.contains(&Token::new(2)));
    assert_eq!(seen.read_messages.len(), 0);
    assert_eq!(seen.read, None);
}

#[test]
fn the_common_parent_referee_rejects_an_admitted_spawn_failure_without_unread() {
    let mut seen = Seen { owner: Some(Token::new(2)), ..Seen::default() };
    let finding = seen.observe_parent(
        parent::Request::Gone { client: Token::new(1), end: End::Unspawned, detail: Box::default() },
        true,
        true,
    );
    assert_eq!(finding, Err("the unread report precedes Gone".into()));
    assert!(seen.gone.is_none());
}
