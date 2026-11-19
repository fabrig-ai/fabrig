use super::*;
use skein_scratch::Scratch;
use temper_client_view_kit::readers::{has_text_run as has_text, require_named as find};

#[test]
fn names_are_the_promised_ones() {
    let samples = samples::samples();
    let mut names = Vec::new();
    for sample in &samples {
        assert!(!names.contains(&sample.name), "duplicate {}", sample.name);
        names.push(sample.name);
    }
    if temper_test_env::rewrite() {
        fs::write(skein_scratch::paths::package().join("names.txt"), format!("{}\n", names.join("\n"))).unwrap();
    } else {
        let promised: Vec<&str> = include_str!("../names.txt").lines().collect();
        assert_eq!(names, promised);
    }
}

#[test]
fn sheet_entry_links_reach_the_adoption_address() {
    for (name, link) in [("home-add-repository", "Adopt another…"), ("chats-no-repository", "Adopt a repository")] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("sheet entry sample");
        let mut view = view(&sample);
        let node = find(view.tree(), Role::Link, link, name);
        assert_eq!(
            temper_client_view::href(&mut view, node).as_deref(),
            Some(temper_client_domain::address::print(&temper_client_domain::Address::Adopt { project: 1 }).as_ref()),
            "{name}: sheet destination"
        );
    }
}

#[test]
fn a_later_settings_card_replaces_a_withdrawn_one_without_call_objects() {
    let sample = samples::samples().into_iter().find(|sample| sample.name == "ask-chat-replaced").unwrap();
    let view = view(&sample);
    assert!(view.tree().nodes().iter().any(|node| node.role == Role::Paragraph && node.name.as_deref() == Some(b"Replaced by the proposal below")));
}

#[test]
fn retired_chat_style_slots_stay_reserved() {
    let names = &temper_client_view::look::NAMES;
    for (token, expected) in [
        (62, b"temper-reserved-62".as_slice()),
        (295, b"temper-reserved-295"),
        (296, b"temper-reserved-296"),
        (297, b"temper-reserved-297"),
        (298, b"temper-reserved-298"),
        (299, b"temper-reserved-299"),
        (597, b"temper-reserved-597"),
    ] {
        assert_eq!(names.style(skein_ui::Style { owner: names.owner, token }), Some(expected));
    }
}

