use super::fixtures;

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "cli C4: SKEIN_CODEGEN_WRITE selects regeneration of the test's checked document or golden; the test table will pass it"
)]
fn each_adoption_outcome_prints_as_its_golden_file() {
    use temper_client_domain::forge;
    let words = fixtures::page_words();
    let filled = fixtures::adoption_form(true);
    let mut adopted = filled.clone();
    adopted.outcome = Some(forge::Answer::Adopted(fixtures::adopted(forge::Protection::Unknown)));
    let mut not_found = filled.clone();
    not_found.outcome = Some(forge::Answer::ForgeRefused(forge::Reason::NotFound));
    let mut core = filled.clone();
    core.outcome = Some(forge::Answer::CoreRefused(fixtures::role_refusal()));
    for (name, form) in
        [("adopt_adopted", adopted), ("adopt_refused_not_found", not_found), ("adopt_refused_core", core)]
    {
        let tree = fixtures::adoption_page(&form, &words);
        let print = fixtures::print(&tree);
        if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some() {
            std::fs::write(fixtures::golden(name), print.text.as_slice()).expect("write adoption golden file");
        }
        let golden = std::fs::read(fixtures::golden(name)).expect("adoption golden file");
        assert_eq!(print.text.as_slice(), golden, "page {name}");
    }
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "cli C4: SKEIN_CODEGEN_WRITE selects regeneration of the test's checked document or golden; the test table will pass it"
)]
fn each_fixture_page_prints_as_its_golden_file() {
    for (name, tree) in [
        ("settings", fixtures::chat::settings()),
        ("models", fixtures::page(&temper_client_domain::Address::Models { project: 1 }, false)),
        ("sheet_adopt", fixtures::page(&temper_client_domain::Address::Adopt { project: 1 }, false)),
        ("builders", fixtures::page(&temper_client_domain::Address::Builders { project: 1 }, false)),
        ("changes", fixtures::changes_page()),
        ("team", fixtures::team_page()),
        ("inbox", fixtures::inbox_page()),
        ("chats", fixtures::chats_page()),
        ("chat", fixtures::chat::page()),
        ("chat_closing", fixtures::chat::closing_page()),
        ("chat_delegation_running", fixtures::chat::delegation_page(false)),
        ("chat_delegation_landed", fixtures::chat::delegation_page(true)),
        ("first_run", fixtures::chat::first_run()),
        ("task", fixtures::task_page()),
        ("goals", fixtures::goals::page()),
        ("agent_task", fixtures::agent_task_page()),
        ("signed_out", fixtures::signed_out_page()),
        ("every_role", fixtures::every_role()),
        ("control_text", fixtures::control_text()),
    ] {
        let print = fixtures::print(&tree);
        if name == "sheet_adopt" {
            let list = tree
                .nodes()
                .iter()
                .find(|node| node.role == skein_ui::Role::List && node.name.as_deref() == Some(b"Repositories"))
                .unwrap();
            assert_eq!(
                skein_ui_text::command(b"down", Some(print.numbers.as_slice()), &tree, u32::MAX),
                skein_ui_text::Command::Act {
                    node: list.id,
                    first: skein_ui::Act::Near { edge: skein_ui::LogEdge::Foot },
                    submit: false
                }
            );
            let owner = tree
                .nodes()
                .iter()
                .find(|node| node.role == skein_ui::Role::TextField && node.name.as_deref() == Some(b"Owner"))
                .unwrap();
            assert!(print.numbers.as_slice().contains(&owner.id));
        }

        if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some() {
            std::fs::write(fixtures::golden(name), print.text.as_slice()).expect("write golden file");
        }
        let golden = std::fs::read(fixtures::golden(name)).expect("golden file");
        assert_eq!(String::from_utf8_lossy(print.text.as_slice()), String::from_utf8_lossy(&golden), "page {name}");
    }
}
#[test]
fn the_first_line_is_the_title_and_the_link_state() {
    let print = fixtures::print(&fixtures::header_page(Some(b"Inbox"), false));
    let text = String::from_utf8(print.text.as_slice().to_vec()).expect("printable UTF-8");
    assert_eq!(text.lines().next(), Some("== Inbox == Live"));
    assert_eq!(text.matches("Live").count(), 1, "link appears only in header");
    assert_eq!(text.matches("Inbox").count(), 2, "the title and temper navigation each name the inbox");
}
#[test]
fn a_behind_frame_adds_its_sentence_to_the_first_line() {
    let print = fixtures::print(&fixtures::header_page(Some(b"Inbox"), true));
    let text = String::from_utf8(print.text.as_slice().to_vec()).expect("printable UTF-8");
    assert_eq!(text.lines().next(), Some("== Inbox == Live; Behind: waiting for current facts"));
    assert_eq!(text.matches("Behind: waiting for current facts").count(), 1);
    assert!(text.contains("Pat"), "identity is separate");
}
#[test]
fn the_largest_fixture_tree_prints_within_its_bounds() {
    let limits = fixtures::tree_limits();
    let (lines, bytes) = skein_ui_text::render_bound(&limits).expect("fixture print envelope");
    for (name, tree) in [("roles", fixtures::every_role()), ("task", fixtures::task_page())] {
        let print = fixtures::print(&tree);
        assert!(u64::from(print.text.len()) <= bytes, "{name}: bytes");
        assert!(skein_lib::bytes::count(print.text.as_slice(), b"\n", u32::MAX) <= lines, "{name}: lines");
        assert!(print.numbers.len() <= limits.nodes, "{name}: numbers");
        assert!(skein_ui_text::render_bound(&limits).is_some(), "{name}: memory bound");
    }
    let native = crate::limits::view(&crate::limits::native()).expect("native view bounds").0;
    assert!(skein_ui_text::render_bound(&native.tree).is_some(), "native view has a print envelope: {:?}", native.tree);
}

