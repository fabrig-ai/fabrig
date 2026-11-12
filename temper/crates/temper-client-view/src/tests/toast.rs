//! The session toast at the end of every page (client/pages/README.md).
use super::fixtures as f;
use crate::look;
use jig_client::contract as c;
use skein_lib::{Duration, Env, List, Queue, Time, Wall};
use skein_ui::{Act, Role};
use temper_client_domain::{
    self as d, Address, Event,
    pages::{self, needs},
};

const OBJECT: c::ObjectKey = c::ObjectKey::Question { task: f::INBOX_TASK, message: f::INBOX_QUESTION };
fn root() -> d::Domain {
    let mut offers = List::with_capacity(1);
    offers.push(c::Offered { act: c::ActKind::Answer, admission: c::Admission::Allowed, effect: None }).unwrap();
    let page = c::Question {
        task: f::task::document().chip,
        from: None,
        message: f::INBOX_QUESTION,
        words: f::task::document_words(b"Which path?"),
        since: Wall::EPOCH,
        acts: offers,
        ending: None,
    };
    let mut source = f::ReadPage::new(Address::Inbox { filter: d::address::Filter::ALL });
    let rows = List::from_boxed(Box::new([super::documents::entry(OBJECT, f::PROJECT, 1, Wall::EPOCH, None)]));
    source.up(jig_client::Up::Watched {
        watch: source.inbox.unwrap(),
        event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(super::documents::inbox(
            rows,
            1,
            c::Count { value: 0, limited: false },
            0,
            false,
        )))),
    });
    source.collect();
    let reads = source.reads.clone();
    for (read, what) in &reads {
        if *what == (c::Read::Question { task: f::INBOX_TASK, message: f::INBOX_QUESTION }) {
            source.answer(*read, c::ReadResult::Value(c::ReadValue::Question(Box::new(page.clone()))));
        }
    }
    source.root
}
fn toast_at(tree: &skein_ui::Tree) -> Option<usize> {
    for (at, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&look::TOAST) {
            return Some(at);
        }
    }
    None
}
fn act(root: &mut d::Domain, act: pages::Act) {
    f::root_event(root, Event::Act { action: d::Action::Pages(act) });
}
#[expect(clippy::too_many_lines, reason = "the fixture delivers and answers a way act")]
fn way_answer(kept: bool) -> d::Domain {
    use jig_client::{Down, UnlinkedRequest, Up};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut root = f::root_with_ways(Address::Home { project: f::PROJECT }, None);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: f::domain_limits() };
    let mut out = Queue::uncapped();
    d::step(&mut root, &env, Event::Went { place: d::Place::Address(Address::Ask { project: f::PROJECT }) }, &mut out);
    let wanted = d::forge::stock_ways::watch(f::PROJECT, &env.limits.forge).unwrap();
    let mut watch = None;
    for request in &out {
        if let d::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { watch: token, subject } } } =
            request
            && subject == &wanted
        {
            watch = Some(*token);
        }
    }
    let path = w::BranchPath::new(
        &w::CEILINGS,
        w::BranchPathParts { repository: None, branch: b"main".as_slice().into(), stops: List::with_capacity(0) },
    )
    .unwrap();
    let way = w::StockWay::new(
        &w::CEILINGS,
        w::StockWayParts {
            headline: b"".as_slice().into(),
            name: b"A way".as_slice().into(),
            line: b"A line".as_slice().into(),
            path: path.clone(),
            items: List::from_boxed(Box::new([w::ProposalItem::new(
                &w::CEILINGS,
                w::ProposalItemParts { kind: 2, scope: f::PROJECT, number: 4, edit: w::SettingEdit::Removed },
            )
            .unwrap()])),
            delegation: false,
            roles: List::with_capacity(0),
        },
    )
    .unwrap();
    let panel = w::WaysPanel::new(
        &w::CEILINGS,
        w::WaysPanelParts {
            rules: w::PathRules::new(
                &w::CEILINGS,
                w::PathRulesParts { rows: List::with_capacity(0), note: Box::new([]) },
            )
            .unwrap(),
            words: None,
            project: f::PROJECT,
            ways: List::from_boxed(Box::new([way])),
            current: if kept { Some(0) } else { None },
            own: path,
            may_change: true,
        },
    )
    .unwrap();
    f::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch: watch.expect("ways watch"),
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::ways_panel(&panel),
                    },
                },
            },
        },
    );
    let action = if kept {
        pages::Act::KeepWay { project: f::PROJECT }
    } else {
        pages::Act::UseWay { project: f::PROJECT, way: 0 }
    };
    let mut out = Queue::uncapped();
    d::step(&mut root, &env, Event::Act { action: d::Action::Pages(action) }, &mut out);
    let mut attempt = None;
    for request in &out {
        if let d::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Send { attempt: token, .. } } } =
            request
        {
            attempt = Some(*token);
        }
    }
    f::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::Answer {
                    attempt: attempt.expect("way act sent"),
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(
                        c::OutcomeSettingsChanged { revision: 9 },
                    )))),
                    why: None,
                },
            },
        },
    );
    root
}
#[test]
fn toast_is_last_on_home_and_chats_in_both_shells_and_undo_is_only_held() {
    for address in [Address::Home { project: f::PROJECT }, Address::Chats { filter: d::ChatsFilter::All }] {
        for (reply, later, way, stop, said) in [
            (false, false, None, None, "Answered: First"),
            (true, false, None, None, "Answered: Replied: “My words”"),
            (false, true, None, None, "Later: it stays in Needs you, after the others."),
            (false, false, Some(false), None, "A way is this project's way now, as revision 9."),
            (false, false, Some(true), None, "A way is now this project's own, as revision 9."),
            (false, false, None, Some(0), "Applied: the next one is decided and listed. This one still waits for you."),
            (
                false,
                false,
                None,
                Some(1),
                "Proposed: it applies once an owner accepts it. This one still waits for you.",
            ),
            (false, false, None, Some(2), "It keeps asking. Stop asking comes back in 7 days."),
            (false, false, None, Some(3), "temper couldn't keep that. Try Keep asking again."),
        ] {
            let mut root = match (way, stop) {
                (Some(kept), _) => way_answer(kept),
                (None, Some(stop)) => stop_answer(stop),
                (None, None) => root(),
            };
            if stop.is_none() && way.is_none() && later {
                act(&mut root, pages::Act::Needs(needs::Act::Later { object: OBJECT }));
            } else if stop.is_none() && way.is_none() && reply {
                act(&mut root, pages::Act::Needs(needs::Act::OpenReply { object: OBJECT }));
                act(&mut root, pages::Act::CardWords { object: OBJECT, words: b"My words".as_slice().into() });
                act(&mut root, pages::Act::Needs(needs::Act::SendReply { object: OBJECT }));
            } else if stop.is_none() && way.is_none() {
                act(
                    &mut root,
                    pages::Act::Needs(needs::Act::Choose { object: OBJECT, words: b"First".as_slice().into() }),
                );
            }
            f::root_event(&mut root, Event::Went { place: d::Place::Address(address.clone()) });
            for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
                assert_eq!(skein_ui::outline::check(view.tree()), None);
                let nodes = view.tree().nodes();
                let at = toast_at(view.tree()).unwrap();
                let toast = &nodes[at];
                assert_eq!(toast.role, Role::Status);
                assert_eq!(at + usize::try_from(toast.size).unwrap(), nodes.len());
                assert_eq!(nodes[at + 1].role, Role::Text);
                assert_eq!(nodes[at + 1].text.get(0).unwrap().text.as_ref(), said.as_bytes());
                let undo = f::find(view.tree(), Role::Button, b"Undo");
                assert_eq!(undo.is_some(), !later && way.is_none() && stop.is_none());
                if let Some(undo) = undo {
                    assert_eq!(
                        crate::act(&mut view, undo, Act::Press),
                        Some(d::Action::Pages(pages::Act::Needs(needs::Act::Undo { object: OBJECT })))
                    );
                }
                let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
                skein_ui_text::render(view.tree(), b"temper", None, &mut print);
                assert!(skein_lib::bytes::find(print.text.as_slice(), said.as_bytes()).is_some());
            }
            d::fire(
                &mut root,
                &Env {
                    now: Time::ZERO.saturating_add(Duration::from_secs(5)),
                    wall: Wall::EPOCH,
                    limits: f::domain_limits(),
                },
                &mut Queue::uncapped(),
            );
            for view in [f::page_view(&root), f::browser_page_view(&root)] {
                assert_eq!(toast_at(view.tree()), None);
                assert!(f::find(view.tree(), Role::Button, b"Undo").is_none());
            }
        }
    }
}

