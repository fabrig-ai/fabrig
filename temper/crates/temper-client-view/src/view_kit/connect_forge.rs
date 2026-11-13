use super as f;
use skein_lib::List;
use skein_ui::{Role, State};
use temper_client_domain as d;
use temper_client_protocol_wire::v1 as w;

pub const HERE: u16 = 3;

pub const GITHUB: u16 = 2;

pub fn row(number: u16, kind: w::Kind, mode: w::Mode, connection: w::Connection, offer: w::Offer) -> w::ForgeRow {
    w::ForgeRow::new(
        &w::CEILINGS,
        w::ForgeRowParts {
            starts_with: w::StartsWith::Repository,
            makings: List::with_capacity(0),
            host: b"git.example:3000".as_slice().into(),
            number,
            kind,
            mode,
            connection,
            offer,
            first: None,
            installations: List::with_capacity(0),
            ended: None,
            app: None,
            web: None,
            repositories: List::with_capacity(0),
            owned: None,
            admin: false,
            read_only: false,
            ended_stop: None,
        },
    )
    .unwrap()
}

pub fn rows(connection: w::Connection, offer: w::Offer) -> List<w::ForgeRow> {
    let mut rows = List::with_capacity(2);
    rows.push(row(GITHUB, w::Kind::Github, w::Mode::Managed, w::Connection::NotConnected, w::Offer::NotOffered))
        .unwrap();
    rows.push(row(HERE, w::Kind::Forgejo, w::Mode::Owned, connection, offer)).unwrap();
    rows
}

pub fn connected() -> w::Connection {
    w::Connection::Connected(
        w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 42, login: b"temper".as_slice().into() }).unwrap(),
    )
}

#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn node(tree: &skein_ui::Tree, role: Role, name: &[u8]) -> skein_ui::NodeId {
    f::find(tree, role, name).unwrap_or_else(|| panic!("missing {role:?}: {}", String::from_utf8_lossy(name)))
}

#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn words_in_order(tree: &skein_ui::Tree, phrases: &[&str]) {
    let mut bytes = Vec::new();
    for (index, node) in tree.nodes().iter().enumerate() {
        if let Some(name) = &node.name
            && f::node_text(tree, index).is_empty()
        {
            bytes.extend_from_slice(name);
            bytes.push(b'\n');
        }
        for run in &node.text {
            bytes.extend_from_slice(&run.text);
        }
    }
    let words = String::from_utf8_lossy(&bytes);
    let mut remaining = words.as_ref();
    for phrase in phrases {
        let index = remaining.find(phrase).unwrap_or_else(|| panic!("missing or out of order: {phrase}\nin {words}"));
        remaining = &remaining[index.checked_add(phrase.len()).expect("phrase end fits")..];
    }
}

pub fn reread_deployment(root: &mut d::Domain, me: skein_lib::Token, owned: bool) {
    let frame = root.pages().me(root.client()).unwrap().clone();
    f::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch: me, end: jig_client::contract::WatchEnd::Busy },
            },
        },
    );
    let mut out = skein_lib::Queue::uncapped();
    let env = skein_lib::Env { now: d::next_deadline(root).unwrap(), wall: f::NOW, limits: f::domain_limits() };
    d::fire(root, &env, &mut out);
    let mut me = None;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open {
                            watch,
                            subject: jig_client::Watchable::Jig(jig_client::contract::Subject::Me),
                        },
                },
        } = request
        {
            me = Some(watch);
        }
    }
    d::step(
        root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me.expect("me reopens after a gap"),
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        frame,
                    ))),
                },
            },
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
            && temper_client_protocol_wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
                == &w::Read::Deployment
        {
            f::deliver_deployment(root, read, true, owned);
            return;
        }
    }
    panic!("deployment read reaches the world");
}

pub fn drive(root: &mut d::Domain, event: d::Event) -> skein_lib::Queue<d::Request> {
    let limits = f::domain_limits();
    let mut out = skein_lib::Queue::uncapped();
    d::step(root, &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits }, event, &mut out);
    out
}

pub fn act(root: &mut d::Domain, act: d::forge::Act) -> skein_lib::Queue<d::Request> {
    drive(root, d::Event::Act { action: d::Action::Forge(act) })
}

