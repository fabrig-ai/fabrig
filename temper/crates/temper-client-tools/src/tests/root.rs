use crate::tests::fixtures;
use jig_client::{self as client, contract as c};

// Queue the last delivery and its terminal before routing either notification,
// as the native client can do when it drains several wire events together.
fn retained_delivery(root: &mut fixtures::Harness, subject: client::Watchable, delivery: c::Delivery) {
    let env = root.env(super::Limits::new(root.limits.domain, root.limits.tools));
    let watch = root.watch_token(&subject);
    let mut pending = skein_lib::Queue::with_capacity(fixtures::OUTPUTS);
    for up in [
        client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) },
        client::Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded },
    ] {
        super::step_client(&mut root.root, &env, client::Event::Up { up }, &mut pending);
    }
    root.watches.remove(&watch).expect("engine ended watch");
    let mut child = skein_lib::Queue::with_capacity(fixtures::OUTPUTS);
    let mut out = skein_lib::Queue::with_capacity(fixtures::OUTPUTS);
    super::drain(&mut root.root, &env, &mut pending, &mut child, &mut out);
    root.root.changed();
    root.outputs(out);
}

#[test]
fn an_ended_tree_delivery_waits_for_the_tasks_read() {
    for json in [false, true] {
        let argv = fixtures::extended(&["task", "show", "41", "--tree"], if json { &["--json"] } else { &[] });
        let (mut root, call) = fixtures::read_call(argv.as_slice());
        let task = fixtures::ended_task(c::Phase::Done);
        let client::Latest::Jig(delivery) = fixtures::owner_tree_at(task.chip.clone()) else { panic!("owner tree") };
        retained_delivery(&mut root, fixtures::tree_watch(), delivery);
        assert!(root.answer(call).is_none(), "task read is still pending");
        root.serve(call, fixtures::ended_task_result(task, c::Ending::Done));
        let answer = root.answer(call).expect("retained tree and ended task complete the read");
        assert_eq!(answer.status, 0);
        assert!(answer.stderr.is_empty());
        assert!(fixtures::has(&answer, if json { b"\"tree\":" } else { b"tree:\n" }));
        assert_eq!(root.watches.len(), fixtures::SHARED_WATCHES);
    }
}

#[test]
fn a_filled_tree_slot_survives_its_close_while_the_task_read_finishes() {
    let (mut root, call) = fixtures::read_call(&["task", "show", "41", "--tree"]);
    root.deliver_subject(&fixtures::tree_watch(), c::Delivery::Tree(Box::new(fixtures::composite_tree())));
    assert!(root.answer(call).is_none(), "task read is still pending after the tree closes");
    root.serve(call, fixtures::task_result(fixtures::goal(fixtures::TASK, fixtures::waiting_on_me())));
    assert_eq!(root.answer(call).expect("both slots filled").status, 0);
}

#[test]
fn a_wait_checks_its_first_task_document_even_when_the_watch_has_ended() {
    let (mut root, call) = fixtures::waiting_call(fixtures::WAIT_TASK, fixtures::UNTIL_REPLIED);
    retained_delivery(
        &mut root,
        fixtures::task_watch(),
        c::Delivery::Task(Box::new(fixtures::ended_task(c::Phase::Done))),
    );
    let answer = root.answer(call).expect("first retained document already meets replied");
    assert_eq!(answer.status, 0);
    assert!(answer.stdout.ends_with(b",\"met\":true}"));
    assert!(root.reads.is_empty(), "the first document settled the condition");
    assert_eq!(crate::root::next_deadline(&root.root), None);
}
#[test]
fn two_calls_in_flight_are_answered_each_under_its_own_number() {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let a = root.submit(fixtures::TASK_CALL);
    let b = root.submit(fixtures::OTHER_TASK_CALL);
    root.serve(b, fixtures::task_result(fixtures::goal(fixtures::OTHER_TASK, fixtures::waiting_on_me())));
    assert!(root.answer(a).is_none());
    assert_eq!(root.answer(b).expect("second answered").status, 0);
    root.serve(a, fixtures::task_result(fixtures::goal(fixtures::TASK, fixtures::waiting_on_me())));
    assert_eq!(root.answer(a).expect("first answered").status, 0);
}
#[test]
fn an_unknown_asker_tells_no_child() {
    for (asker, facts) in [(fixtures::UNKNOWN_ASKER, 1), (temper_client_domain::owner::Owner::Forge.asker(), 0)] {
        let mut root = fixtures::signed_in(fixtures::PARTY);
        root.client(client::Event::Watch {
            handle: client::Handle { asker, number: 1 },
            subject: client::Watchable::Jig(c::Subject::Me),
        });
        assert_eq!(root.facts, facts, "{asker:?}");
    }
}
#[test]
fn new_calls_start_above_all_kept_call_numbers() {
    let mut root = fixtures::started(fixtures::kept(fixtures::RESTORED_PREFIX));
    assert_eq!(root.root.next, Some(fixtures::RESTORED_PREFIX + 1));
    root.deliver(c::Delivery::Me(Box::new(fixtures::me(fixtures::PARTY))));
    let call = root.submit(fixtures::TASK_CALL);
    root.serve(call, fixtures::task_result(fixtures::goal(fixtures::TASK, fixtures::waiting_on_me())));
    assert_eq!(root.answer(call).expect("new read answered").status, 0);
}
#[test]
fn an_unanswered_adoption_is_named_after_its_me_watch_drops() {
    let records = fixtures::restored_adoption(fixtures::PARTY);
    let mut root = fixtures::started(records.clone());
    root.deliver(c::Delivery::Me(Box::new(fixtures::me(fixtures::PARTY))));
    let watch = root.watch_token(&client::Watchable::Jig(c::Subject::Me));
    root.watches.remove(&watch).expect("stream's me watch");
    root.client(client::Event::Up { up: client::Up::WatchEnded { watch, end: c::WatchEnd::Dropped } });
    let me = root.root.client().document(client::Handle { asker: super::RUNNER.asker(), number: 0 }).expect("me");
    assert_eq!(me.state, client::State::Behind);
    for record in &records {
        assert!(root.root.client().pending(record.request.key).is_some(), "restored adoption stays pending");
    }
    assert_eq!(
        root.root.unanswered().as_ref(),
        b"kept: adopt, first sent 00:00 UTC, unanswered: it clears when the engine answers it, or past its retention; pending lists it\n"
    );
}

