//! Exercise hosted runs through the public boundary (domain/hosts/hub.md).

use crate::{LinkDown, LinkUp};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Time, Token, Wall};

use crate::{
    Ask, CallAnswer, Delivery, DeliveryOutcome, Event, Failure, Finish, Hub, Invalid, Limits, MessageRefusal,
    Preparation, Reason, Refusal, Refused, Request, RunAssignment, Work, Workspace, step,
};

const LIMITS: Limits = Limits {
    slots: 2,
    accounts: 2,
    program_sources: 12,
    source_interval: skein_lib::Duration::from_secs(60),
    source_timeout: skein_lib::Duration::from_secs(10),
    charter_bytes: 64,
    transcript_bytes: 64,
    delivery_evidence_bytes: 64,
    turn_bytes: 64,
    outcome_bytes: 32,
    detail_bytes: 8,
    held: 2,
    inbox: 4,
    event_bytes: 16,
    run_calls: 2,
    facts: 64,
    told: 2,
    turns: 2,
    turn_queue_bytes: 128,
    piece_bytes: 4096,
};

struct Harness {
    domain: Hub,
    env: Env<Limits>,
    out: Queue<Request>,
}

impl Harness {
    fn new() -> Self {
        Self {
            domain: Hub::new(&LIMITS),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS },
            out: Queue::uncapped(),
        }
    }

    fn step(&mut self, event: Event) -> Box<[Request]> {
        step(&mut self.domain, &self.env, event, &mut self.out);
        let mut requests = List::with_capacity(self.out.len());
        while let Some(request) = self.out.pop() {
            requests.push(request).expect("bounded output");
        }
        requests.into_boxed()
    }

    fn assign(&mut self, run: u64) -> (Token, Token) {
        let assignment = assignment(run);
        let requests = self.step(fixtures::assign(assignment));
        let [Request::Prepare { owner, workspace }] = &*requests else { panic!("expected prepare: {requests:?}") };
        assert_eq!(
            *workspace,
            Workspace { workstream: run, items: Token::new(run.checked_add(100).expect("small run")) }
        );
        (*owner, Token::new(run.checked_add(200).expect("small run")))
    }

    fn live(&mut self, run: u64) -> (Token, Token, Token) {
        let (owner, workspace) = self.assign(run);
        let requests = self.step(Event::Prepared { owner, workspace });
        let [Request::Agent(crate::ToAgent::Start { owner: started, start: prepared })] = &*requests else {
            panic!("expected start: {requests:?}")
        };
        assert_eq!((*started, prepared.workspace), (owner, Some(workspace)));
        let agent = Token::new(run.checked_add(300).expect("small run"));
        assert!(self.step(Event::Agent(crate::FromAgent::Started { owner, agent })).is_empty());
        (owner, workspace, agent)
    }
}

fn assignment(run: u64) -> RunAssignment {
    RunAssignment {
        identity: [0; 16],
        run: Token::new(run),
        attempt: Token::new(run.checked_add(1000).expect("small run")),
        workspace: Some(Workspace { workstream: run, items: Token::new(run.checked_add(100).expect("small run")) }),
        save: true,
        live: crate::LiveAsk::NONE,
        charter: Box::from(&b"charter"[..]),
        grants: Box::new([]),
    }
}

#[test]
fn a_run_prepares_and_starts_from_opaque_workspace_items() {
    let mut h = Harness::new();
    let (owner, workspace, agent) = h.live(1);
    assert_eq!(h.domain.hosting(owner).expect("run is hosted").run, Token::new(1));
    assert_eq!(workspace, Token::new(201));
    assert_eq!(agent, Token::new(301));
}

#[test]
fn an_itemless_run_starts_and_ends_without_a_workspace() {
    let mut h = Harness::new();
    let mut assignment = assignment(1);
    assignment.workspace = None;
    let run = assignment.run;
    let attempt = assignment.attempt;
    let requests = h.step(fixtures::assign(assignment));
    let [Request::Agent(crate::ToAgent::Start { owner, start })] = &*requests else {
        panic!("an itemless run starts without preparation: {requests:?}");
    };
    assert_eq!(start.workspace, None);
    let owner = *owner;
    let agent = Token::new(301);
    assert!(h.step(Event::Agent(crate::FromAgent::Started { owner, agent })).is_empty());
    let call = Token::new(42);
    assert_eq!(
        &*h.step(fixtures::called(owner, call, fixtures::deliver(Box::from(&b"work"[..])))),
        [Request::Agent(crate::ToAgent::Answer {
            agent,
            call: Box::from(call.raw().to_be_bytes()),
            answer: CallAnswer::Unavailable
        })]
    );
    assert_eq!(
        &*h.step(fixtures::finished(owner, Finish::Ended { outcome: Box::from(&b"done"[..]) })),
        [Request::Agent(crate::ToAgent::Cancel { agent })]
    );
    let requests = h.step(Event::Agent(crate::FromAgent::Gone { owner, detail: Box::new([]) }));
    let [
        Request::Up(LinkUp::Answer {
            run: answered,
            attempt: answered_attempt,
            answer: crate::RunAnswer { ending: crate::Ending::Ended { work, .. }, .. },
            ..
        }),
    ] = &*requests
    else {
        panic!("the itemless run answers without workspace requests: {requests:?}")
    };
    assert_eq!((*answered, *answered_attempt, work.left, work.saved), (run, attempt, None, None));
}

