use jig_brief_world::{LIMITS, World, referee};
use jig_domain_brief::{
    Core, GatherEvent, GatherFailure, GatherFate, GatherMissing, GatherPlaced, GatherRequest, Planned,
};
use skein_lib::{Duration, Time, Token};

fn connector(token: u64, required: bool, priority: u16) -> Planned {
    named_connector(token, b"section", required, priority)
}

fn begin(world: &mut World, sections: Box<[Planned]>, budget: u32) -> Vec<GatherRequest> {
    world.step(planned(sections, budget, Time::ZERO.saturating_add(Duration::from_secs(10))))
}

#[test]
fn a_connector_section_stays_with_its_owner_until_the_root_takes_it() {
    let mut world = World::new(LIMITS);
    let token = Token::new(2);
    world.put(token, b"connector words");
    let requests = begin(
        &mut world,
        Box::new([
            Planned::Core { kind: Core::Task, text: b"task".as_slice().into(), limit: 40, priority: 0, required: true },
            connector(2, true, 1),
        ]),
        40,
    );
    assert!(world.held(token), "brief has only the token");
    assert!(matches!(requests.as_slice(), [GatherRequest::Gather { connector: 3, section, .. }] if *section == token));
    let done = world.settle(requests);
    assert!(referee::within_budget(&done, 40));
    assert!(matches!(done.as_slice(), [GatherRequest::Complete { order, .. }]
        if matches!(order.as_ref(), [GatherPlaced::Core { kind: Core::Task, .. }, GatherPlaced::Connector { token: kept, .. }] if *kept == token)));
    assert!(world.closed(token));
    assert!(!world.held(token));
}

#[test]
fn required_sections_claim_room_before_optional_sections_and_cuts_go_to_the_owner() {
    let mut world = World::new(LIMITS);
    let token = Token::new(2);
    world.put(token, &[b'x'; 90]);
    let requests = begin(
        &mut world,
        Box::new([
            connector(2, false, 8),
            Planned::Core {
                kind: Core::Task,
                text: b"required task words".as_slice().into(),
                limit: 80,
                priority: 0,
                required: true,
            },
        ]),
        45,
    );
    let done = world.settle(requests);
    assert!(referee::within_budget(&done, 45));
    assert!(matches!(done.as_slice(), [GatherRequest::Complete { order, .. }]
        if matches!(order.as_ref(), [GatherPlaced::Connector { size, .. }, GatherPlaced::Core { kind: Core::Task, .. }] if *size < 90)));
    assert!(world.closed(token));
}

#[test]
fn a_required_section_missing_fails_the_brief_and_drops_its_token() {
    let mut world = World::new(LIMITS);
    let asked = begin(&mut world, Box::new([connector(2, true, 0)]), 40);
    let done = world.settle(asked);
    assert_eq!(
        done,
        [GatherRequest::Failed {
            brief: Token::new(1),
            why: GatherMissing::Failed,
            failure: GatherFailure {
                fate: GatherFate::Transient,
                section: b"section",
                words: Box::from(b"unavailable".as_slice())
            }
        }]
    );
    assert!(world.closed(Token::new(2)));
}

#[test]
fn a_late_optional_section_is_marked_missing_and_dropped() {
    let mut world = World::new(LIMITS);
    let token = Token::new(2);
    world.put(token, b"late words");
    world.pause(token);
    let asked = begin(&mut world, Box::new([connector(2, false, 0)]), 40);
    assert!(matches!(asked.as_slice(), [GatherRequest::Gather { .. }]));
    let expired = world.fire(Time::ZERO.saturating_add(Duration::from_secs(10)));
    let done = world.settle(expired);
    assert!(matches!(done.as_slice(), [GatherRequest::Complete { order, .. }]
        if matches!(order.as_ref(), [GatherPlaced::Missing { why: GatherMissing::Late, .. }])));
    assert!(world.closed(token));
}

#[test]
fn an_amendment_while_gathering_drops_every_section_gathered() {
    let mut world = World::new(LIMITS);
    for number in [2, 3] {
        let token = Token::new(number);
        world.put(token, b"held");
        world.pause(token);
    }
    let asked = begin(&mut world, Box::new([connector(2, false, 0), connector(3, false, 1)]), 40);
    assert_eq!(asked.len(), 2);
    let abandoned = world.step(GatherEvent::Abandon { brief: Token::new(1) });
    let done = world.settle(abandoned);
    assert!(done.is_empty());
    assert!(world.closed(Token::new(2)) && world.closed(Token::new(3)));
}

