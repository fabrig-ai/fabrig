//! Local policy consumes each typed parent variant; process ownership is
//! exercised separately by the spawned lifecycle unit and composed worlds.
use super::{Domain, State, parent_request};
use crate::{
    AccountFailure, AgentFailure, Checks, Config, Contract, Event, Failure, Limits, Notice, Request, Undelivered,
};
use alloc::boxed::Box;
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use smith_domain::{run, session};
use smith_host_domain::{self as host, parent, process};

fn running(config: Config) -> (Domain, Env<Limits>, Queue<Request>) {
    let mut limits = crate::tests::limits();
    limits.agent = crate::AgentLimits::Spawned(crate::tests::host_limits());
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.clone() };
    let domain = Domain::running_parent(config, &limits, 71);
    let out = Queue::uncapped();
    (domain, env, out)
}

fn plain_delivery_config() -> Config {
    let mut config = crate::tests::config();
    config.reserve.time = crate::tests::limits().smith.run.check_timeout;
    config.budget.time = Duration::from_secs(120);
    config.contract = Contract::Change(run::outcome::ChangeSpec {
        checks_must_pass: false,
        fields: Box::from([run::outcome::FieldRule { name: b"title".as_slice().into(), max: 64 }]),
    });
    config.workspace = Some(run::Workspace {
        directories: Box::from([run::Directory {
            host: Box::from(b"/host".as_slice()),
            name: b"work".as_slice().into(),
            root: Token::new(9),
            writable: true,
            git: false,
            conflicts: Box::default(),
        }]),
    });
    config
}

fn call_delivery(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    parent_request(
        domain,
        env,
        parent::Request::Called {
            client: Token::new(1),
            logical_run: Token::new(1),
            call: Token::new(55),
            name: host::CallName { activation: 1, completion: 2, position: 1 },
            deadline: Time::from_nanos(99),
            ask: host::Ask::Deliver {
                fields: host::Fields::new(
                    run::outcome::Change {
                        fields: Box::from([run::outcome::Field {
                            name: b"title".as_slice().into(),
                            value: b"keep the change".as_slice().into(),
                        }]),
                    },
                    1024,
                )
                .expect("bounded fields"),
            },
        },
        out,
    );
}

fn saved_intent(out: &mut Queue<Request>) -> Box<crate::DeliveryRecord> {
    let Some(Request::SaveDelivery { record }) = out.pop() else { panic!("save intent") };
    let crate::DeliveryState::Intent(_) = &record.state else { panic!("save a delivery intent") };
    record
}

fn assert_store_failure(out: &mut Queue<Request>, turn: Option<u32>) {
    let Some(Request::Notice {
        notice: Notice::Failure(Failure::Store { turn: found, reason: crate::StoreFailure::Write }),
    }) = out.pop()
    else {
        panic!("store failure notice")
    };
    assert_eq!(found, turn);
}

#[test]
fn started_binds_the_handle_and_admission_keeps_an_empty_person_queue() {
    let (_, env, mut out) = running(crate::tests::config());
    let mut domain = Domain::starting_parent(crate::tests::config(), &env.limits, 71);
    parent_request(
        &mut domain,
        &env,
        parent::Request::Started { client: Token::new(1), agent: Token::new(9) },
        &mut out,
    );
    parent_request(&mut domain, &env, parent::Request::Admitted { client: Token::new(1) }, &mut out);
    assert_eq!(domain.child.run(), Some(Token::new(9)));
    assert!(out.is_empty());
}

#[test]
fn the_first_drop_cause_survives_store_failure_and_the_childs_later_fault() {
    for store_first in [false, true] {
        let (mut domain, env, mut out) = running(crate::tests::config());
        let failed_store = Event::StoreFailed { turn: Some(2), reason: crate::StoreFailure::Write };
        if store_first {
            crate::step(&mut domain, &env, failed_store, &mut out);
            crate::step(&mut domain, &env, Event::Drop, &mut out);
        } else {
            crate::step(&mut domain, &env, Event::Drop, &mut out);
            crate::step(&mut domain, &env, failed_store, &mut out);
        }
        let cause = if store_first {
            crate::EndCause::Host { turn: Some(2), reason: crate::StoreFailure::Write }
        } else {
            crate::EndCause::Cancelled
        };
        assert_eq!(domain.end_cause(), Some(cause));
        parent_request(
            &mut domain,
            &env,
            parent::Request::Faulted { client: Token::new(1), fault: host::Fault::Exited },
            &mut out,
        );
        parent_request(
            &mut domain,
            &env,
            crate::fixture::gone(
                host::End::Exited {
                    exit: Some(smith_host_domain::Exit::Code(0)),
                    forced: smith_host_domain::Forced::No,
                },
                Box::default(),
            ),
            &mut out,
        );
        assert_eq!(domain.end_cause(), Some(cause), "later child terminals cannot replace the cause");
        let mut status = None;
        for _ in 0..out.len() {
            let Some(request) = out.pop() else { break };
            if let Request::Exit { status: value } = request {
                status = Some(value);
            }
        }
        assert_eq!(
            status,
            Some(if store_first { crate::ExitStatus::RunFailed } else { crate::ExitStatus::Interrupted })
        );
    }
}

