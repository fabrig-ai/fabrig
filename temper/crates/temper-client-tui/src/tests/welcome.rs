//! The welcome's states and hand-written frames (temper/docs/design/client/tui/box.md).
use super::{fixtures as f, goldens};
use crate::{BoxLine, Local, Scheme, Told, Tui, look};
use jig_client::{Up, contract as c};
use skein_lib::{List, Queue, Secret};
use skein_ui::{Role, Style};
use temper_client_domain::{self as d, Address, Domain, Event, Request};
use temper_client_protocol_wire::{self as wire, v1 as w};

// The request's review size; the secret is thirty-two zero bytes, never a credential.
const FULL: skein_ui_screen::Size = skein_ui_screen::Size { columns: 80, rows: 24 };
const PRINTED: &[u8] = b"http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const OPEN: &str = "Welcome. Set temper up in your browser: open this link on this machine.";
const ONCE: &str = "It signs you in as this temper's owner, once, within 2 hours.";
const FOLLOW: &str = "Setting temper up in your browser";
const FOLLOWED: &str = "The steps are the browser's; this screen follows them.";
const LABELS: [&str; 4] = ["1 Welcome", "2 Forge", "3 Model", "4 Ready"];
const AHEAD: [Style; 4] = [look::STEP_AHEAD; 4];
const WELCOME: [Style; 4] = [look::STEP_NOW, look::STEP_AHEAD, look::STEP_AHEAD, look::STEP_AHEAD];
const FORGE: [Style; 4] = [look::STEP_DONE, look::STEP_NOW, look::STEP_AHEAD, look::STEP_AHEAD];
const MODEL: [Style; 4] = [look::STEP_DONE, look::STEP_DONE, look::STEP_NOW, look::STEP_AHEAD];
const READY: [Style; 4] = [look::STEP_DONE, look::STEP_DONE, look::STEP_DONE, look::STEP_NOW];