#[test]
fn call_number_exhaustion_exits_1_without_sending() {
    let mut root = fixtures::started(fixtures::kept(u32::MAX));
    root.deliver(c::Delivery::Me(Box::new(fixtures::me(fixtures::PARTY))));
    let call = root.submit(fixtures::TASK_CALL);
    let answer = root.answer(call).expect("exhaustion answered locally");
    assert_eq!((answer.status, answer.stderr.as_ref()), (1, b"call number exhausted\n".as_slice()));
    assert_eq!(root.reads.len(), 0);
}

#[test]
fn every_call_handle_uses_its_prefix_under_one_owner() {
    use crate::jig::runner;
    let mut root = fixtures::signed_in(fixtures::PARTY);
    for prefix in [1, fixtures::RESTORED_PREFIX] {
        let mut out = skein_lib::Queue::with_capacity(fixtures::OUTPUTS);
        let env =
            skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: root.limits.tools };
        runner::call(
            &mut root.root.runner,
            &root.root.client,
            &env,
            runner::Called {
                number: fixtures::CORRELATION,
                call_number: prefix,
                call: fixtures::call(fixtures::TASK_CALL),
                json: false,
                value: None,
            },
            &mut out,
        );
        let runner::Out::Client { event: client::Event::Read { handle, .. } } = out.pop().expect("read once") else {
            panic!("read event")
        };
        assert_eq!(handle, client::Handle { asker: super::RUNNER.asker(), number: (u64::from(prefix) << 32u32) | 1 });
    }
}

// A delayed read and the previous act's answer can arrive in the same
// client batch, leaving owner requests queued beside client retirements.
fn queued_owner_outputs(
    root: &mut fixtures::Harness,
    previous: u32,
    follow_on: u32,
) -> skein_lib::Queue<super::Request> {
    let (read, _) = root.reads.remove(&follow_on).expect("follow-on read pending");
    let (attempt, _, _) = root.sends.get(&previous).expect("previous act sent");
    let env = root.env(super::Limits::new(root.limits.domain, root.limits.tools));
    let mut client = skein_lib::Queue::uncapped();
    let mut child = skein_lib::Queue::uncapped();
    let mut out = skein_lib::Queue::uncapped();
    super::step_client(
        &mut root.root,
        &env,
        client::Event::Up {
            up: client::Up::ReadDone {
                read,
                result: client::Readout::Jig(fixtures::task_result(fixtures::offered(&[c::ActKind::Message]))),
            },
        },
        &mut client,
    );
    // Route the completed read to its owner, retaining that owner's follow-on
    // outputs until the previous act's terminal is ready beside them.
    while let Some(request) = client.pop() {
        match request {
            super::ClientOutput::Linked(client::Request::Told { told })
            | super::ClientOutput::Unlinked(client::UnlinkedRequest::Told { told }) => {
                super::route(&mut root.root, &env, told, &mut child, &mut out);
            }
            super::ClientOutput::Linked(request) => out.push(super::Request::Client { request }),
            super::ClientOutput::Unlinked(request) => out.push(super::Request::Unlinked { request }),
        }
    }
    super::step_client(
        &mut root.root,
        &env,
        client::Event::Up {
            up: client::Up::Answer {
                attempt: *attempt,
                envelope: skein_lib::RequestEnvelope::Final,
                answer: Some(client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                    c::DocumentOutcome::Amended { task: 9 },
                )))),
                why: None,
            },
        },
        &mut client,
    );
    super::drain(&mut root.root, &env, &mut client, &mut child, &mut out);
    root.root.changed();
    out
}