#[test]
fn waiting_and_both_long_notices_reach_the_person_by_kind() {
    let notices = [
        (parent::Request::Waiting { client: Token::new(1), read: None }, Notice::Waiting),
        (
            parent::Request::Long {
                client: Token::new(1),
                operation: host::Operation::Checks,
                span: Duration::from_secs(9),
            },
            Notice::Checking(Checks::Started),
        ),
        (parent::Request::LongDone { client: Token::new(1) }, Notice::Checking(Checks::Ended)),
    ];
    let (mut domain, env, mut out) = running(crate::tests::config());
    for (notice, expected) in notices {
        parent_request(&mut domain, &env, notice, &mut out);
        let Some(Request::Notice { notice }) = out.pop() else { panic!("one person notice") };
        assert_eq!(notice, expected);
        assert!(out.is_empty());
    }
}

#[test]
fn completion_and_command_spans_do_not_claim_checks_are_running() {
    let (mut domain, env, mut out) = running(crate::tests::config());
    for operation in [host::Operation::Completion, host::Operation::Command] {
        parent_request(
            &mut domain,
            &env,
            parent::Request::Long { client: Token::new(1), operation, span: Duration::from_secs(9) },
            &mut out,
        );
    }
    assert!(out.is_empty());
    for _ in 0_u32..2 {
        parent_request(&mut domain, &env, parent::Request::LongDone { client: Token::new(1) }, &mut out);
        assert!(out.is_empty());
    }
}

#[test]
fn each_account_notice_keeps_its_account() {
    let (mut domain, mut env, mut out) = running(crate::tests::config());
    env.wall = Wall::from_nanos(36_000_000_000_000);
    parent_request(
        &mut domain,
        &env,
        parent::Request::Unauthorized { client: Token::new(1), account: 7, generation: 4 },
        &mut out,
    );
    let Some(Request::Credential { account }) = out.pop() else { panic!("refresh the named account") };
    assert_eq!(account, 7);
    for (request, expected) in account_notices() {
        parent_request(&mut domain, &env, request, &mut out);
        let Some(Request::Notice { notice }) = out.pop() else { panic!("account notice") };
        assert_eq!(notice, Notice::Failure(Failure::Account(expected.clone())), "{expected:?}");
        assert!(out.is_empty());
    }
}

fn account_notices() -> [(parent::Request, AccountFailure); 4] {
    let client = Token::new(1);
    [
        (
            parent::Request::Limited { client, account: 7, reset: Some(Duration::from_secs(2)) },
            AccountFailure::Limited { account: 7, reset_at: Some(Wall::from_nanos(36_002_000_000_000)) },
        ),
        (
            parent::Request::Limited { client, account: 8, reset: Some(Duration::ZERO) },
            AccountFailure::Limited { account: 8, reset_at: Some(Wall::from_nanos(36_000_000_000_000)) },
        ),
        (
            parent::Request::Limited { client, account: 9, reset: None },
            AccountFailure::Limited { account: 9, reset_at: None },
        ),
        (
            parent::Request::Refused {
                client,
                account: 10,
                code: b"billing".as_slice().into(),
                reason: b"contact support".as_slice().into(),
            },
            AccountFailure::Refused {
                account: 10,
                code: b"billing".as_slice().into(),
                reason: b"contact support".as_slice().into(),
            },
        ),
    ]
}

#[test]
fn told_and_withdrawn_do_not_abandon_a_durable_delivery() {
    let (mut domain, env, mut out) = running(plain_delivery_config());
    call_delivery(&mut domain, &env, &mut out);
    let Some(Request::PlainStatus { owner, .. }) = out.pop() else { panic!("survey delivery") };
    assert_eq!(owner, Token::new(55));
    crate::step(&mut domain, &env, Event::PlainStatus { owner, changed: true }, &mut out);
    let record = saved_intent(&mut out);
    assert!(out.is_empty());
    parent_request(
        &mut domain,
        &env,
        parent::Request::Told { client: Token::new(1), body: b"fact".as_slice().into() },
        &mut out,
    );
    parent_request(
        &mut domain,
        &env,
        parent::Request::Withdrawn { client: Token::new(1), call: Token::new(55) },
        &mut out,
    );
    assert!(out.is_empty());
    crate::step(&mut domain, &env, Event::DeliverySaved { name: record.name }, &mut out);
    let Some(Request::SaveDelivery { record: answer }) = out.pop() else { panic!("save delivery answer") };
    assert_eq!(answer.name, record.name);
    let crate::DeliveryState::Answer(run::Delivery::Delivered(delivered)) = answer.state else {
        panic!("withdrawn delivery still lands")
    };
    assert_eq!(delivered.receipts()[0].text(), b"files kept");
    assert!(out.is_empty());
}

#[test]
fn a_parent_turn_moves_its_concrete_value_to_the_durable_store() {
    let (mut domain, env, mut out) = running(crate::tests::config());
    let turn = smith_domain::Turn { usage: session::llm::Usage::NONE, spent: 29, ..crate::fixture::turn() };
    let expected = turn.clone();
    parent_request(
        &mut domain,
        &env,
        parent::Request::Turn {
            client: Token::new(1),
            turn: host::Turn {
                opens_window: true,
                number: 1,
                read: None,
                spent: 29,
                body: host::TurnValue::new(turn, crate::tests::host_limits().turn_bytes).expect("bounded native turn"),
            },
        },
        &mut out,
    );
    let Some(Request::SaveTurn { turn, number: 1, .. }) = out.pop() else { panic!("durable native turn") };
    assert_eq!(turn, expected);
}