pub fn attempt(mut out: skein_lib::Queue<d::Request>) -> skein_lib::Token {
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            return attempt;
        }
    }
    panic!("request sent")
}

pub fn answer(root: &mut d::Domain, attempt: skein_lib::Token, outcome: w::Outcome) {
    f::root_event(root, answer_event(attempt, outcome));
}

pub fn answer_event(attempt: skein_lib::Token, outcome: w::Outcome) -> d::Event {
    d::Event::Client {
        event: jig_client::Event::Up {
            up: jig_client::Up::Answer {
                attempt,
                envelope: skein_lib::RequestEnvelope::Final,
                why: None,
                answer: Some(jig_client::Answered::Family(jig_client::Family {
                    family: temper_client_protocol_wire::FORGE,
                    version: temper_client_protocol_wire::CLIENT_VERSION,
                    bytes: temper_client_protocol_wire::answer(
                        &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap(),
                    ),
                })),
            },
        },
    }
}

pub fn first_rows(connection: w::Connection, state: w::FirstState) -> List<w::ForgeRow> {
    let first = w::FirstRepository::new(
        &w::CEILINGS,
        w::FirstRepositoryParts {
            playground: false,
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            state,
            label: Box::from(b"temper-host".as_slice()),
        },
    )
    .unwrap();
    let mut rows = List::with_capacity(1);
    rows.push(
        w::ForgeRow::new(
            &w::CEILINGS,
            w::ForgeRowParts {
                starts_with: w::StartsWith::Repository,
                makings: List::with_capacity(0),
                host: b"git.example:3000".as_slice().into(),
                number: HERE,
                kind: w::Kind::Forgejo,
                mode: w::Mode::Owned,
                connection,
                offer: w::Offer::Offered,
                first: Some(first),
                installations: List::with_capacity(0),
                ended: None,
                app: None,
                web: None,
                repositories: List::with_capacity(0),
                owned: None,
                admin: false,
                read_only: false,
                ended_stop: None,
            },
        )
        .unwrap(),
    )
    .unwrap();
    rows
}

pub fn deliver(root: &mut d::Domain, watch: skein_lib::Token, rows: List<w::ForgeRow>) -> skein_lib::Queue<d::Request> {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: temper_client_protocol_wire::FORGE,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                        bytes: temper_client_protocol_wire::forges(
                            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, linked: false }).unwrap(),
                        ),
                    },
                },
            },
        },
    )
}

pub fn made() -> w::FirstState {
    w::FirstState::Made(w::Made::new(&w::CEILINGS, w::MadeParts { id: 42 }).unwrap())
}

