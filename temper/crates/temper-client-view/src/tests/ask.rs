//! The request's Ask temper page in both shells (temper/docs/design/client/pages/README.md).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests compare semantic subtrees and terminal text"
)]
use super::fixtures as f;
use crate::look;
use jig_client::contract as c;
use skein_ui::{Act, Node, Role, State, Style, Tree};
use temper_client_domain::{self as d, Address, pages};

const ENTRY: &[u8] = b"Ask temper, or tell it how you work";
const LEDE: &str = "Pick a way for Shop, or ask temper to build your own.";
const DRAFT: &[u8] = b"My way of working";
const UNKNOWN: u32 = 99;
// The request has three stock ways; its fourth tab is Yours.
const STOCK: u32 = 3;
const CURRENT: u32 = 1;
const REVIEW: u32 = 3;
const NEWEST: u32 = 8;
// Exactly one tab is current, and only the shown tab has a panel (contracts, 3).
const ONE_PANEL: usize = 1;
const ONE_NOW: usize = 1;
const NAMES: [&[u8]; 3] = [b"You approve", b"Agents review", b"Draft first"];
const LINES: [&[u8]; 3] = [
    b"An agent makes each change; nothing reaches main until you approve it.",
    b"An agent makes each change, another reviews it, and code lands it.",
    b"You and an agent write a design draft; agents build it on its own branch, and it lands whole.",
];
use jig_client::{Down, Up};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn root() -> d::Domain {
    f::playground_root(Address::Ask { project: f::PROJECT }, f::PROJECT, true)
}
fn input(root: &mut d::Domain, act: pages::Act) {
    f::root_event(root, d::Event::Act { action: d::Action::Pages(act) });
}
fn named<'a>(tree: &'a Tree, role: Role, name: &[u8]) -> &'a Node {
    tree.nodes().iter().find(|n| n.role == role && n.name.as_deref() == Some(name)).expect("named node")
}
fn styled(tree: &Tree, style: Style) -> &[Node] {
    let at = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&style)).expect("styled node");
    let end = at.checked_add(usize::try_from(tree.nodes()[at].size).unwrap()).unwrap();
    &tree.nodes()[at..end]
}
fn children(nodes: &[Node]) -> Vec<&Node> {
    let mut out = Vec::new();
    let mut at = 1_usize;
    while at < nodes.len() {
        out.push(&nodes[at]);
        at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    }
    out
}
fn parts(tree: &Tree, style: Style) -> Vec<(Role, Vec<Style>)> {
    children(styled(tree, style)).into_iter().map(|n| (n.role, n.styles.as_slice().to_vec())).collect()
}
fn print(view: &crate::View) -> skein_ui_text::Print {
    let mut out = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", None, &mut out);
    out
}
fn disabled(node: &Node) -> bool {
    node.states.as_slice().contains(&State::Disabled)
}

fn subtree<'a>(tree: &'a Tree, node: &Node) -> &'a [Node] {
    let at = tree.nodes().iter().position(|n| n.id == node.id).unwrap();
    let end = at.checked_add(usize::try_from(node.size).unwrap()).unwrap();
    &tree.nodes()[at..end]
}
fn leaves(tree: &Tree, node: &Node) -> Vec<Vec<u8>> {
    children(subtree(tree, node))
        .iter()
        .map(|n| {
            assert_eq!(n.role, Role::Text);
            n.text.iter().flat_map(|run| run.text.iter().copied()).collect()
        })
        .collect()
}