#[test]
fn the_parent_answer_keeps_its_host_spend_until_gone() {
    let (mut domain, env, mut out) = running(crate::tests::config());
    parent_request(
        &mut domain,
        &env,
        parent::Request::Answered {
            client: Token::new(1),
            answer: host::Answer { read: None, turns: 3, spent: 29, result: host::RunResult::Parked },
            detail: Box::default(),
        },
        &mut out,
    );
    assert!(out.is_empty());
    let State::Answered { answer, .. } = &domain.chat.phase else { panic!("typed host answer retained") };
    assert_eq!(answer.spent, 29);
    assert_eq!(answer.turns, 3);
    parent_request(
        &mut domain,
        &env,
        crate::fixture::gone(
            host::End::Exited { exit: Some(smith_host_domain::Exit::Code(0)), forced: smith_host_domain::Forced::No },
            Box::default(),
        ),
        &mut out,
    );
    let Some(Request::Notice { notice }) = out.pop() else { panic!("answer shown after Gone") };
    assert_eq!(notice, Notice::Answer(crate::Said::Parked));
    assert!(domain.child.is_gone());
}

#[test]
fn an_agent_fault_keeps_the_current_failure_class() {
    let (mut domain, env, mut out) = running(crate::tests::config());
    parent_request(
        &mut domain,
        &env,
        parent::Request::Faulted { client: Token::new(1), fault: host::Fault::Rules },
        &mut out,
    );
    assert!(domain.child.failure().is_some());
    parent_request(&mut domain, &env, crate::fixture::gone(host::End::Settled, Box::new([])), &mut out);
    let Some(Request::Unanswered { .. }) = out.pop() else { panic!("unanswered terminal") };
    let Some(Request::Notice { notice }) = out.pop() else { panic!("failure notice") };
    assert_eq!(
        notice,
        Notice::Failure(Failure::Agent {
            failure: AgentFailure::Fault(host::Fault::Rules),
            end: host::End::Settled,
            detail: Box::new([])
        })
    );
}

#[test]
fn every_message_refusal_ends_the_message_without_failing_the_agent() {
    for reason in [
        host::MessageRefusal::Full,
        host::MessageRefusal::TooLarge,
        host::MessageRefusal::NameInUse,
        host::MessageRefusal::Ending,
    ] {
        let (mut domain, env, mut out) = running(crate::tests::config());
        parent_request(
            &mut domain,
            &env,
            parent::Request::MessageRefused { client: Token::new(1), name: Token::new(3), reason },
            &mut out,
        );
        assert!(domain.child.failure().is_none());
        assert!(domain.chat.phase.is_running());
        let Some(Request::Notice { notice }) = out.pop() else { panic!("message terminal notice") };
        assert_eq!(notice, Notice::NotDelivered { line: Some(Token::new(3)), why: Undelivered::Refused(reason) });
        assert!(out.is_empty());
    }
}

#[test]
fn unread_names_reach_the_parent_without_an_agent_failure_or_another_activation() {
    let (mut domain, env, mut out) = running(crate::tests::config());
    parent_request(
        &mut domain,
        &env,
        parent::Request::Unread { client: Token::new(1), names: Box::from([Token::new(3)]) },
        &mut out,
    );
    assert!(domain.child.failure().is_none());
    assert!(domain.chat.phase.is_running());
    assert!(out.is_empty());
}

#[test]
fn a_message_refused_after_the_last_word_keeps_the_single_transcript_reload() {
    let (mut domain, env, mut out) = running(crate::tests::config());
    parent_request(
        &mut domain,
        &env,
        parent::Request::Answered {
            client: Token::new(1),
            answer: host::Answer {
                read: None,
                turns: 0,
                spent: 0,
                result: host::RunResult::Failed {
                    failure: host::RunFailure::Transcript(host::TranscriptRefusal::Version),
                },
            },
            detail: Box::default(),
        },
        &mut out,
    );
    parent_request(
        &mut domain,
        &env,
        crate::fixture::gone(
            host::End::Exited { exit: Some(smith_host_domain::Exit::Code(0)), forced: smith_host_domain::Forced::No },
            Box::default(),
        ),
        &mut out,
    );
    parent_request(
        &mut domain,
        &env,
        parent::Request::MessageRefused {
            client: Token::new(1),
            name: Token::new(3),
            reason: host::MessageRefusal::Ending,
        },
        &mut out,
    );
    let Some(Request::Notice { notice }) = out.pop() else { panic!("transcript refusal") };
    assert_eq!(
        notice,
        Notice::Failure(Failure::Run {
            failure: host::RunFailure::Transcript(host::TranscriptRefusal::Version),
            detail: Box::new([])
        })
    );
    let Some(Request::Answered { .. }) = out.pop() else { panic!("answer reaches its event writer") };
    let Some(Request::Load) = out.pop() else { panic!("one transcript reload") };
    let Some(Request::Notice { notice }) = out.pop() else { panic!("late message terminal") };
    assert_eq!(
        notice,
        Notice::NotDelivered { line: Some(Token::new(3)), why: Undelivered::Refused(host::MessageRefusal::Ending) }
    );
    assert!(out.is_empty());
    assert!(domain.chat.phase.is_loading());
    assert!(domain.child.failure().is_none());
}