#[test]
fn turns_stay_in_the_host_until_their_own_ack_and_credit_returns() {
    let mut h = Harness::new();
    let mut assignment = assignment(1);
    assignment.workspace = None;
    let (run, attempt) = (assignment.run, assignment.attempt);
    let requests = h.step(Event::Down(LinkDown::Assign {
        assignment: crate::Assignment {
            messages: Box::new([]),
            assignment,
            turns: Box::new([]),
            answered: Box::new([]),
        },
    }));
    let [Request::Agent(crate::ToAgent::Start { owner, start })] = &*requests else {
        panic!("an itemless run starts: {requests:?}");
    };
    assert_eq!(start.workspace, None);
    let owner = *owner;
    let agent = Token::new(301);
    assert!(h.step(Event::Agent(crate::FromAgent::Started { owner, agent })).is_empty());
    for number in 1..=2 {
        let usage = u64::from(number == 1);
        let turn = crate::Turn {
            opens_window: false,
            refused: Box::new([]),
            turn: number,
            spent: usage,
            read: None,
            body: Box::from([7_u8; 64]),
        };
        let requests = h.step(Event::Agent(crate::FromAgent::Turn { owner, turn }));
        let [
            Request::Up(LinkUp::Turn { turn: sent, .. }),
            Request::Agent(crate::ToAgent::Acknowledge { agent: credited, turn: ack }),
        ] = &*requests
        else {
            panic!("a turn is sent with the next credit: {requests:?}");
        };
        assert_eq!((*credited, *ack), (agent, number));
        assert_eq!(sent.turn, number);
        assert_eq!(sent.spent, usage, "spend passes through without a host decision");
    }
    assert_eq!(h.domain.retained_turns(), 2);
    assert!(h.step(Event::Down(LinkDown::TurnKept { run, attempt: Token::new(999), turn: 1 })).is_empty());
    assert!(h.step(Event::Down(LinkDown::TurnKept { run, attempt, turn: 3 })).is_empty());
    assert_eq!(h.domain.retained_turns(), 2);
    assert!(h.step(Event::Down(LinkDown::TurnKept { run, attempt, turn: 1 })).is_empty());
    assert_eq!(h.domain.retained_turns(), 1);
    assert!(h.step(Event::Down(LinkDown::TurnKept { run, attempt, turn: 1 })).is_empty());
    assert_eq!(h.domain.turn(run, attempt, 2).expect("second turn remains").body, Box::from([7_u8; 64]));
}

#[test]
fn a_turn_credit_bound_must_hold_a_maximum_body() {
    assert!(!crate::valid_hub(&Limits { turn_queue_bytes: 63, ..LIMITS }));
}

#[test]
fn a_bad_charter_is_refused_before_a_workspace_is_touched() {
    let mut h = Harness::new();
    let mut assignment = assignment(1);
    assignment.charter = Box::from([0_u8; 65]);
    let requests = h.step(fixtures::assign(assignment));
    let [
        Request::Up(LinkUp::Answer {
            answer: crate::RunAnswer { ending: crate::Ending::Refused(Refusal::Invalid(Invalid::Charter)), .. },
            ..
        }),
    ] = &*requests
    else {
        panic!("expected invalid refusal: {requests:?}")
    };
}

#[test]
fn a_preparation_failure_answers_without_starting_an_agent() {
    let mut h = Harness::new();
    let (owner, _) = h.assign(1);
    let requests = h.step(Event::Unprepared {
        owner,
        failure: Preparation::Permanent { resource: Some(Token::new(77)) },
        detail: Box::new([]),
    });
    let [
        Request::Up(LinkUp::Answer {
            answer:
                crate::RunAnswer {
                    ending:
                        crate::Ending::Failed {
                            failure: Failure::Unprepared(Preparation::Permanent { resource: Some(resource) }),
                            ..
                        },
                    ..
                },
            ..
        }),
    ] = &*requests
    else {
        panic!("expected preparation failure: {requests:?}")
    };
    assert_eq!(*resource, Token::new(77));
}

#[test]
fn a_delivery_is_answered_after_the_workspace_reports_what_it_left() {
    let mut h = Harness::new();
    let (owner, workspace, agent) = h.live(1);
    let call = Token::new(8);
    let requests = h.step(fixtures::called(owner, call, fixtures::deliver(Box::from(&b"ship"[..]))));
    let [Request::Deliver { owner: delivery, workspace: target, .. }] = &*requests else {
        panic!("expected workspace delivery: {requests:?}")
    };
    assert_eq!(*target, workspace);
    let left = Token::new(900);
    let delivered = Delivery { outcome: DeliveryOutcome::Delivered, left, changed: true };
    let requests = h.step(Event::Delivered { owner: *delivery, delivery: delivered });
    assert_eq!(
        &*requests,
        [Request::Agent(crate::ToAgent::Answer {
            agent,
            call: Box::from(call.raw().to_be_bytes()),
            answer: CallAnswer::Delivered(delivered)
        })]
    );
    let requests = h.step(fixtures::finished(owner, Finish::Ended { outcome: Box::from(&b"done"[..]) }));
    assert_eq!(&*requests, [Request::Agent(crate::ToAgent::Cancel { agent })]);
    let requests = h.step(Event::Agent(crate::FromAgent::Gone { owner, detail: Box::new([]) }));
    let [
        Request::Release { workspace: released },
        Request::Up(LinkUp::Answer {
            answer: crate::RunAnswer { ending: crate::Ending::Ended { work, .. }, .. }, ..
        }),
    ] = &*requests
    else {
        panic!("expected release and answer: {requests:?}")
    };
    assert_eq!((*released, work.left, work.saved), (workspace, Some(left), None));
}

#[test]
fn stopping_waits_for_a_delivery_even_after_the_agent_is_gone() {
    let mut h = Harness::new();
    let (owner, workspace, agent) = h.live(1);
    let call = Token::new(42);
    let requests = h.step(fixtures::called(owner, call, fixtures::deliver(Box::from(&b"ship"[..]))));
    let [Request::Deliver { owner: delivery, .. }] = &*requests else { panic!("delivery is in flight: {requests:?}") };
    let delivery = *delivery;
    assert_eq!(
        &*h.step(fixtures::finished(owner, Finish::Ended { outcome: Box::from(&b"done"[..]) })),
        [Request::Agent(crate::ToAgent::Cancel { agent })]
    );
    assert!(h.step(Event::Agent(crate::FromAgent::Gone { owner, detail: Box::new([]) })).is_empty());
    let changed = Delivery { outcome: DeliveryOutcome::Delivered, left: Token::new(900), changed: true };
    let requests = h.step(Event::Delivered { owner: delivery, delivery: changed });
    let [
        Request::Agent(crate::ToAgent::Answer {
            agent: answered_agent,
            call: answered_call,
            answer: CallAnswer::Delivered(outcome),
        }),
        Request::Release { workspace: released },
        Request::Up(LinkUp::Answer {
            answer: crate::RunAnswer { ending: crate::Ending::Ended { work, .. }, .. }, ..
        }),
    ] = &*requests
    else {
        panic!("the delivery settles before release and answer: {requests:?}")
    };
    assert_eq!((*answered_agent, fixtures::callback(answered_call), *outcome), (agent, call, changed));
    assert_eq!((*released, work.left), (workspace, Some(changed.left)));
}