#[test]
fn team_hand_opens_the_sheet_with_the_first_area_and_role() {
    let sample = samples::samples().into_iter().find(|s| s.name == "team-hand").expect("hand-off sample");
    let root = samples::domain(&sample);
    let open = root.pages().hand_area().expect("hand-off sheet is open");
    assert_eq!(open.area.as_deref(), Some(b"core".as_slice()));
    assert_eq!(open.role, Some(52));
    assert!(open.apply);
    let (view, _, _) = built_root(&sample, ViewInputs::new(), &root);
    find(view.tree(), Role::Dialog, "Hand an area to a role", sample.name);
    let mut last = 0;
    for title in ["Prover", "Prover, from the stock", "Porter, from the stock", "Write one with Ask temper"] {
        let id = find(view.tree(), Role::Choice, title, sample.name);
        let position = view.tree().nodes().iter().position(|n| n.id == id).unwrap();
        assert!(position > last, "role choices follow the gallery's order");
        last = position;
    }
    for words in [
        "Owns its area's tests: triages their reds and flakes and makes the repairs.",
        "Keeps its area's port in step with its source.",
    ] {
        find(view.tree(), Role::Paragraph, words, sample.name);
    }
}
#[test]
<<<<<<< HEAD
=======
#[expect(clippy::too_many_lines, reason = "the gallery table walk checks each kind of named part")]
fn each_state_holds_what_its_name_says() {
    assert_eq!(HOLDS.len(), samples::N, "every promised row has a sample");
    let inputs = ViewInputs::new();
    for sample in samples::samples() {
        let mut row = None;
        for holds in &HOLDS {
            if holds.name == sample.name {
                assert!(row.is_none(), "{}: duplicate holds row", sample.name);
                row = Some(holds);
            }
        }
        let row = row.unwrap_or_else(|| panic!("{}: every sample has a HOLDS row", sample.name));
        let root = samples::domain(&sample);
        let (mut view, bounds, _) = built_root(&sample, inputs.clone(), &root);
        let tree = view.tree();
        let mut main = TreeFace::new(tree);
        let level = if matches!(
            root.pages().address(),
            temper_client_domain::Address::Goals { .. } | temper_client_domain::Address::Changes { .. }
        ) {
            2
        } else {
            1
        };
        let h1 = in_main(Role::Heading { level }, row.heading);
        if row.heading.is_empty() {
            assert_eq!(main.find(&h1), Found::Absent, "{} has no main heading", sample.name);
        } else {
            assert!(matches!(main.find(&h1), Found::Node(_)), "{}: heading {}", sample.name, row.heading);
        }
        // Count-only names may repeat; described parts follow conversation order.
        let mut described = tree.nodes().iter();
        let mut destinations = Vec::new();
        for part in row.parts {
            match part {
                Part::WelcomeFrame { box_note } => {
                    let banner = descendants(tree, Role::Banner, "", sample.name);
                    let mut children = Vec::new();
                    let mut rest = &banner[1..];
                    while let Some(node) = rest.first() {
                        children.push(node);
                        rest = &rest[usize::try_from(node.size).expect("banner child span")..];
                    }
                    assert_eq!(children.len(), 4, "{}: brand, steps, end, night switch", sample.name);
                    assert_eq!(
                        (children[0].role, children[0].name.as_deref()),
                        (Role::Link, Some(b"temper".as_slice()))
                    );
                    assert_eq!(
                        (children[1].role, children[1].name.as_deref()),
                        (Role::List, Some(b"Steps".as_slice()))
                    );
                    assert!(children[2].styles.as_slice().contains(&temper_client_view::look::TOP_END));
                    assert_eq!(
                        (children[3].role, children[3].name.as_deref(), children[3].styles.as_slice()),
                        (
                            Role::Button,
                            Some(b"Night mode".as_slice()),
                            [temper_client_view::look::NIGHT_SWITCH].as_slice()
                        )
                    );
                    let end_at = banner.len() - usize::try_from(children[3].size).expect("switch span");
                    let end = &banner[end_at - usize::try_from(children[2].size).expect("end span")..end_at];
                    assert_eq!(
                        end[1].region,
                        Some(skein_ui::Region { owner: temper_client_view::look::NAMES.owner, number: 0 })
                    );
                    assert_eq!(
                        usize::try_from(end[1].size).expect("region span") + 1,
                        end.len(),
                        "{}: link alone",
                        sample.name
                    );
                    let notes: Vec<_> = tree
                        .nodes()
                        .iter()
                        .filter(|n| n.styles.as_slice().contains(&temper_client_view::look::WELCOME_BOX_NOTE))
                        .collect();
                    assert_eq!(notes.len(), usize::from(*box_note), "{}: box paragraph on Welcome alone", sample.name);
                    if *box_note {
                        assert_eq!(notes[0].role, Role::Paragraph);
                        assert!(descendants(tree, Role::Main, "", sample.name).iter().any(|n| n.id == notes[0].id));
                    }
                }
                Part::Projects { rows } => {
                    let region = descendants(tree, Role::Region, "Projects", sample.name);
                    let actual: Vec<_> = region
                        .iter()
                        .enumerate()
                        .filter(|(_, node)| node.role == Role::Item)
                        .map(|(start, node)| {
                            let end = start + usize::try_from(node.size).expect("project span");
                            let row = &region[start..end];
                            let link = row.iter().find(|node| node.role == Role::Link).expect("project link");
                            let health = row
                                .iter()
                                .position(|node| node.styles.as_slice().contains(&temper_client_view::look::CHIP))
                                .expect("project health");
                            let end = health + usize::try_from(row[health].size).expect("health span");
                            let words: Vec<_> = row[health..end]
                                .iter()
                                .flat_map(|node| node.text.iter())
                                .flat_map(|run| run.text.iter().copied())
                                .collect();
                            (
                                String::from_utf8(link.name.as_deref().expect("project name").to_vec()).expect("UTF-8"),
                                String::from_utf8(words).expect("UTF-8"),
                            )
                        })
                        .collect();
                    let expected: Vec<_> =
                        rows.iter().map(|(name, health)| ((*name).to_owned(), (*health).to_owned())).collect();
                    assert_eq!(actual, expected, "{}: projects and their health", sample.name);
                }
                Part::StyleCount { style, count } => {
                    let actual = tree.nodes().iter().filter(|node| node.styles.as_slice().contains(style)).count();
                    assert_eq!(actual, *count, "{}: {style:?} rows", sample.name);
                }
                Part::HomeCards { primary, projects } => {
                    let nodes = tree.nodes();
                    let cards: Vec<_> = nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, node)| node.styles.as_slice().contains(&temper_client_view::look::NEED_CARD))
                        .collect();
                    assert_eq!(cards.len(), primary.len(), "{}: Home cards", sample.name);
                    for (index, ((start, card), primary)) in cards.iter().zip(*primary).enumerate() {
                        let end = start + usize::try_from(card.size).expect("card size");
                        let card = &nodes[*start..end];
                        if !projects.is_empty() {
                            assert!(
                                card.iter().any(|node| node
                                    .text
                                    .iter()
                                    .any(|run| run.text.as_ref() == projects[index].as_bytes())),
                                "{}: card {index} project",
                                sample.name
                            );
                        }
                        let acts = card
                            .iter()
                            .position(|node| node.styles.as_slice().contains(&temper_client_view::look::NEED_ACTS))
                            .expect("card acts");
                        let end = acts + usize::try_from(card[acts].size).expect("acts size");
                        let first = card[acts..end]
                            .iter()
                            .find(|node| matches!(node.role, Role::Button | Role::Link))
                            .expect("primary act");
                        assert_eq!(
                            first.name.as_deref(),
                            Some(primary.as_bytes()),
                            "{}: card {index} primary act",
                            sample.name
                        );
                    }
                }
                Part::Current { name } => {
                    let id = find(tree, Role::Link, name, sample.name);
                    assert!(
                        tree.find(id).expect("current link").states.as_slice().contains(&skein_ui::State::Current),
                        "{}: {name} current",
                        sample.name
                    );
                }
                Part::Destination { name, href } => {
                    let Found::Node(node) = main.find(&Find::exactly_named(Role::Link, name.as_bytes())) else {
                        panic!("{}: missing link {name}", sample.name)
                    };
                    destinations.push((node, href));
                }
                Part::FirstButton { disclosure, name } => {
                    let nodes = descendants(tree, Role::Disclosure, disclosure, sample.name);
                    let first = nodes.iter().find(|node| node.role == Role::Button).expect("switcher's first row");
                    assert_eq!(first.name.as_deref(), Some(name.as_bytes()), "{}: first project row", sample.name);
                }
                Part::CardChanges { title, changes } => {
                    let link = find(tree, Role::Link, title, sample.name);
                    let nodes = tree.nodes();
                    let at = nodes.iter().position(|node| node.id == link).expect("card's link");
                    let (start, card) = nodes
                        .iter()
                        .enumerate()
                        .take(at)
                        .rev()
                        .find(|(start, node)| {
                            node.styles.as_slice().contains(&temper_client_view::look::WORK_CARD)
                                && at < start + usize::try_from(node.size).expect("card size")
                        })
                        .expect("link's card");
                    let end = start + usize::try_from(card.size).expect("card size");
                    let links: Vec<_> = nodes[start..end]
                        .iter()
                        .filter(|node| node.role == Role::Link && node.id != link)
                        .map(|node| std::str::from_utf8(node.name.as_deref().expect("change name")).expect("English"))
                        .collect();
                    assert_eq!(links, *changes, "{}: {title} changes", sample.name);
                }
                Part::Cards { section, from } => {
                    let nodes = descendants(tree, Role::Region, section, sample.name);
                    let cards: Vec<_> = nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, node)| node.styles.as_slice().contains(&temper_client_view::look::WORK_CARD))
                        .collect();
                    assert_eq!(cards.len(), from.len(), "{}: {section} card count", sample.name);
                    for ((start, card), expected) in cards.iter().zip(*from) {
                        let end = start + usize::try_from(card.size).expect("card subtree");
                        let card = &nodes[*start..end];
                        let start = card
                            .iter()
                            .position(|node| {
                                node.role == Role::Paragraph
                                    && node.styles.as_slice().contains(&temper_client_view::look::WORK_FROM)
                            })
                            .expect("card's from line");
                        let end = start + usize::try_from(card[start].size).expect("from subtree");
                        let text: Vec<_> = card[start..end]
                            .iter()
                            .flat_map(|node| node.text.iter().flat_map(|run| run.text.iter().copied()))
                            .collect();
                        assert_eq!(String::from_utf8_lossy(&text), *expected, "{}: {section} from line", sample.name);
                    }
                }
                Part::GroupLines { name, lines } => {
                    let group = descendants(tree, Role::Group, name, sample.name);
                    let drawn: Vec<_> = group
                        .iter()
                        .filter(|node| node.role == Role::Item)
                        .map(|node| std::str::from_utf8(node.name.as_deref().expect("line name")).expect("English"))
                        .collect();
                    assert_eq!(drawn, *lines, "{}: {name} lines in order", sample.name);
                }
                Part::StyledText { style, text } => {
                    let nodes = tree.nodes();
                    let start = nodes
                        .iter()
                        .position(|node| node.styles.as_slice().contains(style))
                        .unwrap_or_else(|| panic!("{}: missing row {style:?}", sample.name));
                    let end = start + usize::try_from(nodes[start].size).expect("row span");
                    let actual: Vec<_> = nodes[start..end]
                        .iter()
                        .flat_map(|node| node.text.iter())
                        .map(|run| std::str::from_utf8(&run.text).expect("row words"))
                        .collect();
                    assert_eq!(actual, *text, "{}: row text", sample.name);
                }
                Part::StockWays { current } => {
                    let menu = descendants(tree, Role::Menu, "Stock ways", sample.name);
                    let items: Vec<_> = menu.iter().filter(|node| node.role == Role::MenuItem).collect();
                    assert_eq!(items.len(), 3, "{}: stock ways alone", sample.name);
                    for (index, (item, name)) in
                        items.into_iter().zip(["You approve", "Agents review", "Draft first"]).enumerate()
                    {
                        assert_eq!(item.name.as_deref(), Some(name.as_bytes()), "{}: read order", sample.name);
                        assert_eq!(
                            item.states.as_slice().contains(&skein_ui::State::Selected),
                            *current == Some(index),
                            "{}: running way ticked",
                            sample.name
                        );
                    }
                    assert!(!tree.nodes().iter().any(|node| node.role == Role::Tabs), "{}: no tabs", sample.name);
                    let how =
                        sample.name == "how" || sample.name.starts_with("how-") || sample.name.starts_with("night-how");
                    let foot = tree
                        .nodes()
                        .iter()
                        .position(|n| n.styles.as_slice().contains(&temper_client_view::look::WAYS_FOOT))
                        .unwrap();
                    let end = foot + usize::try_from(tree.nodes()[foot].size).unwrap();
                    let buttons: Vec<_> = tree.nodes()[foot..end]
                        .iter()
                        .filter(|n| n.role == Role::Button)
                        .map(|n| n.name.as_deref())
                        .collect();
                    assert_eq!(buttons, if how { vec![] } else { vec![Some(b"How does temper work?".as_slice())] });
                    if how {
                        assert!(
                            !tree
                                .nodes()
                                .iter()
                                .any(|n| n.styles.as_slice().contains(&temper_client_view::look::WAY_ASK))
                        );
                    }
                    if matches!(sample.name, "ask-ways-menu" | "ask-ways-yours-menu" | "how-menu") {
                        assert_eq!(sample.open_menu(), Some(b"Stock ways".as_slice()));
                        let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
                        assert!(html.contains("aria-expanded=\"true\">Stock ways"), "{}: menu open", sample.name);
                        assert!(
                            !html.contains(&format!("id=\"skein-ui-menu-{}\" hidden", menu[0].id.0)),
                            "{}: menu visible",
                            sample.name
                        );
                    }
                }
                Part::Path { stops } => {
                    let path = descendants(tree, Role::List, "The way a change goes", sample.name);
                    let actual: Vec<_> = path
                        .iter()
                        .filter(|node| node.role == Role::Item)
                        .map(|node| std::str::from_utf8(node.name.as_deref().expect("stop words")).expect("UTF-8 stop"))
                        .collect();
                    assert_eq!(actual, *stops, "{}: path stops and actors", sample.name);
                }
                Part::Drawing { style, count } => {
                    let drawing = tree
                        .nodes()
                        .iter()
                        .find_map(|node| {
                            node.drawing.as_ref().filter(|drawing| drawing.kind == temper_client_view::paths::KIND)
                        })
                        .expect("way drawing");
                    let mut scene = Scene::new(&bounds.tree);
                    assert!(draw(drawing.kind, &drawing.data, 1000, 0, &mut scene), "{}: way scene", sample.name);
                    assert!(!scene.refused(), "{}: scene fits", sample.name);
                    let actual = (0..u32::try_from(scene.marks().len()).expect("scene marks"))
                        .filter(|index| scene.styles(*index).contains(style))
                        .count();
                    assert_eq!(actual, *count, "{}: drawing {style:?}", sample.name);
                }
                Part::Expanded { name } => {
                    let id = find(tree, Role::Button, name, sample.name);
                    assert!(
                        tree.find(id).expect("stop").states.as_slice().contains(&skein_ui::State::Expanded),
                        "{}: open stop",
                        sample.name
                    );
                }
                Part::Folded { name } => {
                    let id = find(tree, Role::Disclosure, name, sample.name);
                    assert!(
                        !tree.find(id).expect("disclosure").states.as_slice().contains(&skein_ui::State::Expanded),
                        "{}: {name} folded",
                        sample.name
                    );
                }
                Part::Configured { value } => {
                    let facts = temper_client_domain::pages::home::facts(
                        &root,
                        1,
                        worlds::sample_now(sample.world, sample.name),
                    );
                    assert_eq!(facts.configured, *value, "{}: To begin's first step", sample.name);
                }
                Part::Reader { role } => {
                    let me = root.pages().me(root.client()).expect("signed-in reader");
                    let member = me.projects.iter().find(|member| member.project == 1).expect("ways project");
                    assert_eq!(&member.role, role, "{}: reader's role", sample.name);
                }
                Part::After { first, then } => {
                    let nodes = tree.nodes();
                    let start = nodes.iter().position(|node| node.role == Role::Main).expect("main");
                    let end = start + usize::try_from(nodes[start].size).expect("main span");
                    let at = |style| {
                        nodes[start..end]
                            .iter()
                            .position(|node| node.styles.as_slice().contains(style))
                            .expect("ordered part")
                    };
                    assert!(at(first) < at(then), "{}: part order", sample.name);
                }
                Part::Links { style, names } => {
                    let nodes = tree.nodes();
                    let start =
                        nodes.iter().position(|node| node.styles.as_slice().contains(style)).expect("navigation style");
                    let end = start + usize::try_from(nodes[start].size).expect("navigation size");
                    let links: Vec<_> = nodes[start..end]
                        .iter()
                        .filter(|node| node.role == Role::Link)
                        .map(|node| std::str::from_utf8(node.name.as_deref().expect("link name")).expect("English"))
                        .collect();
                    assert_eq!(links, *names, "{}: navigation", sample.name);
                }
                Part::NoStyle { style } => assert!(
                    !tree.nodes().iter().any(|node| node.styles.as_slice().contains(style)),
                    "{}: absent style",
                    sample.name
                ),
                Part::Selected { name } => {
                    let id = find(tree, Role::Choice, name, sample.name);
                    assert!(
                        tree.find(id).expect("source choice").states.as_slice().contains(&skein_ui::State::Selected),
                        "{}: {name} selected",
                        sample.name
                    );
                }
                Part::NotPressed { name } => {
                    let id = find(tree, Role::Button, name, sample.name);
                    assert!(
                        !tree.find(id).expect("mode button").states.as_slice().contains(&skein_ui::State::Pressed),
                        "{}: {name} not pressed",
                        sample.name
                    );
                }
                Part::Pressed { name } => {
                    let id = find(tree, Role::Button, name, sample.name);
                    assert!(
                        tree.find(id).expect("mode button").states.as_slice().contains(&skein_ui::State::Pressed),
                        "{}: {name} pressed",
                        sample.name
                    );
                }
                Part::Text { text } => {
                    assert!(has_text(tree, text), "{}: missing text {text:?}", sample.name);
                }
                Part::Buttons { style, names } => {
                    let nodes = tree.nodes();
                    let start =
                        nodes.iter().position(|node| node.styles.as_slice().contains(style)).expect("button row");
                    let end = start + usize::try_from(nodes[start].size).expect("row span");
                    let actual: Vec<_> = nodes[start..end]
                        .iter()
                        .filter(|node| node.role == Role::Button)
                        .map(|node| std::str::from_utf8(node.name.as_deref().expect("button name")).expect("English"))
                        .collect();
                    assert_eq!(actual, *names, "{}: buttons in order", sample.name);
                }
                Part::Named { role, name } => {
                    find(tree, *role, name, sample.name);
                }
                Part::Ordered { role, name } => {
                    assert!(
                        described.any(|node| node.role == *role && node.name.as_deref() == Some(name.as_bytes())),
                        "{}: missing or out of order {role:?} {name:?}",
                        sample.name
                    );
                }
                Part::Absent { role, name } => {
                    let query =
                        if *role == Role::Menu { Find::named(*role, name.as_bytes()) } else { in_main(*role, name) };
                    assert_eq!(main.find(&query), Found::Absent, "{}: absent {role:?} {name:?}", sample.name);
                }
                Part::NoRole { role } => {
                    assert!(!tree.nodes().iter().any(|node| node.role == *role), "{}: absent {role:?}", sample.name);
                }
                Part::Value { role, name, value } => {
                    let node = find(tree, *role, name, sample.name);
                    let value = value.as_bytes();
                    assert_eq!(
                        tree.find(node).expect("field").value.as_ref().expect("field value").text.as_ref(),
                        value,
                        "{}",
                        sample.name
                    );
                }
                Part::Described { role, name, description } => {
                    let node = described
                        .find(|node| node.role == *role && node.name.as_deref() == Some(name.as_bytes()))
                        .unwrap_or_else(|| panic!("{}: missing {role:?} {name:?}", sample.name));
                    assert_eq!(node.description.as_deref(), Some(description.as_bytes()), "{}", sample.name);
                }
                Part::Goals { rows } => {
                    let actual: Vec<_> = tree
                        .nodes()
                        .iter()
                        .filter(|node| node.styles.as_slice().contains(&temper_client_view::look::GOALS_ROW))
                        .map(|node| String::from_utf8_lossy(node.name.as_deref().expect("goal link name")))
                        .collect();
                    assert_eq!(actual.len(), rows.len(), "{}: goal rows", sample.name);
                    for (actual, expected) in actual.iter().zip(*rows) {
                        assert!(actual.starts_with(expected), "{}: expected {expected:?}, got {actual:?}", sample.name);
                    }
                }
                Part::Chats { live, closed } => {
                    let available = (*live).min(driver::limits().client.domain.rows)
                        + (*closed).min(driver::limits().client.domain.rows);
                    if available == 0 {
                        assert_eq!(main.find(&in_main(Role::List, "Your chats")), Found::Absent, "{}", sample.name);
                        continue;
                    }
                    let list = find(tree, Role::List, "Your chats", sample.name);
                    let nodes = tree.nodes();
                    let start = nodes.iter().position(|node| node.id == list).expect("chats list");
                    let end = start + usize::try_from(nodes[start].size).expect("chats subtree");
                    let mut count = 0;
                    for node in &nodes[start..end] {
                        if node.role == Role::Link
                            && let Some(name) = &node.name
                            && name
                                .strip_prefix(b"T")
                                .is_some_and(|number| !number.is_empty() && number.iter().all(u8::is_ascii_digit))
                        {
                            count += 1;
                        }
                    }
                    assert_eq!(count, available, "{}: windowed chat rows", sample.name);
                }
                Part::Plan { live, title } => {
                    let plan = find(tree, Role::Tree, "Plan", sample.name);
                    let row = Find::named(Role::Paragraph, title.as_bytes()).within(Box::new([
                        skein_fake_person::Target::Role { role: Role::Tree, name: Box::from(b"Plan".as_slice()) },
                    ]));
                    assert!(matches!(main.find(&row), Found::Node(_)), "{}: delegate title", sample.name);
                    let nodes = tree.nodes();
                    let start = nodes.iter().position(|node| node.id == plan).expect("plan node");
                    let end = start + usize::try_from(nodes[start].size).expect("plan subtree");
                    let count = nodes[start..end].iter().filter(|node| node.role == Role::TreeItem).count();
                    let window = driver::limits().client.domain.rows;
                    assert_eq!(
                        count,
                        usize::try_from((*live).min(window)).expect("row count"),
                        "{}: plan window",
                        sample.name
                    );
                }
            }
        }
        for (node, expected) in destinations {
            assert_eq!(
                temper_client_view::href(&mut view, node).as_deref(),
                Some(expected.as_bytes()),
                "{}: destination",
                sample.name
            );
        }
        each_chat_reads_its_allotment(&sample, &view);
        no_state_reads_of_zero(&sample, &view);
        no_named_control_draws_a_text_child_in_any_state(&sample, &view);
        navigation_links_have_addresses_and_install_action_links_keep_their_binding(&sample, &mut view);
    }
}
#[test]
>>>>>>> refs/manual/8669
fn goal_pages_hold_reports_counts_and_only_their_needs() {
    use temper_client_domain::pages::goals::State;
    for (name, number, done, total, state, report) in [
        ("goal-running", 108, 6, 9, State::Running, true),
        ("goal-needs-funds", 108, 6, 9, State::Needs { funds: true }, true),
        ("goal-needs-question", 118, 1, 3, State::Needs { funds: false }, true),
        ("goal-planning", 122, 0, 0, State::Planning, false),
        ("goal-paused", 108, 6, 9, State::Paused, true),
        ("goal-landed", 123, 2, 2, State::Landed, true),
        ("goal-landed-open", 123, 2, 2, State::Landed, true),
        ("goal-closed", 118, 1, 3, State::Closed, false),
        ("goal-failed", 122, 0, 0, State::Failed, false),
        ("goal-no-report", 108, 6, 9, State::Running, false),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("goal sample");
        let root = samples::domain(&sample);
        let page = root.pages().task().expect("task page");
        let task = page.document(root.client()).expect("goal document");
        let goal = page.goal.as_ref().expect("goal page reads");
        assert_eq!(task.chip.task, number, "{name}");
        assert_eq!(goal.line.as_ref().expect("own line read").chip.task, number, "{name}");
        assert_eq!(goal.counts(task, None), temper_client_domain::pages::goal::Counts { done, total }, "{name}");
        assert_eq!(goal.report.is_some(), report, "{name}");
        assert!(page.cards.is_empty(), "{name}: needs belong to the filtered inbox");
        let inbox = root.pages().inbox().expect("goal inbox");
        let open = root.pages().needs().open(inbox);
        assert_eq!(open.len(), usize::from(matches!(state, State::Needs { .. })), "{name}");
        assert_eq!(goal.state(task, None, !open.is_empty()), state, "{name}");
        for card in &open {
            assert!(card.page.is_some(), "{name}: need document delivered");
        }
        let view = view(&sample);
        let bylines = view
            .tree()
            .nodes()
            .iter()
            .any(|node| node.styles.as_slice().contains(&temper_client_view::look::GOAL_BYLINE));
        assert_eq!(bylines, report, "{name}: only a report has a byline");
        if name == "goal-needs-question" {
            let choice = find(view.tree(), Role::Button, "In jig’s store", name);
            assert!(
                view.tree()
                    .find(choice)
                    .expect("recommended choice")
                    .styles
                    .as_slice()
                    .contains(&temper_client_view::look::GOAL_PRIMARY)
            );
        }
        if matches!(state, State::Landed | State::Closed | State::Failed) {
            assert!(
                !view.tree().nodes().iter().any(|node| node.role == Role::TextArea),
                "{name}: ended page has no composer"
            );
        }
    }
}
#[test]
fn replacing_a_gallery_removes_the_whole_previous_print() {
    let scratch = Scratch::new("temper-client-gallery-replace");
    let out = scratch.path().join("gallery");
    replace_gallery(&out).expect("new gallery directory");
    assert!(out.is_dir(), "missing output created");
    let old_bundle = out.join("bundle").join("old-digest");
    fs::create_dir_all(&old_bundle).expect("previous bundle");
    fs::write(old_bundle.join("old.css"), b"old styles").expect("previous stylesheet");
    fs::write(out.join("index.html"), b"previous index").expect("previous index");
    fs::write(out.join("stale.html"), b"obsolete").expect("stale page");
    replace_gallery(&out).expect("replace owned gallery");
    assert!(out.is_dir(), "replacement output exists");
    assert_eq!(fs::read_dir(&out).expect("replacement directory").count(), 0, "whole previous print removed");
}
#[test]
fn print_refuses_a_directory_not_its_own() {
    let scratch = Scratch::new("temper-client-gallery-refusal");
    let file = scratch.path().join("precious");
    fs::write(&file, b"keep me").expect("foreign file");
    match print(scratch.path()) {
        Err(Refusal::NotGallery { out }) => assert_eq!(out, scratch.path()),
        other => panic!("expected NotGallery, got {other:?}"),
    }
    assert_eq!(fs::read(&file).expect("file untouched"), b"keep me");
    assert_eq!(fs::read_dir(scratch.path()).expect("directory untouched").count(), 1);
}