#[test]
fn account_fields_print_hidden() {
    let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
    builder.open(skein_ui::Role::TextField);
    builder.name(b"Key (pasted once)");
    builder.entry(skein_ui::EntryKind::Secret);
    builder.value(b"*", 1);
    builder.close();
    let print = fixtures::print(&builder.finish());
    assert!(print.text.as_slice().windows(b"(hidden)".len()).any(|part| part == b"(hidden)"));
}
#[test]
fn account_sign_in_links_print_the_target() {
    const ADDRESS: &[u8] = b"https://issuer.example/visit";
    let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
    builder.open(skein_ui::Role::Link);
    builder.text(skein_ui::Run::Plain, b"Open the sign-in page");
    builder.external(ADDRESS);
    builder.close();
    let print = fixtures::print(&builder.finish());
    let expected = b"Open the sign-in page <https://issuer.example/visit>";
    assert!(print.text.as_slice().windows(expected.len()).any(|part| part == expected));
}

#[test]
fn the_first_run_prints_settings_and_the_playground_banner() {
    let print = fixtures::print(&fixtures::chat::first_run());
    let text = String::from_utf8_lossy(print.text.as_slice());
    assert!(text.contains("] Deployment\n"));
    assert!(text.contains("Use the wizard to set up temper"));
}