struct Ways {
    root: d::Domain,
    watch: Token,
    settings: Token,
}
fn event(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        event,
        &mut out,
    );
    out
}
fn up(root: &mut d::Domain, up: Up) -> Queue<d::Request> {
    event(root, d::Event::Client { event: jig_client::Event::Up { up } })
}
fn loading_fixture(configured: bool) -> Ways {
    let (mut root, me, _, _, _, _, settings, _, _) = f::root_at_with_values(
        Address::Home { project: f::PROJECT },
        &f::domain_limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        None,
        false,
        false,
    );
    let asking = f::asking_me(root.pages().me(root.client()).unwrap().clone());
    f::deliver_me(&mut root, me, asking);
    up(
        &mut root,
        Up::FamilyServed {
            family: wire::FORGE,
            versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
        },
    );
    let out = event(&mut root, d::Event::Went { place: d::Place::Address(Address::Ask { project: f::PROJECT }) });
    let wanted = d::forge::stock_ways::watch(f::PROJECT, &f::domain_limits().forge).unwrap();
    let watch = out
        .iter()
        .find_map(|r| {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } },
            } = r
                && *subject == wanted
            {
                Some(*watch)
            } else {
                None
            }
        })
        .expect("ways watch");
    let mut f = Ways { root, watch, settings: settings.expect("settings watch") };
    settings_document(&mut f, configured);
    f
}
fn fixture(current: Option<u32>, may_change: bool, configured: bool) -> Ways {
    let mut f = loading_fixture(configured);
    show(&mut f, &panel(current, may_change));
    f
}
fn settings_document(f: &mut Ways, configured: bool) {
    let mut settings = List::uncapped();
    if configured {
        settings
            .push(c::SettingLine {
                setting: c::Setting::Application { kind: 2, scope: f::PROJECT, number: 1 },
                name: None,
                last: Some(NEWEST),
                follows: c::SettingFollows::Change,
                application: None,
            })
            .unwrap();
    }
    up(
        &mut f.root,
        Up::Watched {
            watch: f.settings,
            event: c::WatchEvent::Delivered(c::Delivery::Settings(Box::new(c::SettingsDocument {
                newest: NEWEST,
                settings,
                held: List::uncapped(),
                pending: List::uncapped(),
            }))),
        },
    );
}
fn show(f: &mut Ways, panel: &w::WaysPanel) {
    up(
        &mut f.root,
        Up::FamilyDelivered {
            watch: f.watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::ways_panel(panel),
            },
        },
    );
}
fn panel(current: Option<u32>, may_change: bool) -> w::WaysPanel {
    let ways = NAMES
        .into_iter()
        .zip(LINES)
        .enumerate()
        .map(|(index, (name, line))| {
            w::StockWay::new(
                &w::CEILINGS,
                w::StockWayParts {
                    headline: b"".as_slice().into(),
                    name: name.into(),
                    line: line.into(),
                    path: path(index),
                    items: List::from_boxed(Box::new([w::ProposalItem::new(
                        &w::CEILINGS,
                        w::ProposalItemParts { kind: 7, scope: f::PROJECT, number: 1, edit: w::SettingEdit::Removed },
                    )
                    .unwrap()])),
                    delegation: false,
                    roles: List::with_capacity(0),
                },
            )
            .unwrap()
        })
        .collect();
    w::WaysPanel::new(
        &w::CEILINGS,
        w::WaysPanelParts {
            words: None,
            project: f::PROJECT,
            ways: List::from_boxed(ways),
            current,
            own: path(0),
            may_change,
        },
    )
    .unwrap()
}
#[expect(clippy::too_many_lines, reason = "the three paths keep their stops and supplied cards in one fixture")]
fn path(index: usize) -> w::BranchPath {
    use w::{
        Actor::{Agent, Bar, Code, Person},
        PathStep as S,
    };
    let paths: [&[(w::Actor, S, &[u8])]; 3] = [
        &[
            (Person, S::Ask, b"you ask"),
            (Agent, S::Produce, b"produce"),
            (Code, S::Check, b"check"),
            (Agent, S::Repair, b"repair"),
            (Code, S::Check, b"check"),
            (Person, S::Approve, b"you approve"),
            (Code, S::Conflict, b"conflict"),
            (Agent, S::Resolve, b"resolve"),
            (Code, S::Merge, b"merge"),
            (Bar, S::Branch, b"main"),
        ],
        &[
            (Person, S::Ask, b"you ask"),
            (Agent, S::Produce, b"produce"),
            (Code, S::Check, b"check"),
            (Agent, S::Review, b"review"),
            (Agent, S::Repair, b"repair"),
            (Code, S::Check, b"check"),
            (Code, S::Conflict, b"conflict"),
            (Agent, S::Resolve, b"resolve"),
            (Code, S::Merge, b"merge"),
            (Bar, S::Branch, b"main"),
        ],
        &[
            (Person, S::Draft, b"draft"),
            (Agent, S::Plan, b"plan"),
            (Agent, S::ProduceMany, "produce ×n".as_bytes()),
            (Code, S::Merge, b"merge"),
            (Code, S::Check, b"check"),
            (Agent, S::Repair, b"repair"),
            (Bar, S::Branch, b"goal branch"),
            (Code, S::Check, b"check"),
            (Code, S::Conflict, b"conflict"),
            (Agent, S::Resolve, b"resolve"),
            (Code, S::Merge, b"merge"),
            (Code, S::Verify, b"verify"),
            (Bar, S::Branch, b"main"),
        ],
    ];
    let stops = paths[index]
        .iter()
        .enumerate()
        .map(|(at, (actor, step, words))| {
            let code = match step {
                S::Merge => Some((
                    w::Opens::Queue,
                    "code · lands the change",
                    "the queue",
                    "Does",
                    "merges exactly the commit its checks saw",
                )),
                S::Check if index == 2 && at == 4 => Some((
                    w::Opens::Tier(
                        w::OpensTier::new(&w::CEILINGS, w::OpensTierParts { name: b"slow".as_slice().into() }).unwrap(),
                    ),
                    "code · checks the goal's branch",
                    "slow",
                    "Runs",
                    "the tier slow on the goal's branch",
                )),
                S::Check => Some((
                    w::Opens::Rule(
                        w::OpensRule::new(&w::CEILINGS, w::OpensRuleParts { kind: 1, scope: 0, number: 1 }).unwrap(),
                    ),
                    "code · holds a change until CI passes",
                    "CI",
                    "Checks",
                    "the repository's CI, at the change's head",
                )),
                S::Prove => {
                    Some((w::Opens::Turn, "code · runs the test gates at the candidate", "the turn", "Gates", "unit"))
                }
                S::Conflict => Some((
                    w::Opens::Queue,
                    "code · main has moved in a way that clashes with the change",
                    "the queue",
                    "When",
                    "the change reaches the front of the queue: main is merged into it, never by an agent",
                )),
                S::Verify => Some((
                    w::Opens::Tier(
                        w::OpensTier::new(&w::CEILINGS, w::OpensTierParts { name: b"slow".as_slice().into() }).unwrap(),
                    ),
                    "code · checks what landed",
                    "slow",
                    "Runs",
                    "the tier slow after each landing",
                )),
                S::Ask
                | S::Draft
                | S::Plan
                | S::Produce
                | S::ProduceMany
                | S::Review
                | S::Repair
                | S::Approve
                | S::Resolve
                | S::Prepare
                | S::Turn
                | S::CatchUp
                | S::Branch
                | S::Design
                | S::CloseOut => None,
            };
            let code = if *actor == Code { code } else { None };
            w::Stop::new(
                &w::CEILINGS,
                w::StopParts {
                    needed: false,
                    over: Box::default(),
                    actor: actor.clone(),
                    step: step.clone(),
                    words: (*words).into(),
                    opens: if let Some((opens, ..)) = &code {
                        opens.clone()
                    } else if *step == S::Review {
                        w::Opens::Gate(
                            w::OpensGate::new(&w::CEILINGS, w::OpensGateParts { number: 1, charter: 4 }).unwrap(),
                        )
                    } else if *step == S::Produce {
                        w::Opens::Charter(
                            w::OpensCharter::new(&w::CEILINGS, w::OpensCharterParts { charter: 3 }).unwrap(),
                        )
                    } else {
                        w::Opens::Nothing
                    },
                    card: if let Some((_, who, name, label, value)) = code {
                        w::Card::new(
                            &w::CEILINGS,
                            w::CardParts {
                                who: who.as_bytes().into(),
                                name: name.as_bytes().into(),
                                facts: List::from_boxed(Box::new([w::Fact::new(
                                    &w::CEILINGS,
                                    w::FactParts { label: label.as_bytes().into(), value: value.as_bytes().into() },
                                )
                                .unwrap()])),
                            },
                        )
                        .unwrap()
                    } else {
                        w::Card::new(
                            &w::CEILINGS,
                            w::CardParts {
                                who: "an agent · holds a change until it approves".as_bytes().into(),
                                name: if *step == S::Produce {
                                    b"the producer".as_slice().into()
                                } else {
                                    b"the reviewer".as_slice().into()
                                },
                                facts: List::from_boxed(
                                    [
                                        (b"Model".as_slice(), "Claude Opus 5.5 · high".as_bytes()),
                                        (b"Can", b"read the change and its checks; never edit, push or merge"),
                                        (b"Asked to", b"Review the change at the named head."),
                                    ]
                                    .into_iter()
                                    .map(|(label, value)| {
                                        w::Fact::new(
                                            &w::CEILINGS,
                                            w::FactParts { label: label.into(), value: value.into() },
                                        )
                                        .unwrap()
                                    })
                                    .collect(),
                                ),
                            },
                        )
                        .unwrap()
                    },
                },
            )
            .unwrap()
        })
        .collect();
    w::BranchPath::new(
        &w::CEILINGS,
        w::BranchPathParts { repository: None, branch: b"main".as_slice().into(), stops: List::from_boxed(stops) },
    )
    .unwrap()
}
fn tab(tree: &Tree, index: usize) -> &Node {
    children(styled(tree, look::WAYS_TABS))[index]
}
fn press(root: &mut d::Domain, browser: bool, role: Role, label: &[u8]) {
    let mut view = if browser { f::browser_page_view(root) } else { f::page_view(root) };
    let id = named(view.tree(), role, label).id;
    let action = crate::act(&mut view, id, Act::Press).expect("press action");
    f::root_event(root, d::Event::Act { action });
}