#[test]
fn sections_that_do_not_fit_are_cut_in_their_kinds_order_and_say_how_much() {
    let mut world = World::new(LIMITS);
    let attempts = b"early attempt\nrecent attempt\n";
    let asked = begin(
        &mut world,
        Box::new([
            Planned::Core {
                kind: Core::TranscriptTail,
                text: b"last transcript turn".as_slice().into(),
                limit: 30,
                priority: 2,
                required: false,
            },
            Planned::Core {
                kind: Core::Task,
                text: b"do work\n".as_slice().into(),
                limit: 40,
                priority: 0,
                required: true,
            },
            Planned::Core {
                kind: Core::Attempts,
                text: attempts.as_slice().into(),
                limit: 25,
                priority: 1,
                required: false,
            },
        ]),
        33,
    );
    assert!(referee::within_budget(&asked, 33));
    let [GatherRequest::Complete { order, .. }] = asked.as_slice() else { panic!("completed brief: {asked:?}") };
    let [
        GatherPlaced::CoreMissing { kind: Core::TranscriptTail, why: GatherMissing::Budget },
        GatherPlaced::Core { kind: Core::Task, text: task },
        GatherPlaced::Core { kind: Core::Attempts, text: cut },
    ] = order.as_ref()
    else {
        panic!("source order retained: {order:?}")
    };
    assert_eq!(task.as_ref(), b"do work\n");
    assert!(cut.len() <= 25);
    assert!(cut.ends_with(b"attempt\n"), "newest attempt bytes retained: {cut:?}");
    let mut counted = false;
    for lost in 1..attempts.len() {
        let marker = format!("[{lost} bytes cut]\n");
        if cut.starts_with(marker.as_bytes()) {
            assert_eq!(&cut[marker.len()..], &attempts[lost..]);
            counted = true;
        }
    }
    assert!(counted, "cut names exactly the omitted bytes: {cut:?}");
}

#[test]
fn a_required_section_missing_at_the_deadline_fails_before_completion() {
    let mut world = World::new(LIMITS);
    let token = Token::new(2);
    world.put(token, b"required words");
    world.pause(token);
    let asked = begin(&mut world, Box::new([connector(2, true, 0)]), 40);
    assert!(matches!(asked.as_slice(), [GatherRequest::Gather { .. }]));
    let expired = world.fire(Time::ZERO.saturating_add(Duration::from_secs(10)));
    let done = world.settle(expired);
    assert_eq!(
        done,
        [GatherRequest::Failed {
            brief: Token::new(1),
            why: GatherMissing::Late,
            failure: GatherFailure {
                fate: GatherFate::Infrastructure,
                section: b"section",
                words: Box::from(b"not gathered by its deadline".as_slice())
            }
        }]
    );
    assert!(world.closed(token));
}

fn failure(fate: GatherFate, section: &'static [u8], words: &'static [u8]) -> GatherFailure {
    GatherFailure { fate, section, words: Box::from(words) }
}

fn named_connector(token: u64, name: &'static [u8], required: bool, priority: u16) -> Planned {
    Planned::Connector {
        connector: 3,
        kind: 7,
        name,
        token: Token::new(token),
        size: 0,
        limit: LIMITS.read_bytes,
        priority,
        required,
    }
}

fn planned(sections: Box<[Planned]>, budget: u32, deadline: Time) -> GatherEvent {
    GatherEvent::Plan { brief: Token::new(1), sections, budget, deadline }
}

#[test]
fn a_full_table_or_a_busy_brief_refuses_for_capacity() {
    // The first row has spare table room but reuses the active brief.
    // The second fills the table and asks for a different brief.
    for (capacity, next) in [(LIMITS.briefs, 1), (1, 2)] {
        let mut world = World::new(jig_domain_brief::Limits { briefs: capacity, ..LIMITS });
        begin(&mut world, Box::new([connector(2, true, 0)]), LIMITS.brief_bytes);
        let refused = world.step(GatherEvent::Plan {
            brief: Token::new(next),
            budget: LIMITS.brief_bytes,
            deadline: Time::ZERO.saturating_add(Duration::from_secs(10)),
            sections: Box::new([]),
        });
        assert_eq!(
            refused,
            [GatherRequest::Refused {
                brief: Token::new(next),
                failure: failure(GatherFate::Capacity, b"brief", b"no brief slot is free"),
            }]
        );
        let abandoned = world.step(GatherEvent::Abandon { brief: Token::new(1) });
        assert!(world.settle(abandoned).is_empty());
        assert!(world.closed(Token::new(2)));
    }
}