#[test]
fn paused() {
    let root = super::goal::with_goal_answer(
        pages::Act::Pause,
        c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Stopped { task: 9 })),
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(
            skein_lib::bytes::find(
                &f::node_text(view.tree(), toast_at(view.tree()).unwrap()),
                b"Paused: running tasks stop, nothing new starts."
            )
            .is_some()
        );
    }
}
#[test]
fn resumed() {
    let root = super::goal::with_goal_answer(
        pages::Act::Resume,
        c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Released { task: 9 })),
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(
            skein_lib::bytes::find(&f::node_text(view.tree(), toast_at(view.tree()).unwrap()), b"Resumed.").is_some()
        );
    }
}
#[test]
fn a_moved_refusal_says_look_again_and_dismisses() {
    let refusal = c::Refusal::Moved { now: c::Revision { value: 18 }, what: c::Moved::Spec };
    let mut root =
        super::goal::with_goal_answer(pages::Act::Pause, c::Answer::Refused { refusal: refusal.clone(), at: None });
    let expected = f::words().say_word(jig_client::words::Word::refusal(&refusal), &[]);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let at = toast_at(view.tree()).unwrap();
        assert!(skein_lib::bytes::find(&f::node_text(view.tree(), at), &expected).is_some());
        let again = f::find(view.tree(), Role::Button, b"Look again").unwrap();
        assert_eq!(
            crate::act(&mut view, again, Act::Press),
            Some(d::Action::Pages(pages::Act::Needs(needs::Act::Dismiss)))
        );
    }
    act(&mut root, pages::Act::Needs(needs::Act::Dismiss));
    assert!(toast_at(f::browser_page_view(&root).tree()).is_none());
}