#[test]
fn requested_opening_keeps_order_roles_and_tokens() {
    let f = fixture(Some(CURRENT), true, false);
    for view in [f::page_view(&f.root), f::browser_page_view(&f.root)] {
        let tree = view.tree();
        for (style, expected) in [
            (
                look::STARTER,
                vec![
                    (Role::Paragraph, vec![look::STARTER_LEDE, look::WAYS_LEDE]),
                    (Role::Region, vec![look::STARTER_WAYS, look::WAYS]),
                ],
            ),
            (
                look::WAYS,
                vec![
                    (Role::Tabs, vec![look::WAYS_TABS]),
                    (Role::Paragraph, vec![look::WAYS_AXIS]),
                    (Role::Group, vec![look::WAYS_PANEL]),
                ],
            ),
            (
                look::WAYS_PANEL,
                vec![
                    (Role::Paragraph, vec![look::WAYS_SAYS]),
                    (Role::Drawing, vec![look::WAYS_PATH]),
                    (Role::Slot, vec![look::WAYS_FOOT]),
                ],
            ),
            (look::WAYS_FOOT, vec![(Role::Paragraph, vec![look::WAYS_LEGEND]), (Role::Slot, vec![look::WAYS_ACTS])]),
            (
                look::WAYS_LEGEND,
                vec![
                    (Role::Text, vec![look::WAYS_KIND_AGENT]),
                    (Role::Text, vec![look::WAYS_KIND_MAYBE]),
                    (Role::Text, vec![look::WAYS_KIND_PERSON]),
                    (Role::Text, vec![look::WAYS_KIND_CODE]),
                    (Role::Text, vec![look::WAYS_KIND_LINE]),
                ],
            ),
            (look::WAYS_ACTS, vec![(Role::Button, vec![look::WAYS_USE])]),
        ] {
            assert_eq!(parts(tree, style), expected, "{style:?}");
        }
        assert_eq!(leaves(tree, &styled(tree, look::WAYS_LEDE)[0]), [LEDE.as_bytes()]);
        for (style, name) in [
            (look::WAYS, b"Ways of working".as_slice()),
            (look::WAYS_TABS, b"Ways of working"),
            (look::WAYS_AXIS, b"From more of you to more of your agents"),
            (look::WAYS_PANEL, b"Agents review"),
            (look::WAYS_PATH, b"The way a change goes"),
            (look::WAYS_LEGEND, b"An agent, an agent when needed, a person, code; no merge tool above the dotted line"),
        ] {
            assert_eq!(styled(tree, style)[0].name.as_deref(), Some(name));
        }
        assert!(tree.nodes().iter().all(|n| n.role != Role::Heading { level: 2 }));
    }
}