#[test]
fn live_agent_facts_wait_within_the_hosts_bound_and_drop_when_full() {
    let mut h = Harness::new();
    let (owner, _, _) = h.live(1);
    while h.domain.pop_activity().is_some() {}
    let lost = h.domain.activity_lost();
    for count in 1..=3 {
        assert!(
            h.step(Event::Agent(crate::FromAgent::Activity {
                owner,
                activity: crate::AgentActivity::Count {
                    kind: crate::CountKind::TextArrived,
                    elapsed: skein_lib::Duration::ZERO,
                    count
                }
            }))
            .is_empty()
        );
    }
    assert_eq!(h.domain.activity_lost(), lost + 1);
    for count in 1..=2 {
        assert_eq!(
            h.domain.pop_activity(),
            Some(LinkUp::Activity {
                run: Token::new(1),
                attempt: Token::new(1001),
                activity: crate::AgentActivity::Count {
                    kind: crate::CountKind::TextArrived,
                    elapsed: skein_lib::Duration::ZERO,
                    count
                }
            })
        );
    }
    assert_eq!(h.domain.pop_activity(), None);
}

#[test]
fn cancelling_a_preparing_run_waits_for_the_workspace_terminal() {
    let mut h = Harness::new();
    let (owner, _) = h.assign(1);
    let requests = h.step(Event::Down(LinkDown::Cancel { run: Token::new(1), attempt: Token::new(1001) }));
    assert_eq!(&*requests, [Request::Abort { owner }]);
    let requests = h.step(Event::Unprepared { owner, failure: Preparation::Transient, detail: Box::new([]) });
    let [
        Request::Up(LinkUp::Answer {
            answer:
                crate::RunAnswer {
                    ending: crate::Ending::Failed { failure: Failure::Cancelled(Reason::Engine), .. }, ..
                },
            ..
        }),
    ] = &*requests
    else {
        panic!("expected cancelled answer: {requests:?}")
    };
}

#[test]
fn held_finish_messages_reach_the_agent_after_start() {
    for finish_requested in [false, true] {
        let mut h = Harness::new();
        let (owner, workspace) = h.assign(1);
        let name = Token::new(42);
        h.step(Event::Down(LinkDown::Message {
            run: Token::new(1),
            attempt: Token::new(1001),
            name,
            sender: Box::from(b"requester".as_slice()),
            words: Box::from(b"wrap up".as_slice()),
            finish_requested,
        }));
        h.step(Event::Prepared { owner, workspace });
        let agent = Token::new(301);
        assert_eq!(
            &*h.step(Event::Agent(crate::FromAgent::Started { owner, agent })),
            &[Request::Agent(crate::ToAgent::Message {
                agent,
                name,
                sender: Box::from(b"requester".as_slice()),
                words: Box::from(b"wrap up".as_slice()),
                finish_requested,
            })]
        );
    }
}

#[test]
fn messages_held_during_a_failed_prepare_are_returned_by_name() {
    let mut h = Harness::new();
    let (owner, _) = h.assign(1);
    let run = Token::new(1);
    let attempt = Token::new(1001);
    let name = Token::new(42);
    assert!(h.step(fixtures::inbound(run, attempt, name, Box::from(&b"question"[..]))).is_empty());
    let requests = h.step(Event::Unprepared { owner, failure: Preparation::Transient, detail: Box::new([]) });
    assert_eq!(
        &*requests,
        [Request::Up(LinkUp::Answer {
            run,
            attempt,
            answer: crate::RunAnswer {
                read: None,
                refused: Box::from([Refused { name, reason: MessageRefusal::Ending }]),
                turns: 0,
                spent: 0,
                ending: crate::Ending::Failed {
                    failure: Failure::Unprepared(Preparation::Transient),
                    detail: Box::new([]),
                    work: Work { left: None, saved: None },
                }
            },
        }),]
    );
}

#[test]
fn cancellation_returns_messages_held_before_the_agent_starts() {
    let mut h = Harness::new();
    let (owner, workspace) = h.assign(1);
    let run = Token::new(1);
    let attempt = Token::new(1001);
    let name = Token::new(42);
    let requests = h.step(Event::Prepared { owner, workspace });
    let [Request::Agent(crate::ToAgent::Start { .. })] = &*requests else { panic!("the agent starts: {requests:?}") };
    assert!(h.step(fixtures::inbound(run, attempt, name, Box::from(&b"question"[..]))).is_empty());
    assert_eq!(&*h.step(Event::Down(LinkDown::Cancel { run, attempt })), []);
    let agent = Token::new(301);
    assert_eq!(
        &*h.step(Event::Agent(crate::FromAgent::Started { owner, agent })),
        [Request::Agent(crate::ToAgent::Cancel { agent })]
    );
    let requests = h.step(Event::Agent(crate::FromAgent::Gone { owner, detail: Box::new([]) }));
    assert_eq!(&*requests, [Request::Save { owner, workspace }]);
    let requests = h.step(Event::Saved { owner, at: None });
    let [
        Request::Release { workspace: released },
        Request::Up(LinkUp::Answer {
            answer: crate::RunAnswer { ending: crate::Ending::Failed { failure, .. }, .. },
            ..
        }),
    ] = &*requests
    else {
        panic!("the run releases and answers after save: {requests:?}")
    };
    assert_eq!((*released, *failure), (workspace, Failure::Cancelled(Reason::Engine)));
}

mod fixtures {
    // Host records used by the lifecycle scripts. Callback tokens are encoded
    // as opaque names; names are opaque bytes.
    use crate::{Ask, Assignment, Event, Finish, LinkDown, RunAssignment};
    use skein_lib::{Reader, Token};
    const ANALYST: u32 = 3;
    pub fn account_grant(generation: u64) -> crate::Grant {
        crate::Grant {
            form: crate::Form::Credential(crate::Scheme::Bearer),
            account: ANALYST,
            generation,
            valid: skein_lib::Duration::from_secs(30),
        }
    }