fn front(link: Option<&[u8]>) -> Tui {
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Front);
    tui.step(Told::Stage { stage: crate::BoxStage::Running });
    if let Some(link) = link {
        tui.step(Told::BoxLine { line: BoxLine::Link { url: Secret::new(link.into()) } });
    }
    tui
}
fn build(root: &Domain, tui: &Tui) -> crate::View {
    let mut view = f::view();
    f::build(&mut view, tui, root);
    view
}
fn body(tree: &skein_ui::Tree) -> List<Box<[u8]>> {
    let mut result = List::with_capacity(tree.len());
    for region in tree.nodes() {
        if region.region != Some(look::TOP) && region.region != Some(look::SECTIONS) {
            continue;
        }
        let start = f::index(tree, region.id);
        let end = start.checked_add(usize::try_from(region.size).unwrap()).unwrap();
        for node in &tree.nodes()[start..end] {
            if node.role == (Role::Heading { level: 1 })
                || node.role == Role::Paragraph
                || node.role == Role::Alert
                || node.role == Role::Item
            {
                result.push(f::text(tree, node)).unwrap();
            }
        }
    }
    result
}
#[derive(Debug)]
enum Expect {
    Before,
    After { marks: [Style; 4], forge: Option<&'static str> },
    Elsewhere { words: &'static str },
}
#[test]
fn welcome_commands_wait_until_the_sign_in_lands() {
    let signed_out = f::signed_out();
    let (starting, _) = f::starting();
    let (live, _) = f::live(Some(f::me(&[])));
    let mut tui = front(None);
    tui.step(Told::Local { local: Local::Signing });
    tui.step(Told::CommandWords { value: Box::from(*b"/") });
    for (root, listed) in [(&signed_out, false), (&starting, false), (&live, true)] {
        let view = build(root, &tui);
        assert_eq!(
            f::count(view.tree(), Role::Menu),
            usize::from(listed),
            "link {:?}, list expected {listed}",
            root.client().link().state
        );
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table checks welcome dispatch, link, step facts, words, tokens and bounds"
)]
#[expect(
    clippy::large_stack_arrays,
    reason = "the fixed state table holds Tui values with talk prompts and terminal-login facts"
)]
fn the_welcome_says_each_state() {
    let empty = welcome_root(&[], false, Some(false), None);
    let linked = welcome_root(&[], true, Some(false), None);
    let pending = welcome_root(&[], true, None, None);
    let connected = row(w::Kind::Forgejo, connected());
    let model = welcome_root(core::slice::from_ref(&connected), false, Some(false), None);
    let ready = welcome_root(&[connected], false, Some(true), None);
    let connecting = welcome_root(&[row(w::Kind::Github, w::Connection::Connecting)], false, Some(false), None);
    let fetching = welcome_root(&[row(w::Kind::Forgejo, setting(Some(w::SetUpStep::Fetch)))], false, Some(false), None);
    let setting_up = welcome_root(&[row(w::Kind::Forgejo, setting(None))], false, Some(false), None);
    let forgejo = welcome_root(&[row(w::Kind::Forgejo, w::Connection::Connecting)], false, Some(false), None);
    let github_setup = welcome_root(&[row(w::Kind::Github, setting(None))], false, Some(false), None);
    let unaccepted = welcome_root(&[row(w::Kind::Github, self::connected())], false, Some(false), None);
    let mut making = row(w::Kind::Forgejo, w::Connection::Connecting).into_parts();
    making.first = Some(
        w::FirstRepository::new(
            &w::CEILINGS,
            w::FirstRepositoryParts {
                org: b"work".as_slice().into(),
                repository: b"hello".as_slice().into(),
                state: w::FirstState::Making,
                label: Box::new([]),
                playground: false,
            },
        )
        .unwrap(),
    );
    // A connected Forgejo counts as connected in the domain, including while its first repository is made.
    making.connection = self::connected();
    making.number = 1;
    let making = welcome_root(&[w::ForgeRow::new(&w::CEILINGS, making).unwrap()], false, Some(false), None);
    let failed = welcome_root(
        &[row(
            w::Kind::Forgejo,
            w::Connection::SetUpFailed(
                w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"stopped".as_slice().into(), at: None })
                    .unwrap(),
            ),
        )],
        false,
        Some(false),
        None,
    );
    let mut fake = row(w::Kind::Forgejo, self::connected()).into_parts();
    fake.mode = w::Mode::Fake;
    fake.number = 7;
    let fake = welcome_root(&[w::ForgeRow::new(&w::CEILINGS, fake).unwrap()], true, Some(false), Some(7));
    let mut skipped = row(w::Kind::Forgejo, w::Connection::Connecting).into_parts();
    skipped.number = 1;
    let skipped = welcome_root(
        &[w::ForgeRow::new(&w::CEILINGS, skipped).unwrap(), row(w::Kind::Github, w::Connection::Connecting)],
        false,
        Some(false),
        Some(1),
    );
    let quiet_first = welcome_root(
        &[row(w::Kind::Forgejo, w::Connection::NotSetUp), {
            let mut second = row(w::Kind::Github, w::Connection::Connecting).into_parts();
            second.number = 3;
            w::ForgeRow::new(&w::CEILINGS, second).unwrap()
        }],
        false,
        Some(false),
        None,
    );
    let first_failed = welcome_root(
        &[{
            let mut failed = row(w::Kind::Forgejo, w::Connection::NotSetUp).into_parts();
            failed.first = Some(
                w::FirstRepository::new(
                    &w::CEILINGS,
                    w::FirstRepositoryParts {
                        org: b"work".as_slice().into(),
                        repository: b"hello".as_slice().into(),
                        state: w::FirstState::Failed(
                            w::CreateFailed::new(
                                &w::CEILINGS,
                                w::CreateFailedParts { step: w::CreateStep::Repository },
                            )
                            .unwrap(),
                        ),
                        label: Box::new([]),
                        playground: false,
                    },
                )
                .unwrap(),
            );
            w::ForgeRow::new(&w::CEILINGS, failed).unwrap()
        }],
        false,
        Some(false),
        None,
    );
    let fake_first = welcome_root(
        &[
            {
                let mut fake = row(w::Kind::Forgejo, setting(Some(w::SetUpStep::Fetch))).into_parts();
                fake.number = 1;
                fake.mode = w::Mode::Fake;
                w::ForgeRow::new(&w::CEILINGS, fake).unwrap()
            },
            row(w::Kind::Github, w::Connection::Connecting),
        ],
        false,
        Some(false),
        None,
    );
    let (unread, _) = f::starting();
    let mut stale = welcome_root(&[], false, Some(false), None);
    f::client(&mut stale, jig_client::Event::Up { up: Up::Served { served: false, versions: None } });
    let (project, _) = f::live(Some(f::me(&[b"Hello"])));
    let chat = super::launchpad::question_root();
    let mut signing = front(Some(PRINTED));
    signing.step(Told::Local { local: Local::Signing });
    let mut latest = front(Some(b"the previous printed link"));
    latest.step(Told::BoxLine { line: BoxLine::Link { url: Secret::new(PRINTED.into()) } });
    let mut unkept = front(None);
    unkept.step(Told::Kept { written: false });
    let mut noted = front(None);
    noted.step(Told::Notice { notice: Some(crate::Notice::Unknown { words: b"/unknown".as_slice().into() }) });
    let long = [b'x'; 4096];
    let rows = [
        ("outside front", &empty, Tui::new(f::ADDRESS), Expect::Elsewhere { words: "No project yet" }),
        ("connecting from the first frame", &f::signed_out(), signing, Expect::Before),
        ("no printed link yet", &empty, front(None), Expect::Before),
        ("the printed link whole", &empty, front(Some(PRINTED)), Expect::Before),
        ("the latest printed link", &empty, latest, Expect::Before),
        ("linked before facts", &pending, front(None), Expect::After { marks: AHEAD, forge: None }),
        ("me unread", &unread, front(None), Expect::Before),
        (
            "sign-in fallback",
            &f::signed_out(),
            front(None),
            Expect::Elsewhere { words: "Sign in this terminal to 127.0.0.1:8367" },
        ),
        ("first row alone decides its words", &quiet_first, front(None), Expect::After { marks: FORGE, forge: None }),
        (
            "fake forge excluded",
            &fake_first,
            front(None),
            Expect::After { marks: FORGE, forge: Some("connecting GitHub…") },
        ),
        (
            "first repository failed",
            &first_failed,
            front(None),
            Expect::After { marks: FORGE, forge: Some("stopped: the browser says why") },
        ),
        ("empty store", &linked, front(None), Expect::After { marks: WELCOME, forge: None }),
        (
            "GitHub connecting",
            &connecting,
            front(None),
            Expect::After { marks: FORGE, forge: Some("connecting GitHub…") },
        ),
        ("Forgejo fetching", &fetching, front(None), Expect::After { marks: FORGE, forge: Some("fetching Forgejo…") }),
        (
            "Forgejo setting up",
            &setting_up,
            front(None),
            Expect::After { marks: FORGE, forge: Some("setting up Forgejo…") },
        ),
        (
            "Forgejo connecting",
            &forgejo,
            front(None),
            Expect::After { marks: FORGE, forge: Some("connecting Forgejo…") },
        ),
        (
            "GitHub setting up",
            &github_setup,
            front(None),
            Expect::After { marks: FORGE, forge: Some("connecting GitHub…") },
        ),
        (
            "GitHub not accepted",
            &unaccepted,
            front(None),
            Expect::After { marks: FORGE, forge: Some("connecting GitHub…") },
        ),
        ("first repository making", &making, front(None), Expect::After { marks: MODEL, forge: None }),
        (
            "set up failed",
            &failed,
            front(None),
            Expect::After { marks: FORGE, forge: Some("stopped: the browser says why") },
        ),
        ("playground fake alone", &fake, front(None), Expect::After { marks: WELCOME, forge: None }),
        (
            "playground forge excluded",
            &skipped,
            front(None),
            Expect::After { marks: FORGE, forge: Some("connecting GitHub…") },
        ),
        ("model remains", &model, front(None), Expect::After { marks: MODEL, forge: None }),
        ("Ready remains", &ready, front(None), Expect::After { marks: READY, forge: None }),
        ("project launchpad", &project, front(None), Expect::Elsewhere { words: "temper · Hellobox" }),
        ("out of date", &stale, front(None), Expect::Before),
        ("chat in front", &chat, front(None), Expect::Elsewhere { words: "" }),
        ("long printed link", &empty, front(Some(&long)), Expect::Before),
        ("non UTF-8 link", &empty, front(Some(b"link\xff")), Expect::Before),
        ("credential not kept", &empty, unkept, Expect::Before),
        ("terminal notes", &empty, noted, Expect::Before),
    ];
    for (name, root, tui, expected) in rows {
        if name == "first repository making" {
            // Check the drawing at the Forge boundary too: the domain's connected row decides Model.
            let mut part = crate::view::welcome::read(root, &tui, &f::words(), f::NOW);
            let forge = part.steps.get_mut(1).unwrap();
            assert_eq!(forge.detail.as_deref(), Some("making its first repository…".as_bytes()));
            for (step, style) in part.steps.iter_mut().zip(FORGE) {
                step.style = style;
            }
            let mut drawn = f::view();
            crate::view::signed(
                &mut drawn,
                &crate::view::Signed {
                    body: crate::view::Body::Welcome(part),
                    foot: crate::view::foot::read(root, &tui, &f::words(), f::NOW),
                },
                &mut Queue::uncapped(),
            );
            f::find(drawn.tree(), Role::Text, "making its first repository…".as_bytes());
            super::limits::fits(drawn.tree(), &crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap());
        }
        if name == "the latest printed link" || name == "the printed link whole" {
            assert!(tui.printed().unwrap().expose() == PRINTED, "{name}: the fixture's printed link");
        }
        let view = build(root, &tui);
        let tree = view.tree();
        let lines = body(tree);
        let mut want = List::with_capacity(16);
        match expected {
            Expect::Before | Expect::After { .. } => {
                want.push(Box::from(b"temperbox".as_slice())).unwrap();
                if root.client().link().state == jig_client::Link::OutOfDate {
                    want.push(f::words().view().say(temper_client_view::words::Word::OutOfDateMain, &[])).unwrap();
                } else {
                    want.push(Box::from("temper box · on this machine, to try it".as_bytes())).unwrap();
                }
                if tui.unkept() {
                    want.push(f::words().say(crate::words::Word::NotKept, &[])).unwrap();
                }
            }
            Expect::Elsewhere { .. } => {}
        }
        match expected {
            Expect::Before => {
                want.push(OPEN.as_bytes().into()).unwrap();
                if let Some(link) = tui.printed() {
                    let printed =
                        if name == "non UTF-8 link" { "link�".as_bytes().into() } else { link.expose().into() };
                    want.push(printed).unwrap();
                    want.push(ONCE.as_bytes().into()).unwrap();
                }
                want.push(b"In the browser".as_slice().into()).unwrap();
                want.push("1 Welcome · 2 Forge · 3 Model · 4 Ready".as_bytes().into()).unwrap();
            }
            Expect::After { marks, forge } => {
                want.push(FOLLOW.as_bytes().into()).unwrap();
                want.push(b"Signed in as the owner".as_slice().into()).unwrap();
                for (index, label) in LABELS.iter().enumerate() {
                    let words = if index == 1
                        && let Some(forge) = forge
                    {
                        skein_lib::bytes::concat(&[label.as_bytes(), " · ".as_bytes(), forge.as_bytes()])
                    } else if index == 2 && name == "Ready remains" {
                        "3 Model · OpenAI API key".as_bytes().into()
                    } else {
                        label.as_bytes().into()
                    };
                    want.push(words).unwrap();
                }
                want.push(FOLLOWED.as_bytes().into()).unwrap();
                let mut items = List::with_capacity(5);
                for node in tree.nodes() {
                    if node.role == Role::Item {
                        items.push(node).unwrap();
                    }
                }
                assert_eq!(items.len(), 5, "{name}: owner and four steps");
                for (node, style) in items.iter().zip([look::STEP_DONE].into_iter().chain(marks)) {
                    assert!(node.styles.as_slice().contains(&style), "{name}: step marker");
                }
                if let Some(forge) = forge {
                    let node = f::find(tree, Role::Text, forge.as_bytes());
                    assert!(node.styles.as_slice().contains(&look::HINT), "{name}: forge hint");
                }
            }
            Expect::Elsewhere { words } => {
                let mut found = false;
                for line in &lines {
                    assert!(
                        line.as_ref() != OPEN.as_bytes() && line.as_ref() != FOLLOW.as_bytes(),
                        "{name}: no welcome"
                    );
                    found |= line.as_ref() == words.as_bytes();
                }
                if words.is_empty() {
                    assert_eq!(root.pages().address(), &Address::Chat { task: 9 }, "{name}: chat");
                    assert_eq!(f::count(tree, Role::Log), 1, "{name}: conversation");
                } else {
                    assert!(found, "{name}: other screen");
                }
                continue;
            }
        }
        if tui.notice().is_some() {
            let notice = f::find(tree, Role::Text, "Unknown command /unknown · /help lists them".as_bytes());
            assert_eq!(notice.styles.as_slice(), &[look::SIGNAL]);
        }
        super::limits::fits(tree, &crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap());
        for (words, style) in [(b"temper".as_slice(), look::TITLE), (b"box".as_slice(), look::ADDRESS)] {
            assert!(f::find(tree, Role::Text, words).styles.as_slice().contains(&style), "{name}: header token");
        }
        if root.client().link().state != jig_client::Link::OutOfDate {
            for (words, style) in
                [(b"temper box".as_slice(), look::NAME), (b"on this machine, to try it".as_slice(), look::HINT)]
            {
                assert!(f::find(tree, Role::Text, words).styles.as_slice().contains(&style), "{name}: box line token");
            }
        }
        // Equality without dumping the printed link in a failed assertion.
        assert!(lines == want, "{name}: the welcome's complete text, including the printed link");
        assert_eq!(view.tick(), None, "{name}: no welcome tick");
        for region in tree.nodes() {
            if region.region != Some(look::TOP) && region.region != Some(look::SECTIONS) {
                continue;
            }
            let start = f::index(tree, region.id);
            let end = start + usize::try_from(region.size).unwrap();
            for node in &tree.nodes()[start..end] {
                assert!(node.binding.is_none(), "{name}: read only");
            }
        }
        if name == "connecting from the first frame" {
            let mut connecting = false;
            for node in tree.nodes() {
                if node.role == Role::Status && f::text(tree, node).ends_with(b"Connecting") {
                    connecting = true;
                }
            }
            assert!(connecting, "{name}: Connecting");
        }
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the golden table embeds its hand-written frames")]
fn the_welcomes_are_their_goldens() {
    let before = welcome_root(&[], false, Some(false), None);
    let after = welcome_root(&[row(w::Kind::Github, w::Connection::Connecting)], true, Some(false), None);
    let printed = front(Some(PRINTED));
    let following = front(None);
    let first = build(&before, &printed);
    let second = build(&after, &following);
    let signing = f::signed_out();
    let mut typed = front(Some(PRINTED));
    typed.step(Told::Local { local: Local::Signing });
    typed.step(Told::CommandWords { value: Box::from(*b"fa") });
    let third = build(&signing, &typed);
    let rows = [
        goldens::Golden {
            name: "welcome-80x24-dark",
            tree: first.tree(),
            scheme: Scheme::Dark,
            background: Some(crate::theme::palette(Scheme::Dark)[0].1),
            size: FULL,
            focus: skein_ui_screen::Focus::default(),
            want: include_bytes!("golden/welcome-80x24-dark.txt"),
        },
        goldens::Golden {
            name: "welcome-steps-80x24-dark",
            tree: second.tree(),
            scheme: Scheme::Dark,
            background: Some(crate::theme::palette(Scheme::Dark)[0].1),
            size: FULL,
            focus: skein_ui_screen::Focus::default(),
            want: include_bytes!("golden/welcome-steps-80x24-dark.txt"),
        },
        goldens::Golden {
            name: "welcome-typed-80x24-dark",
            tree: third.tree(),
            scheme: Scheme::Dark,
            background: Some(crate::theme::palette(Scheme::Dark)[0].1),
            size: FULL,
            focus: skein_ui_screen::Focus::default(),
            want: include_bytes!("golden/welcome-typed-80x24-dark.txt"),
        },
    ];
    goldens::check(&rows);
}

fn connected() -> w::Connection {
    w::Connection::Connected(
        w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() }).unwrap(),
    )
}
fn setting(step: Option<w::SetUpStep>) -> w::Connection {
    w::Connection::SettingUp(w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step, fetched: None }).unwrap())
}
fn row(kind: w::Kind, connection: w::Connection) -> w::ForgeRow {
    w::ForgeRow::new(
        &w::CEILINGS,
        w::ForgeRowParts {
            number: 2,
            host: b"git.example".as_slice().into(),
            kind,
            mode: w::Mode::Owned,
            connection,
            offer: w::Offer::NotOffered,
            first: None,
            starts_with: w::StartsWith::Repository,
            installations: List::with_capacity(0),
            ended: None,
            app: None,
            web: None,
            repositories: List::with_capacity(0),
            owned: None,
            admin: false,
            ended_stop: None,
            read_only: false,
            makings: List::with_capacity(0),
        },
    )
    .unwrap()
}
fn append(out: &mut Queue<Request>, mut more: Queue<Request>) {
    while let Some(request) = more.pop() {
        out.push(request);
    }
}
fn accounts(usable: bool) -> c::Accounts {
    let mut accounts = List::with_capacity(1);
    if usable {
        accounts
            .push(c::AccountLine {
                list: None,
                number: 1,
                place: 1,
                uses: None,
                plan: false,
                provider: Some(18),
                provider_name: Some(c::DocumentText::complete(c::Text::new(b"openai".as_slice().into(), 32).unwrap())),
                manage_usage: None,
                display: c::DocumentText::complete(c::Text::new(b"Model".as_slice().into(), 32).unwrap()),
                holder: c::AccountHolder::Party(73),
                sources: List::from_boxed(Box::new([c::AccountSource {
                    name: None,
                    number: 1,
                    place: 1,
                    kind: c::AccountKind::Key,
                    state: c::AccountState::Granted,
                    usable,
                    expiring: false,
                    attention: false,
                    spent_until: None,
                }])),
                facts: List::with_capacity(0),
                cut: false,
            })
            .unwrap();
    }
    c::Accounts {
        program_hosts: List::with_capacity(0),
        accounts,
        signing: List::with_capacity(0),
        lendable: false,
        lending: None,
        lent_end: None,
        last_end: None,
        usable,
        settled: true,
        cut: false,
        readiness: List::with_capacity(0),
        readiness_cut: false,
    }
}
// Watches and reads are answered only through the root's public deliveries.
fn welcome_root(rows: &[w::ForgeRow], linked: bool, model: Option<bool>, playground: Option<u16>) -> Domain {
    if let Some(usable) = model {
        return welcome_read(rows, linked, Some(accounts(usable)), playground);
    }
    welcome_read(rows, linked, None, playground)
}

