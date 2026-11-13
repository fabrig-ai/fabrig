//! Model's forge pane (temper/docs/design/client/pages/README.md).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "test assertions inspect the semantic tree and its printed words"
)]
use super::{connect_forge as forge, fixtures as f};
use crate::look;
use skein_ui::{Fraction, Node, Role, Run, Tree};
use temper_client_domain::{Address, Step};
use temper_client_protocol_wire::v1 as w;

fn underway() -> skein_lib::List<w::ForgeRow> {
    forge::first_rows(
        w::Connection::SettingUp(
            w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(w::SetUpStep::Fetch), fetched: None })
                .unwrap(),
        ),
        w::FirstState::Making,
    )
}
fn fetching(binary: w::SetUpBinary, total: Option<u64>) -> w::Connection {
    w::Connection::SettingUp(
        w::SettingUp::new(
            &w::CEILINGS,
            w::SettingUpParts {
                step: Some(w::SetUpStep::Fetch),
                fetched: Some(
                    w::SetUpFetched::new(&w::CEILINGS, w::SetUpFetchedParts { binary, done: 41_999_999, total })
                        .unwrap(),
                ),
            },
        )
        .unwrap(),
    )
}
fn fetch_rows(binary: w::SetUpBinary, total: Option<u64>) -> skein_lib::List<w::ForgeRow> {
    forge::first_rows(fetching(binary, total), w::FirstState::Making)
}
#[test]
fn the_fetch_fills_its_bar_and_says_its_figure() {
    for (binary, name) in
        [(w::SetUpBinary::Forgejo, "Fetching Forgejo"), (w::SetUpBinary::Runner, "Fetching its runner")]
    {
        let view = browser(fetch_rows(binary, Some(99_000_000)));
        let bars: Vec<_> = view.tree().nodes().iter().filter(|n| n.role == Role::Progress).collect();
        assert_eq!(bars.len(), 1);
        let bar = bars[0];
        assert_eq!(bar.styles.as_slice(), &[look::FLOW_METER]);
        assert_eq!(bar.name.as_deref(), Some(name.as_bytes()));
        assert_eq!(bar.text.len(), 1);
        assert_eq!(bar.text.as_slice()[0].run, Run::Plain);
        assert_eq!(bar.text.as_slice()[0].text.as_ref(), b"42%");
        assert_eq!(bar.fraction, Some(Fraction { done: 41_999_999, total: 99_000_000 }));
        assert_eq!(bar.size, 1);
    }
}
#[test]
fn with_no_size_the_bar_keeps_moving_and_says_its_bytes() {
    for (binary, name) in
        [(w::SetUpBinary::Forgejo, "Fetching Forgejo"), (w::SetUpBinary::Runner, "Fetching its runner")]
    {
        let view = browser(fetch_rows(binary, None));
        let bars: Vec<_> = view.tree().nodes().iter().filter(|n| n.role == Role::Progress).collect();
        assert_eq!(bars.len(), 1);
        let bar = bars[0];
        assert_eq!(bar.styles.as_slice(), &[look::FLOW_METER]);
        assert_eq!(bar.name.as_deref(), Some(name.as_bytes()));
        assert_eq!(bar.text.len(), 1);
        assert_eq!(bar.text.as_slice()[0].run, Run::Plain);
        assert_eq!(bar.text.as_slice()[0].text.as_ref(), b"41 MB");
        assert_eq!(bar.fraction, None);
        assert_eq!(bar.size, 1);
    }
}
#[test]
fn a_fetch_no_line_has_reached_keeps_the_busy_bar() {
    for step in [None, Some(w::SetUpStep::Fetch), Some(w::SetUpStep::Prepare), Some(w::SetUpStep::Run)] {
        let connection = w::Connection::SettingUp(
            w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step, fetched: None }).unwrap(),
        );
        let view = browser(forge::first_rows(connection, w::FirstState::Making));
        let bars: Vec<_> = view.tree().nodes().iter().filter(|n| n.role == Role::Progress).collect();
        assert_eq!(bars.len(), 1);
        let bar = bars[0];
        assert_eq!(bar.styles.as_slice(), &[look::FLOW_BUSY]);
        assert!(bar.text.is_empty());
        assert_eq!(bar.fraction, None);
        assert_eq!(bar.size, 1);
    }
}
#[test]
fn the_detail_names_the_binary_and_the_name_says_both() {
    for (binary, detail) in [
        (w::SetUpBinary::Forgejo, "Setting up: fetching Forgejo"),
        (w::SetUpBinary::Runner, "Setting up: fetching its runner"),
    ] {
        for (total, figure) in [(Some(99_000_000), "42%"), (None, "41 MB")] {
            let view = browser(fetch_rows(binary.clone(), total));
            let line = ["Forgejo on this machine · work/hello · ", detail, " · ", figure].concat();
            assert_stack(view.tree(), &line, &["work/hello", detail], false);
        }
    }
}
#[test]
fn the_first_step_says_the_figure_until_it_is_fetched() {
    for (step, total, suffix) in [
        (w::SetUpStep::Fetch, Some(99_000_000), " · 42%"),
        (w::SetUpStep::Fetch, None, " · 41 MB"),
        (w::SetUpStep::Prepare, Some(99_000_000), ""),
    ] {
        for pins in [false, true] {
            let connection = if step == w::SetUpStep::Fetch {
                fetching(w::SetUpBinary::Forgejo, total)
            } else {
                setting_up(step.clone())
            };
            let mut rows = forge::first_rows(connection, w::FirstState::Making);
            if pins {
                let mut parts = rows.as_slice()[0].clone().into_parts();
                parts.owned = Some(
                    w::OwnedForge::new(
                        &w::CEILINGS,
                        w::OwnedForgeParts {
                            forgejo: b"17.0.1".as_slice().into(),
                            runner: Some(b"13.2.1".as_slice().into()),
                            password: b"/fake/first-password".as_slice().into(),
                        },
                    )
                    .unwrap(),
                );
                rows = skein_lib::List::with_capacity(1);
                rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
            }
            let welcome = root(rows.clone(), None);
            let connect =
                f::forge_root(Address::ConnectForge { forge: Some(temper_client_domain::Choice::Forge(3)) }, rows);
            for root in [&welcome, &connect] {
                for view in [f::browser_page_view(root), f::page_view(root)] {
                    let list = if *root.pages().address() == Address::Forges {
                        let nodes = view.tree().nodes();
                        let (start, node) = nodes
                            .iter()
                            .enumerate()
                            .find(|(_, n)| n.styles.as_slice().contains(&look::CONNECT_LIVE))
                            .unwrap();
                        &nodes[start..start + usize::try_from(node.size).unwrap()]
                    } else {
                        subtree(view.tree(), Role::List, b"Each step")
                    };
                    let first = list.iter().find(|n| !n.text.is_empty()).unwrap();
                    let verb = if step == w::SetUpStep::Fetch { "Fetching" } else { "Fetched" };
                    let versions = if pins { "Forgejo 17.0.1 and its runner 13.2.1" } else { "Forgejo and its runner" };
                    assert_eq!(
                        first.text.as_slice()[0].text.as_ref(),
                        [verb, " ", versions, suffix].concat().as_bytes()
                    );
                }
            }
        }
    }
}
fn failed() -> skein_lib::List<w::ForgeRow> {
    forge::first_rows(
        w::Connection::SetUpFailed(
            w::SetUpFailed::new(
                &w::CEILINGS,
                w::SetUpFailedParts { why: b"fetch failed".as_slice().into(), at: Some(w::FailedStep::Fetch) },
            )
            .unwrap(),
        ),
        w::FirstState::Making,
    )
}
fn root(
    rows: skein_lib::List<w::ForgeRow>,
    way: Option<temper_client_domain::ModelWay>,
) -> temper_client_domain::Domain {
    let (mut root, watch, _, _, _, _) = f::root_at_with_forges(
        Address::Welcome { step: Step::Model { way } },
        &f::domain_limits(),
        b"Shop",
        None,
        None,
        Some(rows),
    );
    let plans = super::connect_model::plans_root();
    let me = temper_client_domain::pages::accounts::current_me(plans.client()).unwrap().clone();
    f::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        me,
                    ))),
                },
            },
        },
    );
    root
}
fn browser(rows: skein_lib::List<w::ForgeRow>) -> crate::View {
    f::browser_page_view(&root(rows, None))
}
fn subtree<'a>(tree: &'a Tree, role: Role, name: &[u8]) -> &'a [Node] {
    let start =
        tree.nodes().iter().position(|n| n.role == role && n.name.as_deref() == Some(name)).expect("named node");
    &tree.nodes()[start..start.checked_add(usize::try_from(tree.nodes()[start].size).unwrap()).unwrap()]
}
fn region(tree: &Tree) -> &[Node] {
    subtree(tree, Role::Complementary, b"Forge and model")
}
fn paragraph(tree: &Tree) -> &[Node] {
    let name = region(tree).iter().find(|n| n.role == Role::Paragraph).unwrap().name.as_deref().unwrap();
    subtree(tree, Role::Paragraph, name)
}
fn assert_stack(tree: &Tree, name: &str, details: &[&str], done: bool) {
    let paragraph = paragraph(tree);
    assert_eq!(paragraph[0].name.as_deref(), Some(name.as_bytes()));
    assert_eq!(paragraph[0].styles.as_slice().contains(&look::FLOW_DONE), done);
    let texts: Vec<_> = paragraph.iter().filter(|n| n.role == Role::Text).collect();
    assert_eq!(texts.len(), details.len().checked_add(1).unwrap());
    assert_eq!(texts[0].text.len(), 1);
    assert_eq!(texts[0].text.as_slice()[0].run, Run::Strong);
    assert_eq!(texts[0].text.as_slice()[0].text.as_ref(), name.split(" · ").next().unwrap().as_bytes());
    for (node, detail) in texts[1..].iter().zip(details) {
        assert!(node.styles.as_slice().contains(&look::FLOW_MUTED));
        assert_eq!(node.text.len(), 1);
        assert_eq!(node.text.as_slice()[0].text.as_ref(), detail.as_bytes());
    }
    for node in paragraph {
        for run in &node.text {
            let text = String::from_utf8_lossy(&run.text);
            assert!(!text.contains('✓') && !text.contains('·'));
        }
    }
}
#[test]
fn the_forge_is_a_region_after_the_choices_and_before_the_foot() {
    for way in [
        None,
        Some(temper_client_domain::ModelWay { provider: 17, kind: temper_client_domain::WayKind::Key, source: None }),
    ] {
        let root = root(underway(), way);
        let view = f::browser_page_view(&root);
        let nodes = view.tree().nodes();
        let heading = nodes.iter().position(|n| n.role == Role::Heading { level: 1 }).unwrap();
        let body = nodes
            .iter()
            .rposition(|n| {
                n.styles.as_slice().contains(&look::FLOW_CARD)
                    || n.styles.as_slice().contains(&look::FLOW_CHOICE)
                    || n.styles.as_slice().contains(&look::WELCOME_PANEL)
            })
            .unwrap();
        let pane = nodes.iter().position(|n| n.role == Role::Complementary).unwrap();
        let back =
            nodes.iter().position(|n| n.role == Role::Link && n.name.as_deref() == Some("‹ Back".as_bytes())).unwrap();
        assert!(heading < body && body < pane && pane < back);
        assert!(nodes[pane].styles.as_slice().contains(&look::WELCOME_STATUS));
        assert_eq!(nodes[pane].name.as_deref(), Some(b"Forge and model".as_slice()));
        assert!(!nodes[..heading].iter().any(|n| n.styles.as_slice().contains(&look::WELCOME_STATUS)));
    }
}
#[test]
fn under_way_the_browser_stacks_the_line() {
    let view = browser(underway());
    assert_stack(
        view.tree(),
        "Forgejo on this machine · work/hello · Setting up: fetching",
        &["work/hello", "Setting up: fetching"],
        false,
    );
    assert!(region(view.tree()).iter().any(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Each step")));
}
#[test]
fn done_is_marked_by_style_alone() {
    let view = browser(forge::first_rows(forge::connected(), forge::made()));
    assert_stack(
        view.tree(),
        "Forgejo on this machine · connected as temper · work/hello made",
        &["connected as temper", "work/hello made"],
        true,
    );
    assert!(!region(view.tree()).iter().any(|n| n.role == Role::Disclosure));
}
#[test]
fn a_connected_github_has_its_one_detail() {
    let view = browser(forge::github_rows(true));
    assert_stack(view.tree(), "GitHub · Connected as my-app[bot] (9)", &["Connected as my-app[bot] (9)"], true);
    assert!(!region(view.tree()).iter().any(|n| n.role == Role::Disclosure));
}
#[test]
fn failed_keeps_its_shape_and_its_steps() {
    let view = browser(failed());
    assert_stack(
        view.tree(),
        "Forgejo on this machine · work/hello · Set up failed: fetch failed",
        &["work/hello", "Set up failed: fetch failed"],
        false,
    );
    assert!(region(view.tree()).iter().any(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Each step")));
}
#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::print_stdout,
    reason = "the terminal prints are read in the landing account"
)]
fn the_terminal_says_the_status_with_no_bar() {
    for (state, rows, line, steps) in [
        (
            "fetch-percent",
            fetch_rows(w::SetUpBinary::Forgejo, Some(99_000_000)),
            "Forgejo on this machine · work/hello · Setting up: fetching Forgejo · 42%",
            true,
        ),
        (
            "fetch-bytes",
            fetch_rows(w::SetUpBinary::Runner, None),
            "Forgejo on this machine · work/hello · Setting up: fetching its runner · 41 MB",
            true,
        ),
        ("under-way", underway(), "Forgejo on this machine · work/hello · Setting up: fetching", true),
        (
            "done",
            forge::first_rows(forge::connected(), forge::made()),
            "✓ Forgejo on this machine · connected as temper · work/hello made",
            false,
        ),
        ("failed", failed(), "Forgejo on this machine · work/hello · Set up failed: fetch failed", true),
    ] {
        let root = root(rows, None);
        let view = f::page_view(&root);
        assert!(!view.tree().nodes().iter().any(|node| node.role == Role::Progress));
        let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
        skein_ui_text::render(view.tree(), b"temper", None, &mut print);
        let text = String::from_utf8_lossy(print.text.as_slice());
        println!("{state}:\n{text}");
        assert!(!text.contains("Setting up: fetching: "));
        let heading = text.find("Connect a model").unwrap();
        let choices = text.rfind("Claude").unwrap();
        let forge = text.find("Forge and model:\n").unwrap();
        let words = text.find(line).unwrap();
        let back = text.find("‹ Back").unwrap();
        assert!(heading < choices && choices < forge && forge < words && words < back);
        assert!(text.lines().any(|row| row.trim_start() == line && row.starts_with(' ')));
        assert_eq!(text.contains("Each step"), steps);
        assert!(text.lines().any(|row| row.trim() == "Model none yet" && row.starts_with(' ')));
        assert!(words < text.find("Model none yet").unwrap() && text.find("Model none yet").unwrap() < back);
    }
}

#[test]
fn the_bar_sits_between_the_line_and_each_step() {
    for (connection, status) in [
        (setting_up(w::SetUpStep::Fetch), "Setting up: fetching"),
        (setting_up(w::SetUpStep::Prepare), "Setting up: preparing"),
        (setting_up(w::SetUpStep::Run), "Setting up: starting"),
        (w::Connection::Connecting, "Connecting"),
        (forge::connected(), "Connected as temper (42)"),
    ] {
        let view = browser(forge::first_rows(connection, w::FirstState::Making));
        let tree = view.tree();
        assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::Progress).count(), 1);
        let nodes = region(tree);
        let line = nodes.iter().position(|node| node.role == Role::Paragraph).unwrap();
        let bar = nodes.iter().position(|node| node.role == Role::Progress).unwrap();
        let steps = nodes.iter().position(|node| node.role == Role::Disclosure).unwrap();
        assert_eq!(bar, line.checked_add(usize::try_from(nodes[line].size).unwrap()).unwrap());
        assert_eq!(bar.checked_add(1).unwrap(), steps);
        assert_eq!(nodes[bar].styles.as_slice(), &[look::FLOW_BUSY]);
        assert_eq!(nodes[bar].name.as_deref(), Some(status.as_bytes()));
        assert!(nodes[bar].text.is_empty());
        assert!(nodes[bar].value.is_none());
        assert_eq!(nodes[bar].size, 1);
        assert_eq!(nodes[steps].name.as_deref(), Some(b"Each step".as_slice()));
    }
}
fn setting_up(step: w::SetUpStep) -> w::Connection {
    w::Connection::SettingUp(
        w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(step), fetched: None }).unwrap(),
    )
}
#[test]
fn done_failed_and_a_waiting_github_have_no_bar() {
    for rows in [forge::first_rows(forge::connected(), forge::made()), failed(), forge::github_rows(false)] {
        let view = browser(rows);
        assert!(!view.tree().nodes().iter().any(|node| node.role == Role::Progress));
    }
}
#[test]
fn the_line_says_the_repository_before_the_status() {
    for (rows, line, status) in [
        (underway(), "Forgejo on this machine · work/hello · Setting up: fetching", "Setting up: fetching"),
        (failed(), "Forgejo on this machine · work/hello · Set up failed: fetch failed", "Set up failed: fetch failed"),
    ] {
        let view = browser(rows);
        assert_stack(view.tree(), line, &["work/hello", status], false);
    }
}

#[test]
fn the_claude_card_says_this_machine() {
    let root = root(underway(), None);
    let browser = f::browser_page_view(&root);
    let card = browser
        .tree()
        .nodes()
        .iter()
        .find(|node| node.styles.as_slice().contains(&look::FLOW_CHOICE) && node.name.as_deref() == Some(b"Claude"))
        .expect("Claude card");
    let nodes = subtree(browser.tree(), card.role, b"Claude");
    let terminal = f::page_view(&root);
    let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
    skein_ui_text::render(terminal.tree(), b"temper", None, &mut print);
    let text = String::from_utf8_lossy(print.text.as_slice());
    for sentence in [
        "Your Claude subscription, through a Claude Code temper keeps apart from your own.",
        "One command in a terminal, on this machine",
    ] {
        let paragraph = nodes
            .iter()
            .find(|node| node.role == Role::Paragraph && node.name.as_deref() == Some(sentence.as_bytes()))
            .expect(sentence);
        let words = subtree(browser.tree(), paragraph.role, sentence.as_bytes());
        assert_eq!(f::nodes_text(words).as_ref(), sentence.as_bytes());
        assert!(text.lines().any(|line| line.trim() == sentence), "terminal: {sentence}");
    }
}
