//! Workers as settings (temper/docs/design/client/pages/workers.md, 1–2).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic subtrees and their words"
)]
use super::fixtures as f;
use crate::look;
use jig_client::{Down, UnlinkedRequest, Watchable, contract as c};
use skein_lib::{List, Queue, Token};
use skein_ui::{Role, State, Tree};
use temper_client_domain::{self as d, Address};

fn address() -> Address {
    Address::Builders { project: f::PROJECT }
}
fn drive(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        event,
        &mut out,
    );
    out
}
fn opened(role: bool) -> (d::Domain, Token, Option<Token>) {
    let (mut root, me_watch, _) = f::root_at(address(), &f::domain_limits());
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.deployment_role = role.then_some(c::HeldRole::Owner);
    let out = drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    let watch = out.iter().find_map(|r| match r {
        d::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { watch, subject } } }
            if *subject == Watchable::Jig(c::Subject::Deployment) =>
        {
            Some(*watch)
        }
        d::Request::Client { .. }
        | d::Request::Unlinked { .. }
        | d::Request::Show { .. }
        | d::Request::Leave { .. }
        | d::Request::SignOut
        | d::Request::Dropped { .. }
        | d::Request::Screen { .. }
        | d::Request::KeepLook { .. }
        | d::Request::KeepSignIn { .. }
        | d::Request::Tab { .. }
        | d::Request::Copy { .. }
        | d::Request::Push(_)
        | d::Request::Fact { .. } => None,
    });
    (root, me_watch, watch)
}
fn document() -> c::Deployment {
    let limits = f::domain_limits();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let mut doc = jig_client_protocol_link::translate::documents::deployment(
        jig_client_test_records::deployment_rows_record(),
        &link,
    )
    .unwrap();
    doc.hosts = List::uncapped();
    doc.worker_settings = List::uncapped();
    doc.cut = false;
    doc
}
fn host(name: Option<&[u8]>, connected: bool, draining: bool) -> c::Host {
    c::Host {
        name: name.map(Into::into),
        channel: 7,
        slots: 2,
        used: 1,
        free: 1,
        connected,
        draining,
        awaiting_logins: false,
        rule: c::HostRule::Open,
        programs: List::with_capacity(0),
        kind: c::HostKind::Worker,
        grace_until: None,
        grace_remaining: None,
        runs: List::with_capacity(0),
        cut: false,
    }
}
fn delivered(doc: c::Deployment) -> d::Domain {
    let (mut root, _, watch) = opened(true);
    drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Deployment(Box::new(doc))),
                },
            },
        },
    );
    root
}
fn main(tree: &Tree) -> &[skein_ui::Node] {
    let (index, node) = tree.nodes().iter().enumerate().find(|(_, n)| n.role == Role::Main).unwrap();
    &tree.nodes()[index..index.checked_add(usize::try_from(node.size).unwrap()).unwrap()]
}
fn named<'a>(tree: &'a Tree, role: Role, name: &str) -> &'a skein_ui::Node {
    main(tree)
        .iter()
        .find(|n| n.role == role && n.name.as_deref() == Some(name.as_bytes()))
        .unwrap_or_else(|| panic!("missing {name}: {}", String::from_utf8_lossy(&f::nodes_text(main(tree)))))
}
fn rows(tree: &Tree) -> &[skein_ui::Node] {
    let (index, node) =
        tree.nodes().iter().enumerate().find(|(_, n)| n.styles.as_slice().contains(&look::SETTINGS_LINES)).unwrap();
    &tree.nodes()[index..index.checked_add(usize::try_from(node.size).unwrap()).unwrap()]
}
fn worker(number: u32, name: &[u8]) -> c::WorkerState {
    c::WorkerState {
        number,
        name: name.into(),
        places: 4,
        digest: Some([0xab; 32]),
        withdrawn: false,
        linked: false,
        since: None,
        pause: c::WorkerPause::Taking,
    }
}
fn workers_document() -> c::Deployment {
    let mut doc = document();
    let mut up = worker(1, b"w1");
    up.linked = true;
    up.since = Some(f::NOW);
    let mut down = worker(2, b"w2");
    down.places = 1;
    down.since = Some(skein_lib::Wall::from_nanos(f::NOW.as_nanos().saturating_sub(60_000_000_000)));
    let mut withdrawn = worker(4, b"withdrawn");
    withdrawn.withdrawn = true;
    let mut local = worker(5, b"local");
    local.digest = None;
    doc.worker_settings = List::from_boxed(Box::new([up, down, worker(3, b"w3"), withdrawn, local]));
    doc.hosts.push(host(Some(b"w1"), true, false)).unwrap();
    // A disconnected host's retained runs are not the worker's connected runs.
    doc.hosts.push(host(Some(b"w2"), false, true)).unwrap();
    doc
}
fn lines(tree: &Tree) -> Vec<&[skein_ui::Node]> {
    let mut nodes = &rows(tree)[1..];
    let mut lines = Vec::new();
    while let Some(node) = nodes.first() {
        let (line, rest) = nodes.split_at(usize::try_from(node.size).unwrap());
        lines.push(line);
        nodes = rest;
    }
    lines
}
fn line<'a>(tree: &'a Tree, name: &[u8]) -> &'a [skein_ui::Node] {
    lines(tree).into_iter().find(|line| line.iter().any(|node| node.name.as_deref() == Some(name))).unwrap()
}
fn send(root: &mut d::Domain, act: d::pages::builders::Act) -> Queue<d::Request> {
    drive(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::Builders(act)) })
}
#[test]
fn each_worker_is_a_line_with_places_runs_digest_and_link_time() {
    for draining in [false, true] {
        let mut doc = workers_document();
        doc.hosts.get_mut(0).unwrap().draining = draining;
        let root = delivered(doc);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            assert_eq!(skein_ui::outline::check(tree), None);
            named(tree, Role::Heading { level: 2 }, "Workers · 4");
            let names: Vec<_> = lines(tree)
                .iter()
                .map(|row| row.iter().find(|n| n.role == Role::Text).unwrap().name.as_deref().unwrap())
                .collect();
            assert_eq!(names, [b"w1".as_slice(), b"w2", b"w3", b"local"]);
            for (name, detail, when) in [
                (
                    b"w1".as_slice(),
                    "4 places · 1 run · digest abababab…",
                    if draining {
                        "Draining".into()
                    } else {
                        format!(
                            "Up since {}",
                            String::from_utf8_lossy(&f::words().time(crate::words::TimeForm::Clock, f::NOW, f::NOW).0)
                        )
                    },
                ),
                (
                    b"w2",
                    "1 place · no runs · digest abababab…",
                    format!(
                        "Down since {}",
                        String::from_utf8_lossy(
                            &f::words()
                                .time(
                                    crate::words::TimeForm::Clock,
                                    skein_lib::Wall::from_nanos(f::NOW.as_nanos().saturating_sub(60_000_000_000)),
                                    f::NOW
                                )
                                .0
                        )
                    ),
                ),
                (b"w3", "4 places · no runs · digest abababab…", "Down".into()),
                (b"local", "4 places · no runs · in the box", "Down".into()),
            ] {
                let row = line(tree, name);
                assert!(
                    row.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_DETAIL)
                        && n.name.as_deref() == Some(detail.as_bytes())),
                    "{detail}"
                );
                assert!(
                    row.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_WHEN)
                        && n.name.as_deref() == Some(when.as_bytes())),
                    "{when}"
                );
            }
            assert!(!f::nodes_text(main(tree)).windows(b"withdrawn".len()).any(|part| part == b"withdrawn"));
            named(tree, Role::Paragraph, "Each agent’s role names which workers it runs on. Ask temper to change it.");
        }
    }
}
#[test]
fn no_worker_settings_says_none_even_with_a_host() {
    let mut doc = document();
    doc.hosts.push(host(Some(b"host-only"), true, false)).unwrap();
    let root = delivered(doc);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), Role::Heading { level: 2 }, "Workers · 0");
        let empty = named(view.tree(), Role::Paragraph, "No worker is added yet.");
        assert!(empty.styles.as_slice().contains(&look::SETTINGS_EMPTY));
        assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_LINES)));
    }
}
#[test]
fn no_role_is_one_line() {
    let (root, _, watch) = opened(false);
    assert!(watch.is_none());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        named(tree, Role::Paragraph, "Workers are seen by the deployment’s owners.");
        assert!(!main(tree).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_SECTION)));
    }
}
#[test]
fn an_unread_document_claims_nothing() {
    for end in [None, Some(c::WatchEnd::NotAllowed), Some(c::WatchEnd::Closed)] {
        let (mut root, _, watch) = opened(true);
        if let Some(end) = end {
            drive(
                &mut root,
                d::Event::Client {
                    event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch: watch.unwrap(), end } },
                },
            );
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            named(view.tree(), Role::Heading { level: 2 }, "Workers");
            assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_LINES)
                || n.styles.as_slice().contains(&look::SETTINGS_EMPTY)
                || n.styles.as_slice().contains(&look::SETTINGS_FACTS)));
        }
    }
    let (mut root, me, _) = opened(false);
    drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch: me, end: c::WatchEnd::Closed } },
        },
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), Role::Heading { level: 2 }, "Workers");
    }
}
#[test]
fn a_cut_document_says_so() {
    for cut in [false, true] {
        let mut doc = document();
        doc.cut = cut;
        let root = delivered(doc);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(
                main(view.tree()).iter().any(|n| n.name.as_deref() == Some(b"Some worker facts were omitted.")),
                cut
            );
        }
    }
}
#[test]
fn a_member_has_the_workers_but_no_field_or_press() {
    let (mut root, me_watch, watch) = opened(true);
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.deployment_role = Some(c::HeldRole::Policy { role: 1 });
    drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Deployment(Box::new(workers_document()))),
                },
            },
        },
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), Role::Heading { level: 2 }, "Workers · 4");
        assert!(!main(view.tree()).iter().any(|n| n.role == Role::TextField || n.role == Role::Button));
        assert!(rows(view.tree()).iter().all(|n| n.binding.is_none()));
    }
}
#[test]
fn owners_fields_and_presses_bind_in_both_shells_and_gate_add() {
    use d::pages::builders::Act;
    let mut root = delivered(workers_document());
    for browser in [false, true] {
        let mut view = if browser { f::browser_page_view(&root) } else { f::page_view(&root) };
        let tree = view.tree();
        let field = named(tree, Role::TextField, "Places for w1");
        assert_eq!(field.value.as_ref().unwrap().text.as_ref(), b"4");
        assert!(
            line(tree, b"w1")
                .iter()
                .any(|n| n.name.as_deref() == Some(b"Change") && n.states.as_slice().contains(&State::Disabled))
        );
        assert!(!line(tree, b"local").iter().any(|n| n.name.as_deref() == Some(b"Remove")));
        assert_eq!(named(tree, Role::TextField, "The worker’s name").placeholder.as_deref(), Some(b"w1".as_slice()));
        assert_eq!(named(tree, Role::TextField, "Places").value.as_ref().unwrap().text.as_ref(), b"4");
        assert_eq!(
            named(tree, Role::TextField, "Its secret’s digest").placeholder.as_deref(),
            Some(b"64 hexadecimal digits".as_slice())
        );
        assert!(named(tree, Role::Button, "Add").states.as_slice().contains(&State::Disabled));
        for (name, value, act, submit) in [
            (
                "Places for w1",
                b"8".as_slice(),
                Act::Places { number: 1, value: b"8".as_slice().into() },
                Act::Change(1),
            ),
            ("The worker’s name", b"w9", Act::Name(b"w9".as_slice().into()), Act::Add),
            ("Places", b"8", Act::AddPlaces(b"8".as_slice().into()), Act::Add),
            ("Its secret’s digest", b"ab", Act::Digest(b"ab".as_slice().into()), Act::Add),
        ] {
            let id = named(view.tree(), Role::TextField, name).id;
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Input { value: value.into() }),
                Some(d::Action::Pages(d::pages::Act::Builders(act)))
            );
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Submit),
                Some(d::Action::Pages(d::pages::Act::Builders(submit)))
            );
        }
        let remove = line(view.tree(), b"w1").iter().find(|n| n.name.as_deref() == Some(b"Remove")).unwrap().id;
        assert_eq!(
            crate::act(&mut view, remove, skein_ui::Act::Press),
            Some(d::Action::Pages(d::pages::Act::Builders(Act::Remove(1))))
        );
    }
    send(&mut root, Act::Places { number: 1, value: b"8".as_slice().into() });
    send(&mut root, Act::Name(b"w9".as_slice().into()));
    send(&mut root, Act::Digest(b"abababababababababababababababababababababababababababababababab".as_slice().into()));
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let change = line(view.tree(), b"w1").iter().find(|n| n.name.as_deref() == Some(b"Change")).unwrap();
        assert!(!change.states.as_slice().contains(&State::Disabled));
        let id = change.id;
        assert_eq!(
            crate::act(&mut view, id, skein_ui::Act::Press),
            Some(d::Action::Pages(d::pages::Act::Builders(Act::Change(1))))
        );
        let add = named(view.tree(), Role::Button, "Add");
        assert!(!add.states.as_slice().contains(&State::Disabled));
        let id = add.id;
        assert_eq!(
            crate::act(&mut view, id, skein_ui::Act::Press),
            Some(d::Action::Pages(d::pages::Act::Builders(Act::Add)))
        );
    }
    for invalid in [
        Act::Name(b"w2".as_slice().into()),
        Act::AddPlaces(b"0".as_slice().into()),
        Act::Digest(b"AB".as_slice().into()),
    ] {
        send(&mut root, invalid);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(named(view.tree(), Role::Button, "Add").states.as_slice().contains(&State::Disabled));
            named(view.tree(), Role::Text, "w2 is already a worker.");
        }
    }
}
#[test]
fn a_linked_refusal_is_under_its_worker_not_the_add_form() {
    let mut root = delivered(workers_document());
    let out = send(&mut root, d::pages::builders::Act::Remove(1));
    let read = out
        .iter()
        .find_map(|request| match request {
            d::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Read { read, .. } } } => Some(*read),
            d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Push(_)
            | d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Copy { .. }
            | d::Request::Fact { .. } => None,
        })
        .unwrap();
    let out = drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Settings(Box::new(
                        c::SettingsDocument {
                            newest: 5,
                            settings: List::uncapped(),
                            held: List::uncapped(),
                            pending: List::uncapped(),
                        },
                    )))),
                },
            },
        },
    );
    let attempt = out
        .iter()
        .find_map(|request| match request {
            d::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Send { attempt, .. } } } => {
                Some(*attempt)
            }
            d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Push(_)
            | d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Copy { .. }
            | d::Request::Fact { .. } => None,
        })
        .unwrap();
    drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                        refusal: c::Refusal::Malformed {
                            at: 0,
                            why: c::Malformed::Settings(c::MalformedSettings {
                                setting: c::Setting::Worker(c::SettingNumber { number: 1 }),
                                fault: c::SettingFault::Linked,
                                detail: None,
                            }),
                        },
                        at: None,
                    })),
                    why: None,
                },
            },
        },
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let message = named(view.tree(), Role::Paragraph, "w1 is up: stop it first. Its runs end interrupted.");
        assert!(line(view.tree(), b"w1").iter().any(|node| node.id == message.id));
        assert!(!line(view.tree(), b"w2").iter().any(|node| node.id == message.id));
        assert!(line(view.tree(), b"w1").iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_REFUSAL)));
    }
}
#[test]
fn the_page_says_builder() {
    let root = delivered(document());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let text = String::from_utf8_lossy(&f::nodes_text(main(view.tree()))).to_lowercase();
        assert!(text.contains("worker"));
        assert!(!text.contains("host"));
        assert!(!text.contains("builder"));
    }
}
#[test]
fn a_full_document_fits_the_tree_bound() {
    let mut doc = document();
    let count = f::domain_limits().client.domain.rows;
    let name = vec![b'b'; usize::try_from(c::PROGRAM_HOST_NAME).unwrap()];
    for number in 1..=count {
        doc.worker_settings.push(worker(number, &name)).unwrap();
    }
    let root = delivered(doc);
    for words in [f::words(), f::browser_words()] {
        let limits = crate::Limits::of(&f::domain_limits(), &words).unwrap();
        let mut view = crate::View::new(&limits, &f::domain_limits(), words, f::FORGE);
        f::build(&mut view, &root);
        assert_eq!(
            rows(view.tree()).iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_WHEN)).count(),
            usize::try_from(count).unwrap()
        );
        named(
            view.tree(),
            Role::Paragraph,
            "Each agent’s role names which workers it runs on. Ask temper to change it.",
        );
    }
}
#[test]
fn an_unlisted_project_keeps_its_missing_message() {
    let root = f::root_at(Address::Builders { project: 3 }, &f::domain_limits()).0;
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), Role::Paragraph, "Project 3 is not one of yours.");
        assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_SECTION)));
    }
}