#[test]
fn malformed_plans_say_why_they_are_refused_permanently() {
    let deadline = Time::ZERO.saturating_add(Duration::from_secs(10));
    let too_many = (0..=LIMITS.sections).map(|n| connector(u64::from(n), false, 0)).collect();
    for (event, section, words) in [
        (
            planned(too_many, LIMITS.brief_bytes, deadline),
            b"brief".as_slice(),
            b"it has more sections than a brief may hold".as_slice(),
        ),
        (planned(Box::new([]), LIMITS.brief_bytes + 1, deadline), b"brief", b"its budget is past the limit"),
        (planned(Box::new([]), LIMITS.brief_bytes, Time::ZERO), b"brief", b"its deadline has passed"),
        (
            planned(Box::new([connector(2, false, 0), connector(2, false, 1)]), LIMITS.brief_bytes, deadline),
            b"brief",
            b"a section is planned twice",
        ),
        (
            planned(
                Box::new([Planned::Core {
                    kind: Core::TranscriptTail,
                    text: vec![b'x'; usize::try_from(LIMITS.brief_bytes).expect("small byte limit") + 1].into(),
                    limit: LIMITS.brief_bytes,
                    priority: 0,
                    required: false,
                }]),
                LIMITS.brief_bytes,
                deadline,
            ),
            b"transcript",
            b"its text is past the brief's limit",
        ),
    ] {
        let mut world = World::new(LIMITS);
        assert_eq!(
            world.step(event),
            [GatherRequest::Refused { brief: Token::new(1), failure: failure(GatherFate::Permanent, section, words) }]
        );
        assert!(world.domain.is_idle());
    }
}

#[test]
fn plans_at_the_admission_limits_are_accepted() {
    let deadline = Time::ZERO.saturating_add(Duration::from_nanos(1));
    let at_section_limit = (0..LIMITS.sections).map(|n| connector(u64::from(n), false, 0)).collect();
    for sections in [
        Box::new([]) as Box<[Planned]>,
        at_section_limit,
        Box::new([Planned::Core {
            kind: Core::Task,
            text: vec![b'x'; usize::try_from(LIMITS.brief_bytes).expect("small byte limit")].into(),
            limit: LIMITS.brief_bytes,
            priority: 0,
            required: true,
        }]),
    ] {
        let mut world = World::new(LIMITS);
        let asked = world.step(planned(sections, LIMITS.brief_bytes, deadline));
        assert!(!asked.iter().any(|r| matches!(r, GatherRequest::Refused { .. })));
        let done = world.settle(asked);
        assert!(matches!(done.as_slice(), [GatherRequest::Complete { .. }]));
        assert!(referee::within_budget(&done, LIMITS.brief_bytes));
    }
}

#[test]
fn a_required_connectors_failure_passes_through_unchanged() {
    for fate in [GatherFate::Capacity, GatherFate::Transient, GatherFate::Permanent] {
        let mut world = World::new(LIMITS);
        begin(&mut world, Box::new([connector(2, true, 0)]), LIMITS.brief_bytes);
        let why = failure(fate, b"pull", b"words");
        let asked = world.step(GatherEvent::Missing { brief: Token::new(1), section: Token::new(2), why: why.clone() });
        let done = world.settle(asked);
        assert_eq!(done, [GatherRequest::Failed { brief: Token::new(1), why: GatherMissing::Failed, failure: why }]);
        assert!(world.closed(Token::new(2)));
    }
}