#[test]
fn tabs_draw_in_read_order_with_only_the_shown_panel() {
    for browser in [false, true] {
        let mut f = fixture(Some(CURRENT), true, false);
        let paths: [&[&[u8]]; 3] = [
            &[
                b"you ask, a person",
                b"produce, an agent",
                b"check, code",
                b"repair, an agent",
                b"check, code",
                b"you approve, a person",
                b"conflict, code",
                b"resolve, an agent",
                b"merge, code",
                b"main, a branch",
            ],
            &[
                b"you ask, a person",
                b"produce, an agent",
                b"check, code",
                b"review, an agent",
                b"repair, an agent",
                b"check, code",
                b"conflict, code",
                b"resolve, an agent",
                b"merge, code",
                b"main, a branch",
            ],
            &[
                b"draft, a person",
                b"plan, an agent",
                "produce ×n, an agent".as_bytes(),
                b"merge, code",
                b"check, code",
                b"repair, an agent",
                b"goal branch, a branch",
                b"check, code",
                b"conflict, code",
                b"resolve, an agent",
                b"merge, code",
                b"verify, code",
                b"main, a branch",
            ],
        ];
        for (index, (name, expected)) in NAMES.into_iter().zip(paths).enumerate() {
            let mut view = if browser { f::browser_page_view(&f.root) } else { f::page_view(&f.root) };
            let tabs = children(styled(view.tree(), look::WAYS_TABS));
            assert_eq!(tabs.len(), usize::try_from(STOCK).unwrap());
            for (at, (node, word)) in tabs.into_iter().zip(NAMES).enumerate() {
                assert_eq!(node.role, Role::Tab);
                assert_eq!(node.name, None);
                let expected = if at == usize::try_from(CURRENT).unwrap() {
                    if browser { b"Agents reviewnow".as_slice() } else { b"Agents review, now".as_slice() }
                } else {
                    word
                };
                assert_eq!(
                    f::node_text(view.tree(), view.tree().nodes().iter().position(|n| n.id == node.id).unwrap())
                        .as_ref(),
                    expected
                );
            }
            let id = tab(view.tree(), index).id;
            let action = crate::act(&mut view, id, Act::Press).unwrap();
            assert_eq!(
                action,
                d::Action::Pages(pages::Act::WayTab { project: f::PROJECT, tab: u32::try_from(index).unwrap() })
            );
            f::root_event(&mut f.root, d::Event::Act { action });
            let view = if browser { f::browser_page_view(&f.root) } else { f::page_view(&f.root) };
            assert_eq!(styled(view.tree(), look::WAYS_PANEL)[0].name.as_deref(), Some(name));
            assert_eq!(
                view.tree().nodes().iter().filter(|n| n.styles.as_slice().contains(&look::WAYS_PANEL)).count(),
                ONE_PANEL
            );
            for (at, node) in children(styled(view.tree(), look::WAYS_TABS)).into_iter().enumerate() {
                assert_eq!(node.states.as_slice().contains(&State::Selected), at == index);
            }
            assert_eq!(leaves(view.tree(), &styled(view.tree(), look::WAYS_SAYS)[0]), [LINES[index]]);
            assert_eq!(
                styled(view.tree(), look::WAYS_PATH)
                    .iter()
                    .filter(|n| n.role == Role::Item)
                    .map(|n| n.name.as_deref().unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                view.tree().nodes().iter().filter(|n| n.styles.as_slice().contains(&look::WAYS_NOW)).count(),
                ONE_NOW
            );
        }
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "the existing interaction rows cover agent and code cards in both shells")]
fn stop_presses_open_move_close_and_tabs_close_cards() {
    for browser in [false, true] {
        let mut f = fixture(Some(CURRENT), true, false);
        let mut view = if browser { f::browser_page_view(&f.root) } else { f::page_view(&f.root) };
        let drawing = styled(view.tree(), look::WAYS_PATH)[0].id;
        for data in [vec![], vec![0, 1]] {
            assert_eq!(crate::act(&mut view, drawing, Act::Drawing { data: data.into() }), None);
        }
        let action =
            crate::act(&mut view, drawing, Act::Drawing { data: Box::new([u8::try_from(REVIEW).unwrap()]) }).unwrap();
        assert_eq!(action, d::Action::Pages(pages::Act::WayStop { project: f::PROJECT, stop: REVIEW }));
        f::root_event(&mut f.root, d::Event::Act { action });
        let mut view = if browser { f::browser_page_view(&f.root) } else { f::page_view(&f.root) };
        assert_eq!(styled(view.tree(), look::WAYS_ZOOM)[0].name.as_deref(), Some(b"Review".as_slice()));
        assert_eq!(
            parts(view.tree(), look::WAYS_ZOOM),
            [(Role::Slot, vec![look::WAYS_ZOOM_HEAD]), (Role::Group, vec![])]
        );
        assert_eq!(
            parts(view.tree(), look::WAYS_ZOOM_HEAD),
            [(Role::Heading { level: 3 }, vec![]), (Role::Text, vec![]), (Role::Link, vec![])]
        );
        for (role, words) in [
            (Role::Term, b"Model".as_slice()),
            (Role::Description, "Claude Opus 5.5 · high".as_bytes()),
            (Role::Term, b"Can"),
            (Role::Description, b"read the change and its checks; never edit, push or merge"),
            (Role::Term, b"Asked to"),
            (Role::Description, b"Review the change at the named head."),
        ] {
            named(view.tree(), role, words);
        }
        let open = named(view.tree(), Role::Link, "Open the reviewer ›".as_bytes()).id;
        assert_eq!(crate::href(&mut view, open).as_deref(), Some(b"/projects/1/settings/roles/4".as_slice()));
        let review = named(view.tree(), Role::Button, b"review, an agent");
        assert!(review.states.as_slice().contains(&State::Expanded));
        let id = review.id;
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(d::Action::Pages(pages::Act::WayStop { project: f::PROJECT, stop: REVIEW }))
        );
        for (label, heading, link) in [
            (
                b"produce, an agent".as_slice(),
                Some(b"Produce".as_slice()),
                Some(("Open the producer ›", b"/projects/1/settings/roles/3".as_slice())),
            ),
            (b"produce, an agent", None, None),
            (
                b"review, an agent",
                Some(b"Review".as_slice()),
                Some(("Open the reviewer ›", b"/projects/1/settings/roles/4".as_slice())),
            ),
        ] {
            press(&mut f.root, browser, Role::Button, label);
            let mut view = if browser { f::browser_page_view(&f.root) } else { f::page_view(&f.root) };
            if let Some((label, href)) = link {
                let id = named(view.tree(), Role::Link, label.as_bytes()).id;
                assert_eq!(crate::href(&mut view, id).as_deref(), Some(href));
            }
            assert_eq!(
                view.tree()
                    .nodes()
                    .iter()
                    .find(|n| n.styles.as_slice().contains(&look::WAYS_ZOOM))
                    .and_then(|n| n.name.as_deref()),
                heading
            );
        }
        for (label, heading, who, fact, value, link) in [
            (
                b"check, code".as_slice(),
                b"Check".as_slice(),
                "code · holds a change until CI passes",
                b"Checks".as_slice(),
                b"the repository's CI, at the change's head".as_slice(),
                Some("Open CI ›"),
            ),
            (
                b"merge, code",
                b"Merge",
                "code · lands the change",
                b"Does",
                b"merges exactly the commit its checks saw",
                None,
            ),
        ] {
            press(&mut f.root, browser, Role::Button, label);
            let mut view = if browser { f::browser_page_view(&f.root) } else { f::page_view(&f.root) };
            assert_eq!(styled(view.tree(), look::WAYS_ZOOM)[0].name.as_deref(), Some(heading));
            let head = children(styled(view.tree(), look::WAYS_ZOOM_HEAD));
            assert_eq!(head[0].role, Role::Heading { level: 3 });
            assert_eq!(head[0].name.as_deref(), Some(heading));
            assert_eq!(head[1].role, Role::Text);
            assert_eq!(
                head[1].text.iter().flat_map(|run| run.text.iter().copied()).collect::<Vec<_>>(),
                who.as_bytes()
            );
            let card = styled(view.tree(), look::WAYS_ZOOM);
            let facts: Vec<_> = card.iter().filter(|n| n.role == Role::Term || n.role == Role::Description).collect();
            assert_eq!(facts.len(), 2);
            assert_eq!((facts[0].role, facts[0].name.as_deref()), (Role::Term, Some(fact)));
            assert_eq!((facts[1].role, facts[1].name.as_deref()), (Role::Description, Some(value)));
            let links: Vec<_> = card.iter().filter(|n| n.role == Role::Link).collect();
            assert_eq!(links.len(), usize::from(link.is_some()));
            if let Some(link) = link {
                let id = named(view.tree(), Role::Link, link.as_bytes()).id;
                assert_eq!(crate::href(&mut view, id).as_deref(), Some(b"/projects/1/settings/rules/1/1".as_slice()));
            }
        }
        let mut view = if browser { f::browser_page_view(&f.root) } else { f::page_view(&f.root) };
        let id = tab(view.tree(), 0).id;
        let action = crate::act(&mut view, id, Act::Press).unwrap();
        f::root_event(&mut f.root, d::Event::Act { action });
        let view = if browser { f::browser_page_view(&f.root) } else { f::page_view(&f.root) };
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::WAYS_ZOOM)));
    }
}