    pub fn application_grant(generation: u64) -> crate::Grant {
        crate::Grant { form: crate::Form::Application, ..account_grant(generation) }
    }
    pub fn program_grant(generation: u64) -> crate::Grant {
        crate::Grant {
            form: crate::Form::Program(crate::SourceName::new(b"claude").expect("source name")),
            ..account_grant(generation)
        }
    }
    pub fn assign(assignment: RunAssignment) -> Event {
        Event::Down(LinkDown::Assign {
            assignment: Assignment { messages: Box::new([]), assignment, turns: Box::new([]), answered: Box::new([]) },
        })
    }
    pub fn inbound(run: Token, attempt: Token, name: Token, words: Box<[u8]>) -> Event {
        Event::Down(LinkDown::Message { run, attempt, name, sender: Box::new([]), words, finish_requested: false })
    }
    pub fn called(owner: Token, call: Token, ask: Ask) -> Event {
        Event::Agent(crate::FromAgent::Called { owner, call: Box::from(call.raw().to_be_bytes()), ask })
    }

    pub fn finished(owner: Token, finish: Finish) -> Event {
        Event::Agent(crate::FromAgent::Finished { read: None, owner, turns: 0, spent: 0, finish })
    }
    pub fn deliver(title: Box<[u8]>) -> Ask {
        Ask::Deliver { title, body: Box::new([]) }
    }

    pub fn callback(name: &[u8]) -> Token {
        Token::new(Reader::new(name).u64().expect("encoded callback"))
    }
}

mod pieces;
mod progress;

fn assigned_grants(h: &mut Harness, grants: Box<[crate::Grant]>) -> (Token, Box<crate::Start>) {
    let mut assignment = assignment(1);
    assignment.workspace = None;
    assignment.grants = grants;
    let requests = h.step(fixtures::assign(assignment));
    assert_eq!(requests.len(), 1);
    let mut requests = requests.into_iter();
    let Some(Request::Agent(crate::ToAgent::Start { owner, start })) = requests.next() else {
        panic!("expected start")
    };
    (owner, start)
}

fn renewal(h: &mut Harness, grant: crate::Grant) -> Box<[Request]> {
    h.step(Event::Down(LinkDown::Grant { run: Token::new(1), attempt: Token::new(1001), grant }))
}

fn kept_grants(h: &Harness, owner: Token) -> &[crate::Grant] {
    h.domain.hosted.get(skein_lib::Id::from_token(owner)).expect("hosted run").grants()
}

#[test]
fn a_grant_keeps_its_form_through_the_start() {
    let mut h = Harness::new();
    let (_, start) = assigned_grants(&mut h, Box::new([fixtures::account_grant(1)]));
    assert_eq!(&*start.grants, &[fixtures::account_grant(1)]);
}

#[test]
fn a_program_grant_reaches_the_agents_start_and_renewals() {
    let mut h = Harness::new();
    let (owner, start) = assigned_grants(&mut h, Box::new([fixtures::program_grant(1)]));
    assert_eq!(&*start.grants, &[fixtures::program_grant(1)]);
    let agent = Token::new(301);
    h.step(Event::Agent(crate::FromAgent::Started { owner, agent }));
    assert_eq!(
        &*renewal(&mut h, fixtures::program_grant(2)),
        &[Request::Agent(crate::ToAgent::Grant { agent, grant: fixtures::program_grant(2) })]
    );
}

#[test]
fn a_program_renewal_while_starting_reaches_the_agent() {
    let mut h = Harness::new();
    let (owner, _) = assigned_grants(&mut h, Box::new([fixtures::program_grant(1)]));
    renewal(&mut h, fixtures::program_grant(2));
    let agent = Token::new(301);
    assert_eq!(
        &*h.step(Event::Agent(crate::FromAgent::Started { owner, agent })),
        &[Request::Agent(crate::ToAgent::Grant { agent, grant: fixtures::program_grant(2) })]
    );
}

#[test]
fn two_grants_of_one_number_in_different_tables_are_admitted() {
    let mut h = Harness::new();
    let (owner, _) = assigned_grants(&mut h, Box::new([fixtures::account_grant(1), fixtures::application_grant(1)]));
    assert_eq!(kept_grants(&h, owner), &[fixtures::account_grant(1), fixtures::application_grant(1)]);
}

#[test]
fn an_applications_grant_is_kept_and_never_handed_to_the_agent() {
    let mut h = Harness::new();
    let (owner, start) = assigned_grants(&mut h, Box::new([fixtures::application_grant(1)]));
    assert!(start.grants.is_empty());
    assert!(renewal(&mut h, fixtures::application_grant(2)).is_empty());
    assert!(h.step(Event::Agent(crate::FromAgent::Started { owner, agent: Token::new(301) })).is_empty());
    assert!(renewal(&mut h, fixtures::application_grant(3)).is_empty());
    assert_eq!(kept_grants(&h, owner), &[fixtures::application_grant(3)]);
}

#[test]
fn an_accounts_grant_renews_the_agents_grant() {
    let mut h = Harness::new();
    let (owner, _) = assigned_grants(&mut h, Box::new([fixtures::account_grant(1), fixtures::application_grant(1)]));
    let agent = Token::new(301);
    h.step(Event::Agent(crate::FromAgent::Started { owner, agent }));
    let newest = crate::Grant { form: crate::Form::Credential(crate::Scheme::Key), ..fixtures::account_grant(2) };
    assert_eq!(&*renewal(&mut h, newest), &[Request::Agent(crate::ToAgent::Grant { agent, grant: newest })]);
    assert_eq!(kept_grants(&h, owner), &[newest, fixtures::application_grant(1)]);
}

