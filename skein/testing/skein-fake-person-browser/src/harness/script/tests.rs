use super::{Play, Script, Step};
use crate::harness::{
    Harness, Names, Outcome, Output, Printed, Runtime, Scripted, Sent, ServerState, Setup, Shown, page, person, run,
};
use crate::tests::{face, found, seen};
use crate::{BrowserFace, NestedHas};
use skein_browser::boundary::{Event, Key, Request};
use skein_fake_person::{Find, Next, Step as PersonStep};
use skein_lib::{Duration, Time, Token};
use skein_ui::Role;

#[test]
fn trouble_named_by_a_later_step_does_not_fail_the_run() {
    let mut script = Script::new(Box::new([
        Step::Key(Key::Escape),
        Step::Trouble { words: b"frame-ancestors".as_slice().into(), within: skein_lib::Duration::from_secs(1) },
    ]));
    assert_eq!(script.expected(), vec![Box::from(b"frame-ancestors".as_slice())]);
    let mut face = face();
    face.event(&skein_browser::boundary::Event::Trouble {
        page: Token::new(8),
        trouble: skein_browser::boundary::Trouble::Exception,
        url: Box::default(),
        text: b"frame-ancestors".as_slice().into(),
    });
    assert_eq!(face.trouble(), &[]);
    face.event(&skein_browser::boundary::Event::Trouble {
        page: Token::new(7),
        trouble: skein_browser::boundary::Trouble::Exception,
        url: Box::default(),
        text: b"frame-ancestors".as_slice().into(),
    });
    script.next(&mut face, Time::ZERO);
    script.next(&mut face, Time::ZERO);
    script.next(&mut face, Time::ZERO);
    assert_eq!(script.at(), 2);
    assert_eq!(face.failure(), None);
}
#[test]
fn a_trouble_step_not_met_within_its_bound_fails_naming_its_words() {
    let mut script = Script::new(Box::new([Step::Trouble {
        words: b"frame-ancestors".as_slice().into(),
        within: skein_lib::Duration::from_secs(1),
    }]));
    let mut face = face();
    script.next(&mut face, Time::ZERO);
    script.check_deadline(&mut face, Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1)));
    assert_eq!(face.expired(), Some(Duration::from_secs(1)));
    assert_eq!(face.failure().as_deref(), Some("did not receive trouble containing frame-ancestors within its bound"));
}
#[test]
fn a_request_step_with_another_status_fails_naming_it() {
    for now in [Time::ZERO, Time::ZERO.saturating_add(Duration::from_secs(1))] {
        let (mut script, mut face) = unanswered_request();
        script.answered(403);
        script.check_deadline(&mut face, now);
        assert_eq!(face.expired(), None);
        assert_eq!(face.failure().as_deref(), Some("expected status 200, saw 403"));
    }
}

fn unanswered_request() -> (Script, BrowserFace<'static>) {
    let mut script = Script::new(Box::new([Step::Request {
        method: skein_http::Method::Get,
        path: b"/".as_slice().into(),
        host: b"other.test".as_slice().into(),
        headers: vec![],
        body: Box::new([]),
        status: 200,
        within: skein_lib::Duration::from_secs(1),
    }]));
    let mut face = face();
    script.next(&mut face, Time::ZERO);
    assert_eq!(script.request().unwrap().host.as_ref(), b"other.test");
    (script, face)
}

#[test]
fn unanswered_steps_expire_even_while_the_face_is_pending() {
    let within = Duration::from_secs(1);
    let until = Time::ZERO.saturating_add(within);
    let before = Time::from_nanos(until.as_nanos() - 1);
    let (mut script, mut face) = unanswered_request();
    script.check_deadline(&mut face, before);
    assert_eq!(face.failure(), None);
    script.check_deadline(&mut face, until);
    assert_eq!(face.expired(), Some(within));
    assert_eq!(face.failure().as_deref(), Some("did not receive status 200 within its bound"));

    let find = Find::text(b"answer");
    for step in [PersonStep::See { find: find.clone(), within }, PersonStep::Gone { find, within }] {
        let mut script = Script::new(Box::new([Step::Person(step)]));
        let mut face = crate::tests::face();
        assert_eq!(script.next(&mut face, Time::ZERO), Next::Wait { until: None });
        assert_eq!(script.next(&mut face, Time::ZERO), Next::Wait { until: Some(until) });
        assert!(face.pending());
        script.check_deadline(&mut face, before);
        assert_eq!(face.failure(), None);
        script.check_deadline(&mut face, until);
        assert_eq!(face.expired(), Some(within));
        assert!(face.failure().is_some());
    }

    let find = Find::text(b"missing");
    for step in [
        PersonStep::Press { find: find.clone() },
        PersonStep::Type { find: find.clone(), words: b"words".as_slice().into() },
    ] {
        let mut script = Script::new(Box::new([Step::Person(step)]));
        let mut face = crate::tests::face();
        script.next(&mut face, Time::ZERO);
        script.next(&mut face, Time::ZERO);
        found(&mut face, None);
        assert_eq!(
            script.next(&mut face, Time::ZERO),
            Next::Failed { step: 0, why: skein_fake_person::Why::Missing { find: find.clone() } }
        );
        assert_eq!(face.expired(), None);
    }
}
const SEE_TEXT: &[u8] = b"Ready";
const SEE_WITHIN: Duration = Duration::from_secs(10);