#[test]
fn acts_follow_current_permission_and_configuration() {
    for (current, may_change, configured, shown, expected) in [
        (Some(CURRENT), true, false, 0, Some(b"Use this".as_slice())),
        (Some(CURRENT), true, false, CURRENT, Some(b"Keep it".as_slice())),
        (None, true, false, STOCK, None),
        (Some(CURRENT), false, false, 0, None),
        (Some(CURRENT), true, true, CURRENT, None),
    ] {
        let mut f = fixture(current, may_change, configured);
        input(&mut f.root, pages::Act::WayTab { project: f::PROJECT, tab: shown });
        for mut view in [f::page_view(&f.root), f::browser_page_view(&f.root)] {
            let act = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&look::WAYS_USE));
            assert_eq!(
                act.and_then(|n| n.name.as_deref()),
                expected,
                "current={current:?}, may_change={may_change}, configured={configured}"
            );
            if let Some(node) = act {
                let id = node.id;
                let act = crate::act(&mut view, id, Act::Press);
                let expected = if shown == CURRENT {
                    pages::Act::KeepWay { project: f::PROJECT }
                } else {
                    pages::Act::UseWay { project: f::PROJECT, way: shown }
                };
                assert_eq!(act, Some(d::Action::Pages(expected)));
            }
            if current.is_none() {
                assert_eq!(styled(view.tree(), look::WAYS_PANEL)[0].name.as_deref(), Some(b"Yours".as_slice()));
                assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::WAYS_SAYS)));
            }
        }
    }
}

#[test]
fn pending_way_acts_stay_visible_disabled_and_refuse_in_words() {
    for (tab, label, act) in [
        (0, b"Use this".as_slice(), pages::Act::UseWay { project: f::PROJECT, way: 0 }),
        (CURRENT, b"Keep it".as_slice(), pages::Act::KeepWay { project: f::PROJECT }),
    ] {
        let mut f = fixture(Some(CURRENT), true, false);
        input(&mut f.root, pages::Act::WayTab { project: f::PROJECT, tab });
        let sent = event(&mut f.root, d::Event::Act { action: d::Action::Pages(act) });
        for mut view in [f::page_view(&f.root), f::browser_page_view(&f.root)] {
            let node = named(view.tree(), Role::Button, label);
            assert!(disabled(node));
            let id = node.id;
            assert_eq!(crate::act(&mut view, id, Act::Press), None);
        }
        let out = event(&mut f.root, d::Event::Client { event: jig_client::Event::Stored });
        let attempt = sent
            .iter()
            .chain(out.iter())
            .find_map(|r| {
                if let d::Request::Unlinked {
                    request: jig_client::UnlinkedRequest::Down { down: Down::Send { attempt, .. } },
                } = r
                {
                    Some(*attempt)
                } else {
                    None
                }
            })
            .unwrap();
        let why = b"Forge 2: owner approval asks 2 people; only 1 can approve.";
        up(
            &mut f.root,
            Up::Answer {
                attempt,
                envelope: skein_lib::RequestEnvelope::Final,
                why: None,
                answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                    refusal: c::Refusal::Judged {
                        at: 0,
                        setting: c::Setting::Application { kind: 7, scope: f::PROJECT, number: 1 },
                        why: c::Text::new(why.as_slice().into(), 128).unwrap(),
                    },
                    at: None,
                })),
            },
        );
        for view in [f::page_view(&f.root), f::browser_page_view(&f.root)] {
            let refusal = styled(view.tree(), look::WAYS_REFUSED);
            assert_eq!(refusal[0].role, Role::Alert);
            assert!(f::text(view.tree()).windows(why.len()).any(|s| s == why));
            assert!(!disabled(named(view.tree(), Role::Button, label)));
            assert_eq!(children(styled(view.tree(), look::WAYS_PANEL)).last().unwrap().id, refusal[0].id);
        }
    }
}

#[test]
fn loading_and_ended_ways_keep_the_lede_and_composer() {
    let mut f = loading_fixture(false);
    let watch = f.watch;
    for (ended, expected) in [(false, vec![]), (true, vec![(Role::Paragraph, vec![look::WAYS_SAYS])])] {
        if ended {
            up(&mut f.root, Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded });
        }
        for view in [f::page_view(&f.root), f::browser_page_view(&f.root)] {
            assert_eq!(parts(view.tree(), look::WAYS), expected);
            if ended {
                assert_eq!(
                    leaves(view.tree(), &styled(view.tree(), look::WAYS_SAYS)[0]),
                    [b"The ways can't be read now."]
                );
            }
            named(view.tree(), Role::TextArea, ENTRY);
            assert_eq!(leaves(view.tree(), &styled(view.tree(), look::WAYS_LEDE)[0]), [LEDE.as_bytes()]);
        }
    }
}