#[test]
fn a_renewal_while_starting_sends_the_agent_accounts_grants_only() {
    let mut h = Harness::new();
    let (owner, _) = assigned_grants(&mut h, Box::new([fixtures::account_grant(1), fixtures::application_grant(1)]));
    assert!(renewal(&mut h, fixtures::account_grant(2)).is_empty());
    assert!(renewal(&mut h, fixtures::application_grant(2)).is_empty());
    let agent = Token::new(301);
    assert_eq!(
        &*h.step(Event::Agent(crate::FromAgent::Started { owner, agent })),
        &[Request::Agent(crate::ToAgent::Grant { agent, grant: fixtures::account_grant(2) })]
    );
}

#[test]
fn an_equal_or_older_generation_does_not_renew_an_agent() {
    let mut h = Harness::new();
    let (owner, _) = assigned_grants(&mut h, Box::new([fixtures::account_grant(2)]));
    h.step(Event::Agent(crate::FromAgent::Started { owner, agent: Token::new(301) }));
    assert!(renewal(&mut h, fixtures::account_grant(2)).is_empty());
    assert!(renewal(&mut h, fixtures::account_grant(1)).is_empty());
}

#[test]
fn an_agent_cannot_supply_the_hubs_turn_refusals() {
    let mut h = Harness::new();
    let (owner, workspace, agent) = h.live(1);
    let turn = crate::Turn {
        turn: 1,
        spent: 0,
        read: None,
        opens_window: false,
        refused: Box::from([Refused { name: Token::new(42), reason: MessageRefusal::Full }]),
        body: Box::new([]),
    };
    assert_eq!(
        &*h.step(Event::Agent(crate::FromAgent::Turn { owner, turn })),
        &[Request::Agent(crate::ToAgent::Cancel { agent })]
    );
    assert_eq!(
        &*h.step(Event::Agent(crate::FromAgent::Gone { owner, detail: Box::new([]) })),
        &[Request::Save { owner, workspace }]
    );
    let requests = h.step(Event::Saved { owner, at: None });
    let [Request::Release { .. }, Request::Up(LinkUp::Answer { answer, .. })] = &*requests else {
        panic!("one failed answer")
    };
    assert_eq!(
        answer.ending,
        crate::Ending::Failed {
            failure: Failure::Agent(crate::AgentFailure::Rules),
            detail: Box::new([]),
            work: Work { left: None, saved: None }
        }
    );
    assert!(answer.refused.is_empty());
}

fn named_relay(h: &mut Harness, owner: Token, name: &[u8]) {
    let requests = h.step(Event::Agent(crate::FromAgent::Called {
        owner,
        call: Box::from(name),
        ask: Ask::Relay {
            tool: Box::from(&b"read"[..]),
            writes: false,
            input: Box::from(&b"{}"[..]),
            deadline: skein_lib::Duration::from_secs(1),
        },
    }));
    let [Request::Up(LinkUp::Call { call, .. })] = &*requests else {
        panic!("expected named relay: {requests:?}");
    };
    assert_eq!(call.as_ref(), name);
}

fn named_answer(run: u64, name: &[u8]) -> Event {
    Event::Down(LinkDown::Relayed {
        run: Token::new(run),
        attempt: Token::new(run.checked_add(1000).expect("small run")),
        call: Box::from(name),
        answer: crate::Relayed::Answer { error: false, text: Box::from(&b"record"[..]) },
    })
}

#[test]
fn a_named_relay_answer_reaches_its_waiting_call() {
    let mut h = Harness::new();
    let (first, _, _) = h.live(1);
    let (second, _, agent) = h.live(2);
    named_relay(&mut h, first, b"read-call");
    named_relay(&mut h, second, b"read-call");
    let requests = h.step(named_answer(2, b"read-call"));
    assert_eq!(
        &*requests,
        &[Request::Agent(crate::ToAgent::Answer {
            agent,
            call: Box::from(&b"read-call"[..]),
            answer: CallAnswer::Relayed { error: false, text: Box::from(&b"record"[..]) },
        })]
    );
    assert!(h.domain.is_relayed(Token::new(1), Token::new(1001), b"read-call"));
}

#[test]
fn an_answer_naming_no_relay_of_its_attempt_is_dropped() {
    let mut h = Harness::new();
    let (first, _, _) = h.live(1);
    let (second, _, _) = h.live(2);
    named_relay(&mut h, first, b"first-call");
    named_relay(&mut h, second, b"second-call");
    assert!(h.step(named_answer(2, b"first-call")).is_empty());
    assert!(h.domain.is_relayed(Token::new(1), Token::new(1001), b"first-call"));
    assert!(h.domain.is_relayed(Token::new(2), Token::new(1002), b"second-call"));
}

#[test]
fn an_assignment_is_answered_by_its_run_and_attempt() {
    let mut h = Harness::new();
    let (owner, workspace, agent) = h.live(7);
    assert_eq!(
        &*h.step(fixtures::finished(owner, Finish::Parked)),
        &[Request::Agent(crate::ToAgent::Cancel { agent })]
    );
    assert_eq!(
        &*h.step(Event::Agent(crate::FromAgent::Gone { owner, detail: Box::new([]) })),
        &[Request::Save { owner, workspace }]
    );
    let requests = h.step(Event::Saved { owner, at: None });
    let [Request::Release { .. }, Request::Up(LinkUp::Answer { run, attempt, answer })] = &*requests else {
        panic!("the assignment answers: {requests:?}");
    };
    assert_eq!((*run, *attempt), (Token::new(7), Token::new(1007)));
    assert_eq!(answer.ending, crate::Ending::Parked { work: Work { left: None, saved: None } });
}

#[test]
fn an_assignment_is_refused_by_its_run_and_attempt() {
    let mut h = Harness::new();
    let mut next = assignment(9);
    next.charter = Box::new([0; 65]);
    assert_eq!(
        &*h.step(fixtures::assign(next)),
        &[Request::Up(LinkUp::Answer {
            run: Token::new(9),
            attempt: Token::new(1009),
            answer: crate::RunAnswer {
                read: None,
                refused: Box::new([]),
                turns: 0,
                spent: 0,
                ending: crate::Ending::Refused(Refusal::Invalid(Invalid::Charter)),
            },
        })]
    );
}