pub(super) fn lending_root(kind: Option<c::AccountKind>) -> Domain {
    let mut document = accounts(kind.is_some());
    if let Some(kind) = kind {
        for row in document.accounts.iter_mut() {
            row.plan = kind == c::AccountKind::SignIn;
            let provider = if kind == c::AccountKind::Program { b"anthropic".as_slice() } else { b"openai" };
            row.provider_name = Some(c::DocumentText::complete(c::Text::new(provider.into(), 32).unwrap()));
            for source in row.sources.iter_mut() {
                source.kind = kind;
                source.name = Some(b"work".as_slice().into());
            }
        }
    }
    welcome_read(&[row(w::Kind::Forgejo, connected())], false, Some(document), None)
}

#[expect(clippy::manual_map, reason = "the fixture answers the optional playground without a closure")]
fn welcome_read(rows: &[w::ForgeRow], linked: bool, model: Option<c::Accounts>, playground: Option<u16>) -> Domain {
    let mut root = f::signed_out();
    let mut out = f::event(
        &mut root,
        Event::Start {
            address: Address::Welcome { step: d::Step::WelcomeAsked },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
    );
    append(&mut out, f::client(&mut root, jig_client::Event::Up { up: Up::Served { served: true, versions: None } }));
    while let Some(request) = out.pop() {
        let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        let up = match down {
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) } => {
                let mut me = f::me(&[]);
                me.deployment_role = Some(c::HeldRole::Owner);
                Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) }
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Accounts) } => {
                let Some(accounts) = &model else { continue };
                Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(accounts.clone()))),
                }
            }
            jig_client::Down::Open { watch, subject }
                if Some(&subject) == d::forge::forges::watch(&f::limits().forge).as_ref() =>
            {
                Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::forges(
                            &w::Forges::new(
                                &w::CEILINGS,
                                w::ForgesParts { forges: List::from_boxed(rows.into()), linked },
                            )
                            .unwrap(),
                        ),
                    },
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } } => {
                let Some(_) = model else { continue };
                let record = match wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read() {
                    w::Read::Deployment => Some(wire::deployment_answer(
                        &w::DeploymentAnswer::new(
                            &w::CEILINGS,
                            w::DeploymentAnswerParts { push: None, welcome: true, owned: true, program_login: None },
                        )
                        .unwrap(),
                    )),
                    w::Read::Playground => match playground {
                        Some(forge) => Some(wire::playground_answer(
                            &w::PlaygroundAnswer::new(
                                &w::CEILINGS,
                                w::PlaygroundAnswerParts {
                                    state: w::PlaygroundState::Running,
                                    project: None,
                                    forge,
                                    provider: 17,
                                    owner: b"demo".as_slice().into(),
                                    name: b"demo".as_slice().into(),
                                    cues: List::with_capacity(0),
                                },
                            )
                            .unwrap(),
                        )),
                        None => None,
                    },
                    w::Read::Setting(_)
                    | w::Read::Finding(_)
                    | w::Read::ProposalItems(_)
                    | w::Read::RevisionSetting(_)
                    | w::Read::ProgramEnvironment
                    | w::Read::View(_) => continue,
                };
                Up::ReadDone { read, result: jig_client::Readout::Family { family: wire::FORGE, record } }
            }
            jig_client::Down::Open { watch, .. } => Up::WatchEnded { watch, end: c::WatchEnd::Unknown },
            jig_client::Down::Save { .. }
            | jig_client::Down::Erase { .. }
            | jig_client::Down::Send { .. }
            | jig_client::Down::Continue { .. }
            | jig_client::Down::Close { .. }
            | jig_client::Down::Read { .. } => continue,
        };
        append(&mut out, f::client(&mut root, jig_client::Event::Up { up }));
    }
    assert_eq!(root.forge().linked(), Some(linked), "the fixture delivered the forges document");
    root
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the stage golden table embeds its frames")]
fn the_starting_box_welcomes_are_their_goldens() {
    use crate::BoxStage;
    let rows = [
        (
            "welcome-finding-80x24-dark",
            BoxStage::Finding,
            None,
            "Finding temper-box…",
            look::HINT,
            include_bytes!("golden/welcome-finding-80x24-dark.txt").as_slice(),
        ),
        (
            "welcome-fetching-80x24-dark",
            BoxStage::Fetching { done: 12_300_000, total: Some(24_100_000) },
            None,
            "Downloading temper-box, once for this build: 12.3 of 24.1 MB",
            look::HINT,
            include_bytes!("golden/welcome-fetching-80x24-dark.txt").as_slice(),
        ),
        (
            "welcome-fetching-unsized-80x24-dark",
            BoxStage::Fetching { done: 12_300_000, total: None },
            None,
            "Downloading temper-box, once for this build: 12.3 MB",
            look::HINT,
            include_bytes!("golden/welcome-fetching-unsized-80x24-dark.txt").as_slice(),
        ),
        (
            "welcome-starting-80x24-dark",
            BoxStage::Starting,
            Some(b"10:41:01 Starting serve".as_slice()),
            "Starting the box…",
            look::HINT,
            include_bytes!("golden/welcome-starting-80x24-dark.txt").as_slice(),
        ),
        (
            "welcome-failed-80x24-dark",
            BoxStage::Failed { why: b"WHY".as_slice().into() },
            None,
            "temper-box could not start: WHY",
            look::REFUSED,
            include_bytes!("golden/welcome-failed-80x24-dark.txt").as_slice(),
        ),
    ];
    let root = f::signed_out();
    let mut views = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (_, stage, log, line, style, _) in &rows {
        let mut tui = front(None);
        tui.step(Told::Size { columns: FULL.columns });
        tui.step(Told::Local { local: Local::Signing });
        tui.step(Told::Stage { stage: stage.clone() });
        if let Some(line) = log {
            tui.step(Told::BoxLine { line: BoxLine::Log { line: (*line).into() } });
        }
        let view = build(&root, &tui);
        let node = f::find(view.tree(), Role::Paragraph, line.as_bytes());
        assert!(node.styles.as_slice().contains(style));
        for line in &body(view.tree()) {
            assert_ne!(line.as_ref(), ONCE.as_bytes());
        }
        super::limits::fits(view.tree(), &crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap());
        views.push(view).unwrap();
    }
    let mut checks = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for ((name, _, _, _, _, want), view) in rows.iter().zip(&views) {
        checks
            .push(goldens::Golden {
                name,
                tree: view.tree(),
                scheme: Scheme::Dark,
                background: Some(crate::theme::palette(Scheme::Dark)[0].1),
                size: FULL,
                focus: skein_ui_screen::Focus::default(),
                want,
            })
            .unwrap();
    }
    goldens::check(checks.as_slice());
    startup_details();
}