#[test]
fn no_project_reads_the_same_in_the_terminal() {
    use jig_client::contract::{Delivery, Subject, WatchEvent};
    use skein_lib::{Env, List, Queue, Time};
    use temper_client_domain::{self as client, Event, Request};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let limits = crate::limits::native().client;
    let env = Env { now: Time::ZERO, wall: fixtures::NOW, limits };
    let mut root = client::unlinked(&limits, 16);
    let mut out = Queue::uncapped();
    for event in [
        Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
        Event::Start {
            address: client::Address::Chats { filter: client::ChatsFilter::All },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } } },
    ] {
        client::step(&mut root, &env, event, &mut out);
    }
    while let Some(request) = out.pop() {
        let up = match request {
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(Subject::Me) },
                    },
            } => {
                let mut me = jig_client_kit::stories::frame(1);
                me.projects.clear();
                Some(jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(me))) })
            }
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } if wire::decode_reading(&record.bytes, &w::CEILINGS).expect("forge read").read()
                == &w::Read::Deployment =>
            {
                let answer = w::DeploymentAnswer::new(
                    &w::CEILINGS,
                    w::DeploymentAnswerParts { push: None,  welcome: false, owned: false, program_login: None },
                )
                .expect("deployment answer");
                Some(jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::deployment_answer(&answer)),
                    },
                })
            }
            Request::Client { .. }
            | Request::Unlinked { .. }
            | Request::Show { .. }
            | Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::KeepSignIn { .. }
            | Request::Tab { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Copy { .. }
            | Request::Fact { .. } => None,
        };
        if let Some(up) = up {
            client::step(&mut root, &env, Event::Client { event: jig_client::Event::Up { up } }, &mut out);
        }
    }
    let mut view = temper_client_view::View::new(
        &temper_client_view::Limits { tree: fixtures::tree_limits() },
        &limits,
        fixtures::page_words(),
        wire::FORGE,
    );
    temper_client_view::build(
        &mut view,
        &root,
        fixtures::NOW,
        &mut Queue::with_capacity(fixtures::tree_limits().patches),
    );
    let print = fixtures::print(view.tree());
    let text = String::from_utf8_lossy(print.text.as_slice());
    assert_eq!(text.lines().next(), Some("== No project yet == Live"));
    assert!(text.lines().any(|line| line == "Make your first one, or ask an owner to add you to theirs."), "{text}");
    assert!(text.contains("Project name"), "{text}");
    assert!(text.contains("Create"), "{text}");
    assert!(!text.contains("Set up temper"), "{text}");
    assert!(!text.contains("Try the playground"), "{text}");
}

#[test]
fn a_chats_or_agents_long_line_draws_whole() {
    use jig_client::contract as c;
    use temper_client_domain::Address;
    for length in [1149, 10_660] {
        let bytes = vec![b'x'; length];
        for address in [Address::Chat { task: 1 }, Address::Task { task: 1 }] {
            let tree = fixtures::chat::long_line_page(
                address,
                &c::Activity::Words {
                    message: 1,
                    by: c::Requester::Person(1),
                    kind: c::MessageKind::Words,
                    words: temper_fixtures::first_run::text(&bytes),
                },
            );
            assert_long_line(&tree, &bytes);
        }
    }
}

fn assert_long_line(tree: &skein_ui::Tree, bytes: &[u8]) {
    assert!(tree.nodes().iter().any(|node| {
        node.text.as_slice().iter().flat_map(|run| run.text.iter().copied()).eq(bytes.iter().copied())
    }));
    let limits = crate::limits::view(&crate::limits::native()).expect("native bounds").0;
    let mut print = skein_ui_text::Print::new(&limits.tree).expect("native print bounds");
    skein_ui_text::render(tree, b"temper", None, &mut print);
    assert!(skein_lib::bytes::find(print.text.as_slice(), bytes).is_some(), "whole words in the print");
}

#[test]
fn an_agents_heard_or_notified_long_word_draws_whole() {
    use jig_client::contract as c;
    for length in [1149, 10_660] {
        let bytes = vec![b'x'; length];
        let word = c::Word {
            number: 1,
            from: c::Requester::Task(2),
            kind: c::MessageKind::Words,
            words: temper_fixtures::first_run::text(&bytes),
            at: skein_lib::Wall::EPOCH,
            hits: 1,
            eligible: true,
        };
        for event in
            [c::Activity::Heard(word.clone()), c::Activity::Notified { recipient: c::Requester::Person(1), word }]
        {
            let tree = fixtures::chat::long_line_page(temper_client_domain::Address::Task { task: 1 }, &event);
            assert_long_line(&tree, &bytes);
        }
    }
}