#[test]
fn times_read_against_each_mockup_worlds_fixed_now() {
    for (name, expected) in [
        ("chats-solo", "1 h 28 min"),
        ("chats-reference", "50 min"),
        ("inbox-solo", "a proposal"),
        ("inbox-solo", "12 min"),
        ("chat-solo-T10", "waiting 12 min"),
        ("chat-reference-T1", "Today · Thursday 22 October"),
        ("chat-solo-T10", "Today · Tuesday 27 October"),
        ("chat-first-run-4-T4", "21:05"),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("named sample");
        let view = view(&sample);
        assert!(
            view.tree()
                .nodes()
                .iter()
                .any(|node| node.text.iter().any(|run| String::from_utf8_lossy(&run.text).contains(expected))),
            "{name}: {expected}"
        );
    }
}

#[test]
fn gallery_chat_clocks_are_separate_leaves_under_the_speakers() {
    for (name, clock) in [("chat-reference-T1", "06:56"), ("chat-solo-T10", "09:18")] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("sample");
        let view = view(&sample);
        let tree = view.tree();
        let who = tree
            .nodes()
            .iter()
            .enumerate()
            .find(|(_, node)| node.styles.as_slice().contains(&temper_client_view::look::WHO))
            .expect("speaker")
            .0;
        assert_eq!(tree.nodes()[who + 1].name.as_deref(), Some(b"You".as_slice()));
        let clock_at = who + 1 + usize::try_from(tree.nodes()[who + 1].size).expect("speaker size");
        assert_eq!(tree.nodes()[clock_at].role, Role::Time);
        assert_eq!(tree.nodes()[clock_at].name.as_deref(), Some(clock.as_bytes()));
    }
}

