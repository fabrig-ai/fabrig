//! How work runs' parts in both shells (temper/docs/design/client/README.md, 5.13).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic tables and terminal lines"
)]
use super::fixtures as f;
use crate::look;
use jig_client::{Down, Up, contract as c};
use skein_lib::{List, Queue, Token};
use skein_ui::{Act, Node, Role, State, Style, Tree};
use temper_client_domain::{self as d, Address, pages};
use temper_client_protocol_wire::{self as wire, v1 as w};

const CHANGE: &str =
    "Ask temper to change any of it, in words; an owner accepts its proposal, and new work follows it.";
const NAMES: [&[u8]; 3] = [b"You approve", b"Agents review", b"Draft first"];
const LINES: [&[u8]; 3] = [b"You approve its changes.", b"Agents review its changes.", b"Start from a draft."];
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
#[expect(clippy::struct_excessive_bools, reason = "the state table varies four independent page facts")]
struct Case {
    name: &'static str,
    read: u8,
    current: Option<u32>,
    shown: u32,
    goals: bool,
    budget: c::TemplateBudget,
    figure: Option<&'static [u8]>,
    may_change: bool,
    asking: bool,
    open: bool,
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
        Some(d::forge::ways::Ways { goals: case.goals, away: true, way: 11, ..d::forge::ways::Ways::STOCK }),
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
            w::Stop::new(&w::CEILINGS, w::StopParts { actor, step, words: words.as_bytes().into(), opens, card })
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
    reason = "one state table checks the page's heading, lanes, budget and shared controls in both shells"
)]
fn parts_draw_the_current_way_and_the_shown_path_in_lanes() {
    let base = || Case {
        name: "agents",
        read: 2,
        current: Some(1),
        shown: 1,
        goals: true,
        budget: c::TemplateBudget::Given,
        figure: None,
        may_change: true,
        asking: true,
        open: false,
    };
    for case in [
        Case { name: "reading", read: 0, ..base() },
        Case { name: "ended", read: 1, ..base() },
        base(),
        Case {
            name: "draft amount",
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
        Case { name: "Yours", current: None, shown: 3, ..base() },
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
            let heading = if case.read < 2 {
                "How work runs in Shop.".as_bytes()
            } else {
                case.current.map_or(b"A way of its own".as_slice(), |i| NAMES[usize::try_from(i).unwrap()])
            };
            named(main, Role::Heading { level: 1 }, heading);
            let lede = if case.read < 2 {
                CHANGE.to_owned()
            } else {
                let line = case.current.map_or("Shop runs settings of its own, drawn below.", |i| {
                    core::str::from_utf8(LINES[usize::try_from(i).unwrap()]).unwrap()
                });
                format!("{line} {CHANGE}")
            };
            named(styled(tree, look::SETTINGS_INTRO), Role::Paragraph, lede.as_bytes());
            assert!(styled(tree, look::SETTINGS_COL)[0].styles.as_slice().contains(&look::HOW_WIDE));
            assert_eq!(main.iter().any(|n| n.name.as_deref() == Some(b"Ask temper to change this")), case.asking);
            let headings: Vec<_> = main
                .iter()
                .filter(|n| n.role == Role::Heading { level: 2 })
                .map(|n| n.name.as_deref().unwrap())
                .collect();
            let mut expected = vec![b"The path of a change".as_slice()];
            if case.figure.is_some() {
                expected.push(b"Budgets");
            }
            expected.extend([
                "What needs you".as_bytes(),
                "While you’re away, agents may decide".as_bytes(),
                b"What the project shows",
            ]);
            if case.goals {
                expected.push(b"Ways of working");
            }
            assert_eq!(headings, expected, "{}", case.name);
            for removed in
                [b"Status".as_slice(), b"Funds", b"Telling you", b"Not built yet: this is what temper does today."]
            {
                assert!(!main.iter().any(|n| n.name.as_deref() == Some(removed)));
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
            let rows = children(styled(tree, look::HOW_PATH));
            assert_eq!(rows.len(), 5);
            let draft = case.shown == 2;
            let headers = children(subtree(tree, rows[0]));
            assert_eq!(
                headers.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
                [
                    b"".as_slice(),
                    if draft { b"Draft" } else { b"Ask" },
                    b"Produce",
                    b"Check",
                    b"Review",
                    if draft { b"goal branch" } else { b"main" },
                    b"Verify"
                ]
            );
            assert_eq!(children(subtree(tree, rows[3])).len(), 1);
            let authority = children(subtree(tree, rows[3]))[0];
            assert_eq!(authority.spans, Some(skein_ui::Spans { rows: 1, columns: 7 }));
            assert!(authority.styles.as_slice().contains(&look::HOW_LINE));
            assert_eq!(f::nodes_text(subtree(tree, authority)).as_ref(), b"authority: no merge tool above this line");
            let mut buttons = Vec::new();
            for (row, label, style, stops) in [
                (
                    1,
                    b"You".as_slice(),
                    look::HOW_PERSON,
                    vec![(0, if draft { b"you draft".as_slice() } else { b"you ask" })],
                ),
                (2, b"Agents", look::HOW_AGENT, vec![(1, b"the coding agent".as_slice()), (3, b"the reviewer")]),
                (4, b"Code", look::HOW_CODE, vec![(2, b"CI".as_slice()), (5, b"slow")]),
            ] {
                let cells = children(subtree(tree, rows[row]));
                assert_eq!(cells.len(), 7);
                assert_eq!(cells[0].name.as_deref(), Some(label));
                assert_eq!(cells[0].styles.as_slice(), &[look::HOW_LANE]);
                for (column, cell) in cells.iter().skip(1).enumerate() {
                    assert_eq!(cell.styles.as_slice().contains(&look::HOW_BAR), column == 4);
                    let content = children(subtree(tree, cell));
                    if let Some((stop, name)) = stops.iter().find(|(stop, _)| *stop == column) {
                        assert_eq!(content.len(), 1, "{} {label:?} column {column}", case.name);
                        let button = content[0];
                        assert_eq!(button.size, if browser == 0 { 2 } else { 1 }, "one visible label per stop");
                        assert_eq!(button.name.as_deref(), Some(*name));
                        assert_eq!(button.styles.as_slice(), &[style]);
                        assert_eq!(
                            button.states.as_slice().contains(&State::Current),
                            case.open && column == usize::try_from(REVIEW).unwrap()
                        );
                        buttons.push((button.id, u32::try_from(*stop).unwrap()));
                    } else {
                        assert!(content.is_empty());
                    }
                }
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
                for label in ["You |", "Agents |", "Code |", "authority: no merge tool above this line"] {
                    assert_eq!(lines.iter().filter(|line| line.contains(label)).count(), 1, "{text}");
                }
                assert!(lines.iter().any(|line| line.contains(" | Produce | Check | Review |")), "{text}");
                for (label, words) in [
                    ("You |", vec![if draft { "you draft" } else { "you ask" }]),
                    ("Agents |", vec!["the coding agent", "the reviewer"]),
                    ("Code |", vec!["CI", "slow"]),
                ] {
                    let line = lines.iter().find(|line| line.contains(label)).unwrap();
                    for word in words {
                        assert!(line.contains(word), "{line}");
                    }
                }
                if case.open {
                    assert!(text.contains("Open the reviewer ›"));
                }
                if let Some(act) = act {
                    assert!(text.contains(core::str::from_utf8(act).unwrap()));
                }
            }
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
}