fn startup_details() {
    use crate::{BoxStage, view};
    for (done, total, want) in [
        (12_349_999, Some(24_199_999), "Downloading temper-box, once for this build: 12.3 of 24.1 MB"),
        (0, None, "Downloading temper-box, once for this build: 0.0 MB"),
        (u64::MAX, None, "Downloading temper-box, once for this build: 18446744073709.5 MB"),
    ] {
        let mut tui = front(None);
        tui.step(Told::Stage { stage: BoxStage::Fetching { done, total } });
        assert_eq!(view::front::stage(&tui, &f::words()), Some((want.as_bytes().into(), look::HINT)));
    }
    let (starting, _) = f::starting();
    let (live, _) = f::live(Some(f::me(&[])));
    let signed_out = f::signed_out();
    for (root, local, last) in
        [(&signed_out, Local::Signing, true), (&starting, Local::Ended, true), (&live, Local::Idle, false)]
    {
        let mut tui = front(Some(PRINTED));
        tui.step(Told::Local { local });
        tui.step(Told::Size { columns: 12 });
        tui.step(Told::BoxLine { line: BoxLine::Log { line: b"old".as_slice().into() } });
        tui.step(Told::BoxLine { line: BoxLine::Log { line: "ab界cd界ef".as_bytes().into() } });
        let part = view::welcome::read(root, &tui, &f::words(), f::NOW);
        assert_eq!(part.last_log.as_deref(), if last { Some("ab界cd界".as_bytes()) } else { None });
        assert!(part.stage.is_none(), "the printed link wins over a stage");
        assert!(!part.signed_in, "the browser's sign-in is not the TUI's");
    }
    let mut tui = front(None);
    assert!(view::front::stage(&tui, &f::words()).is_none(), "running without a link draws nothing");
    tui.step(Told::Stage { stage: BoxStage::Failed { why: b"\xff\x80".as_slice().into() } });
    assert_eq!(
        view::front::stage(&tui, &f::words()),
        Some(("temper-box could not start: ��".as_bytes().into(), look::REFUSED))
    );
    let mut tui = front(None);
    tui.step(Told::Local { local: Local::Signing });
    tui.step(Told::Stage { stage: BoxStage::Failed { why: Box::new([0xff; 513]) } });
    let view = build(&signed_out, &tui);
    super::limits::fits(view.tree(), &crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap());
}