#[test]
fn a_follow_on_named_request_waits_for_the_previous_erase() {
    const NAMED_CALLS: u32 = 2; // The previous act and its follow-on.
    let (mut root, previous) = fixtures::message();
    let follow_on = root.submit(fixtures::MESSAGE_CALL);
    // Fill the remaining admitted-call quota with reads that stay pending.
    for _ in NAMED_CALLS..root.limits.tools.rows {
        root.submit(fixtures::TASK_CALL);
    }
    let out = queued_owner_outputs(&mut root, previous, follow_on);
    let mut erased = false;
    let mut saved = false;
    for request in &out {
        match request {
            super::Request::Unlinked {
                request: client::UnlinkedRequest::Down { down: client::Down::Erase { .. } },
            } => {
                erased = true;
            }
            super::Request::Unlinked {
                request: client::UnlinkedRequest::Down { down: client::Down::Save { record } },
            } => {
                assert!(erased, "previous erase leaves before the follow-on named request");
                assert_eq!(record.key, fixtures::named("port-answer"));
                saved = true;
            }
            super::Request::Printed { number, .. } if *number == previous => {
                assert!(erased, "previous erase leaves before its result permits another named call");
            }
            super::Request::Client { .. }
            | super::Request::Unlinked { .. }
            | super::Request::Ready
            | super::Request::Record { .. }
            | super::Request::Kept { .. }
            | super::Request::Progress { .. }
            | super::Request::Printed { .. }
            | super::Request::Fact { .. } => {}
        }
    }
    assert!(erased && saved, "both the retirement and the new request leave");
    root.serving = Some(follow_on);
    root.outputs(out);
    root.serving = None;
    root.done(follow_on, c::Outcome::Document(c::DocumentOutcome::Amended { task: 9 }));
    assert_eq!(root.answer(previous).expect("previous call answered once").status, 0);
    assert_eq!(root.answer(follow_on).expect("follow-on call answered once").status, 0);
    assert!(root.answer(previous).is_none() && root.answer(follow_on).is_none());
}

fn retry(root: &mut fixtures::Harness, call: u32, why: client::Retry) {
    let (attempt, _, _) = root.sends.get(&call).expect("sent act");
    root.client(client::Event::Up {
        up: client::Up::Answer {
            attempt: *attempt,
            envelope: skein_lib::RequestEnvelope::Again,
            answer: None,
            why: Some(why),
        },
    });
}
#[test]
fn the_calls_last_retry_survives_its_next_attempt_until_the_answer() {
    for why in [
        client::Retry::Busy,
        client::Retry::NotReady,
        client::Retry::Lost,
        client::Retry::Undecodable,
        client::Retry::Full { held: 2, room_at: skein_lib::Wall::from_nanos(1_000_000_000) },
    ] {
        let (mut root, call) = fixtures::message();
        assert_eq!(root.root.last_retry(), None);
        retry(&mut root, call, why);
        assert_eq!(root.root.last_retry(), Some(why));
        let before = root.sends.get(&call).expect("first attempt").0;
        root.now = skein_lib::Time::from_nanos(2_000_000_000);
        root.fire();
        assert_ne!(root.sends.get(&call).expect("retried").0, before);
        assert_eq!(root.root.last_retry(), Some(why), "in flight keeps the last answer");
        retry(&mut root, call, client::Retry::NotReady);
        assert_eq!(root.root.last_retry(), Some(client::Retry::NotReady), "a later reason replaces it");
        root.now = skein_lib::Time::from_nanos(4_000_000_000);
        root.fire();
        root.done(call, c::Outcome::Document(c::DocumentOutcome::Said { task: fixtures::TASK, message: 1 }));
        assert_eq!(root.answer(call).expect("done").status, 0);
        assert_eq!(root.root.last_retry(), None);
    }
}
#[test]
fn a_retry_belongs_to_the_call_only_when_it_joins_the_kept_request() {
    let mut root = fixtures::started(fixtures::restored_message(fixtures::PARTY));
    root.deliver(c::Delivery::Me(Box::new(fixtures::me(fixtures::PARTY))));
    retry(&mut root, fixtures::EMPTY, client::Retry::Busy);
    assert_eq!(root.root.last_retry(), None, "replay is not the call's act");
    let call = root.submit(fixtures::MESSAGE_CALL);
    root.serve(call, fixtures::task_result(fixtures::offered(&[c::ActKind::Message])));
    assert_eq!(root.root.last_retry(), Some(client::Retry::Busy), "joined before a new attempt");
    root.now = skein_lib::Time::from_nanos(2_000_000_000);
    root.serving = Some(fixtures::EMPTY);
    root.fire();
    root.serving = None;
    assert_eq!(root.root.last_retry(), Some(client::Retry::Busy));
    let read = root.submit(fixtures::TASK_CALL);
    assert_eq!(root.root.last_retry(), None, "another call has no retry of its own");
    root.serve(read, fixtures::task_result(fixtures::offered(&[c::ActKind::Message])));
    assert_eq!(root.answer(read).expect("read").status, 0);
    assert_eq!(root.root.last_retry(), Some(client::Retry::Busy));
    root.done(fixtures::EMPTY, c::Outcome::Document(c::DocumentOutcome::Said { task: fixtures::TASK, message: 1 }));
    assert_eq!(root.answer(call).expect("joined act answered").status, 0);
    assert_eq!(root.root.last_retry(), None);
}