fn pending_start(h: &mut Harness, run: u64) -> (Token, Token) {
    let (owner, workspace) = h.assign(run);
    let requests = h.step(Event::Prepared { owner, workspace });
    let [Request::Agent(crate::ToAgent::Start { owner: started, start })] = &*requests else {
        panic!("expected start: {requests:?}")
    };
    assert_eq!((*started, start.workspace), (owner, Some(workspace)));
    (owner, workspace)
}

#[test]
fn a_refused_start_answers_invalid_or_busy_and_frees_its_slot() {
    for busy in [false, true] {
        let mut h = Harness::new();
        let (owner, workspace) = pending_start(&mut h, 1);
        let _other = h.assign(2);
        let name = Token::new(42);
        assert!(
            h.step(fixtures::inbound(Token::new(1), Token::new(1001), name, Box::from(&b"question"[..]))).is_empty()
        );
        let event = if busy {
            crate::FromAgent::StartBusy { owner }
        } else {
            crate::FromAgent::StartRefused { cause: crate::StartCause::Grants, owner, why: b"budget".as_slice().into() }
        };
        let requests = h.step(Event::Agent(event));
        assert_eq!(
            requests.as_ref(),
            &[
                Request::Release { workspace },
                Request::Up(LinkUp::Answer {
                    run: Token::new(1),
                    attempt: Token::new(1001),
                    answer: crate::RunAnswer {
                        turns: 0,
                        spent: 0,
                        read: None,
                        refused: Box::new([Refused { name, reason: MessageRefusal::Ending }]),
                        ending: crate::Ending::Refused(if busy {
                            Refusal::Busy
                        } else {
                            Refusal::Invalid(Invalid::Start {
                                cause: crate::StartCause::Grants,
                                why: b"budget".as_slice().into(),
                            })
                        }),
                    },
                }),
            ]
        );
        assert_eq!(h.domain.kept_answers(), 0);
        assert!(h.domain.hosting(owner).is_none());
        h.domain.reclaim();
        let _replacement = h.assign(3);
    }
}

#[test]
fn a_refused_starts_words_keep_only_the_bounded_detail_tail() {
    let mut h = Harness::new();
    let (owner, _) = pending_start(&mut h, 1);
    let requests = h.step(Event::Agent(crate::FromAgent::StartRefused {
        cause: crate::StartCause::Other,
        owner,
        why: b"prefix detailed".as_slice().into(),
    }));
    let [Request::Release { .. }, Request::Up(LinkUp::Answer { answer, .. })] = &*requests else {
        panic!("expected refusal: {requests:?}")
    };
    assert_eq!(
        answer.ending,
        crate::Ending::Refused(Refusal::Invalid(Invalid::Start {
            cause: crate::StartCause::Other,
            why: b"detailed".as_slice().into()
        }))
    );
}

#[test]
fn gone_while_starting_still_answers_an_unstarted_agent_failure() {
    let mut h = Harness::new();
    let (owner, workspace) = pending_start(&mut h, 1);
    let requests = h.step(Event::Agent(crate::FromAgent::Gone { owner, detail: b"exit".as_slice().into() }));
    assert_eq!(
        requests.as_ref(),
        &[
            Request::Release { workspace },
            Request::Up(LinkUp::Answer {
                run: Token::new(1),
                attempt: Token::new(1001),
                answer: crate::RunAnswer {
                    turns: 0,
                    spent: 0,
                    read: None,
                    refused: Box::new([]),
                    ending: crate::Ending::Failed {
                        failure: Failure::Agent(crate::AgentFailure::Unstarted),
                        detail: b"exit".as_slice().into(),
                        work: Work { left: None, saved: None },
                    },
                },
            }),
        ]
    );
}

#[test]
fn a_start_refusal_after_cancel_keeps_the_cancellations_answer() {
    for busy in [false, true] {
        let mut h = Harness::new();
        let (owner, _) = pending_start(&mut h, 1);
        assert!(h.step(Event::Down(LinkDown::Cancel { run: Token::new(1), attempt: Token::new(1001) })).is_empty());
        let event = if busy {
            crate::FromAgent::StartBusy { owner }
        } else {
            crate::FromAgent::StartRefused { cause: crate::StartCause::Other, owner, why: b"budget".as_slice().into() }
        };
        let requests = h.step(Event::Agent(event));
        let [Request::Release { .. }, Request::Up(LinkUp::Answer { answer, .. })] = &*requests else {
            panic!("expected cancellation: {requests:?}")
        };
        assert_eq!(
            answer.ending,
            crate::Ending::Failed {
                failure: Failure::Cancelled(Reason::Engine),
                detail: if busy { Box::new([]) } else { b"budget".as_slice().into() },
                work: Work { left: None, saved: None },
            }
        );
    }
}

#[test]
fn two_grants_of_one_account_in_different_forms_are_refused() {
    let mut h = Harness::new();
    let mut assignment = assignment(1);
    assignment.grants = Box::new([
        fixtures::account_grant(1),
        crate::Grant { form: crate::Form::Credential(crate::Scheme::Key), ..fixtures::account_grant(1) },
    ]);
    let requests = h.step(fixtures::assign(assignment));
    let [
        Request::Up(LinkUp::Answer {
            answer: crate::RunAnswer { ending: crate::Ending::Refused(Refusal::Invalid(Invalid::Grants)), .. },
            ..
        }),
    ] = &*requests
    else {
        panic!("expected invalid grants: {requests:?}")
    };
}

fn program_endpoints(count: u32) -> Box<[u32]> {
    let mut endpoints = List::with_capacity(count);
    for endpoint in 0..count {
        endpoints.push(endpoint).expect("one slot per endpoint");
    }
    endpoints.into_boxed()
}

#[test]
fn program_endpoints_are_valid_up_to_the_cap_and_distinct() {
    // domain/hosts/charters.md, 1: 64 distinct endpoint identities per hello.
    const ENDPOINT_CAP: u32 = 64;
    let mut duplicate = program_endpoints(ENDPOINT_CAP);
    *duplicate.last_mut().expect("nonempty endpoint list") = 0;
    for (endpoints, valid_hub) in [
        (program_endpoints(0), true),
        (program_endpoints(ENDPOINT_CAP), true),
        (program_endpoints(ENDPOINT_CAP + 1), false),
        (duplicate, false),
    ] {
        assert_eq!(crate::valid_program_endpoints(&endpoints), valid_hub, "endpoints: {endpoints:?}");
    }
}

