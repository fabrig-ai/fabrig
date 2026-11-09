//! Remembered page states (temper/docs/design/client/README.md, 5.17).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary tests drive domain answers and inspect semantic subtrees"
)]
use super::fixtures as f;
use crate::look;
use jig_client::{Down, Readable, UnlinkedRequest, contract as c};
use skein_lib::{Env, List, Queue, Time, Token};
use skein_ui::{Act as UiAct, Node, Role, Shortcut, State, Tree};
use temper_client_domain::{
    self as d, Address,
    pages::{self, remembered::Act},
};

const TITLE: &[u8] = b"What temper remembers about you";
const FULL: &str = "12 lines, the most Ask temper keeps: it asks you which to change before it keeps another.";
const EMPTY: &str =
    "Ask temper keeps a line here when you ask it to remember something, such as how you like its answers.";
const MOVED: &str = "This line changed meanwhile; it shows as it is now.";
const LONG: &str = "A line is at most 120 bytes.";
// client/README.md, 5.17: twelve entries, each up to 120 bytes.
const ENTRIES: u64 = 12;
const BYTES: usize = 120;
fn drive(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(root, &Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() }, event, &mut out);
    out
}
fn change(root: &mut d::Domain, act: Act) -> Queue<d::Request> {
    drive(root, d::Event::Act { action: d::Action::Pages(pages::Act::Remembered(act)) })
}
fn read(out: &Queue<d::Request>) -> Token {
    out.iter().find_map(|r| if let d::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(c::Read::Notes { .. }) } } } = r { Some(*read) } else { None }).expect("notes read")
}
fn notes(root: &mut d::Domain, token: Token, count: u64) {
    let mut rows = List::uncapped();
    for number in 1..=count {
        rows.push(c::Note {
            number,
            revision: c::Revision { value: 1 },
            scope: c::NoteScope::Party { party: 73 },
            description: c::DocumentText::complete(
                c::Text::new(b"Keep answers short.".as_slice().into(), 256).unwrap(),
            ),
            body: c::DocumentText::complete(c::Text::new(Box::default(), 0).unwrap()),
            writer: if number == 2 {
                c::NoteAuthor::Party { party: 73 }
            } else {
                c::NoteAuthor::Task { task: 14, attempt: 1 }
            },
            tasks: if number == 3 { List::uncapped() } else { List::from_boxed(Box::new([14])) },
            changed: Some(f::NOW),
            acts: List::uncapped(),
            cut: false,
        })
        .unwrap();
    }
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read: token,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(
                        c::NotePage { rows, before: None, after: None, cut: false },
                    )))),
                },
            },
        },
    );
}
fn refuse(root: &mut d::Domain, refusal: c::Refusal) -> Token {
    let mut out = change(root, Act::Save);
    let mut stored = drive(root, d::Event::Client { event: jig_client::Event::Stored });
    while let Some(request) = stored.pop() {
        out.push(request);
    }
    let attempt = out
        .iter()
        .find_map(|r| {
            if let d::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Send { attempt, .. } } } = r {
                Some(*attempt)
            } else {
                None
            }
        })
        .expect("edit sent");
    let out = drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused { refusal, at: None })),
                    why: None,
                },
            },
        },
    );
    read(&out)
}
fn main(tree: &Tree) -> &[Node] {
    let (at, node) = tree.nodes().iter().enumerate().find(|(_, n)| n.role == Role::Main).unwrap();
    &tree.nodes()[at..at.checked_add(usize::try_from(node.size).unwrap()).unwrap()]
}
fn named<'a>(tree: &'a Tree, role: Role, name: &[u8]) -> &'a Node {
    main(tree).iter().find(|n| n.role == role && n.name.as_deref() == Some(name)).expect("page node")
}
#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "table inputs and expected visibility are independent facts")]
struct Case {
    count: u64,
    answered: bool,
    field: Option<usize>,
    refusal: Option<c::Refusal>,
    refreshed: bool,
    expected_rows: usize,
    empty: bool,
    full: bool,
    line: Option<&'static str>,
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the page's read, edit and refusal states")]
fn remembered_draws_only_answered_rows_and_their_acts() {
    for case in [
        Case {
            count: 0,
            answered: false,
            field: None,
            refusal: None,
            refreshed: false,
            expected_rows: 0,
            empty: false,
            full: false,
            line: None,
        },
        Case {
            count: 0,
            answered: true,
            field: None,
            refusal: None,
            refreshed: false,
            expected_rows: 0,
            empty: true,
            full: false,
            line: None,
        },
        Case {
            count: 3,
            answered: true,
            field: None,
            refusal: None,
            refreshed: false,
            expected_rows: 3,
            empty: false,
            full: false,
            line: None,
        },
        Case {
            count: ENTRIES - 1,
            answered: true,
            field: None,
            refusal: None,
            refreshed: false,
            expected_rows: 11,
            empty: false,
            full: false,
            line: None,
        },
        Case {
            count: ENTRIES,
            answered: true,
            field: None,
            refusal: None,
            refreshed: false,
            expected_rows: 12,
            empty: false,
            full: true,
            line: None,
        },
        Case {
            count: 3,
            answered: true,
            field: Some(BYTES),
            refusal: None,
            refreshed: false,
            expected_rows: 3,
            empty: false,
            full: false,
            line: None,
        },
        Case {
            count: 3,
            answered: true,
            field: Some(BYTES),
            refusal: Some(c::Refusal::Moved { now: c::Revision { value: 2 }, what: c::Moved::Note }),
            refreshed: true,
            expected_rows: 3,
            empty: false,
            full: false,
            line: Some(MOVED),
        },
        Case {
            count: 3,
            answered: true,
            field: Some(BYTES + 1),
            refusal: Some(c::Refusal::Limit { which: c::Limit::Words { limit: 120 } }),
            refreshed: true,
            expected_rows: 3,
            empty: false,
            full: false,
            line: Some(LONG),
        },
        Case {
            count: 3,
            answered: true,
            field: Some(BYTES),
            refusal: Some(c::Refusal::Moved { now: c::Revision { value: 2 }, what: c::Moved::Note }),
            refreshed: false,
            expected_rows: 0,
            empty: false,
            full: false,
            line: None,
        },
    ] {
        let (mut root, _, _) = f::root_at(Address::Home { project: f::PROJECT }, &f::domain_limits());
        let out = drive(&mut root, d::Event::Went { place: d::Place::Address(Address::Remembered) });
        let token = read(&out);
        if case.answered {
            notes(&mut root, token, case.count);
        }
        let field = case.field.map(|len| {
            let mut words = skein_lib::bytes::zeroed(len);
            words.fill(b'x');
            words
        });
        if let Some(words) = &field {
            change(&mut root, Act::Change { note: 2 });
            change(&mut root, Act::Words { words: words.clone() });
        }
        if let Some(refusal) = &case.refusal {
            let token = refuse(&mut root, refusal.clone());
            if case.refreshed {
                notes(&mut root, token, case.count);
            }
        }
        let mut view = f::browser_page_view(&root);
        assert_eq!(skein_ui::outline::check(view.tree()), None, "{case:?}");
        named(view.tree(), Role::Heading { level: 1 }, TITLE);
        named(
            view.tree(),
            Role::Paragraph,
            b"What Ask temper keeps about you across your chats, when you ask it to. Only you see these lines.",
        );
        assert!(
            !view.tree().nodes().iter().any(|n| n.role == Role::Link && n.states.as_slice().contains(&State::Current)),
            "{case:?}"
        );
        let body = main(view.tree());
        let words = String::from_utf8_lossy(&f::nodes_text(body)).into_owned();
        assert_eq!(words.contains("Nothing kept yet"), case.empty, "{case:?}");
        assert_eq!(words.contains(EMPTY), case.empty, "{case:?}");
        assert_eq!(words.contains(FULL), case.full, "{case:?}");
        let buttons: Vec<_> = body
            .iter()
            .filter(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Change"))
            .map(|n| n.id)
            .collect();
        assert_eq!(buttons.len(), case.expected_rows, "{case:?}");
        let deletes: Vec<_> = body
            .iter()
            .filter(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Delete"))
            .map(|n| n.id)
            .collect();
        assert_eq!(deletes.len(), case.expected_rows);
        for (index, (change, delete)) in buttons.into_iter().zip(deletes).enumerate() {
            let note = u64::try_from(index).unwrap() + 1;
            for (id, act) in [(change, Act::Change { note }), (delete, Act::Delete { note })] {
                assert_eq!(
                    crate::act(&mut view, id, UiAct::Press),
                    Some(d::Action::Pages(pages::Act::Remembered(act)))
                );
            }
        }
        if case.expected_rows > 0 {
            named(view.tree(), Role::Text, b"Keep answers short.");
            named(view.tree(), Role::Text, b"Ask temper chat 14 \xc2\xb7 Today \xc2\xb7 Thursday 1 January");
            named(view.tree(), Role::Text, b"Changed by you \xc2\xb7 Today \xc2\xb7 Thursday 1 January");
            named(view.tree(), Role::Text, b"Ask temper \xc2\xb7 Today \xc2\xb7 Thursday 1 January");
        }
        if let Some(line) = case.line {
            let refusal = named(view.tree(), Role::Paragraph, line.as_bytes());
            assert!(refusal.styles.as_slice().contains(&look::SETTINGS_REFUSAL));
        }
        let open = case.field.is_some() && case.refusal.is_none();
        assert_eq!(main(view.tree()).iter().any(|n| n.role == Role::TextField), open);
        if open {
            let node = named(view.tree(), Role::TextField, b"Line");
            assert_eq!(node.value.as_ref().map(|v| v.text.as_ref()), field.as_deref());
            let input = node.id;
            assert_eq!(
                crate::act(&mut view, input, UiAct::Input { value: b"new words".as_slice().into() }),
                Some(d::Action::Pages(pages::Act::Remembered(Act::Words { words: b"new words".as_slice().into() })))
            );
            assert_eq!(
                crate::act(&mut view, input, UiAct::Submit),
                Some(d::Action::Pages(pages::Act::Remembered(Act::Save)))
            );
            for (name, act) in [(b"Save".as_slice(), Act::Save), (b"Cancel", Act::Cancel)] {
                let button = named(view.tree(), Role::Button, name);
                if act == Act::Cancel {
                    assert_eq!(button.shortcut, Some(Shortcut::ESCAPE));
                }
                let id = button.id;
                assert_eq!(
                    crate::act(&mut view, id, UiAct::Press),
                    Some(d::Action::Pages(pages::Act::Remembered(act)))
                );
            }
            let cancel = named(view.tree(), Role::Button, b"Cancel").id;
            let event = crate::act_event(&mut view, cancel, UiAct::Press).unwrap();
            drive(&mut root, event);
            f::build(&mut view, &root);
            assert!(!main(view.tree()).iter().any(|n| n.role == Role::TextField));
            let focus: Vec<_> = main(view.tree()).iter().filter(|n| n.focus.is_some()).collect();
            assert_eq!(focus.len(), 1);
            assert_eq!(focus[0].name.as_deref(), Some(b"Change".as_slice()));
            assert!(root.pages().remembered().unwrap().rows().any(|r| match r {
                pages::remembered::Row::Party { note, line, .. } => note == 2 && line == b"Keep answers short.",
                pages::remembered::Row::Task { .. } => false,
            }));
        }
    }
}