#[test]
fn model_samples_link_the_mockup_state_that_draws_them() {
    let mut rows = [
        ("connect-model-choose", Some("connect-model.html?world=reference&bare")),
        ("connect-model-connected", Some("connect-model.html?world=reference&bare")),
        ("connect-model-key-details", Some("connect-model.html?world=reference&way=key&bare")),
        ("connect-model-program-details", Some("connect-model.html?world=reference&way=program&bare")),
        ("connect-model-signing-in", None),
        ("accounts-added", Some("accounts.html?world=reference&bare")),
        (
            "connect-model-declined",
            Some("connect-model.html?world=first-run&step=5&way=chatgpt&bare#other-states-declined"),
        ),
        ("accounts-renewed", Some("accounts.html?world=reference&bare")),
        (
            "connect-model-refused",
            Some("connect-model.html?world=first-run&step=5&way=chatgpt&bare#other-states-refused"),
        ),
        ("connect-model-key-lost", None),
        ("welcome-model-claude-terminal", Some("../web/claude-sign-in/01-model.html")),
        ("welcome-model-claude-signing-in", Some("../web/claude-sign-in/02-signing-in.html")),
        ("welcome-model-claude-signed-in", Some("../web/claude-sign-in/04-signed-in.html")),
        ("welcome-model-claude-no-terminal", Some("../web/claude-sign-in/05-no-terminal.html")),
        ("welcome-model-claude-not-finished", None),
    ];
    rows.sort_by_key(|row| row.0);
    let models: Vec<_> = samples::samples()
        .into_iter()
        .filter(|sample| {
            sample.name.starts_with("connect-model-")
                || sample.name.starts_with("welcome-model-claude-")
                || matches!(sample.name, "accounts-added" | "accounts-renewed")
        })
        .collect();
    assert_eq!(models.len(), rows.len(), "every model sample has a row");
    for (sample, (name, expected)) in models.iter().zip(rows) {
        assert_eq!(sample.name, name);
        assert_eq!(samples::mockup(sample).as_deref(), expected, "{name}");
    }
}

#[test]
fn connection_samples_assign_priority_and_progress_to_the_bundled_nodes() {
    use temper_client_view::look;
    for (name, expected) in [
        ("welcome-forge-choose", vec!["Run on this machine"]),
        ("welcome-forge-details", vec!["Run on this machine", "Run on this machine"]),
        ("welcome-forge-under-way", vec!["Continue"]),
        ("welcome-forge-connected", vec!["Continue"]),
        ("welcome-forge-github", vec!["Continue"]),
        ("welcome-model-choose", vec!["Use your Claude subscription"]),
        ("welcome-model-claude-terminal", vec!["Sign in to Claude Code"]),
        ("welcome-model-claude-signing-in", vec![]),
        ("welcome-model-claude-signed-in", vec!["Continue"]),
        ("welcome-model-claude-no-terminal", vec!["Use your Claude subscription"]),
        ("welcome-model-claude-not-finished", vec!["Sign in to Claude Code"]),
        ("github-app-details", vec!["GitHub"]),
        ("github-install", vec!["Install on GitHub"]),
        ("github-picker", vec![]),
        ("github-adopted", vec![]),
        ("forges-github-install", vec!["Install on GitHub"]),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("sample");
        let view = view(&sample);
        let primary: Vec<_> = view
            .tree()
            .nodes()
            .iter()
            .filter(|node| node.styles.as_slice().contains(&look::FLOW_PRIMARY))
            .map(|node| std::str::from_utf8(node.name.as_deref().expect("primary name")).expect("UTF-8"))
            .collect();
        assert_eq!(primary, expected, "{name}");
        if name == "welcome-forge-details" {
            assert_eq!(
                view.tree()
                    .nodes()
                    .iter()
                    .filter(|node| node.styles.as_slice().contains(&look::WELCOME_OPTION_MORE))
                    .count(),
                1
            );
        }
    }
    let sample =
        samples::samples().into_iter().find(|sample| sample.name == "forge-setting-up-middle").expect("sample");
    let view = view(&sample);
    let progress = view
        .tree()
        .nodes()
        .iter()
        .position(|node| node.styles.as_slice().contains(&look::CONNECT_LIVE))
        .expect("progress");
    let nodes = view.tree().nodes();
    let end = progress + usize::try_from(nodes[progress].size).expect("subtree size");
    let items: Vec<_> = nodes[progress..end].iter().filter(|node| node.role == Role::Item).collect();
    assert_eq!(items.len(), 5);
    assert!(items[0].styles.is_empty());
    assert!(items[1].states.as_slice().contains(&skein_ui::State::Current));
    assert!(items[2].styles.as_slice().contains(&look::CONNECT_LATER));
    assert!(items[2].states.is_empty());
}