fn away_answer(until: Option<Wall>, off: Option<Wall>, refused: bool) -> d::Domain {
    let mut root = f::away_root(until, off);
    let attempt = f::away_toggle(&mut root);
    f::playground_start_answer(
        &mut root,
        attempt,
        if refused {
            temper_client_protocol_wire::v1::Outcome::Unknown
        } else {
            temper_client_protocol_wire::v1::Outcome::Away
        },
    );
    root
}
#[test]
fn on_says_until_and_offers_limits() {
    for until in [Some(Wall::from_nanos(28_800_000_000_000)), None] {
        let root = away_answer(until, Some(Wall::EPOCH), false);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let said = if until.is_some() {
                "Away until 08:00: agents decide within Shop’s limits."
            } else {
                "Away: agents decide within Shop’s limits."
            };
            assert_eq!(f::node_text(view.tree(), toast_at(view.tree()).unwrap()).as_ref(), said.as_bytes());
            let limits = f::find(view.tree(), Role::Button, b"Limits").unwrap();
            assert_eq!(
                crate::act(&mut view, limits, Act::Press),
                Some(d::Action::Pages(pages::Act::Away(pages::away::Act::Limits { project: f::PROJECT })))
            );
        }
    }
}
#[test]
fn off_says_questions_wait() {
    let root = away_answer(None, None, false);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(
            f::node_text(view.tree(), toast_at(view.tree()).unwrap()).as_ref(),
            b"Away is off: questions wait for you again."
        );
        assert!(f::find(view.tree(), Role::Button, b"Limits").is_none());
    }
}
#[test]
fn refused_says_try_again() {
    let root = away_answer(None, None, true);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(
            f::node_text(view.tree(), toast_at(view.tree()).unwrap()).as_ref(),
            "Away didn’t change. Try again.".as_bytes()
        );
        assert!(f::find(view.tree(), Role::Button, b"Limits").is_none());
    }
}