pub fn github_rows(accepted: bool) -> List<w::ForgeRow> {
    let mut row = row(
        GITHUB,
        w::Kind::Github,
        w::Mode::Connected,
        w::Connection::Connected(
            w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 9, login: b"my-app[bot]".as_slice().into() })
                .unwrap(),
        ),
        w::Offer::NotOffered,
    )
    .into_parts();
    row.ended = Some(
        w::ConnectStopped::new(&w::CEILINGS, w::ConnectStoppedParts { why: b"fixture stopped".as_slice().into() })
            .unwrap(),
    );
    row.installations = List::with_capacity(3);
    if accepted {
        row.installations
            .push(
                w::Installation::new(
                    &w::CEILINGS,
                    w::InstallationParts {
                        id: 1,
                        account: Some(
                            w::Account::new(&w::CEILINGS, w::AccountParts { id: 10, login: b"acme".as_slice().into() })
                                .unwrap(),
                        ),
                        suspended: false,
                        stray: false,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    for (id, suspended, stray) in [(2, true, !accepted), (3, false, true)] {
        row.installations
            .push(
                w::Installation::new(&w::CEILINGS, w::InstallationParts { id, account: None, suspended, stray })
                    .unwrap(),
            )
            .unwrap();
    }
    let mut rows = List::with_capacity(1);
    rows.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    rows
}

#[expect(
    clippy::disallowed_methods,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn parent(tree: &skein_ui::Tree, id: skein_ui::NodeId) -> Option<skein_ui::NodeId> {
    let index = tree.nodes().iter().position(|n| n.id == id).unwrap();
    tree.nodes()[..index]
        .iter()
        .enumerate()
        .rev()
        .find(|(start, n)| start.checked_add(usize::try_from(n.size).unwrap()).unwrap() > index)
        .map(|(_, n)| n.id)
}

pub fn failure_cases() -> [(List<w::ForgeRow>, Option<usize>, &'static str); 6] {
    let rows = |at| {
        first_rows(
            w::Connection::SetUpFailed(
                w::SetUpFailed::new(
                    &w::CEILINGS,
                    w::SetUpFailedParts {
                        why: "its port, 127.0.0.1:8368, is taken by another program. temper box’s output has the rest."
                            .as_bytes()
                            .into(),
                        at,
                    },
                )
                .unwrap(),
            ),
            w::FirstState::Making,
        )
    };
    [
        (
            rows(Some(w::FailedStep::Fetch)),
            Some(0),
            "Fetching failed: its port, 127.0.0.1:8368, is taken by another program. temper box’s output has the rest.",
        ),
        (
            rows(Some(w::FailedStep::Prepare)),
            Some(1),
            "Preparing failed: its port, 127.0.0.1:8368, is taken by another program. temper box’s output has the rest.",
        ),
        (
            rows(Some(w::FailedStep::Run)),
            Some(2),
            "Starting failed: its port, 127.0.0.1:8368, is taken by another program. temper box’s output has the rest.",
        ),
        (
            rows(Some(w::FailedStep::Connect)),
            Some(3),
            "Connecting failed: its port, 127.0.0.1:8368, is taken by another program. temper box’s output has the rest.",
        ),
        (
            first_rows(
                w::Connection::Connecting,
                w::FirstState::Failed(
                    w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step: w::CreateStep::File }).unwrap(),
                ),
            ),
            Some(4),
            "Making work/hello failed: adding the starter workflow",
        ),
        (
            rows(None),
            None,
            "Set up failed: its port, 127.0.0.1:8368, is taken by another program. temper box’s output has the rest.",
        ),
    ]
}

#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn failed_progress(tree: &skein_ui::Tree, failed: Option<usize>, why: &str) {
    let list = node(tree, Role::List, b"Each step");
    let start = tree.nodes().iter().position(|n| n.id == list).unwrap();
    let end = start.checked_add(usize::try_from(tree.nodes()[start].size).unwrap()).unwrap();
    let items: Vec<_> = tree.nodes()[start..end].iter().filter(|n| n.role == Role::Item).collect();
    assert_eq!(items.len(), 5);
    let labels = [
        if failed > Some(0) { "Fetched Forgejo and its runner" } else { "Fetching Forgejo and its runner" },
        "Preparing: its database, temper’s admin, your account",
        "Starting Forgejo and its runner",
        "Connecting as temper",
        "Making work/hello",
    ];
    for (index, (item, label)) in items.iter().zip(labels).enumerate() {
        let item_index = tree.nodes().iter().position(|n| n.id == item.id).unwrap();
        assert!(f::node_text(tree, item_index).starts_with(label.as_bytes()), "step {index}: {label}");
        assert_eq!(item.styles.as_slice().contains(&crate::look::FLOW_FAILED), failed == Some(index));
        assert_eq!(item.styles.as_slice().contains(&crate::look::FLOW_DONE), failed.is_some_and(|at| index < at));
        assert!(!item.states.as_slice().contains(&State::Current));
    }
    let alert = node(tree, Role::Alert, why.as_bytes());
    assert!(tree.nodes().iter().position(|n| n.id == alert).unwrap() >= end);
    assert_eq!(
        tree.nodes()[start..end]
            .iter()
            .filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"failed"))
            .count(),
        usize::from(failed.is_some())
    );
}

#[expect(
    clippy::disallowed_methods,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn pinned_rows(connection: w::Connection, pins: bool, runner: bool) -> List<w::ForgeRow> {
    let mut parts = row(HERE, w::Kind::Forgejo, w::Mode::Owned, connection, w::Offer::Offered).into_parts();
    if pins {
        parts.owned = Some(
            w::OwnedForge::new(
                &w::CEILINGS,
                w::OwnedForgeParts {
                    forgejo: b"17.0.0".as_slice().into(),
                    runner: runner.then(|| b"11.2.1".as_slice().into()),
                    password: b"/fake/first-password".as_slice().into(),
                },
            )
            .unwrap(),
        );
    }
    let mut rows = List::with_capacity(1);
    rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
    rows
}