#[test]
fn terminal_prints_tabs_stops_card_legend_and_act_in_order() {
    let mut f = fixture(Some(CURRENT), true, false);
    input(&mut f.root, pages::Act::WayStop { project: f::PROJECT, stop: REVIEW });
    let view = f::page_view(&f.root);
    let out = print(&view);
    let text = String::from_utf8_lossy(out.text.as_slice());
    assert_eq!(text.matches("Ways of working:").count(), 1, "{text}");
    assert_eq!(text.matches("The way a change goes:").count(), 1, "{text}");
    assert!(text.contains("agent, when needed, person, code; no merge tool above"), "{text}");
    let mut rest = text.as_ref();
    for word in [
        "== Ask temper ==",
        LEDE,
        "Ways of working:",
        "You approve",
        "Agents review, now",
        "Draft first",
        "more of you",
        "more of your agents",
        core::str::from_utf8(LINES[1]).unwrap(),
        "you ask, a person",
        "produce, an agent",
        "check, code",
        "review, an agent",
        "repair, an agent",
        "check, code",
        "conflict, code",
        "resolve, an agent",
        "merge, code",
        "main, a branch",
        "Review",
        "an agent · holds a change until it approves",
        "Open the reviewer ›",
        "Model",
        "Claude Opus 5.5 · high",
        "Can",
        "Asked to",
        "agent",
        "when needed",
        "person",
        "code",
        "no merge tool above",
        "Keep it",
        core::str::from_utf8(ENTRY).unwrap(),
        "Send",
    ] {
        let at = rest.find(word).unwrap_or_else(|| panic!("missing or out of order {word}: {text}"));
        rest = &rest[at + word.len()..];
    }
    for removed in [
        "How should temper run",
        "Tell it how you work, in your own words",
        "What it sets",
        "Ways to begin",
        "Small changes, alone",
        "A design draft first",
        "Models by role",
        "Not sure yet?",
        "At scale, as temper builds itself",
        "See how it works",
        "Start from it",
        "How temper builds temper, beside the chat",
    ] {
        assert!(!text.contains(removed), "{removed}: {text}");
    }
}

#[test]
fn lede_uses_the_project_name() {
    for name in [b"Hello".as_slice(), b"Another project"] {
        let (root, _, _, _, _) = f::root_at_full(Address::Ask { project: f::PROJECT }, &f::domain_limits(), name);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(
                leaves(view.tree(), &styled(view.tree(), look::WAYS_LEDE)[0]),
                [format!("Pick a way for {}, or ask temper to build your own.", String::from_utf8_lossy(name))
                    .as_bytes()]
            );
        }
    }
}

fn draft_first() -> Ways {
    let mut f = fixture(Some(CURRENT), true, false);
    input(&mut f.root, pages::Act::WayTab { project: f::PROJECT, tab: 2 });
    let mut view = f::browser_page_view(&f.root);
    let use_this = named(view.tree(), Role::Button, b"Use this").id;
    let action = crate::act(&mut view, use_this, Act::Press).expect("Use this is enabled");
    assert_eq!(action, d::Action::Pages(pages::Act::UseWay { project: f::PROJECT, way: 2 }));
    let out = event(&mut f.root, d::Event::Act { action });
    let attempt = out
        .iter()
        .find_map(|request| {
            let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Send { attempt, .. } },
            } = request
            else {
                return None;
            };
            Some(*attempt)
        })
        .expect("Use this sends the way's settings");
    up(
        &mut f.root,
        Up::Answer {
            attempt,
            envelope: skein_lib::RequestEnvelope::Final,
            why: None,
            answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(
                c::OutcomeSettingsChanged { revision: NEWEST },
            )))),
        },
    );
    settings_document(&mut f, true);
    show(&mut f, &panel(Some(2), true));
    f
}

#[test]
fn asking_composer_uses_the_message_bound_for_send_and_enter() {
    let mut f = draft_first();
    let bound = usize::try_from(f.root.pages().me(f.root.client()).unwrap().words_bytes).unwrap();
    let mut limits = f::domain_limits();
    limits.client.domain.long_text = 10_000;
    for browser in [false, true] {
        let words = if browser { f::browser_words() } else { f::words() };
        let envelope = crate::Limits::of(&limits, &words).unwrap();
        let mut view = crate::View::new(&envelope, &limits, words, f::FORGE);
        for length in [257, bound, bound + 1, 10_001, 257] {
            let words = vec![b'a'; length].into_boxed_slice();
            d::step(
                &mut f.root,
                &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits },
                d::Event::Act { action: d::Action::Pages(pages::Act::ChatWords { words: words.clone() }) },
                &mut Queue::uncapped(),
            );
            f::build(&mut view, &f.root);
            let entry = named(view.tree(), Role::TextArea, ENTRY);
            assert_eq!(entry.value.as_ref().unwrap().text.as_ref(), &words[..length.min(10_000)]);
            assert_eq!(entry.states.as_slice().contains(&State::SubmitsOnEnter), length <= bound);
            let entry = entry.id;
            let send = named(view.tree(), Role::Button, b"Send");
            assert_eq!(disabled(send), length > bound);
            let send = send.id;
            let expected = (length <= bound).then_some(d::Action::Pages(pages::Act::StartChat));
            assert_eq!(crate::act(&mut view, entry, Act::Submit), expected);
            assert_eq!(crate::act(&mut view, send, Act::Press), expected);
            let line = format!(
                "That's {} character{} over what one message holds.",
                String::from_utf8_lossy(&crate::words::grouped(u64::try_from(length.saturating_sub(bound)).unwrap())),
                if length == bound + 1 { "" } else { "s" }
            );
            if length > bound {
                named(view.tree(), Role::Status, line.as_bytes());
            } else {
                assert!(!String::from_utf8_lossy(&f::text(view.tree())).contains("over what one message holds"));
            }
        }
    }
}

#[test]
fn pending_start_disables_send_and_keeps_words() {
    let mut root = root();
    input(&mut root, pages::Act::ChatWords { words: DRAFT.into() });
    input(&mut root, pages::Act::StartChat);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let send = named(view.tree(), Role::Button, b"Send");
        assert!(disabled(send));
        let id = send.id;
        assert_eq!(crate::act(&mut view, id, Act::Press), None);
        let entry = named(view.tree(), Role::TextArea, ENTRY);
        assert_eq!(entry.value.as_ref().unwrap().text.as_ref(), DRAFT);
        let id = entry.id;
        assert_eq!(crate::act(&mut view, id, Act::Submit), None);
    }
}

#[test]
fn empty_composer_is_focused_with_send_disabled() {
    let root = root();
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let entry = named(view.tree(), Role::TextArea, ENTRY);
        assert_eq!(entry.placeholder.as_deref(), Some(ENTRY));
        assert_eq!(entry.focus, Some(0));
        assert_eq!(entry.value.as_ref().unwrap().text.as_ref(), b"");
        assert!(entry.states.as_slice().contains(&State::SubmitsOnEnter));
        let send = named(view.tree(), Role::Button, b"Send");
        assert!(disabled(send));
        let id = send.id;
        assert_eq!(crate::act(&mut view, id, Act::Press), None);
    }
}