#[test]
fn asking_gallery_states_list_the_template_and_send_readiness() {
    for (name, sending, disabled, refusal) in [
        ("ask-first-run", false, true, false),
        ("night-ask", false, true, false),
        ("ask-typing", false, false, false),
        ("ask-sending", true, true, false),
        ("ask-refused", false, false, true),
    ] {
        let sample = samples::samples().into_iter().find(|s| s.name == name).expect("sample");
        let root = samples::domain(&sample);
        let wall = worlds::now(sample.world);
        assert_eq!(root.pages().address(), &temper_client_domain::Address::Ask { project: 1 }, "{name}");
        let facts = temper_client_domain::pages::home::facts(&root, 1, wall);
        assert!(temper_client_domain::pages::home::beginning(&root, 1, wall), "{name}");
        assert_eq!(facts.left(), 1, "{name}: only the way of working is left");
        assert!(matches!(facts.repository, temper_client_domain::pages::home::Repository::In { .. }), "{name}");
        assert!(
            root.pages()
                .me(root.client())
                .expect("me")
                .projects
                .get(0)
                .expect("project")
                .templates
                .as_slice()
                .iter()
                .any(|t| t.number == 5)
        );
        let chats = root.pages().chats().expect("Ask temper");
        assert_eq!(chats.start.is_some(), sending, "{name}");
        assert_eq!(chats.outcome.is_some(), refusal, "{name}");
        let view = view(&sample);
        let send = view
            .tree()
            .nodes()
            .iter()
            .find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Send"))
            .expect("send");
        assert_eq!(send.states.as_slice().contains(&skein_ui::State::Disabled), disabled, "{name}");
        let how = find(view.tree(), Role::Button, "How does temper work?", name);
        let how = view.tree().find(how).expect("how button");
        assert_eq!(how.states.as_slice().contains(&skein_ui::State::Disabled), sending, "{name}");
        assert!(has_text(view.tree(), "How a change is produced and lands in Hello."), "{name}");
    }
}

#[test]
fn new_chat_focuses_the_composer_in_its_print() {
    let sample = samples::samples().into_iter().find(|s| s.name == "chat-new").expect("new chat");
    let view = view(&sample);
    let entry = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::TextArea && n.name.as_deref() == Some(b"What do you want done?"))
        .expect("composer");
    assert_eq!(entry.focus, Some(0));
    let page = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    assert!(page.contains(&format!("data-node=\"{}\" autofocus", entry.id.0)));
}

#[test]
fn goal_and_chat_queries_keep_live_and_ended_rows_separate() {
    use samples::{Then, World};
    use temper_client_domain::{Address, ChatsFilter, GoalsFilter};
    for world in [World::Solo, World::Reference] {
        let (chats, _) = worlds::world(world, Address::Chats { filter: ChatsFilter::All }, Then::Shown);
        let page = chats.pages().chats().expect("chats page");
        let live = &page.live(chats.client()).expect("live chats").rows;
        let jig_client::contract::EndedGroup::Rows { rows: ended, .. } =
            &page.page.as_ref().expect("closed chats").ended
        else {
            panic!("ended chat rows")
        };
        assert!(!live.is_empty() && !ended.is_empty(), "both chat sources have rows");
        for row in live.iter().chain(ended) {
            assert!(row.flags.conversation && row.tracked.is_none(), "{world:?}: chats contain only conversations");
        }
        let (goals, _) = worlds::world(world, Address::Goals { project: 1, filter: GoalsFilter::All }, Then::Shown);
        let page = goals.pages().goals().expect("goals page");
        let live = &page.live(goals.client()).expect("live goals").rows;
        assert!(!live.is_empty() && !page.ended().is_empty(), "both goal sources have rows");
        for row in live.iter().chain(page.ended()) {
            assert!(!row.flags.conversation && row.tracked.is_some(), "{world:?}: goals contain only tracked tasks");
        }
    }
}

#[test]
fn reference_lede_names_the_worlds_templates_in_number_order() {
    use jig_client::contract as c;
    let sample =
        samples::samples().into_iter().find(|sample| sample.name == "goals-reference").expect("reference goals");
    let root = samples::domain(&sample);
    let me = root.pages().me(root.client()).expect("reference me");
    let templates = &me.projects.get(0).expect("reference project").templates;
    let goals: Vec<_> = templates
        .iter()
        .filter(|template| template.tracked.is_some())
        .map(|template| (template.number, template.name.bytes()))
        .collect();
    assert_eq!(
        goals,
        vec![
            (2, b"goal".as_slice()),
            (4, b"goal for a draft".as_slice()),
            (6, b"Plan, then wait for me".as_slice()),
            (7, b"One agent, one branch".as_slice()),
        ]
    );
    assert_eq!(
        templates.iter().find(|template| template.number == 5).expect("Ask temper template").executor,
        c::TemplateExecutor::Charter(11)
    );
    let view = view(&sample);
    let nodes = view.tree().nodes();
    let start = nodes
        .iter()
        .position(|node| node.styles.as_slice().contains(&temper_client_view::look::GOALS_LEDE))
        .expect("goals lede");
    let end = start + usize::try_from(nodes[start].size).expect("lede size");
    let text: Vec<_> =
        nodes[start..end].iter().flat_map(|node| node.text.iter()).flat_map(|run| run.text.iter().copied()).collect();
    assert_eq!(
        String::from_utf8(text).expect("lede UTF-8"),
        "A goal turns a draft into landed work, its own way: task by task, plan, tasks, land whole, plan, then wait for me, one agent, one branch."
    );
}

#[test]
fn each_settings_switch_off_keeps_its_value_and_changes_its_navigation() {
    for (name, off, changes) in [
        ("settings-drafts-off", "Chats can write drafts", true),
        ("settings-goals-off", "Goals", true),
        ("settings-changes-off", "Changes", false),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).expect("switch-off sample");
        let view = view(&sample);
        let tree = view.tree();
        for label in ["Chats can write drafts", "Goals", "Changes"] {
            let check = tree
                .nodes()
                .iter()
                .find(|node| node.role == Role::Check && node.name.as_deref() == Some(label.as_bytes()))
                .expect("exact project switch name");
            assert_eq!(check.states.as_slice().contains(&skein_ui::State::Pressed), label != off, "{name}: {label}");
        }
        let root = samples::domain(&sample);
        let filters = temper_client_domain::pages::work::shown(
            &temper_client_domain::pages::work::Filters {
                goals: 1,
                changes: 1,
                ..temper_client_domain::pages::work::Filters::default()
            },
            temper_client_domain::pages::work::overrides(&root, 1),
        );
        for (label, present, expected) in
            [("Goals", filters.goals, off != "Goals"), ("Changes", filters.changes, changes)]
        {
            assert_eq!(present, expected, "{name}: {label} Work filter");
        }
        let ways = tree
            .nodes()
            .iter()
            .any(|node| node.role == Role::Heading { level: 2 } && node.name.as_deref() == Some(b"Ways of working"));
        assert_eq!(ways, off != "Goals", "{name}: ways");
    }
}

#[test]
fn goal_acts_samples_reach_the_editor_dialog_and_answered_toast() {
    for (name, role, label) in [
        ("goal-amending", Role::Region, "Amend task 108"),
        ("goal-closing", Role::Dialog, "Close this goal?"),
        ("goal-toast", Role::Status, ""),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).unwrap();
        let root = samples::domain(&sample);
        let view = view(&sample);
        if name == "goal-toast" {
            assert_eq!(root.pages().needs().toast(), Some(&temper_client_domain::pages::needs::Toasted::Paused));
            assert!(root.pages().task().unwrap().pending.is_empty());
            assert!(view.tree().nodes().iter().any(|node| node.role == role));
        } else {
            find(view.tree(), role, label, name);
        }
    }
}

#[test]
fn the_open_samples_open_only_their_named_disclosure() {
    for (name, disclosure) in [
        ("chat-model-open", "Model"),
        ("chat-work-open", "Checked settings 3×, read 7 docs"),
        ("rail-all-projects", "vsr"),
        ("rail-across", "All projects"),
    ] {
        let sample = samples::samples().into_iter().find(|s| s.name == name).unwrap();
        let view = view(&sample);
        let node = view
            .tree()
            .nodes()
            .iter()
            .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(disclosure.as_bytes()))
            .unwrap();
        let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
        assert!(html.contains(&format!("data-node=\"{}\" open", node.id.0)), "{name}");
        assert_eq!(html.matches(" open ").count(), 1);
    }
}