struct SeeStory {
    setup: Harness,
    runtime: Runtime,
    script: Script,
    faces: [BrowserFace<'static>; 1],
    done: [bool; 1],
}

impl SeeStory {
    fn new(visible: Option<Time>) -> Self {
        let setup = Harness::new(Setup {
            server: "/test-server".into(),
            args: vec![],
            printed: Printed { sent: Sent::Page, prefix: b"listening ", on: Output::Stdout, names: Names::Address },
            restart: vec![],
            chromium: "/test-browser".into(),
            downloads: None,
            expected: vec![],
            origins: vec![],
        });
        let mut browser = Scripted::new();
        if let Some(visible) = visible {
            browser = browser
                .page(b"/", Shown { nodes: vec![(b"StaticText".as_slice().into(), SEE_TEXT.into())], elements: vec![] })
                .find_after(SEE_TEXT, visible);
        }
        let mut script =
            Script::new(Box::new([Step::Person(PersonStep::See { find: Find::text(SEE_TEXT), within: SEE_WITHIN })]));
        let (mut runtime, _) = Runtime::new(&setup, &[&mut script], run::BrowserStart::Scripted(browser));
        runtime.ask(Request::Page {
            person: person(0),
            page: page(0),
            url: b"http://example.test/".as_slice().into(),
            requests: false,
        });
        let mut story =
            Self { setup, runtime, script, faces: [BrowserFace::new(page(0), "example.test")], done: [false] };
        story.turn(Time::ZERO);
        story.turn(Time::ZERO);
        assert!(story.faces[0].pending(), "the first find is in flight");
        story
    }

    fn turn(&mut self, now: Time) {
        self.runtime.benv.now = now;
        self.runtime.tick(&self.setup, &mut self.faces, &[&mut self.script], now);
        self.runtime.judge_steps(&self.setup, &mut [&mut self.script], &mut self.faces, &self.done, now);
        if self.runtime.outcome.is_none() {
            self.runtime.play(&mut [&mut self.script], &mut self.faces, &mut self.done, now, &self.setup);
        }
    }