#[test]
fn gone_without_an_answer_fails_after_retained_turns_are_saved() {
    let (mut domain, env, mut out) = running(crate::tests::config());
    parent_request(
        &mut domain,
        &env,
        crate::fixture::gone(host::End::Unspawned, b"unavailable".as_slice().into()),
        &mut out,
    );
    let Some(Request::Unanswered { cause: crate::EndCause::Agent { ended: None }, turns: 0, spent: 0, read: None }) =
        out.pop()
    else {
        panic!("unanswered run reaches its event writer")
    };
    let Some(Request::Notice { notice }) = out.pop() else { panic!("missing answer notice") };
    assert_eq!(
        notice,
        Notice::Failure(Failure::Agent {
            failure: AgentFailure::NotStarted,
            end: host::End::Unspawned,
            detail: b"unavailable".as_slice().into()
        })
    );
    let Some(Request::Exit { status: crate::ExitStatus::RunFailed }) = out.pop() else { panic!("failed invocation") };
}

#[test]
fn a_delivery_call_keeps_its_fields_owner_and_deadline_in_local_policy() {
    let config = plain_delivery_config();
    let (mut domain, env, mut out) = running(config);
    let change = run::outcome::Change {
        fields: Box::from([run::outcome::Field {
            name: b"title".as_slice().into(),
            value: b"keep the change".as_slice().into(),
        }]),
    };
    let deadline = Time::from_nanos(99);
    parent_request(
        &mut domain,
        &env,
        parent::Request::Called {
            client: Token::new(1),
            logical_run: Token::new(1),
            call: Token::new(55),
            name: host::CallName { activation: 1, completion: 2, position: 1 },
            deadline,
            ask: host::Ask::Deliver { fields: host::Fields::new(change, 1024).expect("bounded fields") },
        },
        &mut out,
    );
    let Some(Request::PlainStatus { owner, directory, deadline: received }) = out.pop() else {
        panic!("survey typed delivery")
    };
    assert_eq!(owner, Token::new(55));
    assert_eq!(directory, 0);
    assert_eq!(received, deadline);
}

#[test]
fn an_unconfigured_host_call_receives_one_unavailable_parent_answer() {
    let (mut domain, env, mut out, owner) = crate::tests::spawned_chat_with_saved_hello();
    crate::step(
        &mut domain,
        &env,
        Event::Process(process::Event::Received {
            owner,
            message: host::Up::Call {
                call: Token::new(55),
                name: host::CallName { activation: 1, completion: 2, position: 1 },
                deadline: Time::from_nanos(99),
                ask: host::Ask::Host {
                    tool: b"missing".as_slice().into(),
                    effect: host::Effect::Read,
                    body: Box::default(),
                },
            },
        }),
        &mut out,
    );
    assert!(domain.is_ready(), "the routed call keeps its generated answer ready");
    crate::resume(&mut domain, &env, &mut out);
    assert!(!domain.is_ready(), "the answer drained without another external event");
    let mut replies = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Process(process::Request::Send {
            message: host::Down::Answer { call, reply: host::Reply::Unavailable },
            ..
        }) = request
        {
            assert_eq!(call, Token::new(55));
            replies += 1;
        }
    }
    assert_eq!(replies, 1);
}

#[test]
fn closing_input_during_a_refused_start_exits_after_the_notice() {
    let (mut domain, env, mut out) = crate::tests::awaiting_credential(crate::tests::config());
    crate::step(&mut domain, &env, Event::Closed, &mut out);
    assert!(out.is_empty());
    crate::step(
        &mut domain,
        &env,
        Event::NoCredential { account: 7, reason: crate::CredentialFailure::Missing },
        &mut out,
    );
    let Some(Request::Notice { notice: Notice::NotDelivered { line: Some(line), why: Undelivered::Dropped } }) =
        out.pop()
    else {
        panic!("the pending line is dropped before the credential refusal")
    };
    assert_eq!(line, Token::new(1));
    let Some(Request::Notice { notice }) = out.pop() else { panic!("failed grant notice") };
    assert_eq!(notice, Notice::Failure(Failure::Credential { account: 7, reason: crate::CredentialFailure::Missing }));
    let Some(Request::Exit { status: crate::ExitStatus::Accepted }) = out.pop() else { panic!("closed input exits") };
    assert!(out.is_empty());
}

fn answer_with_a_late_state_save(gone_first: bool) {
    let (mut domain, env, mut out) = running(crate::tests::config());
    crate::step(&mut domain, &env, Event::Line { text: b"follow-up".as_slice().into() }, &mut out);
    drop(crate::tests::drain_requests(&mut out));
    parent_request(
        &mut domain,
        &env,
        parent::Request::Answered {
            client: Token::new(1),
            answer: host::Answer { read: None, turns: 0, spent: 0, result: host::RunResult::Parked },
            detail: Box::default(),
        },
        &mut out,
    );
    assert!(domain.chat.phase.is_answered());
    assert!(!domain.child.is_gone());
    let gone = crate::fixture::gone(
        host::End::Exited { exit: Some(host::Exit::Code(0)), forced: host::Forced::No },
        Box::new([]),
    );
    if gone_first {
        parent_request(&mut domain, &env, gone, &mut out);
        assert!(domain.chat.phase.is_answered());
        assert!(domain.child.is_gone());
        super::state_saved(&mut domain, &env, &mut out);
    } else {
        super::state_saved(&mut domain, &env, &mut out);
        assert!(domain.chat.phase.is_answered());
        parent_request(&mut domain, &env, gone, &mut out);
    }
    assert!(domain.chat.phase.is_loading());
    assert_eq!(domain.lines.carry()[0].name, Token::new(1));
    assert!(!domain.chat.state_pending());
}