#[test]
fn the_rail_menu_sample_opens_the_menu() {
    let sample = samples::samples().into_iter().find(|s| s.name == "rail-new-menu").unwrap();
    let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    assert!(html.contains("aria-expanded=\"true\""));
    let menu = html.split("role=\"menu\"").nth(1).unwrap().split('>').next().unwrap();
    assert!(!menu.contains("hidden"), "{menu}");
    assert!(html.contains("Ask for one change."));
    assert!(html.contains("Ask about this project, or change how its work runs."));
    assert!(html.contains("A design session that ends in a goal, best had in the terminal."));
}

#[test]
fn the_person_menu_sample_opens_above_its_row() {
    let sample = samples::samples().into_iter().find(|s| s.name == "rail-person-menu").unwrap();
    let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    let person = html.split("class=\"temper-rail-person\"").nth(1).unwrap();
    assert!(person.contains("aria-expanded=\"true\""));
    let menu = person.split("role=\"menu\"").nth(1).unwrap().split('>').next().unwrap();
    assert!(!menu.contains("hidden"), "{menu}");
    assert!(person.contains("role=\"menuitem\">Accounts</div>"));
    assert!(person.contains("role=\"menuitem\">What temper remembers about you</div>"));
    assert!(person.contains("role=\"menuitem\">Telling you</div>"));
    assert!(person.contains("role=\"menuitem\">Sign out</div>"));
    assert_eq!(person.matches("role=\"menuitem\"").count(), 4);
    let css = fs::read_to_string(STYLESHEETS[0]).unwrap();
    let panel = css
        .lines()
        .find(|line| line.starts_with("#app > .temper-rail > .temper-rail-foot > .temper-rail-person > [role=menu] {"))
        .unwrap();
    assert!(panel.contains("bottom: calc(100% + 6px)"));
    assert!(!panel.contains("top:"));
}

#[test]
fn the_accounts_tabs_sign_out_hides_over_a_phones_width_alone() {
    let css = fs::read_to_string(STYLESHEETS[0]).unwrap();
    let mut selectors = Vec::new();
    let mut start = 0;
    let mut hiding_rules = 0;
    for (at, character) in css.char_indices() {
        match character {
            '{' => {
                selectors.push(css[start..at].trim());
                start = at + 1;
            }
            '}' => {
                let selector = selectors.pop().unwrap();
                let declarations = css[start..at].split_whitespace().collect::<String>();
                if selector.contains("temper-settings-sign-out") && declarations.contains("display:none") {
                    assert!(
                        selectors.contains(&"@media (min-width: 761px)"),
                        "Sign out must hide only over a phone's width: {selector}"
                    );
                    assert_eq!(selector, ".temper-settings-setting.temper-settings-sign-out");
                    hiding_rules += 1;
                }
                start = at + 1;
            }
            _ => {}
        }
    }
    assert_eq!(hiding_rules, 1, "one desktop-only hiding rule");
}

#[test]
fn the_rails_link_state_is_one_word_out_of_the_foot_over_a_phones_width_alone() {
    let css = fs::read_to_string(STYLESHEETS[0]).unwrap();
    let foot = "#app > .temper-rail > .temper-rail-foot";
    // :is() takes its most specific argument: the word's [role=status].temper-behind
    // makes (1, 4, 0), outweighing the item's (1, 3, 0) regardless of source order.
    let item = format!("{foot} > :is(a, [role=status], [data-region=temper-link])");
    let mut item_found = false;
    let rules = [
        (
            " > :is([data-region=temper-link], [role=status].temper-behind)",
            "position:absolute;top:28px;right:20px;display:flex;min-height:24px;padding:0;font-size:13.5px;line-height:24px;",
        ),
        (" > [data-region=temper-link] > *", "gap:6px;"),
        (" > [data-region=temper-link]:has(> .temper-live)", "display:flex;"),
        (":has(> .temper-behind) > [data-region=temper-link]:has(> .temper-live)", "display:none;"),
        (":has(> [data-region=temper-link] > :not(.temper-live)) > .temper-behind", "display:none;"),
        (" > .temper-behind", "gap:6px;"),
        (
            " > .temper-behind::before",
            "content:\"\";width:7px;height:7px;border-radius:50%;flex:none;background:var(--temper-amber);",
        ),
        (
            concat!(
                " > [data-region=temper-link] > :is(.temper-live, .temper-connecting, .temper-offline), ",
                "#app > .temper-rail > .temper-rail-foot > :is([data-region=temper-link], [role=status].temper-behind).temper-behind"
            ),
            "align-items:baseline;",
        ),
        (" > [data-region=temper-link] > .temper-live::before", "width:7px;height:7px;"),
    ];
    let mut found = [false; 9];
    let mut selectors = Vec::new();
    let mut start = 0;
    let mut query = None;
    let mut hiding_rules = 0;
    for (at, character) in css.char_indices() {
        match character {
            '{' => {
                let selector = css[start..at].rsplit("*/").next().unwrap().trim();
                selectors.push((selector, at));
                start = at + 1;
            }
            '}' => {
                let (selector, _) = selectors.pop().unwrap();
                if selector == item {
                    item_found = true;
                }
                let declarations = css[start..at].split_whitespace().collect::<String>();
                for (index, (suffix, expected)) in rules.iter().enumerate() {
                    if selector == format!("{foot}{suffix}") {
                        assert!(!found[index], "one rule for {selector}");
                        found[index] = true;
                        assert_eq!(declarations, *expected, "{selector}");
                        let parent = selectors.last().expect("the word's rules are desktop-only");
                        assert_eq!(parent.0, "@media (min-width: 761px)", "{selector}");
                        assert_eq!(*query.get_or_insert(parent.1), parent.1, "one query for the word's rules");
                    }
                }
                if selector.contains("[data-region=temper-link]:has(> .temper-live)")
                    && declarations.contains("display:none")
                {
                    assert_eq!(
                        selector,
                        format!("{foot}:has(> .temper-behind) > [data-region=temper-link]:has(> .temper-live)"),
                        "Live hides only when Behind replaces it"
                    );
                    hiding_rules += 1;
                }
                start = at + 1;
            }
            _ => {}
        }
    }
    for ((suffix, _), found) in rules.iter().zip(found) {
        assert!(found, "missing {foot}{suffix}");
    }
    assert!(item_found, "the foot's item selector has the pinned lower specificity");
    assert_eq!(hiding_rules, 1, "only Behind replaces Live");
}

#[test]
fn the_chat_log_follows_its_end_until_the_person_scrolls_away() {
    use skein_ui_web::page::{Event as PageEvent, Op};
    use temper_client_domain::{Action, Event, pages::Act};
    let sample = samples::samples().into_iter().find(|s| s.name == "chat-solo-T6").expect("chat sample");
    let mut root = samples::domain(&sample);
    let (mut view, bounds, patches) = built(&sample);
    let log = find(view.tree(), Role::Log, "Conversation", sample.name);
    let mut page = Page::new(
        Look {
            names: &[temper_client_view::look::NAMES],
            pictures: &[],
            components: &[],
            elements: temper_client_view::look::ELEMENTS,
            draw,
        },
        &bounds.tree,
        b"temper",
        Primary::Control,
    );
    let mut ops = Vec::new();
    page.apply(&patches, &mut ops);
    assert!(ops.iter().any(|op| matches!(op, Op::ScrollEnd { node } if *node == log)), "initial chat follows its end");
    for (top, follow, words) in
        [(800, true, b"At the end".as_slice()), (100, false, b"Reading earlier turns"), (800, true, b"Back at the end")]
    {
        ops.clear();
        assert!(page.act(PageEvent::Scrolled { node: log, top, height: 1000, viewport: 200 }, &mut ops).is_none());
        let mut out = Queue::uncapped();
        driver::step(
            &mut root,
            Event::Act { action: Action::Pages(Act::MessageWords { words: words.into() }) },
            &mut out,
        );
        if words == b"At the end" {
            driver::step(&mut root, Event::Act { action: Action::Pages(Act::Send) }, &mut out);
        }
        let mut patches = Queue::with_capacity(bounds.tree.patches);
        temper_client_view::build(&mut view, &root, worlds::sample_now(sample.world, sample.name), &mut patches);
        let patches: Vec<_> = (0..patches.len()).map(|_| patches.pop().expect("queued patch")).collect();
        assert!(!patches.is_empty(), "the chat changes");
        page.apply(&patches, &mut ops);
        assert_eq!(ops.iter().any(|op| matches!(op, Op::ScrollEnd { node } if *node == log)), follow);
    }
}