    fn wake(&mut self, now: Time) -> Option<Time> {
        assert!(self.runtime.events.is_empty(), "nothing else will wake the script");
        self.runtime.play_wake(&[&mut self.script], &self.faces, &self.done, now)
    }
}

#[test]
fn a_see_polls_again_after_a_slow_negative_reply() {
    let poll = skein_browser::Limits::default().poll;
    let arrives = Time::ZERO.saturating_add(poll.saturating_mul(2));
    let retry = arrives.saturating_add(poll);
    let mut story = SeeStory::new(Some(retry));

    // The scripted kit's reply was queued at the request; deliver it on a later turn.
    story.turn(arrives);
    assert_eq!(story.script.at(), 0);
    assert_eq!(story.wake(arrives), Some(retry), "a slow reply must not defer the retry to the step's deadline");

    story.turn(retry);
    assert!(story.faces[0].pending(), "the second find is in flight");
    story.turn(retry);
    assert_eq!(story.script.at(), 1, "the next find sees the target");
    story.turn(retry);
    assert_eq!(story.runtime.outcome, Some(Outcome::Done));
}

#[test]
fn a_see_polls_when_a_reply_is_consumed_after_the_next_poll_is_due() {
    let poll = skein_browser::Limits::default().poll;
    let arrives = Time::ZERO.saturating_add(poll.saturating_mul(2));
    let resume = arrives.saturating_add(poll.saturating_mul(2));
    let mut story = SeeStory::new(Some(resume));

    story.runtime.server_state = ServerState::Stopping;
    story.turn(arrives);
    assert!(!story.faces[0].pending(), "the reply arrived while the script could not play");
    story.runtime.server_state = ServerState::Running;
    story.turn(resume);
    assert!(story.wake(resume).is_some_and(|at| at <= resume), "the overdue poll is runnable work");

    story.turn(resume);
    story.turn(resume);
    assert_eq!(story.script.at(), 1);
}

#[test]
fn a_see_still_expires_at_its_own_deadline_after_a_slow_reply() {
    let until = Time::ZERO.saturating_add(SEE_WITHIN);
    let before = Time::from_nanos(until.as_nanos() - 1);
    let mut story = SeeStory::new(None);
    story.turn(before);
    assert_eq!(story.runtime.outcome, None, "the negative reply is not an early failure");
    assert_eq!(story.wake(before), Some(until), "the step's bound comes before the next poll");
    story.turn(until);
    let Some(Outcome::Failed { person: 0, step: 0, report }) = story.runtime.outcome else {
        panic!("the missing target must expire exactly at the step's deadline");
    };
    assert!(report.starts_with("Missing {"), "{report}");
    let bound = SEE_WITHIN.as_nanos();
    assert!(report.contains(&format!("bound {bound} ns, waited {bound} ns")), "{report}");
}

#[test]
fn a_key_step_sends_the_kits_key() {
    let mut script = Script::new(Box::new([Step::Key(Key::Escape)]));
    let mut face = face();
    script.next(&mut face, Time::ZERO);
    assert_eq!(
        face.requests().collect::<Vec<_>>(),
        vec![Request::Key { page: Token::new(7), op: Token::new(7_000_001), key: Key::Escape, shift: false }]
    );
}
#[test]
fn a_key_with_shift_step_sends_the_key_with_shift_held() {
    let mut script = Script::new(Box::new([Step::KeyWithShift(Key::Enter)]));
    let mut face = face();
    script.next(&mut face, Time::ZERO);
    assert_eq!(
        face.requests().collect::<Vec<_>>(),
        vec![Request::Key { page: Token::new(7), op: Token::new(7_000_001), key: Key::Enter, shift: true }]
    );
}
const ADDRESS_OP: Token = Token::new(7_000_001);
fn address_step(path: &[u8]) -> (Script, BrowserFace<'static>) {
    let mut script = Script::new(Box::new([Step::Address(path.into())]));
    let mut face = face();
    script.next(&mut face, Time::ZERO);
    assert_eq!(face.requests().collect::<Vec<_>>(), vec![Request::Address { page: Token::new(7), op: ADDRESS_OP }]);
    (script, face)
}
#[test]
fn a_failed_address_step_names_both_addresses() {
    let (_, mut face) = address_step(b"/notes/1");
    face.event(&Event::Address { op: ADDRESS_OP, url: b"http://127.0.0.1:9000/notes/2".as_slice().into() });
    assert!(!face.pending());
    assert_eq!(
        face.failure().as_deref(),
        Some("the page's address was http://127.0.0.1:9000/notes/2, expected the path /notes/1")
    );
}
#[test]
fn an_address_step_passes_on_its_path() {
    let (script, mut face) = address_step(b"/notes/1");
    face.event(&Event::Address { op: ADDRESS_OP, url: b"http://127.0.0.1:9000/notes/1".as_slice().into() });
    assert_eq!((face.failure(), script.at()), (None, 1));
}
#[test]
fn an_address_step_compares_the_query_too() {
    let (_, mut face) = address_step(b"/");
    face.event(&Event::Address { op: ADDRESS_OP, url: b"http://127.0.0.1:9000/?secret=1".as_slice().into() });
    assert_eq!(
        face.failure().as_deref(),
        Some("the page's address was http://127.0.0.1:9000/?secret=1, expected the path /")
    );
}
#[test]
fn an_address_answer_to_another_op_is_ignored() {
    let (_, mut face) = address_step(b"/notes/1");
    face.event(&Event::Address { op: Token::new(7_000_002), url: b"http://127.0.0.1:9000/".as_slice().into() });
    assert_eq!((face.pending(), face.failure()), (true, None));
}
#[test]
fn a_point_press_uses_the_found_figures_node() {
    let mut script = Script::new(Box::new([Step::PressAt {
        find: Find::exactly_named(Role::Drawing, b"Notes by label"),
        at: skein_browser::boundary::Point { x: 128, y: 60 },
    }]));
    let mut face = face();
    script.next(&mut face, Time::ZERO);
    script.next(&mut face, Time::ZERO);
    found(&mut face, Some(seen(b"")));
    script.next(&mut face, Time::ZERO);
    assert_eq!(
        face.requests().collect::<Vec<_>>(),
        vec![Request::Press {
            page: Token::new(7),
            op: Token::new(7_000_002),
            node: 9_000_000_000,
            at: Some(skein_browser::boundary::Point { x: 128, y: 60 }),
        }]
    );
}
const FOCUS: &[(bool, Option<&str>)] = &[
    (true, None),
    (
        false,
        Some(
            "node Find { within: [], target: ExactRole { role: TextField, name: [84, 105, 116, 108, 101] }, nth: 0 } is not focused",
        ),
    ),
];
#[test]
fn the_focus_step_checks_the_found_nodes_focus() {
    for &(focused, expected) in FOCUS {
        let mut script = Script::new(Box::new([Step::Focused(Find::exactly_named(Role::TextField, b"Title"))]));
        let mut face = face();
        script.next(&mut face, Time::ZERO);
        script.next(&mut face, Time::ZERO);
        let mut target = seen(b"");
        target.states.focused = focused;
        found(&mut face, Some(target));
        script.next(&mut face, Time::ZERO);
        assert_eq!(face.failure().as_deref(), expected, "focused {focused}");
    }
}

fn keep(selector: &[u8]) -> Step {
    Step::Keep { selector: selector.into(), attribute: None, name: b"read".as_slice().into() }
}

fn keeping(selector: &[u8]) -> (Script, BrowserFace<'static>) {
    let mut script = Script::new(Box::new([keep(selector)]));
    let mut face = face();
    script.next(&mut face, Time::ZERO);
    assert_eq!(
        face.requests().collect::<Vec<_>>(),
        vec![Request::Attribute {
            page: Token::new(7),
            op: ADDRESS_OP,
            selector: selector.into(),
            name: b"value".as_slice().into()
        }]
    );
    (script, face)
}
#[test]
fn a_keep_cannot_nest_has_through_not() {
    let selector = b"details:has(> summary:not(:has(> [role=group] + span)))".as_slice();
    assert_eq!(Script::try_new(Box::new([keep(selector)])).err(), Some(NestedHas { selector: selector.into() }));
}

#[test]
fn a_keep_cannot_nest_has_through_is() {
    let selector = b"a:has(b:is(c:has(d)))".as_slice();
    assert_eq!(Script::try_new(Box::new([keep(selector)])).err(), Some(NestedHas { selector: selector.into() }));
}

#[test]
fn a_keep_cannot_nest_has_in_mixed_case() {
    let selector = b"a:HaS(b:where(c:HAS(d)))".as_slice();
    assert_eq!(Script::try_new(Box::new([keep(selector)])).err(), Some(NestedHas { selector: selector.into() }));
}

#[test]
fn a_nested_has_refusal_displays_the_selector() {
    let refusal = NestedHas { selector: b"a:HaS(b:where(c:HAS(d)))".as_slice().into() };
    assert_eq!(refusal.to_string(), "keep selector a:HaS(b:where(c:HAS(d))) nests :has()");
}

#[test]
fn a_keep_requests_unnested_selectors_unchanged() {
    for selector in [
        b"[role=tree]:has(> x):not(:has(y))".as_slice(),
        b"main:not(:has(input, label, form))",
        b"x:has([title=':has(y)'])",
        br#"x:has([title="\":has(y)"])"#,
        b"x:has(/* :has(y) */ z)",
    ] {
        assert!(Script::try_new(Box::new([keep(selector)])).is_ok());
        let _ = keeping(selector);
    }
}

#[test]
fn a_keep_waits_for_its_terminal_before_advancing() {
    let (mut script, mut face) = keeping(b"#hidden");
    script.next(&mut face, Time::ZERO);
    assert_eq!(script.at(), 0);
    assert!(face.pending());
    face.event(&Event::Attribute {
        op: ADDRESS_OP,
        read: skein_browser::boundary::Read::Value(b"stored".as_slice().into()),
    });
    script.next(&mut face, Time::ZERO);
    assert_eq!(script.at(), 1);
    assert_eq!(face.kept(), Some((b"read".as_slice().into(), b"stored".as_slice().into())));
}
#[test]
fn a_refused_keep_names_the_selector_and_attribute() {
    let (mut script, mut face) = keeping(b"#hidden");
    face.event(&Event::Refused {
        op: ADDRESS_OP,
        why: skein_browser::boundary::Refusal::Protocol,
        detail: Box::default(),
    });
    script.check_deadline(&mut face, Time::ZERO);
    assert!(!face.pending());
    assert_eq!(script.at(), 1);
    assert_eq!(
        face.failure().as_deref(),
        Some("keep selector #hidden attribute value: browser refused read: Protocol")
    );
}
#[test]
fn another_operations_attribute_does_not_complete_a_keep() {
    let (script, mut face) = keeping(b"#hidden");
    face.event(&Event::Attribute { op: Token::new(8), read: skein_browser::boundary::Read::NoElement });
    assert!(face.pending());
    assert_eq!(script.at(), 0);
    assert_eq!(face.failure(), None);
}

fn tabs_script() -> (Script, BrowserFace<'static>) {
    let script = Script::new(Box::new([Step::Tabs {
        tabs: Box::new([
            super::TabSeen {
                title: Some(b"Opening".as_slice().into()),
                address: Some(b"/auth".as_slice().into()),
                opener: Some(false),
            },
            super::TabSeen { title: None, address: None, opener: None },
        ]),
        within: skein_lib::Duration::from_secs(1),
    }]));
    (script, face())
}
fn tab(title: &[u8], url: &[u8], opener: bool) -> skein_browser::boundary::Tab {
    skein_browser::boundary::Tab { title: title.into(), url: url.into(), opener }
}
fn tabs_answer(face: &mut BrowserFace<'_>, op: Token, tabs: Vec<skein_browser::boundary::Tab>, more: u32) {
    let mut list = skein_lib::List::with_capacity(2);
    for tab in tabs {
        list.push(tab).unwrap();
    }
    face.event(&Event::Tabs { op, tabs: list, more });
}
fn correct_tabs() -> Vec<skein_browser::boundary::Tab> {
    vec![tab(b"Opening", b"https://example.test/auth?code=x", false), tab(b"Any title", b"about:blank", true)]
}
#[test]
fn a_tabs_step_polls_until_each_requested_field_count_and_order_match() {
    let mut wrong_title = correct_tabs();
    wrong_title[0].title = b"Other".as_slice().into();
    let mut wrong_address = correct_tabs();
    wrong_address[0].url = b"https://example.test/other".as_slice().into();
    let mut wrong_opener = correct_tabs();
    wrong_opener[0].opener = true;
    let mut wrong_order = correct_tabs();
    wrong_order.reverse();
    for (tabs, more) in
        [(wrong_title, 0), (wrong_address, 0), (wrong_opener, 0), (wrong_order, 0), (vec![], 0), (correct_tabs(), 1)]
    {
        let (mut script, mut face) = tabs_script();
        script.next(&mut face, Time::ZERO);
        assert_eq!(face.requests().collect::<Vec<_>>(), vec![Request::Tabs { page: Token::new(7), op: ADDRESS_OP }]);
        tabs_answer(&mut face, Token::new(99), correct_tabs(), 0);
        assert!(face.pending(), "another operation cannot complete this read");
        tabs_answer(&mut face, ADDRESS_OP, tabs, more);
        script.next(&mut face, Time::ZERO);
        assert_eq!(script.at(), 0, "mismatching tabs must wait");
        let poll = skein_browser::Limits::default().poll;
        script.next(&mut face, Time::from_nanos(poll.as_nanos() - 1));
        assert_eq!(face.requests().count(), 0, "no early poll");
        script.next(&mut face, Time::ZERO.saturating_add(poll));
        let Some(Request::Tabs { op, .. }) = face.requests().next() else { panic!("poll tabs") };
        tabs_answer(&mut face, op, correct_tabs(), 0);
        script.next(&mut face, Time::ZERO.saturating_add(poll));
        assert_eq!(script.at(), 1);
        assert!(face.failure().is_none());
    }
}
#[test]
fn a_tabs_deadline_names_the_last_tabs_while_the_next_read_is_pending() {
    let (mut script, mut face) = tabs_script();
    script.next(&mut face, Time::ZERO);
    face.requests().for_each(drop);
    tabs_answer(&mut face, ADDRESS_OP, vec![tab(b"Waiting", b"about:blank", true)], 2);
    script.next(&mut face, Time::ZERO);
    script.next(&mut face, Time::ZERO.saturating_add(skein_browser::Limits::default().poll));
    assert!(face.pending());
    script.check_deadline(&mut face, Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1)));
    assert_eq!(face.expired(), Some(Duration::from_secs(1)));
    let failure = face.failure().expect("deadline fails");
    assert!(failure.contains("Waiting") && failure.contains("about:blank") && failure.contains("2 more"), "{failure}");
    assert_eq!(script.at(), 1);
}