#[test]
fn an_assignments_live_ask_reaches_the_agents_start() {
    for live in [crate::LiveAsk::NONE, crate::LiveAsk { text: true, reasoning: true, input: true }] {
        let mut h = Harness::new();
        let mut assignment = assignment(1);
        assignment.live = live;
        let requests = h.step(fixtures::assign(assignment));
        let [Request::Prepare { owner, .. }] = &*requests else { panic!("prepare workspace") };
        let requests = h.step(Event::Prepared { owner: *owner, workspace: Token::new(201) });
        let [Request::Agent(crate::ToAgent::Start { start, .. })] = &*requests else { panic!("start agent") };
        assert_eq!(start.live, live);
    }
}

#[test]
fn a_local_call_error_retires_the_relay_and_ignores_late_terminals() {
    let mut h = Harness::new();
    let (owner, workspace, agent) = h.live(1);
    named_relay(&mut h, owner, b"oversized");
    let error = Event::Down(LinkDown::Relayed {
        run: Token::new(1),
        attempt: Token::new(1001),
        call: b"oversized".as_slice().into(),
        answer: crate::Relayed::Answer { error: true, text: b"call too large".as_slice().into() },
    });
    assert_eq!(
        &*h.step(error),
        &[Request::Agent(crate::ToAgent::Answer {
            agent,
            call: b"oversized".as_slice().into(),
            answer: CallAnswer::Relayed { error: true, text: b"call too large".as_slice().into() }
        })]
    );
    h.domain.reclaim();
    assert_eq!(h.domain.calls(), 0);
    assert!(!h.domain.owns_relay(Token::new(1), Token::new(1001), b"oversized"));
    assert!(h.step(named_answer(1, b"oversized")).is_empty());
    assert!(
        h.step(Event::RelayCancelled {
            run: Token::new(1),
            attempt: Token::new(1001),
            call: b"oversized".as_slice().into()
        })
        .is_empty()
    );
    assert_eq!(h.domain.calls(), 0);
    assert_eq!(
        &*h.step(fixtures::finished(owner, Finish::Parked)),
        &[Request::Agent(crate::ToAgent::Cancel { agent })]
    );
    assert_eq!(
        &*h.step(Event::Agent(crate::FromAgent::Gone { owner, detail: Box::new([]) })),
        &[Request::Save { owner, workspace }]
    );
    let requests = h.step(Event::Saved { owner, at: None });
    let [Request::Release { .. }, Request::Up(LinkUp::Answer { .. })] = &*requests else {
        panic!("the drained assignment answers: {requests:?}");
    };
    h.step(Event::Down(LinkDown::Acknowledge { run: Token::new(1), attempt: Token::new(1001) }));
    h.domain.reclaim();
    assert_eq!((h.domain.hosted(), h.domain.calls()), (0, 0));
}

#[test]
fn valid_refuses_each_unrepresentable_limit() {
    for (name, limits, accepted) in [
        ("fixture", LIMITS, true),
        ("turn bytes", Limits { turn_bytes: 0, ..LIMITS }, false),
        ("turn queue", Limits { turn_queue_bytes: LIMITS.turn_bytes - 1, ..LIMITS }, false),
        ("call count", Limits { run_calls: u32::MAX, ..LIMITS }, false),
        ("held count", Limits { held: u32::MAX, ..LIMITS }, true),
        ("source count", Limits { program_sources: u32::MAX, ..LIMITS }, true),
        ("turn window", Limits { turns: u32::MAX, ..LIMITS }, false),
        ("output total", Limits { program_sources: u32::MAX - 4, ..LIMITS }, true),
        ("call capacity", Limits { slots: u32::MAX, run_calls: 2, turns: 0, ..LIMITS }, false),
        ("turn capacity", Limits { slots: u32::MAX, turns: 1, run_calls: 0, ..LIMITS }, false),
        ("source_interval", Limits { source_interval: skein_lib::Duration::ZERO, ..LIMITS }, false),
        ("source_timeout", Limits { source_timeout: skein_lib::Duration::ZERO, ..LIMITS }, false),
    ] {
        assert_eq!(crate::valid_hub(&limits), accepted, "{name}");
    }
}

mod source_validity {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        for (name, limits, accepted) in [
            ("fixture", super::LIMITS, true),
            ("interval", crate::Limits { source_interval: skein_lib::Duration::ZERO, ..super::LIMITS }, false),
            ("timeout", crate::Limits { source_timeout: skein_lib::Duration::ZERO, ..super::LIMITS }, false),
        ] {
            assert_eq!(crate::sources::valid(&limits), accepted, "{name}");
        }
    }
}

#[test]
fn validity_does_not_price_host_payloads() {
    assert!(crate::valid_hub(&Limits {
        event_bytes: u64::MAX,
        charter_bytes: u64::MAX,
        transcript_bytes: u64::MAX,
        turn_queue_bytes: u64::MAX,
        ..LIMITS
    }));
}

#[test]
fn admission_classifies_calls_and_link_work_separately_from_completions() {
    let call = crate::FromAgent::Called {
        owner: Token::new(1),
        call: Box::from(*b"read"),
        ask: Ask::Relay {
            tool: Box::from(*b"logs"),
            writes: false,
            input: Box::new([]),
            deadline: skein_lib::Duration::from_secs(1),
        },
    };
    assert!(call.new_work());
    assert!(Event::Agent(call).new_work());
    let settled = crate::FromAgent::Yielded { owner: Token::new(1) };
    assert!(!settled.new_work());
    assert!(!Event::Agent(settled).new_work());
    assert!(
        Event::Down(LinkDown::SignOut { name: crate::SourceName::new(b"test").expect("configured name") }).new_work()
    );
    assert!(!Event::SourceLost.new_work());
    assert!(!Event::Saved { owner: Token::new(1), at: None }.new_work());
}

mod drain;
mod jobs;

