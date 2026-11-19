use super::*;
use skein_scratch::Scratch;
use temper_client_view_kit::readers::{has_text_run as has_text, require_named as find};

#[test]
fn usage_gallery_states_hold_windows_at_their_fixed_now() {
    use temper_client_view::look;
    for (name, meters, pace, colour) in [
        ("accounts-usage", &["Week"][..], "resets in 4 d 14 h · 0.5%/h · about 84% at reset", None),
        ("accounts-usage-filling", &["5 h", "Week"][..], "full in 1 h", Some(look::USAGE_WARNED)),
        ("accounts-usage-calm", &["Week"][..], "resets in 2 d · 0.4%/h · about 59% at reset", None),
        ("accounts-usage-limited", &["5 h"][..], "resets in 40 min", Some(look::USAGE_LIMITED)),
    ] {
        let sample = samples::samples().into_iter().find(|sample| sample.name == name).unwrap();
        let root = samples::domain(&sample);
        let doc = temper_client_domain::pages::accounts::document(root.client()).unwrap();
        let account = doc.accounts.get(0).unwrap();
        assert_eq!(account.provider_name.as_ref().unwrap().bytes(), b"anthropic");
        let mut facts = 0;
        for fact in &account.facts {
            if fact.kind == jig_client::contract::FACT_USAGE {
                let jig_client::contract::AccountFactValue::Window(window) = fact.value else { panic!("window fact") };
                assert_eq!(fact.seen, sample.now());
                assert!(window.resets.unwrap() > sample.now());
                facts += 1;
            }
        }
        assert!(facts > 0);
        let view = view(&sample);
        let actual: Vec<_> = view
            .tree()
            .nodes()
            .iter()
            .filter(|node| node.role == Role::Meter)
            .map(|node| String::from_utf8_lossy(node.name.as_deref().unwrap()))
            .collect();
        assert_eq!(actual, meters, "{name}: only the windows that should show, shortest first");
        assert!(has_text(view.tree(), pace), "{name}: {pace}");
        if let Some(colour) = colour {
            assert!(view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&colour)), "{name}: colour");
        }
    }
}

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
                0
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
            "min-height:34px;padding:4px12px4px14px;border:1pxsolidvar(--gf-line-2);border-radius:8px;background:var(--gf-surface);font:50013.5px/1.6var(--gf-sans);white-space:nowrap;cursor:pointer;box-shadow:none;display:inline-flex;align-items:center;gap:10px;color:var(--gf-ink);",
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
        (
            ".temper-way-menu [role=menuitem] > [id^=\"skein-ui-description-\"]",
            "margin-left:auto;color:var(--gf-faint);font-size:12.5px;",
            None,
        ),
        (
            ".temper-way-menu [role=menuitem][aria-selected=true]:has(> [id^=\"skein-ui-description-\"])::after",
            "margin-left:10px;",
            None,
        ),
        (".temper-starter:has(> .temper-ways)", "position:relative;", None),
        (".temper-starter:has(> .temper-ways) > .temper-ways-lede", "padding-right:260px;", None),
        (".temper-starter > .temper-ways .temper-way-acts", "position:absolute;top:-3px;right:0;margin:0;", None),
        (
            ".temper-starter:has(> .temper-ways) > .temper-ways-lede",
            "padding-right:0;",
            Some("@media (max-width: 760px)"),
        ),
        (".temper-starter > .temper-ways .temper-way-acts", "position:static;", Some("@media (max-width: 760px)")),
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
                            let how_suffix = match *suffix {
                                ".temper-way > .temper-ways-panel" => ".temper-ways-panel",
                                ".temper-starter:has(> .temper-ways)" => {
                                    ".temper-settings-section:has(> .temper-way-head)"
                                }
                                ".temper-starter:has(> .temper-ways) > .temper-ways-lede" => {
                                    ".temper-settings-section:has(> .temper-way-head) > .temper-settings-section-head"
                                }
                                ".temper-starter > .temper-ways .temper-way-acts" => {
                                    ".temper-settings-section > .temper-way-head > .temper-way-acts"
                                }
                                _ => suffix,
                            };
                            let how = format!("#app > .temper-settings {how_suffix}");
                            if suffix.starts_with(".temper-starter") && parent.is_none() {
                                let expected_how = expected.replace("top:-3px", "top:-6px");
                                let start = css.find(&format!("{how} {{")).expect("How work runs' rule");
                                let body = css[start..].split_once('{').unwrap().1.split_once('}').unwrap().0;
                                assert_eq!(body.split_whitespace().collect::<String>(), expected_how, "{how}");
                            } else {
                                assert!(selector.split(',').any(|s| s.trim() == how), "shared head rule: {selector}");
                            }
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