#[test]
fn a_late_state_save_after_the_answer_is_handled_before_gone() {
    answer_with_a_late_state_save(false);
}

#[test]
fn gone_before_a_late_state_save_keeps_the_line_until_its_save_finishes() {
    answer_with_a_late_state_save(true);
}

#[test]
fn gone_after_a_late_turn_save_shows_the_answer_once() {
    let (mut domain, env, mut out) = running(crate::tests::config());
    domain.turns.unsaved.push((1, 1));
    parent_request(
        &mut domain,
        &env,
        parent::Request::Answered {
            client: Token::new(1),
            answer: host::Answer { read: None, turns: 1, spent: 0, result: host::RunResult::Parked },
            detail: Box::default(),
        },
        &mut out,
    );
    super::turn_saved(&mut domain, &env, 1, &mut out);
    assert!(domain.chat.phase.is_answered());
    parent_request(
        &mut domain,
        &env,
        crate::fixture::gone(
            host::End::Exited { exit: Some(host::Exit::Code(0)), forced: host::Forced::No },
            Box::new([]),
        ),
        &mut out,
    );
    assert!(domain.chat.phase.is_loading());
    assert_eq!(shown_requests(&mut out), 1);
}

#[test]
fn a_run_refused_for_its_effort_shows_the_effort_and_the_choices() {
    use crate::ChoiceNotice;
    let effort = b"high".as_slice();
    let mut config = crate::tests::config();
    config.models[0].effort = Some(effort.into());
    config.efforts[0] = Box::new([effort.into()]);
    let (mut domain, env, mut out) = running(config);
    let result = host::RunResult::Refused { refusal: host::Refusal::Invalid(host::RunInvalid::Effort) };
    let answer = host::Answer { read: None, turns: 0, spent: 0, result };
    let request = parent::Request::Answered { client: Token::new(1), answer, detail: Box::default() };
    parent_request(&mut domain, &env, request, &mut out);
    parent_request(&mut domain, &env, crate::fixture::gone(host::End::Settled, Box::default()), &mut out);
    let Some(Request::Notice { notice }) = out.pop() else { panic!("effort refusal notice") };
    let choice = ChoiceNotice::Effort { effort: effort.into(), choices: Box::new([effort.into()]), dropped: false };
    assert_eq!(notice, Notice::Failure(Failure::EffortRefused(choice)));
}

fn shown_requests(out: &mut Queue<Request>) -> u32 {
    let mut shown = 0_u32;
    for _ in 0..out.len() {
        match out.pop() {
            Some(Request::Notice { notice: Notice::Turn { .. } | Notice::Answer(_) }) => {
                shown = shown.checked_add(1).expect("bounded output");
            }
            Some(
                Request::Input { .. }
                | Request::Notice { .. }
                | Request::Answered { .. }
                | Request::Unanswered { .. }
                | Request::Load
                | Request::SelectChat { .. }
                | Request::SaveState { .. }
                | Request::SaveTurn { .. }
                | Request::SaveDelivery { .. }
                | Request::Git { .. }
                | Request::PlainStatus { .. }
                | Request::Credential { .. }
                | Request::Exit { .. }
                | Request::Lower { .. }
                | Request::Process(_),
            ) => {}
            None => break,
        }
    }
    shown
}

#[test]
fn gone_fills_the_agents_first_cause_with_the_exit_the_kit_reported() {
    let (mut domain, env, mut out) = running(crate::tests::config());
    parent_request(
        &mut domain,
        &env,
        parent::Request::Faulted { client: Token::new(1), fault: host::Fault::Exited },
        &mut out,
    );
    assert_eq!(domain.end_cause(), Some(crate::EndCause::Agent { ended: None }));
    parent_request(
        &mut domain,
        &env,
        crate::fixture::gone(
            host::End::Exited { exit: Some(host::Exit::Signal(9)), forced: host::Forced::Killed },
            Box::default(),
        ),
        &mut out,
    );
    assert_eq!(domain.end_cause(), Some(crate::EndCause::Agent { ended: Some(host::Exit::Signal(9)) }));
    let mut notices = 0_u32;
    let mut completed = 0_u32;
    for _ in 0..out.len() {
        match out.pop() {
            Some(Request::Notice {
                notice:
                    Notice::Failure(Failure::Agent {
                        end: host::End::Exited { exit: Some(host::Exit::Signal(9)), forced: host::Forced::Killed },
                        ..
                    }),
            }) => {
                notices = notices.checked_add(1).expect("bounded output");
            }
            Some(Request::Unanswered {
                cause: crate::EndCause::Agent { ended: Some(host::Exit::Signal(9)) }, ..
            }) => {
                completed = completed.checked_add(1).expect("bounded output");
            }
            _ => {}
        }
    }
    assert_eq!(notices, 1);
    assert_eq!(completed, 1);
}

#[test]
fn gone_without_observed_process_evidence_keeps_ended_absent() {
    for end in [host::End::Unspawned, host::End::Settled, host::End::Exited { exit: None, forced: host::Forced::No }] {
        let (mut domain, env, mut out) = running(crate::tests::config());
        parent_request(&mut domain, &env, crate::fixture::gone(end, Box::default()), &mut out);
        assert_eq!(domain.end_cause(), Some(crate::EndCause::Agent { ended: None }));
        let mut completed = 0_u32;
        for _ in 0..out.len() {
            if let Some(Request::Unanswered { cause: crate::EndCause::Agent { ended: None }, .. }) = out.pop() {
                completed = completed.checked_add(1).expect("bounded output");
            }
        }
        assert_eq!(completed, 1);
    }
}