#[test]
fn a_composer_past_the_bound_prints_its_line() {
    let words = vec![b'x'; 8193];
    let tree = fixtures::chat::composer_over_page(&words);
    let mut print = skein_ui_text::Print::new(&crate::limits::view(&crate::limits::native()).unwrap().0.tree).unwrap();
    skein_ui_text::render(&tree, b"temper", None, &mut print);
    let printed = String::from_utf8_lossy(print.text.as_slice());
    assert!(printed.contains("That's 1 character over what one message holds."));
    assert!(printed.contains(String::from_utf8_lossy(&words).as_ref()));
}

/// The Models print reads a supplied offer, including an off row and the chat default.
pub(super) fn models_frame(
    address: &temper_client_domain::Address,
    mut me: jig_client::contract::Me,
) -> jig_client::contract::Me {
    use jig_client::contract as c;
    use skein_lib::List;
    if !matches!(address, temper_client_domain::Address::Models { .. }) {
        return me;
    }
    let text = |value: &str| c::Text::new(value.as_bytes().into(), 256).unwrap();
    me.providers = List::uncapped();
    for (provider, name) in [(17, "anthropic"), (18, "openai")] {
        me.providers
            .push(c::ProviderOffer {
                provider,
                name: c::DocumentText::complete(text(name)),
                key_bytes: 4096,
                sign_in: true,
                key: true,
                programs: List::with_capacity(0),
                programs_cut: false,
                none: false,
                signed_in_plan: true,
                manage_usage: None,
            })
            .unwrap();
    }
    let project = me.projects.get_mut(0).unwrap();
    project.models = List::uncapped();
    for (provider, name, shown, picker) in [
        (17, "claude-opus-5-5", "Opus 5.5", true),
        (17, "claude-sonnet-5-5", "Sonnet 5.5", false),
        (18, "gpt-6.1-sol", "GPT-6.1 Sol", true),
    ] {
        project
            .models
            .push(c::MeModel {
                new_until: if provider == 18 { Some(skein_lib::Wall::from_nanos(604_800_000_000_000)) } else { None },
                number: 1,
                source: if provider == 18 { c::ModelSource::List } else { c::ModelSource::Stock },
                estimated: false,
                name: text(name),
                provider: Some(provider),
                window: 200_000,
                output: 16_384,
                efforts: List::from_boxed(Box::new([text("low"), text("high")])),
                effort: Some(text("high")),
                shown: Some(text(shown)),
                accounts: List::with_capacity(0),
                picker,
                description: None,
                prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 5, unit: 1000 },
            })
            .unwrap();
    }
    project.defaults = List::from_boxed(Box::new([c::MeDefault {
        template: jig_client::shapes::CHAT,
        choice: c::LlmChoice {
            later: None,
            account: None,
            main: c::ModelEffort { model: text("claude-opus-5-5"), effort: Some(text("high")) },
            subagents: List::with_capacity(0),
        },
    }]));
    me
}

pub(super) fn models_accounts() -> jig_client::contract::Accounts {
    use jig_client::contract as c;
    use skein_lib::List;
    let text = |value: &str| c::DocumentText::complete(c::Text::new(value.as_bytes().into(), 256).unwrap());
    c::Accounts {
        program_hosts: List::with_capacity(0),
        readiness: List::with_capacity(0),
        readiness_cut: false,
        signing: List::with_capacity(0),
        last_end: None,
        usable: true,
        settled: true,
        cut: false,
        accounts: List::from_boxed(Box::new([c::AccountLine {
            facts: List::with_capacity(0),
            list: Some(c::AccountList {
                read: Some(fixtures::NOW),
                tried: Some(fixtures::NOW),
                failure: None,
                left_out: 0,
            }),
            number: 1,
            place: 1,
            uses: None,
            plan: true,
            provider: Some(18),
            provider_name: Some(text("openai")),
            manage_usage: None,
            display: text("Own"),
            holder: c::AccountHolder::Party(1),
            cut: false,
            sources: List::from_boxed(Box::new([c::AccountSource {
                name: None,
                place: 1,
                number: 1,
                kind: c::AccountKind::SignIn,
                state: c::AccountState::Granted,
                usable: true,
                expiring: false,
                attention: false,
                spent_until: None,
            }])),
        }])),
    }
}