#[test]
fn overturned_says_it_asks_again_under_needs_you() {
    let root = super::home::overturn_answer(
        temper_client_protocol_wire::v1::Outcome::Overturned,
        Some(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Said { task: 99, message: 9 }))),
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(
            f::node_text(view.tree(), toast_at(view.tree()).unwrap()).as_ref(),
            b"Overturned: it asks you again, under Needs you."
        );
    }
}
#[test]
fn overturn_ended_says_a_new_goal_can_undo_it() {
    let root = super::home::overturn_answer(
        temper_client_protocol_wire::v1::Outcome::Overturned,
        Some(c::Answer::Refused { refusal: c::Refusal::Unknown, at: None }),
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(
            f::node_text(view.tree(), toast_at(view.tree()).unwrap()).as_ref(),
            b"It has ended; a new goal can undo it."
        );
    }
}
#[test]
fn overturn_already_says_it_was_overturned() {
    let root = super::home::overturn_answer(temper_client_protocol_wire::v1::Outcome::Unknown, None);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(f::node_text(view.tree(), toast_at(view.tree()).unwrap()).as_ref(), b"That was overturned already.");
    }
}

#[test]
fn an_unsent_answer_says_to_press_again_without_an_act() {
    let mut root = root();
    let past_bound = temper_client_protocol_wire::CUE_BYTES.checked_add(1).unwrap();
    let mut words = List::with_capacity(past_bound);
    for _ in 0..past_bound {
        words.push(b'a').unwrap();
    }
    act(&mut root, pages::Act::Needs(needs::Act::Choose { object: OBJECT, words: words.into_boxed() }));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let at = toast_at(view.tree()).unwrap();
        let toast = super::inbox::subtree(view.tree(), at);
        assert_eq!(toast[0].role, Role::Status);
        assert_eq!(f::node_text(view.tree(), at).as_ref(), b"That answer wasn't sent. Press it again.");
        for node in toast {
            assert_ne!(node.role, Role::Button);
        }
    }
}

fn stop_answer(which: u8) -> d::Domain {
    use d::pages::stop_asking::Act as Stop;
    use temper_protocol_reaches::{Kind, Offer, Subject};
    let mut root = super::needs::stop_root(which == 0, false);
    let mut out = Queue::uncapped();
    let mut limits = f::domain_limits();
    limits.client.domain.text = 512;
    limits.client.domain.long_text = 512;
    limits.client.domain.turn_text = 512;
    let env = Env { now: Time::ZERO, wall: f::NOW, limits };
    let object = c::ObjectKey::Question { task: f::INBOX_TASK, message: 1 };
    for action in [
        Stop::Open { object },
        match which {
            0 => Stop::Apply { offer: Offer::Decide(Subject::Kind(Kind::Implementation)) },
            1 => Stop::Propose { offer: Offer::Decide(Subject::Kind(Kind::Implementation)) },
            2 | 3 => Stop::Keep { object },
            _ => panic!("four stop asking outcomes"),
        },
    ] {
        d::step(&mut root, &env, Event::Act { action: d::Action::Pages(pages::Act::StopAsking(action)) }, &mut out);
    }
    d::step(&mut root, &env, Event::Client { event: jig_client::Event::Stored }, &mut out);
    let mut attempt = None;
    for request in &out {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: sent, .. } },
        } = request
        {
            attempt = Some(*sent);
        }
    }
    let attempt = attempt.expect("stop asking sends one act");
    let answer = match which {
        0 => jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged {
            revision: 1,
        }))),
        1 => jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Proposed {
            proposal: 1,
            to: 1,
        }))),
        2 => {
            use temper_client_protocol_wire::{self as wire, v1 as w};
            jig_client::Answered::Family(jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::answer(
                    &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Dismissed }).unwrap(),
                ),
            })
        }
        3 => jig_client::Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None }),
        _ => panic!("four stop asking outcomes"),
    };
    d::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(answer),
                    why: None,
                },
            },
        },
        &mut out,
    );
    root
}