#[test]
fn refused_start_keeps_words_and_puts_refusal_under_composer() {
    let mut root = root();
    input(&mut root, pages::Act::ChatWords { words: DRAFT.into() });
    let mut out = Queue::uncapped();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Act { action: d::Action::Pages(pages::Act::StartChat) },
        &mut out,
    );
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Client { event: jig_client::Event::Stored },
        &mut out,
    );
    let mut sent = None;
    for request in &out {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: Down::Send { attempt, .. } },
        } = request
        {
            sent = Some(*attempt);
        }
    }
    let attempt = sent.expect("sent start");
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                        refusal: c::Refusal::NotOffered,
                        at: None,
                    })),
                },
            },
        },
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let entry = named(view.tree(), Role::TextArea, ENTRY);
        assert_eq!(entry.value.as_ref().unwrap().text.as_ref(), DRAFT);
        assert!(!disabled(named(view.tree(), Role::Button, b"Send")));
        let foot = children(styled(view.tree(), look::SESSION_FOOT));
        assert_eq!(foot.last().unwrap().role, Role::Status);
        assert!(String::from_utf8_lossy(&f::main_text(view.tree())).contains("not offered"));
    }
}

#[test]
fn settings_is_current_in_the_rail_and_phone_tabs() {
    let root = root();
    for (view, tabs) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
        assert_eq!(view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&look::TABBAR)), tabs);
        for style in [look::RAIL, look::TABBAR] {
            // The terminal has no tab bar.
            if let Some(at) = view.tree().nodes().iter().position(|n| n.styles.as_slice().contains(&style)) {
                let end = at.checked_add(usize::try_from(view.tree().nodes()[at].size).unwrap()).unwrap();
                let current: Vec<_> = view.tree().nodes()[at..end]
                    .iter()
                    .filter(|n| n.states.as_slice().contains(&State::Current))
                    .map(|n| n.name.as_deref())
                    .collect();
                assert_eq!(current, [Some(b"Ask temper".as_slice())]);
                if style == look::TABBAR {
                    let tabs = styled(view.tree(), style);
                    assert_eq!(tabs.iter().filter(|node| node.role == Role::Link).count(), 4);
                    let ask = tabs
                        .iter()
                        .find(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Ask temper"))
                        .unwrap();
                    let at = view.tree().nodes().iter().position(|node| node.id == ask.id).unwrap();
                    assert_eq!(f::node_text(view.tree(), at).as_ref(), b"Ask");
                }
            }
        }
    }
}

#[test]
fn unlisted_project_says_what_home_says_in_both_shells() {
    let (root, _, _) = f::root_at(Address::Ask { project: UNKNOWN }, &f::domain_limits());
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(String::from_utf8_lossy(&f::main_text(view.tree())).contains("Project 99 is not one of yours."));
        let node = named(view.tree(), Role::Link, b"Work").id;
        assert_eq!(
            crate::act(&mut view, node, Act::Press),
            Some(d::Action::Pages(pages::Act::Go {
                address: Address::Work { project: f::PROJECT, filter: d::WorkFilter::All }
            }))
        );
        assert!(f::find(view.tree(), Role::TextArea, ENTRY).is_none());
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::STARTER)));
    }
}

// The offer crosses three accounts: two providers, with both a plan and a key at OpenAI.
const CLAUDE: u32 = 18;
const PLAN: u32 = 17;
const KEY: u32 = 19;

fn ask_root(offers: &[(u32, &[u32])], held: &[u32], preview: &[u8]) -> d::Domain {
    let (sample, _) = super::choice::provider_byline_root(
        Address::NewChat { draft: false },
        [c::Readiness::Usable, c::Readiness::Usable],
        [true, true],
        true,
    );
    let mut me = f::asking_me(sample.pages().me(sample.client()).unwrap().clone());
    let mut document = sample.pages().accounts_document(sample.client()).unwrap().clone();
    let mut key = document.accounts.iter().find(|row| row.number == PLAN).unwrap().clone();
    key.number = KEY;
    key.plan = false;
    key.sources.get_mut(0).unwrap().kind = c::AccountKind::Key;
    let mut accounts = document.accounts.as_slice().to_vec();
    accounts.push(key);
    document.accounts = List::from_boxed(accounts.into_iter().filter(|row| held.contains(&row.number)).collect());
    let project = me.projects.get_mut(0).unwrap();
    for template in project.templates.iter_mut() {
        template.llm = None;
    }
    let sample_model = project.models.get(0).unwrap().clone();
    project.models = List::with_capacity(u32::try_from(offers.len()).unwrap());
    for (provider, accounts) in offers {
        let mut model = sample_model.clone();
        let (name, shown) = if *provider == CLAUDE {
            (b"claude-opus-5-5".as_slice(), b"Opus 5.5".as_slice())
        } else {
            (b"gpt-6-astra".as_slice(), b"Astra 6".as_slice())
        };
        model.name = f::first_run::text(name);
        model.shown = Some(f::first_run::text(shown));
        model.provider = Some(*provider);
        model.accounts = List::from_boxed(accounts.to_vec().into_boxed_slice());
        model.efforts = List::from_boxed(Box::new([f::first_run::text(b"low"), f::first_run::text(b"high")]));
        model.effort = Some(f::first_run::text(b"high"));
        project.models.push(model).unwrap();
    }
    project.defaults = List::from_boxed(Box::new([
        c::MeDefault {
            template: jig_client::shapes::CHAT,
            choice: c::LlmChoice {
                later: None,
                account: None,
                main: c::ModelEffort { model: f::first_run::text(b"ordinary-chat"), effort: None },
                subagents: List::with_capacity(0),
            },
        },
        c::MeDefault {
            template: 5,
            choice: c::LlmChoice {
                later: None,
                account: None,
                main: c::ModelEffort { model: f::first_run::text(preview), effort: Some(f::first_run::text(b"high")) },
                subagents: List::with_capacity(0),
            },
        },
    ]));
    let (mut root, me_watch, _, _, accounts_watch) =
        f::root_at_full(Address::Ask { project: f::PROJECT }, &f::domain_limits(), b"Shop");
    f::deliver_me(&mut root, me_watch, me);
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched {
                    watch: accounts_watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(document))),
                },
            },
        },
    );
    root
}