fn after_an_answered_agents_forced_exit(result: host::RunResult) -> Queue<Request> {
    let mut config = crate::tests::config();
    config.mode = crate::Mode::Headless { prompt: b"hello".as_slice().into() };
    let (mut domain, env, mut out) = running(config);
    parent_request(
        &mut domain,
        &env,
        parent::Request::Answered {
            client: Token::new(1),
            answer: host::Answer { read: None, turns: 0, spent: 0, result },
            detail: Box::new([]),
        },
        &mut out,
    );
    while out.pop().is_some() {}
    parent_request(
        &mut domain,
        &env,
        crate::fixture::gone(
            host::End::Exited { exit: Some(host::Exit::Signal(9)), forced: host::Forced::Killed },
            Box::new([]),
        ),
        &mut out,
    );
    out
}

#[test]
fn a_forced_exit_after_a_parked_or_cancelled_answer_keeps_its_terminal() {
    for (result, expected) in [
        (host::RunResult::Parked, crate::ExitStatus::NeedsInput),
        (host::RunResult::Failed { failure: host::RunFailure::Cancelled }, crate::ExitStatus::Interrupted),
    ] {
        let mut out = after_an_answered_agents_forced_exit(result);
        let Some(Request::Notice {
            notice: Notice::Warning(crate::Warning::ForcedExit { forced: host::Forced::Killed, .. }),
        }) = out.pop()
        else {
            panic!("warning comes before the answer")
        };
        let mut exit = None;
        while let Some(request) = out.pop() {
            if let Request::Exit { status } = request {
                exit = Some(status);
            }
        }
        assert_eq!(exit, Some(expected));
    }
}

#[test]
fn the_host_answer_clock_precedes_teardown_and_durable_saves() {
    let (mut domain, mut env, mut out) = running(crate::tests::config());
    env.wall = Wall::from_nanos(36_000_000_000_000);
    crate::step(&mut domain, &env, Event::Line { text: b"follow-up".as_slice().into() }, &mut out);
    drop(crate::tests::drain_requests(&mut out));
    assert_eq!(domain.answer_wall(), None);
    parent_request(
        &mut domain,
        &env,
        parent::Request::Answered {
            client: Token::new(1),
            answer: host::Answer { read: None, turns: 3, spent: 29, result: host::RunResult::Parked },
            detail: Box::default(),
        },
        &mut out,
    );
    assert!(out.is_empty());
    let answered_at = Some(env.wall);
    env.wall = Wall::from_nanos(39_600_000_000_000);
    parent_request(&mut domain, &env, crate::fixture::gone(host::End::Settled, Box::default()), &mut out);
    assert_eq!(domain.answer_wall(), answered_at);
    assert!(out.is_empty());
    crate::step(&mut domain, &env, Event::StateSaved, &mut out);
    assert_eq!(domain.answer_wall(), answered_at);
    let Some(Request::Notice { notice: Notice::Answer(_) }) = out.pop() else { panic!("settled answer is shown") };
    let Some(Request::Answered { .. }) = out.pop() else { panic!("settled answer reaches the shell") };
}

mod older {
    use super::*;
    #[test]
    fn an_older_load_before_any_run_preserves_its_reason_through_notice_cause_and_exit_one() {
        for why in [
            crate::Older::Unrecorded,
            crate::Older::Layout(crate::StoreFamily::Store),
            crate::Older::Layout(crate::StoreFamily::Transcript),
        ] {
            let limits = crate::tests::limits();
            let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.clone() };
            let mut domain = Domain::new(crate::tests::config(), &limits, 71).expect("loading parent");
            let mut out = Queue::uncapped();
            let reason = crate::StoreFailure::OlderBuild { why };
            crate::step(&mut domain, &env, Event::StoreFailed { turn: None, reason }, &mut out);
            assert_eq!(domain.ending.cause(), Some(crate::EndCause::Host { turn: None, reason }));
            let Some(Request::Notice { notice: Notice::Failure(Failure::Store { turn: None, reason: found }) }) =
                out.pop()
            else {
                panic!("older notice before exit");
            };
            assert_eq!(found, reason);
            let Some(Request::Exit { status }) = out.pop() else {
                panic!("older-load exit");
            };
            assert_eq!(status, crate::ExitStatus::RunFailed);
            assert!(out.is_empty());
        }
    }
}

#[test]
fn a_late_metadata_save_cannot_show_an_answer_before_its_turn_is_durable() {
    let (mut domain, env, mut out) = running(crate::tests::config());
    crate::step(&mut domain, &env, Event::Line { text: b"follow-up".as_slice().into() }, &mut out);
    let turn = crate::fixture::turn();
    parent_request(
        &mut domain,
        &env,
        parent::Request::Turn {
            client: Token::new(1),
            turn: host::Turn {
                opens_window: true,
                number: 1,
                read: None,
                spent: 0,
                body: host::TurnValue::new(turn, crate::tests::host_limits().turn_bytes).expect("bounded native turn"),
            },
        },
        &mut out,
    );
    parent_request(
        &mut domain,
        &env,
        parent::Request::Answered {
            client: Token::new(1),
            answer: host::Answer { read: None, turns: 1, spent: 0, result: host::RunResult::Parked },
            detail: Box::default(),
        },
        &mut out,
    );
    parent_request(&mut domain, &env, crate::fixture::gone(host::End::Settled, Box::default()), &mut out);
    drop(crate::tests::drain_requests(&mut out));
    crate::step(&mut domain, &env, Event::StateSaved, &mut out);
    assert!(out.is_empty());
    assert!(domain.chat.phase.is_answered());
    crate::step(&mut domain, &env, Event::TurnSaved { number: 1 }, &mut out);
    let requests = crate::tests::drain_requests(&mut out);
    let mut answered = 0_u32;
    for request in requests {
        if let Request::Answered { .. } = request {
            answered += 1;
        }
    }
    assert_eq!(answered, 1);
    assert!(domain.chat.phase.is_loading());
}