#[test]
fn a_required_connectors_failure_frees_its_slot_and_drops_the_other_gather() {
    for fate in [GatherFate::Capacity, GatherFate::Transient, GatherFate::Permanent] {
        let mut world = World::new(jig_domain_brief::Limits { briefs: 1, ..LIMITS });
        let paused = Token::new(3);
        world.put(paused, b"still gathering");
        world.pause(paused);
        let asked = begin(
            &mut world,
            Box::new([named_connector(2, b"pull", true, 0), named_connector(3, b"ci", true, 1)]),
            LIMITS.brief_bytes,
        );
        assert_eq!(
            asked,
            [
                GatherRequest::Gather { connector: 3, section: Token::new(2), budget: LIMITS.read_bytes },
                GatherRequest::Gather { connector: 3, section: paused, budget: LIMITS.read_bytes },
            ]
        );
        let why = failure(fate, b"pull", b"the change's head moved");
        let failed =
            world.step(GatherEvent::Missing { brief: Token::new(1), section: Token::new(2), why: why.clone() });
        let done = world.settle(failed);
        assert_eq!(done, [GatherRequest::Failed { brief: Token::new(1), why: GatherMissing::Failed, failure: why }]);
        assert!(world.closed(paused));
        assert!(!world.held(paused));
        assert!(world.closed(Token::new(2)));
        assert_eq!(world.env.now, Time::ZERO);
        assert_eq!(world.domain.next_deadline(), None);

        world.put(Token::new(4), b"next brief");
        let admitted = world.step(GatherEvent::Plan {
            brief: Token::new(2),
            budget: LIMITS.brief_bytes,
            deadline: Time::ZERO.saturating_add(Duration::from_secs(10)),
            sections: Box::new([connector(4, true, 0)]),
        });
        assert_eq!(
            admitted,
            [GatherRequest::Gather { connector: 3, section: Token::new(4), budget: LIMITS.read_bytes }]
        );
        assert_eq!(
            world.settle(admitted),
            [GatherRequest::Complete {
                brief: Token::new(2),
                order: Box::new([GatherPlaced::Connector { connector: 3, kind: 7, token: Token::new(4), size: 10 }]),
            }]
        );
    }
}

#[test]
fn a_required_sections_deadline_frees_its_slot_and_drops_the_other_gather() {
    let mut world = World::new(jig_domain_brief::Limits { briefs: 1, ..LIMITS });
    for token in [Token::new(2), Token::new(3)] {
        world.put(token, b"still gathering");
        world.pause(token);
    }
    let asked = begin(
        &mut world,
        Box::new([named_connector(2, b"pull", true, 0), named_connector(3, b"ci", true, 1)]),
        LIMITS.brief_bytes,
    );
    assert_eq!(asked.len(), 2);
    assert!(world.settle(asked).is_empty());
    let now = Time::ZERO.saturating_add(Duration::from_secs(11));
    let expired = world.fire(now);
    assert_eq!(
        world.settle(expired),
        [GatherRequest::Failed {
            brief: Token::new(1),
            why: GatherMissing::Late,
            failure: failure(GatherFate::Infrastructure, b"pull", b"not gathered by its deadline"),
        }]
    );
    for token in [Token::new(2), Token::new(3)] {
        assert!(world.closed(token));
        assert!(!world.held(token));
    }
    assert_eq!(world.domain.next_deadline(), None);

    world.put(Token::new(4), b"next brief");
    let admitted = world.step(GatherEvent::Plan {
        brief: Token::new(2),
        budget: LIMITS.brief_bytes,
        deadline: now.saturating_add(Duration::from_secs(10)),
        sections: Box::new([connector(4, true, 0)]),
    });
    assert_eq!(admitted, [GatherRequest::Gather { connector: 3, section: Token::new(4), budget: LIMITS.read_bytes }]);
    assert_eq!(
        world.settle(admitted),
        [GatherRequest::Complete {
            brief: Token::new(2),
            order: Box::new([GatherPlaced::Connector { connector: 3, kind: 7, token: Token::new(4), size: 10 }]),
        }]
    );
}

#[test]
fn a_required_answer_past_its_read_or_cut_bound_fails_permanently() {
    const BUDGET: u32 = 40; // Below the connector's 128-byte read limit, so it must cut.
    for (preliminary, answer) in [
        (Vec::new(), GatherEvent::Ready { brief: Token::new(1), section: Token::new(2), size: LIMITS.read_bytes + 1 }),
        (
            vec![GatherEvent::Ready { brief: Token::new(1), section: Token::new(2), size: BUDGET + 1 }],
            GatherEvent::Cut { brief: Token::new(1), section: Token::new(2), size: BUDGET + 1 },
        ),
    ] {
        let mut world = World::new(LIMITS);
        begin(&mut world, Box::new([named_connector(2, b"pull", true, 0)]), BUDGET);
        for event in preliminary {
            world.step(event);
        }
        let asked = world.step(answer);
        let done = world.settle(asked);
        assert_eq!(
            done,
            [GatherRequest::Failed {
                brief: Token::new(1),
                why: GatherMissing::Failed,
                failure: failure(GatherFate::Permanent, b"pull", b"answered past its bound")
            }]
        );
        assert!(world.closed(Token::new(2)));
    }
}