#[test]
fn ask_temper_says_its_own_default() {
    for (offers, held, preview, expected) in [
        (
            vec![(CLAUDE, &[CLAUDE][..]), (PLAN, &[PLAN][..])],
            vec![CLAUDE, PLAN],
            b"claude-opus-5-5".as_slice(),
            "Claude Opus 5.5 · high",
        ),
        (vec![(PLAN, &[PLAN][..])], vec![PLAN], b"gpt-6-astra", "Astra 6 · high"),
    ] {
        let root = ask_root(&offers, &held, preview);
        assert!(root.pages().chat().is_none());
        assert!(root.pages().choice().requested.is_none(), "a preview stays out of the draft");
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let byline = named(view.tree(), Role::Disclosure, b"Model");
            assert_eq!(byline.description.as_deref(), Some(expected.as_bytes()));
            assert!(byline.states.as_slice().contains(&State::FoldsAway));
            assert!(byline.styles.as_slice().contains(&look::MODEL));
        }
    }
}

#[test]
fn ask_picker_buttons_edit_the_draft() {
    use pages::choice::Act as Choice;
    for browser in [false, true] {
        let mut root = ask_root(&[(CLAUDE, &[CLAUDE]), (PLAN, &[PLAN])], &[CLAUDE, PLAN], b"claude-opus-5-5");
        for (label, expected) in
            [(b"Astra 6".as_slice(), Choice::Model { index: 1, account: None }), (b"low", Choice::Effort { index: 0 })]
        {
            let mut view = if browser { f::browser_page_view(&root) } else { f::page_view(&root) };
            let node = named(view.tree(), Role::Button, label).id;
            let action = crate::act(&mut view, node, Act::Press).expect("model choice");
            assert_eq!(action, d::Action::Pages(pages::Act::Choice(expected)));
            f::root_event(&mut root, d::Event::Act { action });
        }
        let choice = root.pages().choice().requested.as_ref().unwrap();
        assert_eq!(choice.main.model.bytes(), b"gpt-6-astra");
        assert_eq!(choice.main.effort.as_ref().unwrap().bytes(), b"low");
        assert!(root.pages().chat().is_none());
    }
}

#[test]
fn asking_chat_picker_applies_to_its_next_run() {
    use pages::choice::Act as Choice;
    let mut task = super::chat::asking_task();
    task.llm = Some(c::LlmChoice {
        later: None,
        account: None,
        main: c::ModelEffort { model: f::first_run::text(b"chosen"), effort: None },
        subagents: List::with_capacity(0),
    });
    let mut acts = task.acts.as_slice().to_vec();
    acts.push(c::Offered { act: c::ActKind::Amend, admission: c::Admission::Allowed, effect: None });
    task.acts = List::from_boxed(acts.into_boxed_slice());
    for browser in [false, true] {
        let mut root = super::chat::asking_task_root(task.clone(), true, 37);
        let mut view = if browser { f::browser_page_view(&root) } else { f::page_view(&root) };
        let byline = named(view.tree(), Role::Disclosure, b"Model");
        assert_eq!(byline.description.as_deref(), Some(b"chosen".as_slice()));
        assert!(byline.styles.as_slice().contains(&look::MODEL));
        let settings = named(view.tree(), Role::Link, b"How work runs").id;
        assert_eq!(crate::href(&mut view, settings).as_deref(), Some(b"/projects/1/settings".as_slice()));
        let id = named(view.tree(), Role::Button, b"second").id;
        let action = crate::act(&mut view, id, Act::Press).expect("picker action");
        assert_eq!(action, d::Action::Pages(pages::Act::Choice(Choice::Model { index: 1, account: None })));
        f::root_event(&mut root, d::Event::Act { action });
        assert_eq!(root.pages().choice().requested.as_ref().unwrap().main.model.bytes(), b"second");
        assert!(root.pages().choice().pending.is_some(), "the next-run amendment was sent");
    }
}

#[test]
fn composer_and_path_emit_the_requested_acts() {
    let mut root = root();
    input(&mut root, pages::Act::ChatWords { words: DRAFT.into() });
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let entry = named(view.tree(), Role::TextArea, ENTRY).id;
        assert_eq!(
            crate::act(&mut view, entry, Act::Input { value: b"Changed".as_slice().into() }),
            Some(d::Action::Pages(pages::Act::ChatWords { words: b"Changed".as_slice().into() }))
        );
        assert_eq!(crate::act(&mut view, entry, Act::Submit), Some(d::Action::Pages(pages::Act::StartChat)));
        let send = named(view.tree(), Role::Button, b"Send").id;
        assert_eq!(crate::act(&mut view, send, Act::Press), Some(d::Action::Pages(pages::Act::StartChat)));
        let links: Vec<_> = children(styled(view.tree(), look::TRAIL)).into_iter().map(|n| n.id).collect();
        let back = styled(view.tree(), look::BACK)[0].id;
        for (node, address) in
            [(back, Address::Home { project: f::PROJECT }), (links[0], Address::Home { project: f::PROJECT })]
        {
            assert_eq!(crate::href(&mut view, node).as_deref(), Some(d::address::print(&address).as_ref()));
            assert_eq!(crate::act(&mut view, node, Act::Press), Some(d::Action::Pages(pages::Act::Go { address })));
        }
    }
}

#[test]
fn absent_asking_template_disables_the_composer() {
    let (mut absent, watch, _) = f::root_at(Address::Ask { project: f::PROJECT }, &f::domain_limits());
    let mut me = absent.pages().me(absent.client()).unwrap().clone();
    let project = me.projects.get_mut(0).unwrap();
    project.templates =
        List::from_boxed(project.templates.iter().filter(|template| template.number != 5).cloned().collect());
    f::deliver_me(&mut absent, watch, me);
    input(&mut absent, pages::Act::ChatWords { words: b"How does temper work?".as_slice().into() });
    for view in [f::page_view(&absent), f::browser_page_view(&absent)] {
        let tree = view.tree();
        assert!(disabled(named(tree, Role::Button, b"Send")));
        assert!(disabled(named(tree, Role::TextArea, ENTRY)));
        assert!(String::from_utf8_lossy(&f::text(tree)).contains("Ask temper isn't set up in this project."));
    }
}