#[test]
fn an_unknown_delivery_settles_at_the_next_load_of_the_same_invocation() {
    let mut config = plain_delivery_config();
    config.workspace.as_mut().expect("delivery workspace").directories[0].git = true;
    let (mut domain, env, mut out) = running(config);
    domain.chat.live_mut().heads.insert(0, b"before".as_slice().into()).expect("one repository");
    call_delivery(&mut domain, &env, &mut out);
    let Some(Request::Git { owner, op: crate::GitOp::Status, .. }) = out.pop() else { panic!("survey repository") };
    crate::step(
        &mut domain,
        &env,
        Event::Git {
            owner,
            result: crate::GitResult::Status { changed: true, merging: None, head: b"before".as_slice().into() },
        },
        &mut out,
    );
    let record = saved_intent(&mut out);
    crate::step(&mut domain, &env, Event::DeliverySaved { name: record.name }, &mut out);
    let Some(Request::Git { op: crate::GitOp::Commit { .. }, .. }) = out.pop() else { panic!("execute commit") };
    crate::step(
        &mut domain,
        &env,
        Event::Git {
            owner,
            result: crate::GitResult::Uncertain {
                reason: run::DeliveryReason::TimedOut,
                diagnostic: Box::new(run::Diagnostic::empty()),
            },
        },
        &mut out,
    );
    let Some(Request::Git { op: crate::GitOp::Inspect { name }, .. }) = out.pop() else { panic!("inspect commit") };
    assert_eq!(name, record.name);
    crate::step(
        &mut domain,
        &env,
        Event::Git {
            owner,
            result: crate::GitResult::Failed {
                reason: run::DeliveryReason::Broken,
                diagnostic: Box::new(run::Diagnostic::empty()),
            },
        },
        &mut out,
    );
    assert!(out.is_empty(), "unknown keeps the intent for the next load");
    parent_request(
        &mut domain,
        &env,
        parent::Request::Answered {
            client: Token::new(1),
            answer: host::Answer { read: None, turns: 0, spent: 0, result: host::RunResult::Parked },
            detail: Box::default(),
        },
        &mut out,
    );
    parent_request(&mut domain, &env, crate::fixture::gone(host::End::Settled, Box::default()), &mut out);
    let Some(Request::Notice { notice: Notice::Answer(_) }) = out.pop() else { panic!("show answer") };
    let Some(Request::Answered { .. }) = out.pop() else { panic!("answer reaches shell") };
    let Some(Request::Load) = out.pop() else { panic!("reload same chat") };
    assert!(out.is_empty());
    crate::step(
        &mut domain,
        &env,
        Event::Loaded { state: None, transcript: None, deliveries: Box::from([*record]) },
        &mut out,
    );
    let Some(Request::Git { owner, op: crate::GitOp::Inspect { name }, .. }) = out.pop() else {
        panic!("reconcile intent")
    };
    assert_eq!(owner, Token::new(0));
    crate::step(
        &mut domain,
        &env,
        Event::Git { owner, result: crate::GitResult::Inspected { head: b"landed".as_slice().into(), named: true } },
        &mut out,
    );
    let Some(Request::SaveDelivery { record }) = out.pop() else { panic!("save reconciled answer") };
    assert_eq!(record.name, name);
    let crate::DeliveryState::Answer(run::Delivery::Delivered(_)) = &record.state else {
        panic!("save a delivered answer")
    };
    crate::step(&mut domain, &env, Event::DeliverySaved { name }, &mut out);
    assert!(domain.chat.phase.is_idle());
    assert!(out.is_empty());
}

#[test]
fn a_delivery_save_landing_after_a_store_failure_is_ignored() {
    let (mut domain, env, mut out) = running(plain_delivery_config());
    call_delivery(&mut domain, &env, &mut out);
    let Some(Request::PlainStatus { owner, .. }) = out.pop() else { panic!("survey delivery") };
    assert_eq!(owner, Token::new(55));
    crate::step(&mut domain, &env, Event::PlainStatus { owner, changed: true }, &mut out);
    let record = saved_intent(&mut out);
    assert!(out.is_empty());
    crate::step(&mut domain, &env, Event::StoreFailed { turn: Some(2), reason: crate::StoreFailure::Write }, &mut out);
    assert_store_failure(&mut out, Some(2));
    assert!(out.is_empty());
    crate::step(&mut domain, &env, Event::DeliverySaved { name: record.name }, &mut out);
    assert!(out.is_empty());
}