#[test]
fn home_away_deliveries_draw_the_mock_words_and_off_span() {
    use temper_client_view::look;
    for (name, count, headline, expected_kicker, overturns, landed, overturned) in [
        ("home-decided", 2, "57 changes landed. One thing needs you.", "Tuesday, 08:04", 2, false, false),
        (
            "home-away",
            4,
            "57 changes landed. Four things were decided for you.",
            "While you were away · Mon 23:10 to now",
            4,
            false,
            false,
        ),
        (
            "home-away-one",
            1,
            "57 changes landed. One thing was decided for you.",
            "While you were away · Mon 23:10 to 08:03",
            0,
            true,
            false,
        ),
        (
            "home-away-overturned",
            4,
            "57 changes landed. Four things were decided for you.",
            "While you were away · Mon 23:10 to now",
            2,
            true,
            true,
        ),
    ] {
        let sample = samples::samples().into_iter().find(|s| s.name == name).expect("Home away sample");
        let view = view(&sample);
        let tree = view.tree();
        assert!(
            tree.nodes()
                .iter()
                .any(|n| n.role == (Role::Heading { level: 1 }) && n.name.as_deref() == Some(headline.as_bytes()))
        );
        assert_eq!(tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::DECISION)).count(), count);
        assert!(
            tree.nodes().iter().any(|n| n.name.as_deref() == Some("welcome-stall · 01:05".as_bytes())),
            "the decision task read supplies its title"
        );
        if landed {
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_LATE)
                && n.styles.as_slice().contains(&look::DECISION_LANDED)
                && n.name.as_deref() == Some(b"landed 03:12")));
        }
        let kicker =
            tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::HOME_KICKER)).expect("away kicker");
        assert_eq!(kicker.name.as_deref(), Some(expected_kicker.as_bytes()));
        assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_ALLOWED)
            && n.name.as_deref() == Some("Allowed: implementation choices.".as_bytes())));
        assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_TOOK)
            && n.name.as_deref() == Some(b"In temper serve".as_slice())));
        assert_eq!(
            tree.nodes()
                .iter()
                .filter(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Overturn".as_slice()))
                .count(),
            overturns
        );
        if overturned {
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_LATE)
                && n.name.as_deref() == Some(b"Overturned: it asks you again".as_slice())));
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_OVERTURNED)));
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::DECISION_LATE)
                && n.styles.as_slice().contains(&look::DECISION_LANDED)
                && n.name.as_deref() == Some(b"landed 03:12")));
        }
    }
}

#[test]
fn agent_work_runs_its_line_and_reads_away_from_the_end() {
    let sample = samples::samples().into_iter().find(|sample| sample.name == "agent-work").expect("agent-work");
    assert_eq!(sample.address, temper_client_domain::Address::Task { task: 1 });
    let view = view(&sample);
    let styles = temper_client_view::look::WORK_RUNNING;
    let line = view.tree().nodes().iter().find(|node| node.styles.as_slice().contains(&styles)).expect("running run");
    assert_eq!(line.description.as_deref(), Some(b"checking template 2".as_slice()));
    assert!(!view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&temper_client_view::look::TYPING)));
    let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    assert!(html.contains("temper-agent-page"));
    assert!(html.contains("data-log-away"));
    assert!(html.contains("temper-latest"));
}

#[test]
fn the_browser_draws_each_model_name_once() {
    const ONCE: usize = 1; // One row of each model under the single account.
    let sample = samples::samples().into_iter().find(|s| s.name == "chat-model-open").unwrap();
    let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    assert_eq!(html.matches(">Opus 5.5").count(), ONCE);
    assert_eq!(html.matches(">Sonnet 5.5").count(), ONCE);
}

#[test]
fn a_settings_cards_later_reply_follows_it() {
    for (name, expected) in [
        ("ask-chat-accepted", &["Card 1", "Reply"][..]),
        ("ask-chat-replaced", &["Card 1", "Reply", "Card 2", "Reply"][..]),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).unwrap();
        let view = view(&sample);
        let mut order = Vec::new();
        for node in view.tree().nodes() {
            if let Some(key) = node.key
                && key.owner == temper_client_view::frame::owner()
                && key.kind == 1
                && key.a == 10
            {
                order.push(if key.b == 1 { "Card 1" } else { "Card 2" });
            }
            if node.text.iter().any(|run| run.text.as_ref() == b"Tell me what you would change.") {
                order.push("Reply");
            }
        }
        assert_eq!(order, expected, "{name}");
    }
}

fn finding_home(root: &mut temper_client_domain::Domain) {
    use jig_client::{Down, Readable, Watchable, contract as c};
    use skein_lib::List;
    let mut out = Queue::uncapped();
    crate::driver::step(
        root,
        temper_client_domain::Event::Went {
            place: temper_client_domain::Place::Address(temper_client_domain::Address::AllHome),
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        match crate::driver::down(request) {
            Some(Down::Open { watch, subject: Watchable::Jig(c::Subject::Inbox { .. }) }) => {
                crate::driver::deliver(
                    root,
                    watch,
                    c::Delivery::Inbox(Box::new(temper_fixtures::client_documents::inbox(
                        List::with_capacity(0),
                        0,
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    ))),
                    &mut out,
                );
            }
            Some(Down::Read { read, what: Readable::Jig(c::Read::Inbox { .. }) }) => {
                crate::driver::answer(
                    root,
                    read,
                    c::ReadValue::Inbox(Box::new(c::InboxPage {
                        selection: None,
                        before: None,
                        rows: List::with_capacity(0),
                        after: None,
                        cut: false,
                    })),
                    &mut out,
                );
            }
            Some(_) | None => {}
        }
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "Home and Needs you share the finding's facts and opened forms")]
fn finding_gallery_cards_keep_facts_forms_and_responsive_hooks() {
    use temper_client_domain::{
        Action,
        pages::{
            Act,
            findings::{Act as F, Id},
        },
    };
    use temper_client_view::look;
    const ID: Id = Id { project: 1, number: 7 };
    for sample in samples::samples().into_iter().filter(|sample| sample.name.starts_with("needs-finding")) {
        let mut root = samples::domain(&sample);
        assert_eq!(temper_client_domain::pages::parts::needs_facts(&root, worlds::now(samples::World::Solo)).open, 1);
        let (mut view, _, _) = built_root(&sample, ViewInputs::new(), &root);
        for (role, name) in [
            (Role::Text, "Finding"),
            (Role::Paragraph, "feedback · high · no area"),
            (Role::Paragraph, "Reached you as fabrig’s owner: findings that no area owns come to you."),
            (Role::Paragraph, "Handed on by task 55: “Your area owns the view”"),
            (Role::Button, "Backlog"),
        ] {
            find(view.tree(), role, name, sample.name);
        }
        assert!(view.tree().nodes().iter().any(|node| {
            node.name
                .as_deref()
                .is_some_and(|name| name.starts_with("fabrig › 1/7 The phone card is clipped".as_bytes()))
        }));
        // The browser's single tree carries both the laptop list and the phone's one-card bar.
        for style in [look::ASK, look::ASK_CHOICES, look::NEEDS_BAR, look::NEEDS_HEAD] {
            assert!(
                view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&style)),
                "{}: responsive hook {style:?}",
                sample.name
            );
        }
        let backlog = find(view.tree(), Role::Button, "Backlog", sample.name);
        assert_eq!(
            temper_client_view::act(&mut view, backlog, skein_ui::Act::Press),
            Some(Action::Pages(Act::Finding(F::Backlog { id: ID })))
        );
        if sample.name == "needs-finding-hand" {
            find(view.tree(), Role::Button, "view", sample.name);
            let field = find(view.tree(), Role::TextField, "Why hand it on", sample.name);
            assert_eq!(
                temper_client_view::act(
                    &mut view,
                    field,
                    skein_ui::Act::Input { value: b"The view owns this".as_slice().into() }
                ),
                Some(Action::Pages(Act::Finding(F::Why { id: ID, words: b"The view owns this".as_slice().into() })))
            );
        }
        if sample.name == "needs-finding-drop" {
            let field = find(view.tree(), Role::TextField, "Reason to drop", sample.name);
            assert_eq!(
                temper_client_view::act(
                    &mut view,
                    field,
                    skein_ui::Act::Input { value: b"Duplicate".as_slice().into() }
                ),
                Some(Action::Pages(Act::Finding(F::Reason { id: ID, words: b"Duplicate".as_slice().into() })))
            );
        }
        if sample.name == "needs-finding-route" {
            assert!(
                !root.pages().needs().findings().goals(ID).is_empty(),
                "the live-goal menu has the project's goals"
            );
        }
        if sample.name == "needs-finding-refused" {
            find(view.tree(), Role::Paragraph, "not a live goal of the project", sample.name);
        }
        finding_home(&mut root);
        let (mut home, _, _) = built_root(&sample, ViewInputs::new(), &root);
        for style in [
            look::NEED_CARD,
            look::NEED_FROM,
            look::FINDING_ROWS,
            look::FINDING_WHEN,
            look::NEED_ACTS,
            look::NEED_MORE,
            look::WHY,
        ] {
            assert!(
                home.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&style)),
                "{}: Home hook {style:?}",
                sample.name
            );
        }
        let more = find(home.tree(), Role::Button, "More", sample.name);
        assert_eq!(
            temper_client_view::act(&mut home, more, skein_ui::Act::Press),
            Some(Action::Pages(Act::Finding(F::Open { id: ID })))
        );
        find(home.tree(), Role::Link, "The phone card is clipped", sample.name);
        if sample.name == "needs-finding-hand" {
            find(home.tree(), Role::TextField, "Why hand it on", sample.name);
        }
        if sample.name == "needs-finding-drop" {
            find(home.tree(), Role::TextField, "Reason to drop", sample.name);
        }
        if sample.name == "needs-finding-route" {
            find(home.tree(), Role::Disclosure, "Goal", sample.name);
        }
    }
}

