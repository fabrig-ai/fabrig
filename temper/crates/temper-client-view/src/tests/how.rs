//! How work runs' parts in both shells (temper/docs/design/client/pages/settings.md, 2).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic drawings and terminal lines"
)]
use super::fixtures as f;
use crate::{look, paths};
use jig_client::{Down, Up, contract as c};
use skein_lib::{List, Queue, Token};
use skein_ui::{Act, Node, Role, State, Style, Tree};
use temper_client_domain::{self as d, Address, pages};
use temper_client_protocol_wire::{self as wire, v1 as w};

const CHANGE: &str =
    "Ask temper to change any of it, in words; an owner accepts its proposal, and new work follows it.";
const NAMES: [&[u8]; 3] = [b"You approve", b"Agents review", b"Draft first"];
const LINES: [&[u8]; 3] = [b"You approve its changes.", b"Agents review its changes.", b"Start from a draft."];
const HEADLINES: [&[u8]; 3] = [b"You approve every change", b"Agents review every change, and code lands it", b""];
const REVIEW: u32 = 3;

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
fn up(root: &mut d::Domain, up: Up) {
    event(root, d::Event::Client { event: jig_client::Event::Up { up } });
}
#[expect(clippy::struct_excessive_bools, reason = "the state table varies independent page facts")]
struct Case {
    name: &'static str,
    read: u8,
    words: Option<(&'static [u8], &'static [u8])>,
    heading: &'static str,
    lede: &'static str,
    current: Option<u32>,
    shown: u32,
    goals: bool,
    budget: c::TemplateBudget,
    figure: Option<&'static [u8]>,
    may_change: bool,
    asking: bool,
    open: bool,
    roles: Option<bool>,
}
#[expect(clippy::too_many_lines, reason = "one fixture delivers the project, budget and ways watch for each state")]
fn fixture(case: &Case) -> (d::Domain, Token) {
    let (mut root, me, ..) = f::root_at_with_values(
        Address::Home { project: f::PROJECT },
        &f::domain_limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        Some(d::forge::ways::Ways { goals: case.goals, way: 11, ..d::forge::ways::Ways::STOCK }),
        false,
        false,
    );
    let mut offered = root.pages().me(root.client()).unwrap().clone();
    if case.asking {
        offered = f::asking_me(offered);
    }
    let stock = f::chat::messaging_root();
    let mut goal = stock.pages().me(stock.client()).unwrap().projects.get(0).unwrap().templates.get(0).unwrap().clone();
    goal.number = 11;
    goal.tracked = Some(1);
    goal.budget = case.budget;
    let mut templates = List::uncapped();
    for template in &offered.projects.get(0).unwrap().templates {
        templates.push(template.clone()).unwrap();
    }
    templates.push(goal).unwrap();
    offered.projects.get_mut(0).unwrap().templates = templates;
    f::deliver_me(&mut root, me, offered);
    up(
        &mut root,
        Up::FamilyServed {
            family: wire::FORGE,
            versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
        },
    );
    let out = event(&mut root, d::Event::Went { place: d::Place::Address(Address::Settings { project: f::PROJECT }) });
    if let Some(populated) = case.roles {
        let read = out
            .iter()
            .find_map(|request| {
                if let d::Request::Unlinked {
                    request:
                        jig_client::UnlinkedRequest::Down {
                            down: Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                        },
                } = request
                    && let w::Read::View(view) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
                    && view.address() == b"roles"
                {
                    Some(*read)
                } else {
                    None
                }
            })
            .expect("roles read on Settings");
        let roles = if populated {
            [
                (7, "Chat", "For each chat"),
                (2, "Coding agent", "For each change"),
                (9, "Status agent", "When you look at Home after landings"),
            ]
            .into_iter()
            .map(|(charter, name, who)| {
                let mut facts = vec![("Can", "read; never merge"), ("Asked to", "Help the person.")];
                if charter != 2 {
                    facts.insert(0, ("Model", "The model · high"));
                }
                w::CharterView::new(
                    &w::CEILINGS,
                    w::CharterViewParts {
                        charter,
                        card: w::Card::new(
                            &w::CEILINGS,
                            w::CardParts {
                                name: name.as_bytes().into(),
                                who: who.as_bytes().into(),
                                facts: List::from_boxed(
                                    facts
                                        .into_iter()
                                        .map(|(label, value)| {
                                            w::Fact::new(
                                                &w::CEILINGS,
                                                w::FactParts {
                                                    label: label.as_bytes().into(),
                                                    value: value.as_bytes().into(),
                                                },
                                            )
                                            .unwrap()
                                        })
                                        .collect(),
                                ),
                            },
                        )
                        .unwrap(),
                    },
                )
                .unwrap()
            })
            .collect()
        } else {
            Box::default()
        };
        let view = w::View::Roles(
            w::RolesView::new(&w::CEILINGS, w::RolesViewParts { roles: List::from_boxed(roles) }).unwrap(),
        );
        let answer = w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view }).unwrap();
        up(
            &mut root,
            Up::ReadDone {
                read,
                result: jig_client::Readout::Family { family: wire::FORGE, record: Some(wire::view_answer(&answer)) },
            },
        );
    }
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
        .expect("ways watch on Settings");
    if case.read == 1 {
        up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::Closed });
    } else if case.read == 2 {
        let ways = NAMES
            .into_iter()
            .zip(LINES)
            .enumerate()
            .map(|(index, (name, line))| {
                w::StockWay::new(
                    &w::CEILINGS,
                    w::StockWayParts {
                        headline: HEADLINES[index].into(),
                        name: name.into(),
                        line: line.into(),
                        path: path(index == 2),
                        items: List::from_boxed(Box::new([w::ProposalItem::new(
                            &w::CEILINGS,
                            w::ProposalItemParts {
                                kind: 7,
                                scope: f::PROJECT,
                                number: 1,
                                edit: w::SettingEdit::Removed,
                            },
                        )
                        .unwrap()])),
                        delegation: false,
                    },
                )
                .unwrap()
            })
            .collect();
        let panel = w::WaysPanel::new(
            &w::CEILINGS,
            w::WaysPanelParts {
                rules: w::PathRules::new(
                    &w::CEILINGS,
                    w::PathRulesParts { rows: List::with_capacity(0), note: Box::new([]) },
                )
                .unwrap(),
                words: case.words.map(|(headline, lede)| {
                    w::ProjectWords::new(
                        &w::CEILINGS,
                        w::ProjectWordsParts { headline: headline.into(), lede: lede.into() },
                    )
                    .unwrap()
                }),
                project: f::PROJECT,
                ways: List::from_boxed(ways),
                current: case.current,
                own: path(false),
                may_change: case.may_change,
            },
        )
        .unwrap();
        up(
            &mut root,
            Up::FamilyDelivered {
                watch,
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::ways_panel(&panel),
                },
            },
        );
        event(
            &mut root,
            d::Event::Act { action: d::Action::Pages(pages::Act::WayTab { project: f::PROJECT, tab: case.shown }) },
        );
        if case.open {
            event(
                &mut root,
                d::Event::Act { action: d::Action::Pages(pages::Act::WayStop { project: f::PROJECT, stop: REVIEW }) },
            );
        }
    }
    (root, watch)
}
fn path(draft: bool) -> w::BranchPath {
    use w::{
        Actor::{Agent, Bar, Code, Person},
        PathStep as S,
    };
    let rows = [
        (Person, if draft { S::Draft } else { S::Ask }, if draft { "you draft" } else { "you ask" }, "the person"),
        (Agent, if draft { S::ProduceMany } else { S::Produce }, "produce", "the coding agent"),
        (Code, S::Check, "check", "CI"),
        (Agent, S::Review, "review", "the reviewer"),
        (Agent, S::Repair, "repair", "the coding agent"),
        (Code, S::Conflict, "conflict", "the queue"),
        (Bar, S::Branch, if draft { "goal branch" } else { "main" }, ""),
        (Code, S::Verify, "verify", "slow"),
    ];
    let stops = rows
        .into_iter()
        .map(|(actor, step, words, name)| {
            let opens = if step == S::Review {
                w::Opens::Gate(w::OpensGate::new(&w::CEILINGS, w::OpensGateParts { number: 1, charter: 4 }).unwrap())
            } else {
                w::Opens::Nothing
            };
            let card = w::Card::new(
                &w::CEILINGS,
                w::CardParts {
                    who: b"from the engine".as_slice().into(),
                    name: name.as_bytes().into(),
                    facts: List::from_boxed(Box::new([w::Fact::new(
                        &w::CEILINGS,
                        w::FactParts { label: b"Can".as_slice().into(), value: b"read, never merge".as_slice().into() },
                    )
                    .unwrap()])),
                },
            )
            .unwrap();
            w::Stop::new(
                &w::CEILINGS,
                w::StopParts {
                    needed: step == S::Repair || step == S::Conflict,
                    over: if step == S::Repair { b"tests, review".as_slice().into() } else { Box::default() },
                    actor,
                    step,
                    words: words.as_bytes().into(),
                    opens,
                    card,
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
fn view(root: &d::Domain, browser: bool) -> crate::View {
    let words = if browser { f::browser_words() } else { f::words() };
    let limits = crate::Limits::of(&f::domain_limits(), &words).unwrap();
    let mut view = crate::View::new(&limits, &f::domain_limits(), words, f::FORGE);
    f::build(&mut view, root);
    view
}
fn subtree<'a>(tree: &'a Tree, node: &Node) -> &'a [Node] {
    let at = tree.nodes().iter().position(|n| n.id == node.id).unwrap();
    &tree.nodes()[at..at.checked_add(usize::try_from(node.size).unwrap()).unwrap()]
}
fn styled(tree: &Tree, style: Style) -> &[Node] {
    subtree(tree, tree.nodes().iter().find(|n| n.styles.as_slice().contains(&style)).unwrap())
}
fn children(nodes: &[Node]) -> Vec<&Node> {
    let mut at = 1;
    let mut result = Vec::new();
    while at < nodes.len() {
        let node = &nodes[at];
        result.push(node);
        at = at.checked_add(usize::try_from(node.size).unwrap()).unwrap();
    }
    result
}
fn named<'a>(nodes: &'a [Node], role: Role, name: &[u8]) -> &'a Node {
    nodes
        .iter()
        .find(|n| n.role == role && n.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing {role:?}: {}", String::from_utf8_lossy(name)))
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one state table checks the page's heading, drawing, budget and shared controls in both shells"
)]
fn parts_draw_the_current_way_and_the_shown_path() {
    let base = || Case {
        name: "agents",
        read: 2,
        words: None,
        heading: "Agents review every change, and code lands it",
        lede: "Shop runs the stock way Agents review. Agents review its changes.",
        current: Some(1),
        shown: 1,
        goals: true,
        budget: c::TemplateBudget::Given,
        figure: None,
        may_change: true,
        asking: true,
        open: false,
        roles: None,
    };
    for case in [
        Case { name: "reading", read: 0, heading: "How work runs in Shop.", lede: "", ..base() },
        Case { name: "ended", read: 1, heading: "How work runs in Shop.", lede: "", ..base() },
        base(),
        Case { name: "roles", roles: Some(true), ..base() },
        Case { name: "empty roles", roles: Some(false), ..base() },
        Case {
            name: "draft amount, empty headline",
            heading: "Draft first",
            lede: "Shop runs the stock way Draft first. Start from a draft.",
            current: Some(2),
            shown: 2,
            budget: c::TemplateBudget::Amount(80),
            figure: Some(b"80"),
            ..base()
        },
        Case {
            name: "cap",
            budget: c::TemplateBudget::AsAllowed { cap: Some(90) },
            figure: Some(b"up to 90"),
            ..base()
        },
        Case { name: "uncapped", budget: c::TemplateBudget::AsAllowed { cap: None }, ..base() },
        Case { name: "goals hidden", goals: false, budget: c::TemplateBudget::Amount(80), ..base() },
        Case {
            name: "Yours without words",
            current: None,
            shown: 3,
            heading: "A way of its own",
            lede: "Shop runs settings of its own, drawn below.",
            ..base()
        },
        Case {
            name: "Yours with words",
            current: None,
            shown: 3,
            words: Some((b"Our own headline", b"Our own lede.")),
            heading: "Our own headline",
            lede: "Our own lede.",
            ..base()
        },
        Case { name: "stock ignores project words", words: Some((b"Our own headline", b"Our own lede.")), ..base() },
        Case { name: "open reviewer", open: true, ..base() },
        Case { name: "member", may_change: false, ..base() },
        Case { name: "Ask absent", asking: false, ..base() },
        Case { name: "shown differs", shown: 0, ..base() },
    ] {
        let (root, _) = fixture(&case);
        for (browser, mut view) in [view(&root, false), view(&root, true)].into_iter().enumerate() {
            let tree = view.tree();
            assert_eq!(skein_ui::outline::check(tree), None, "{}", case.name);
            let main = subtree(tree, tree.nodes().iter().find(|n| n.role == Role::Main).unwrap());
            named(main, Role::Heading { level: 1 }, case.heading.as_bytes());
            let lede = if case.lede.is_empty() { CHANGE.to_owned() } else { format!("{} {CHANGE}", case.lede) };
            named(styled(tree, look::SETTINGS_INTRO), Role::Paragraph, lede.as_bytes());
            assert!(styled(tree, look::SETTINGS_COL)[0].styles.as_slice().contains(&look::HOW_WIDE));
            assert_eq!(main.iter().any(|n| n.name.as_deref() == Some(b"Ask temper to change this")), case.asking);
            let headings: Vec<_> = main
                .iter()
                .filter(|n| n.role == Role::Heading { level: 2 })
                .map(|n| n.name.as_deref().unwrap())
                .collect();
            let mut expected = vec![b"The path of a change".as_slice(), b"What reaches you"];
            if case.roles == Some(true) {
                expected.push(b"Roles");
            }
            if case.figure.is_some() {
                expected.push(b"Budgets");
            }
            expected.push(b"What the project shows");
            if case.goals {
                expected.push(b"Ways of working");
            }
            assert_eq!(headings, expected, "{}", case.name);
            for removed in
                [b"Status".as_slice(), b"Funds", b"Telling you", b"Not built yet: this is what temper does today."]
            {
                assert!(!main.iter().any(|n| n.name.as_deref() == Some(removed)));
            }
            let mut role_links = Vec::new();
            if case.roles == Some(true) {
                named(main, Role::Text, b"every agent Shop runs");
                let rows = children(styled(tree, look::HOW_ROLES));
                assert_eq!(
                    rows.iter().map(|row| row.name.as_deref().unwrap()).collect::<Vec<_>>(),
                    [b"Chat".as_slice(), b"Coding agent", b"Status agent"]
                );
                for (row, (charter, who, model)) in rows.into_iter().zip([
                    (7, "For each chat", true),
                    (2, "For each change", false),
                    (9, "When you look at Home after landings", true),
                ]) {
                    assert_eq!(row.role, Role::Disclosure);
                    assert_eq!(row.description.as_deref(), Some(who.as_bytes()));
                    assert!(!row.states.as_slice().contains(&State::Expanded), "starts closed");
                    let opened = subtree(tree, row);
                    assert_eq!(
                        opened
                            .iter()
                            .filter(|node| node.role == Role::Term)
                            .map(|node| node.name.as_deref().unwrap())
                            .collect::<Vec<_>>(),
                        if model {
                            vec![b"Model".as_slice(), b"Can", b"Asked to"]
                        } else {
                            vec![b"Can".as_slice(), b"Asked to"]
                        }
                    );
                    named(opened, Role::Description, b"read; never merge");
                    named(opened, Role::Description, b"Help the person.");
                    if model {
                        named(opened, Role::Description, "The model · high".as_bytes());
                    }
                    let link = named(opened, Role::Link, "Read its instructions ›".as_bytes());
                    role_links.push((link.id, charter));
                }
            } else {
                assert!(!main.iter().any(|node| node.styles.as_slice().contains(&look::HOW_ROLES)));
            }
            if let Some(figure) = case.figure {
                assert_eq!(
                    f::nodes_text(styled(tree, look::HOW_ROW)).as_ref(),
                    [b"Goalseach given its own when it starts".as_slice(), figure].concat()
                );
            }
            if case.read < 2 {
                assert!(!main.iter().any(|n| n.role == Role::Table || n.role == Role::Tabs));
                assert_eq!(
                    main.iter().any(|n| n.name.as_deref() == Some(b"The ways can't be read now.")),
                    case.read == 1
                );
                continue;
            }
            let way =
                case.current.map_or("its own", |i| core::str::from_utf8(NAMES[usize::try_from(i).unwrap()]).unwrap());
            assert_eq!(f::nodes_text(styled(tree, look::HOW_WAY)).as_ref(), format!("the way: {way}").as_bytes());
            let tabs = children(styled(tree, look::WAYS_TABS));
            assert_eq!(tabs.len(), if case.current.is_none() { 4 } else { 3 });
            for (index, tab) in tabs.iter().enumerate() {
                assert_eq!(
                    tab.states.as_slice().contains(&State::Selected),
                    index == usize::try_from(case.shown).unwrap()
                );
                let now = case.current.map_or(index == 3, |i| index == usize::try_from(i).unwrap());
                let name = if index == 3 { b"Yours".as_slice() } else { NAMES[index] };
                assert_eq!(
                    f::nodes_text(subtree(tree, tab)).as_ref(),
                    [
                        name,
                        if !now {
                            b"".as_slice()
                        } else if browser == 1 {
                            b"now"
                        } else {
                            b", now"
                        }
                    ]
                    .concat()
                );
            }
            let panel = styled(tree, look::WAYS_PANEL);
            assert!(!panel.iter().any(|n| n.role == Role::Table));
            let drawing = panel
                .iter()
                .find(|n| n.drawing.as_ref().is_some_and(|d| d.kind == paths::KIND))
                .expect("shown way has a path drawing");
            let mut scene = skein_ui::scene::Scene::new(&view.limits().tree);
            assert!(paths::draw(paths::KIND, &drawing.drawing.as_ref().unwrap().data, 720, 0, &mut scene));
            assert_eq!(
                scene.height(),
                Some(if case.open { 154 } else { 140 }),
                "open card has 14 pixels for its caret"
            );
            assert_eq!(
                (0..scene.marks().len())
                    .any(|index| scene.styles(u32::try_from(index).unwrap()).contains(&look::WAYS_CARET)),
                case.open
            );
            let drawing_id = drawing.id;
            let fallback = subtree(tree, drawing);
            let draft = case.shown == 2;
            let expected = [
                if draft { "you draft, a person" } else { "you ask, a person" },
                "produce, an agent",
                "check, code",
                "review, an agent",
                "repair, an agent, when needed (tests, review)",
                "conflict, code, when needed",
                if draft { "goal branch, a branch" } else { "main, a branch" },
                "verify, code",
            ];
            assert_eq!(
                fallback
                    .iter()
                    .filter(|n| n.role == Role::Item)
                    .map(|n| core::str::from_utf8(n.name.as_deref().unwrap()).unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
            let mut buttons = Vec::new();
            for (index, item) in fallback.iter().filter(|n| n.role == Role::Item).enumerate() {
                let content = children(subtree(tree, item));
                if index == 6 {
                    assert_eq!(content[0].role, Role::Text, "a branch bar has no act");
                } else {
                    let button = content[0];
                    assert_eq!(button.role, Role::Button);
                    assert_eq!(button.name, item.name);
                    assert_eq!(
                        button.states.as_slice().contains(&State::Expanded),
                        case.open && index == usize::try_from(REVIEW).unwrap()
                    );
                    buttons.push((button.id, u32::try_from(index).unwrap()));
                }
            }
            let foot = children(styled(tree, look::WAYS_FOOT));
            assert!(foot[0].styles.as_slice().contains(&look::WAYS_LEGEND), "legend precedes the way's act");
            assert_eq!(
                f::nodes_text(styled(tree, look::WAYS_LEGEND)).as_ref(),
                if browser == 1 {
                    b"agentwhen neededpersoncodeno merge tool above".as_slice()
                } else {
                    b"agent, when needed, person, code; no merge tool above".as_slice()
                }
            );
            let panel_children = children(panel);
            let drawing_at = panel_children.iter().position(|n| n.id == drawing.id).unwrap();
            assert!(
                panel_children[drawing_at + 1 + usize::from(case.open)].styles.as_slice().contains(&look::WAYS_FOOT)
            );
            if case.open {
                assert!(
                    panel_children[drawing_at + 1].styles.as_slice().contains(&look::WAYS_ZOOM),
                    "card follows drawing, leaving its caret's room"
                );
            }
            let act = match case.current {
                Some(current) if case.may_change => {
                    Some(if case.shown == current { b"Keep it".as_slice() } else { b"Use this".as_slice() })
                }
                Some(_) | None => None,
            };
            assert_eq!(
                main.iter().find(|n| n.styles.as_slice().contains(&look::WAYS_USE)).and_then(|n| n.name.as_deref()),
                act
            );
            assert_eq!(main.iter().any(|n| n.styles.as_slice().contains(&look::WAYS_ZOOM)), case.open);
            if case.open {
                named(main, Role::Link, "Open the reviewer ›".as_bytes());
            }
            if browser == 0 {
                let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
                skein_ui_text::render(tree, b"temper", None, &mut print);
                let text = core::str::from_utf8(print.text.as_slice()).unwrap();
                let lines: Vec<_> = text.lines().collect();
                let actual: Vec<_> = lines
                    .iter()
                    .filter_map(|line| expected.iter().find(|words| line.contains(**words)).copied())
                    .collect();
                assert_eq!(actual, expected, "one fallback line per stop, in path order: {text}");
                if case.roles == Some(true) {
                    let mut after = 0;
                    for name in ["Chat", "Coding agent", "Status agent"] {
                        after += text[after..].find(name).expect("role prints in order") + name.len();
                    }
                    assert!(text.contains("Read its instructions ›"));
                }
                if case.open {
                    assert!(text.contains("Open the reviewer ›"));
                }
                if let Some(act) = act {
                    assert!(text.contains(core::str::from_utf8(act).unwrap()));
                }
            }
            for (id, charter) in role_links {
                assert_eq!(
                    crate::act(&mut view, id, Act::Press),
                    Some(d::Action::Pages(pages::Act::Go { address: Address::Role { project: f::PROJECT, charter } }))
                );
            }
            assert_eq!(
                crate::act(&mut view, drawing_id, Act::Drawing { data: Box::new([u8::try_from(REVIEW).unwrap()]) }),
                Some(d::Action::Pages(pages::Act::WayStop { project: f::PROJECT, stop: REVIEW }))
            );
            for (id, stop) in buttons {
                assert_eq!(
                    crate::act(&mut view, id, Act::Press),
                    Some(d::Action::Pages(pages::Act::WayStop { project: f::PROJECT, stop }))
                );
            }
            if case.asking {
                let id = named(view.tree().nodes(), Role::Link, b"Ask temper to change this").id;
                assert_eq!(crate::href(&mut view, id).as_deref(), Some(b"/projects/1/ask".as_slice()));
                assert_eq!(
                    crate::act(&mut view, id, Act::Press),
                    Some(d::Action::Pages(pages::Act::Go { address: Address::Ask { project: f::PROJECT } }))
                );
            }
        }
    }
    // The shared page drawing and an unpressed reply read the same wire marks.
    let (mut root, _) = fixture(&base());
    event(&mut root, d::Event::Went { place: d::Place::Address(Address::Ask { project: f::PROJECT }) });
    for browser in [false, true] {
        let ask = view(&root, browser);
        let words = if browser { f::browser_words() } else { f::words() };
        let mut builder = skein_ui::Builder::new(&ask.limits().tree);
        let shown = pages::chat::Shown::Drawn(Box::new(w::View::Path(path(false))));
        crate::views::block(&mut builder, b"path main", Some(&shown), &words);
        let reply = builder.finish();
        for (nodes, pressed) in [(ask.tree().nodes(), true), (reply.nodes(), false)] {
            let drawing = nodes
                .iter()
                .find(|node| node.drawing.as_ref().is_some_and(|d| d.kind == paths::KIND))
                .expect("wire path drawn");
            assert_eq!(drawing.binding.is_some(), pressed);
            for line in [b"conflict, code, when needed".as_slice(), b"repair, an agent, when needed (tests, review)"] {
                named(nodes, Role::Item, line);
            }
        }
    }
}