#[test]
fn a_store_failure_while_a_delivered_answer_saves_tells_the_person() {
    let (mut domain, env, mut out) = running(plain_delivery_config());
    call_delivery(&mut domain, &env, &mut out);
    let Some(Request::PlainStatus { owner, .. }) = out.pop() else { panic!("survey delivery") };
    assert_eq!(owner, Token::new(55));
    crate::step(&mut domain, &env, Event::PlainStatus { owner, changed: true }, &mut out);
    let record = saved_intent(&mut out);
    assert!(out.is_empty());
    crate::step(&mut domain, &env, Event::DeliverySaved { name: record.name }, &mut out);
    let Some(Request::SaveDelivery { record }) = out.pop() else { panic!("save delivered answer") };
    let crate::DeliveryState::Answer(run::Delivery::Delivered(_)) = &record.state else {
        panic!("save a delivered answer")
    };
    crate::step(&mut domain, &env, Event::StoreFailed { turn: Some(2), reason: crate::StoreFailure::Write }, &mut out);
    assert_store_failure(&mut out, Some(2));
    let Some(Request::Notice { notice: Notice::Failure(Failure::DeliveryUnsaved) }) = out.pop() else {
        panic!("landed delivery was not saved")
    };
    assert!(out.is_empty());
}

#[test]
fn a_store_failure_while_a_reconciled_answer_saves_exits_with_its_notice() {
    let limits = crate::tests::limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.clone() };
    let mut domain = Domain::new(plain_delivery_config(), &limits, 71).expect("loading chat");
    let mut out = Queue::uncapped();
    crate::resume(&mut domain, &env, &mut out);
    let Some(Request::Load) = out.pop() else { panic!("initial load") };
    crate::step(
        &mut domain,
        &env,
        Event::Loaded {
            state: None,
            transcript: None,
            deliveries: Box::from([crate::DeliveryRecord {
                name: run::CallName { activation: 1, completion: 2, position: 1 },
                state: crate::DeliveryState::Intent(crate::DeliveryIntent {
                    directories: Box::from([crate::IntentDirectory {
                        directory: 0,
                        changed: true,
                        head: None,
                        push: None,
                    }]),
                }),
                landed: Box::default(),
            }]),
        },
        &mut out,
    );
    let Some(Request::SaveDelivery { record }) = out.pop() else { panic!("save reconciled plain delivery") };
    let crate::DeliveryState::Answer(run::Delivery::Delivered(_)) = &record.state else {
        panic!("save a delivered answer")
    };
    crate::step(&mut domain, &env, Event::StoreFailed { turn: None, reason: crate::StoreFailure::Write }, &mut out);
    assert_store_failure(&mut out, None);
    let Some(Request::Notice { notice: Notice::Failure(Failure::DeliveryUnsaved) }) = out.pop() else {
        panic!("unsaved delivery notice before exit")
    };
    let Some(Request::Exit { status: crate::ExitStatus::RunFailed }) = out.pop() else {
        panic!("failed recovery exits")
    };
    assert!(out.is_empty());
}

#[test]
fn an_alien_save_ask_is_refused_without_touching_the_local_tree() {
    use crate::agent::{Output, command};
    let (mut domain, env, mut out) = running(crate::tests::config());
    let charter = domain.charter().0;
    assert!(!charter.grants.save, "local charters grant no save");
    let start = host::Start {
        live: None,
        messages: Box::new([]),
        logical_run: Token::new(1),
        identity: None,
        activation: 1,
        workspace: None,
        charter: host::Charter::new(charter, u64::MAX).expect("local charter"),
        transcript: None,
        answered: Box::new([]),
        directories: Box::new([]),
        grants: Box::new([]),
    };
    command(&mut domain.agent, env.now, env.wall, parent::Event::Spawn { client: Token::new(1), start });
    let Some(Output::Process(process::Request::Spawn { owner, .. })) = domain.agent.pop() else { panic!("spawn") };
    domain.child = crate::child::Child::Starting;
    domain.child.started(owner);
    domain.agent.process(env.now, env.wall, process::Event::Spawned { owner, process: Token::new(2) });
    while let Some(output) = domain.agent.pop() {
        match output {
            Output::Process(process::Request::Send { .. }) => {
                domain.agent.process(env.now, env.wall, process::Event::Sent { owner });
            }
            Output::Parent(_) | Output::Lower { .. } | Output::Process(_) => {}
        }
    }
    domain.agent.process(env.now, env.wall, process::Event::Received { owner, message: host::Up::Admitted });
    while domain.agent.pop().is_some() {}
    domain.agent.process(
        env.now,
        env.wall,
        process::Event::Received {
            owner,
            message: host::Up::Call {
                call: Token::new(55),
                name: host::CallName { activation: 1, completion: 2, position: 0 },
                deadline: Time::from_nanos(99),
                ask: host::Ask::Save,
            },
        },
    );
    let mut refusal = None;
    while let Some(output) = domain.agent.pop() {
        match output {
            Output::Parent(request @ parent::Request::Called { .. }) => {
                parent_request(&mut domain, &env, request, &mut out);
            }
            Output::Process(process::Request::Send { message: host::Down::Answer { call, reply }, .. }) => {
                assert_eq!(call, Token::new(55));
                refusal = Some(reply);
            }
            Output::Parent(_) | Output::Lower { .. } | Output::Process(_) => {}
        }
    }
    let Some(host::Reply::Delivery(host::Delivery::Refused(refused))) = refusal else {
        panic!("save refused: {refusal:?}")
    };
    assert_eq!(refused.explanation(), b"this host does not save");
    assert!(out.is_empty(), "no git or plain-tree operation starts");
}