#[test]
fn a_starting_fault_keeps_its_class_until_gone_without_admission() {
    let mut h = Harness::new();
    let (owner, workspace) = pending_start(&mut h, 1);
    h.step(Event::Down(LinkDown::Message {
        run: Token::new(1),
        attempt: Token::new(1001),
        name: Token::new(9),
        sender: Box::new([]),
        words: b"pending".as_slice().into(),
        finish_requested: false,
    }));
    for fault in [crate::AgentFailure::Rules, crate::AgentFailure::Exited] {
        assert!(h.step(Event::Agent(crate::FromAgent::Faulted { owner, fault })).is_empty());
    }
    let requests = h.step(Event::Agent(crate::FromAgent::Gone { owner, detail: b"prefix detailed".as_slice().into() }));
    let [Request::Release { workspace: released }, Request::Up(LinkUp::Answer { answer, .. })] = &*requests else {
        panic!("fault waits for cleanup: {requests:?}");
    };
    assert_eq!(*released, workspace);
    assert_eq!(answer.refused.as_ref(), &[Refused { name: Token::new(9), reason: MessageRefusal::Ending }]);
    assert_eq!(
        answer.ending,
        crate::Ending::Failed {
            failure: Failure::Agent(crate::AgentFailure::Rules),
            detail: b"detailed".as_slice().into(),
            work: Work { left: None, saved: None },
        }
    );
    while let Some(fact) = h.domain.pop_fact() {
        if let crate::Fact::Started { .. } = fact {
            panic!("a faulted startup is never admitted");
        }
    }
}

#[test]
fn cancellation_before_a_starting_fault_keeps_the_cancellation() {
    let mut h = Harness::new();
    let (owner, _) = pending_start(&mut h, 1);
    h.step(Event::Down(LinkDown::Cancel { run: Token::new(1), attempt: Token::new(1001) }));
    assert!(h.step(Event::Agent(crate::FromAgent::Faulted { owner, fault: crate::AgentFailure::Rules })).is_empty());
    let requests = h.step(Event::Agent(crate::FromAgent::Gone { owner, detail: b"fault".as_slice().into() }));
    let [Request::Release { .. }, Request::Up(LinkUp::Answer { answer, .. })] = &*requests else {
        panic!("cancel waits for cleanup: {requests:?}");
    };
    assert_eq!(
        answer.ending,
        crate::Ending::Failed {
            failure: Failure::Cancelled(Reason::Engine),
            detail: b"fault".as_slice().into(),
            work: Work { left: None, saved: None },
        }
    );
}

#[test]
fn a_list_read_without_a_machine_answers_agent_immediately() {
    let mut h = Harness::new();
    let requests = h.step(Event::Down(LinkDown::ReadList {
        read: 7,
        endpoint: 3,
        grant: crate::Grant {
            form: crate::Form::Credential(crate::Scheme::Key),
            account: 1,
            generation: 2,
            valid: skein_lib::Duration::ZERO,
        },
    }));
    assert_eq!(
        &*requests,
        &[Request::Up(LinkUp::ListRead {
            read: 7,
            outcome: jig_domain_link::ListOutcome::Failed(jig_domain_link::ListFailure::Agent),
        })]
    );
    assert_eq!(h.domain.hosted(), 0);
}

#[test]
fn a_shutdown_says_draining_before_any_cancelled_runs_answer() {
    let mut h = Harness::new();
    let owners = [h.assign(1).0, h.assign(2).0];
    let requests = h.step(Event::CancelAll { reason: Reason::Shutdown });
    assert_eq!(&*requests, [Request::Up(LinkUp::Draining)]);
    for (number, owner) in owners.into_iter().enumerate() {
        crate::resume(&mut h.domain, &h.env, &mut h.out);
        assert_eq!(h.out.pop(), Some(Request::Abort { owner }));
        assert_eq!(h.out.pop(), None);
        let requests = h.step(Event::Unprepared { owner, failure: Preparation::Transient, detail: Box::new([]) });
        let [Request::Up(LinkUp::Answer { run, answer, .. })] = &*requests else {
            panic!("cancelled run answers after draining: {requests:?}");
        };
        assert_eq!(run.raw(), u64::try_from(number).expect("two runs").checked_add(1).expect("two runs"));
        let crate::Ending::Failed { failure, .. } = answer.ending else {
            panic!("shutdown failed the run: {answer:?}");
        };
        assert_eq!(failure, Failure::Cancelled(Reason::Shutdown));
    }
}

#[test]
fn a_shutting_hub_says_draining_again_after_each_opening() {
    let mut h = Harness::new();
    let requests = h.step(jobs::opened());
    let [Request::Hello { .. }] = &*requests else {
        panic!("an opening before shutdown says hello: {requests:?}");
    };
    h.step(Event::CancelAll { reason: Reason::Shutdown });
    for _ in 0_u32..2 {
        let requests = h.step(jobs::opened());
        let [Request::Hello { .. }, Request::Up(LinkUp::Draining)] = &*requests else {
            panic!("each opening while shutting down says draining after hello: {requests:?}");
        };
    }
}

#[test]
fn a_save_is_an_unchecked_delivery_and_shares_the_delivery_slot() {
    let mut h = Harness::new();
    let (owner, workspace, agent) = h.live(1);
    let call = Token::new(8);
    let requests = h.step(fixtures::called(owner, call, Ask::Save));
    let [Request::DeliverUnchecked { owner: delivery, workspace: target }] = requests.as_ref() else {
        panic!("unchecked workspace delivery: {requests:?}")
    };
    assert_eq!(*target, workspace);
    let delivery = *delivery;
    for ask in [Ask::Save, fixtures::deliver(b"checked".as_slice().into())] {
        let requests = h.step(fixtures::called(owner, Token::new(9), ask));
        let [Request::Agent(crate::ToAgent::Answer { answer: CallAnswer::Busy, .. })] = requests.as_ref() else {
            panic!("save shares the delivery slot: {requests:?}")
        };
    }
    let result = Delivery { outcome: DeliveryOutcome::Delivered, left: Token::new(900), changed: true };
    assert_eq!(
        h.step(Event::Delivered { owner: delivery, delivery: result }).as_ref(),
        [Request::Agent(crate::ToAgent::Answer {
            agent,
            call: Box::from(call.raw().to_be_bytes()),
            answer: CallAnswer::Delivered(result)
        })]
    );
}