#[test]
fn a_required_cut_cannot_outlive_the_deadline() {
    const BUDGET: u32 = 40; // A 41-byte answer needs a cut.
    for sections in [
        Box::new([named_connector(2, b"pull", true, 0)]) as Box<[Planned]>,
        Box::new([named_connector(2, b"pull", true, 0), connector(3, false, 1)]),
    ] {
        let mut world = World::new(LIMITS);
        begin(&mut world, sections, BUDGET);
        // Either start cutting now, or defer allocation until the optional section expires.
        world.step(GatherEvent::Ready { brief: Token::new(1), section: Token::new(2), size: BUDGET + 1 });
        let expired = world.fire(Time::ZERO.saturating_add(Duration::from_secs(10)));
        let done = world.settle(expired);
        assert_eq!(
            done,
            [GatherRequest::Failed {
                brief: Token::new(1),
                why: GatherMissing::Late,
                failure: failure(GatherFate::Infrastructure, b"pull", b"not gathered by its deadline")
            }]
        );
        assert!(world.closed(Token::new(2)));
    }
}

#[test]
fn a_deadline_names_the_first_missing_required_section_in_source_order() {
    let mut world = World::new(LIMITS);
    begin(
        &mut world,
        Box::new([connector(2, false, 0), named_connector(3, b"pull", true, 9), named_connector(4, b"ci", true, 1)]),
        LIMITS.brief_bytes,
    );
    let expired = world.fire(Time::ZERO.saturating_add(Duration::from_secs(10)));
    let done = world.settle(expired);
    assert_eq!(
        done,
        [GatherRequest::Failed {
            brief: Token::new(1),
            why: GatherMissing::Late,
            failure: failure(GatherFate::Infrastructure, b"pull", b"not gathered by its deadline")
        }]
    );
    for token in [2, 3, 4] {
        assert!(world.closed(Token::new(token)));
    }
}

#[test]
fn required_core_text_that_cannot_be_cut_fails_with_its_name() {
    const BUDGET: u32 = 1; // Too small for the cut marker of the task text.
    let mut world = World::new(LIMITS);
    let done = begin(
        &mut world,
        Box::new([Planned::Core {
            kind: Core::Task,
            text: b"task".as_slice().into(),
            limit: LIMITS.brief_bytes,
            priority: 0,
            required: true,
        }]),
        BUDGET,
    );
    assert_eq!(
        done,
        [GatherRequest::Failed {
            brief: Token::new(1),
            why: GatherMissing::Budget,
            failure: failure(GatherFate::Permanent, b"task", b"does not fit the brief's budget")
        }]
    );
    assert!(world.domain.is_idle());
}

#[test]
fn a_required_connector_with_no_budget_fails_with_its_name() {
    let mut world = World::new(LIMITS);
    let token = Token::new(2);
    world.put(token, b"words");
    let asked = begin(&mut world, Box::new([named_connector(2, b"pull", true, 0)]), 0);
    let done = world.settle(asked);
    assert_eq!(
        done,
        [GatherRequest::Failed {
            brief: Token::new(1),
            why: GatherMissing::Budget,
            failure: failure(GatherFate::Permanent, b"pull", b"does not fit the brief's budget")
        }]
    );
    assert!(world.closed(token));
}

#[test]
fn a_failure_reason_joins_its_section_and_words() {
    let why = failure(GatherFate::Transient, b"pull", b"words");
    assert_eq!(why.reason().as_ref(), b"pull: words");
}

#[test]
fn required_answers_at_their_read_and_cut_bounds_complete() {
    const CUT_BUDGET: u32 = 40; // Below the 128-byte answer, requiring a cut.
    for budget in [1, CUT_BUDGET, LIMITS.read_bytes] {
        let mut world = World::new(LIMITS);
        let token = Token::new(2);
        world.put(token, &vec![b'x'; usize::try_from(LIMITS.read_bytes).expect("small read limit")]);
        let asked = begin(&mut world, Box::new([named_connector(2, b"pull", true, 0)]), budget);
        let done = world.settle(asked);
        assert_eq!(
            done,
            [GatherRequest::Complete {
                brief: Token::new(1),
                order: Box::new([GatherPlaced::Connector { connector: 3, kind: 7, token, size: budget }]),
            }]
        );
        assert!(referee::within_budget(&done, budget));
        assert!(world.closed(token));
    }
}