#[test]
fn role_talk_shows_provers_open_talk_sent_words_reply_and_leave() {
    use temper_client_view::look;
    let sample = samples::samples().into_iter().find(|sample| sample.name == "role-talk").expect("role-talk");
    assert_eq!(sample.address, temper_client_domain::Address::Task { task: 52 });
    assert!(!samples::samples().iter().any(|sample| sample.name == "night-role-talk"));
    let root = samples::domain(&sample);
    let task = root.pages().task().expect("role's page").document(root.client()).expect("task");
    assert_eq!(task.flags.ended_by, jig_client::contract::EndedBy::Standing);
    assert_eq!(task.talk.as_ref().unwrap().party, root.pages().me(root.client()).unwrap().party.number);
    let view = view(&sample);
    let tree = view.tree();
    find(tree, Role::Heading { level: 1 }, "T52 · Prover", sample.name);
    find(tree, Role::TextArea, "Talk", sample.name);
    find(tree, Role::Button, "Leave", sample.name);
    find(tree, Role::Status, "Sent", sample.name);
    let said: Vec<u8> =
        tree.nodes().iter().flat_map(|node| node.text.iter().flat_map(|run| run.text.iter().copied())).collect();
    let said = String::from_utf8(said).unwrap();
    for words in [
        "From now on, reproduce a flake twice before routing it.",
        "Add it to my rules?",
        "Talk with Prover · it answers here, and the talk stays open until you leave or 30 minutes pass with nothing said.",
        "Or in a terminal: temper ctl DIR role talk 52 TEXT --key KEY",
    ] {
        assert!(said.contains(words), "missing {words}: {said}");
    }
    let log = tree.nodes().iter().position(|node| node.role == Role::Log).unwrap();
    let reply = tree
        .nodes()
        .iter()
        .position(|node| node.text.iter().any(|run| run.text.as_ref() == b"Add it to my rules?"))
        .unwrap();
    assert!(reply > log && reply < log + usize::try_from(tree.nodes()[log].size).unwrap());
    assert!(tree.nodes().iter().any(|node| node.styles.as_slice().contains(&look::AGENT_PAGE)));
    let html = page(&sample, &skein_ui_bundle::Digest(*b"0123456789abcdef"));
    assert!(html.contains("Talk to Prover"));
    assert!(html.contains("temper-agent-page"));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the overlay table holds the shared head's declarations and each page's scope"
)]
fn asking_ways_head_keeps_the_requested_overlay_rules() {
    let css = fs::read_to_string(STYLESHEETS[0]).unwrap();
    let scope = "#app > .temper-session ";
    let rules = [
        (".temper-starter > .temper-way", "margin-top:18px;", None),
        (
            ".temper-way-head",
            "width:100%;display:flex;flex-wrap:wrap;align-items:center;justify-content:space-between;gap:10px16px;",
            None,
        ),
        (
            ".temper-way-line",
            "flex:11auto;min-width:0;margin:0;font-size:15px;line-height:1.55;color:var(--gf-ink-2);",
            None,
        ),
        (".temper-way-acts", "display:flex;gap:8px;margin-left:auto;", None),
        (
            ".temper-way-acts > button",
            "min-height:34px;min-width:84px;padding:4px14px;border:1pxsolidvar(--gf-line-2);border-radius:8px;background:var(--gf-surface);color:var(--gf-ink);font:50013.5px/1.6var(--gf-sans);white-space:nowrap;cursor:pointer;box-shadow:none;",
            None,
        ),
        (
            ".temper-way-acts > .temper-ways-use",
            "border-color:var(--gf-ink);background:var(--gf-ink);color:var(--gf-paper);",
            None,
        ),
        (".temper-way-pick", "position:relative;", None),
        (
            ".temper-way-pick > button[aria-haspopup=menu]",
            "min-height:34px;padding:4px12px4px14px;border:1pxsolidvar(--gf-line-2);border-radius:8px;background:var(--gf-surface);font:50013.5px/1.6var(--gf-sans);white-space:nowrap;cursor:pointer;box-shadow:none;display:inline-flex;align-items:center;gap:10px;color:var(--gf-ink-2);",
            None,
        ),
        (
            ".temper-way-pick > button[aria-haspopup=menu]::after",
            "content:''/'';width:6px;height:6px;margin-top:-3px;border:solidvar(--gf-muted);border-width:01.5px1.5px0;transform:rotate(45deg);",
            None,
        ),
        (".temper-way-pick > button[aria-haspopup=menu][aria-expanded=true]", "background:var(--gf-hover);", None),
        (
            ".temper-way-menu",
            "position:absolute;top:calc(100%+6px);right:0;z-index:10;min-width:100%;width:max-content;margin:0;padding:6px;list-style:none;border:1pxsolidvar(--gf-line);border-radius:12px;background:var(--gf-surface);box-shadow:012px32pxvar(--gf-menu-shadow);",
            None,
        ),
        (
            ".temper-way-menu [role=menuitem]",
            "display:flex;align-items:center;gap:16px;width:100%;min-height:0;margin:0;padding:6px10px;border:0;border-radius:8px;background:transparent;color:var(--gf-ink);font:40014px/1.5var(--gf-sans);text-align:left;cursor:pointer;box-shadow:none;",
            None,
        ),
        (".temper-way-menu [role=menuitem]:hover", "background:var(--gf-hover);", None),
        (".temper-way-menu [role=menuitem][aria-selected=true]", "font-weight:500;", None),
        (
            ".temper-way-menu [role=menuitem][aria-selected=true]::after",
            "content:'✓'/'';margin-left:auto;color:var(--gf-good,var(--gf-control-good));",
            None,
        ),
        (".temper-way > .temper-ways-panel", "margin-top:14px;", None),
        (
            ".temper-way-ask",
            "width:100%;display:flex;flex-wrap:wrap;align-items:center;gap:6px10px;margin:22px00;padding-top:16px;border-top:1pxsolidvar(--gf-line);font-size:14px;line-height:1.55;color:var(--gf-ink-2);",
            None,
        ),
        (".temper-way-ask-mark", "flex:none;color:var(--gf-ink);", None),
        (".temper-way-acts", "margin-left:0;", Some("@media (max-width: 760px)")),
    ];
    let mut found = vec![false; rules.len()];
    let mut parents = Vec::new();
    let mut start = 0;
    for (at, character) in css.char_indices() {
        match character {
            '{' => {
                let selector = css[start..at].rsplit("*/").next().unwrap().trim();
                parents.push(selector);
                start = at + 1;
            }
            '}' => {
                let selector = parents.pop().unwrap();
                let declarations = css[start..at].split_whitespace().collect::<String>();
                for (index, (suffix, expected, parent)) in rules.iter().enumerate() {
                    let ask = format!("{scope}{suffix}");
                    if selector.split(',').any(|s| s.trim() == ask) && parents.last().copied() == *parent {
                        if !matches!(
                            *suffix,
                            ".temper-starter > .temper-way" | ".temper-way-ask" | ".temper-way-ask-mark"
                        ) {
                            let how_suffix = if *suffix == ".temper-way > .temper-ways-panel" {
                                ".temper-ways-panel"
                            } else {
                                suffix
                            };
                            let how = format!("#app > .temper-settings {how_suffix}");
                            assert!(selector.split(',').any(|s| s.trim() == how), "shared head rule: {selector}");
                        }
                        assert!(!found[index], "one rule for {selector} under {parent:?}");
                        found[index] = true;
                        assert_eq!(declarations, *expected, "{selector}");
                    }
                }
                start = at + 1;
            }
            _ => {}
        }
    }
    for ((suffix, _, parent), found) in rules.iter().zip(found) {
        assert!(found, "missing {scope}{suffix} under {parent:?}");
    }
    assert!(!css.contains("#app > .temper-session .temper-ways-tabs"));
    assert!(!css.contains("#app > .temper-session .temper-ways-acts > .temper-ways-use"));
    assert!(!css.contains(".temper-ways-tabs"));
    assert!(!css.contains(".temper-ways-axis"));
    assert!(!css.contains(".temper-ways-now"));
    assert!(!css.contains("#app > .temper-settings .temper-ways-acts"));
    assert!(css.contains("html:root[data-night] #app > .temper-settings .temper-way-acts > .temper-ways-use"));
    assert!(css.contains("html:root[data-night] #app > .temper-session .temper-way-acts > .temper-ways-use"));
}