#[test]
fn a_viewport_step_sends_dimensions_and_holds_the_face_pending() {
    let mut script = Script::new(Box::new([Step::Viewport { width: 420, height: 900 }]));
    let mut face = face();
    script.next(&mut face, Time::ZERO);
    assert_eq!(
        face.requests().collect::<Vec<_>>(),
        vec![Request::Viewport { page: Token::new(7), op: ADDRESS_OP, width: 420, height: 900 }]
    );
    assert!(face.pending());
    face.event(&Event::Done { op: ADDRESS_OP });
    assert!(!face.pending());
}

#[test]
fn the_pressed_step_awaits_the_found_nodes_state_within_its_bound() {
    use skein_fake_person::Why;

    const ROWS: &[(&str, Option<bool>, bool, Option<&str>)] = &[
        ("pressed at first look", Some(true), false, None),
        ("pressed at later look", Some(false), true, None),
        ("unpressed until bound", Some(false), false, Some("unpressed")),
        ("absent until bound", None, false, Some("missing")),
    ];
    for &(name, first, later, expected) in ROWS {
        for deadline in [false, true] {
            let find = Find::exactly_named(Role::Button, b"Closed");
            let within = Duration::from_secs(1);
            let start = Time::from_nanos(7);
            let until = start.saturating_add(within);
            let mut script = Script::new(Box::new([Step::Pressed { find: find.clone(), within }]));
            let mut face = face();
            face.tick(start);
            assert_eq!(script.next(&mut face, start), Next::Wait { until: Some(until) }, "{name}");
            let target = first.map(|pressed| {
                let mut target = seen(b"");
                target.role = b"button".as_slice().into();
                target.name = b"Closed".as_slice().into();
                target.states.pressed = pressed;
                target
            });
            found(&mut face, target);
            script.next(&mut face, start);
            if first != Some(true) {
                assert_eq!(script.at(), 0, "{name} must wait");
                assert_eq!(face.failure(), None, "{name} must not fail early");
                let poll = skein_browser::Limits::default().poll;
                let before_poll = start.saturating_add(Duration::from_nanos(poll.as_nanos() - 1));
                face.tick(before_poll);
                assert_eq!(script.next(&mut face, before_poll), Next::Wait { until: Some(until) }, "{name}");
                assert_eq!(face.requests().count(), 0, "{name}: no early look");
                let again = start.saturating_add(poll);
                face.tick(again);
                assert_eq!(script.next(&mut face, again), Next::Wait { until: Some(until) }, "{name}");
                assert!(face.pending(), "{name}: look again at the poll");
                if later {
                    let mut target = seen(b"");
                    target.states.pressed = true;
                    found(&mut face, Some(target));
                    script.next(&mut face, again);
                } else {
                    let before_bound = Time::from_nanos(until.as_nanos() - 1);
                    script.check_deadline(&mut face, before_bound);
                    assert_eq!(script.at(), 0, "{name}: keep waiting before the bound");
                    assert_eq!(face.failure(), None, "{name}: no failure before the bound");
                    face.tick(until);
                    if deadline {
                        script.check_deadline(&mut face, until);
                    } else {
                        assert_eq!(script.next(&mut face, until), Next::Wait { until: None }, "{name}");
                    }
                }
            }
            let failure = match expected {
                None => None,
                Some("unpressed") => Some(format!("node {find:?} is not pressed within its bound")),
                Some("missing") => Some(format!("{:?}", Why::Missing { find })),
                Some(other) => panic!("unknown outcome {other}"),
            };
            assert_eq!(face.expired(), expected.map(|_| within), "{name}, deadline {deadline}");
            assert_eq!(face.failure(), failure, "{name}, deadline {deadline}");
            assert_eq!(script.at(), 1, "{name}, deadline {deadline}");
            assert_eq!(script.next(&mut face, until), Next::Done, "{name}: script goes on");
        }
    }
}
