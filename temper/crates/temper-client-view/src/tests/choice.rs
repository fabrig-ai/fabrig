#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic trees and allocate maximal model-name fixtures"
)]
use super::fixtures;
pub(super) use super::fixtures::choice::{
    NAME_BYTES, account, delivered, line_fixture, line_models, model_text, no_model_root, offer_row, picker_fixture,
    playground_offers_root, provider_byline_root, provider_offer, source, wait_pointers,
};
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Builder, Role};
use temper_client_domain::{self as d, Address};

fn text(root: &d::Domain) -> String {
    let view = fixtures::page_view(root);
    let mut text = String::from_utf8(fixtures::text(view.tree()).into_vec()).unwrap();
    for node in view.tree().nodes() {
        if let Some(description) = &node.description {
            text.push_str(core::str::from_utf8(description).unwrap());
        }
    }
    text
}
#[test]
fn the_accounts_banner_precedes_main() {
    let view = fixtures::page_view(&banner(Some(false)).0);
    let nodes = view.tree().nodes();
    let banner = nodes.iter().position(|n| n.name.as_deref() == Some(b"Agents need an LLM account to run.")).unwrap();
    assert!(nodes.iter().position(|n| n.role == Role::Navigation).unwrap() < banner);
    assert!(banner < nodes.iter().position(|n| n.role == Role::Main).unwrap());
}
#[test]
fn usable_accounts_remove_the_banner() {
    assert!(!text(&banner(Some(true)).0).contains("Agents need"));
}
#[test]
fn unknown_accounts_have_no_banner() {
    assert!(!text(&banner(None).0).contains("Agents need"));
}
#[test]
fn a_behind_accounts_document_removes_the_banner() {
    let (mut root, watch) = banner(Some(false));
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::Dropped } },
        },
    );
    assert!(!text(&root).contains("Agents need"));
}
#[test]
fn missing_accounts_leave_the_new_chat_composer_available() {
    assert!(fixtures::find(fixtures::page_view(&banner(Some(false)).0).tree(), Role::Button, b"Send").is_some());
}
#[test]
fn the_accounts_page_has_no_accounts_banner() {
    let (mut root, _) = banner(Some(false));
    fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Accounts) });
    assert!(!text(&root).contains("Agents need"));
}
#[test]
fn an_absent_choice_uses_the_unsettled_default() {
    assert!(text(&banner(None).0).contains("the default model"));
}

#[test]
fn the_picker_closed_line_names_no_account() {
    for address in [Address::NewChat { draft: false }, Address::Ask { project: fixtures::PROJECT }] {
        let (root, _) = provider_byline_root(address, [c::Readiness::Usable, c::Readiness::Usable], [true, true], true);
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let node = view
                .tree()
                .nodes()
                .iter()
                .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Model"))
                .unwrap();
            assert_eq!(node.description.as_deref(), Some(b"gpt-6.1-sol".as_slice()));
            assert!(!String::from_utf8_lossy(node.description.as_deref().unwrap()).contains("plan"));
        }
    }
}

const EFFORT_BYTES: u32 = jig_client_protocol_wire::v21::CEILINGS.me_model_efforts_item;
const CUT_MARK: &[u8] = "…".as_bytes();

fn catalogue(name: &[u8], efforts: &[&[u8]]) -> d::Domain {
    let mut declared = List::with_capacity(u32::try_from(efforts.len()).unwrap());
    for effort in efforts {
        declared.push(c::Text::new(Box::from(*effort), EFFORT_BYTES).unwrap()).unwrap();
    }
    let mut models = List::with_capacity(1);
    let mut model = offer_row(name, None);
    model.effort = declared.get(0).cloned();
    model.efforts = declared;
    models.push(model).unwrap();
    catalogue_rows(models)
}
fn catalogue_rows(models: List<c::MeModel>) -> d::Domain {
    let (mut root, me_watch, _) = fixtures::root_at(Address::NewChat { draft: false }, &fixtures::limits());
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let project = me.projects.get_mut(0).unwrap();
    project.templates = List::with_capacity(1);
    project
        .templates
        .push(c::Template {
            resources: List::with_capacity(0),
            number: 1,
            name: c::DocumentText::complete(c::Text::new(Box::from(*b"Chat"), 32).unwrap()),
            executor: c::TemplateExecutor::Charter(1),
            contract: c::TaskContractDocument::Report { words: 32 },
            shallower: 1,
            budget: c::TemplateBudget::AsAllowed { cap: None },
            deadline: c::TemplateDeadline::None,
            tracked: None,
            flags: c::TaskFlags {
                ended_by: jig_client::contract::EndedBy::Result,
                conversation: true,
                listed: false,
                retention: c::Retention::Kept,
            },
            label: 7,
            grace: None,
            llm: None,
        })
        .unwrap();
    project.models = models;
    me.providers = List::with_capacity(2);
    for (number, name) in [(17, b"First".as_slice()), (18, b"Second".as_slice())] {
        let provider = provider_offer(number, name);
        me.providers.push(provider).unwrap();
    }
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    root
}
fn catalogue_view(root: &d::Domain) -> crate::View {
    let mut tree = fixtures::tree_limits();
    tree.text_per_node = 1024;
    let mut view = crate::View::new(&crate::Limits { tree }, &fixtures::limits(), model_words(), fixtures::FORGE);
    fixtures::build(&mut view, root);
    view
}
fn shows_requested(view: &crate::View, label: &[u8]) {
    assert!(fixtures::find(view.tree(), Role::Button, label).is_some());
    let byline = fixtures::find(view.tree(), Role::Disclosure, b"Model").unwrap();
    assert_eq!(view.tree().nodes().iter().find(|n| n.id == byline).unwrap().description.as_deref(), Some(label));
}
fn marked_prefix() -> Vec<u8> {
    [vec![b'm'; prefix_bytes()], CUT_MARK.to_vec()].concat()
}
#[test]
fn a_long_model_name_shows_a_marked_prefix() {
    let mut root = catalogue(&vec![b'm'; usize::try_from(NAME_BYTES).unwrap()], &[]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    shows_requested(&catalogue_view(&root), &marked_prefix());
}
#[test]
fn a_name_at_the_display_bound_shows_whole() {
    let name = vec![b'm'; shown_bytes()];
    let mut root = catalogue(&name, &[]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    shows_requested(&catalogue_view(&root), &name);
}
#[test]
fn a_name_one_past_the_display_bound_shows_a_marked_prefix() {
    let name = vec![b'm'; shown_bytes() + 1];
    let mut root = catalogue(&name, &[]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    shows_requested(&catalogue_view(&root), &marked_prefix());
}

#[test]
fn only_the_selected_duplicate_model_shows_its_efforts() {
    let mut models = List::with_capacity(2);
    for (provider, effort) in [(17, b"first-effort".as_slice()), (18, b"second-effort".as_slice())] {
        let mut efforts = List::with_capacity(1);
        efforts.push(c::Text::new(Box::from(effort), EFFORT_BYTES).unwrap()).unwrap();
        let mut model = offer_row(b"shared", Some(provider));
        model.effort = efforts.get(0).cloned();
        model.efforts = efforts;
        models.push(model).unwrap();
    }
    let mut root = catalogue_rows(models);
    edit(&mut root, d::pages::choice::Act::Model { index: 1, account: None });
    let view = catalogue_view(&root);
    let efforts: Vec<_> = view
        .tree()
        .nodes()
        .iter()
        .filter(|node| {
            node.role == Role::Button
                && [b"first-effort".as_slice(), b"second-effort"].contains(&node.name.as_deref().unwrap_or_default())
        })
        .map(|node| node.name.as_deref())
        .collect();
    assert_eq!(efforts, [Some(b"second-effort".as_slice())]);
}

#[test]
fn the_chosen_model_and_effort_are_pressed() {
    let mut root = catalogue(b"needs-effort", &[b"low", b"high"]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    edit(&mut root, d::pages::choice::Act::Effort { index: 1 });
    let view = catalogue_view(&root);
    let states = |name: &[u8]| {
        let node = view.tree().nodes().iter().find(|n| n.role == Role::Button && n.name.as_deref() == Some(name));
        node.unwrap().states.as_slice().to_vec()
    };
    for name in [b"needs-effort".as_slice(), b"high"] {
        assert!(states(name).contains(&skein_ui::State::Pressed));
        assert!(!states(name).contains(&skein_ui::State::Selected));
    }
    assert!(!states(b"low").contains(&skein_ui::State::Pressed));
}
#[test]
fn a_model_press_enables_start_with_its_default_effort() {
    let mut root = catalogue(b"needs-effort", &[b"high"]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    fixtures::root_event(
        &mut root,
        d::Event::Act { action: d::Action::Pages(d::pages::Act::ChatWords { words: b"Hello".as_slice().into() }) },
    );
    let mut view = catalogue_view(&root);
    let send = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Send"))
        .expect("send");
    assert!(!send.states.as_slice().contains(&skein_ui::State::Disabled));
    let entry = fixtures::find(view.tree(), Role::TextArea, b"What do you want done?").expect("entry");
    assert!(crate::act(&mut view, entry, skein_ui::Act::Submit).is_some());
}

#[test]
fn offered_models_do_not_infer_provider_readiness() {
    for ready in [None, Some(c::Readiness::Unknown), Some(c::Readiness::Unavailable), Some(c::Readiness::Usable)] {
        let root = account_readiness(ready.map(readiness));
        assert_eq!(using_text(&root), "chosen");
        assert!(!text(&root).contains("Account readiness unknown"));
    }
}
#[test]
fn another_payers_readiness_does_not_decide_this_payers() {
    let mut row = readiness(c::Readiness::Unavailable);
    row.payer = c::AccountUse::Project(fixtures::PROJECT);
    assert!(!text(&account_readiness(Some(row))).contains("needs an account"));
}
#[test]
fn the_provider_keeps_its_supplied_name() {
    assert!(text(&task_root()).contains("Supplied Provider"));
}
#[test]
fn the_accounts_banner_is_a_named_region() {
    let view = fixtures::page_view(&banner(Some(false)).0);
    let node = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::ACCOUNTS)).unwrap();
    assert_eq!(node.role, Role::Region);
    assert_eq!(node.name.as_deref(), Some(b"LLM account".as_slice()));
}
#[test]
fn the_picker_is_on_the_composers_hint_row() {
    let view = fixtures::page_view(&task_root());
    let nodes = view.tree().nodes();
    let row = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::FOOT_ROW)).unwrap();
    let picker = nodes.iter().position(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Model")).unwrap();
    assert_eq!(nodes[picker].description.as_deref(), Some(b"chosen".as_slice()));
    assert!(nodes[picker].states.as_slice().contains(&skein_ui::State::FoldsAway));
    assert!(row < picker && picker < row + usize::try_from(nodes[row].size).unwrap());
}

#[test]
fn changing_is_said_in_the_menu() {
    let mut root = task_root();
    edit(&mut root, d::pages::choice::Act::Model { index: 1, account: None });
    assert!(text(&root).contains("Changing…"));
    assert_eq!(using_text(&root), "chosen");
}

#[test]
fn a_refused_next_run_says_why() {
    use c::{HelloRefusal as F, HostKind as H, TaskWaitRefused as R};
    for (name, refusal, expected) in [
        (
            "engine",
            R::Hello { kind: H::Engine, at: fixtures::NOW, why: F::ReservedChannel, figures: None },
            "Next run refused · An engine host was refused at 00:00. Channel zero belongs to the engine's own host.",
        ),
        (
            "worker",
            R::Hello { kind: H::Worker, at: fixtures::NOW, why: F::Position { worker: 7 }, figures: None },
            "Next run refused · A worker was refused at 00:00. Unconfigured or occupied worker position 7.",
        ),
    ] {
        let root = task_root();
        let mut builder = Builder::new(&fixtures::tree_limits());
        let mut times = crate::words::Times::new(fixtures::NOW);
        crate::choice::waiting(
            &mut builder,
            &root,
            &waiting_task(Some(c::TaskRunWait::Refused(refusal))),
            &fixtures::words(),
            &mut crate::frame::Bindings::new(32),
            &mut times,
        );
        assert_eq!(fixtures::text(&builder.finish()).as_ref(), expected.as_bytes(), "{name}");
        let midnight = match refusal {
            R::Hello { .. } => Some(skein_lib::Wall::from_nanos(86_400_000_000_000)),
            R::Bounds { .. } => None,
        };
        assert_eq!(times.next, midnight, "{name}");
        let source = fixtures::chat::messaging_root();
        let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
        task.flags.ended_by = c::EndedBy::Requester;
        task.run_wait = Some(c::TaskRunWait::Refused(refusal));
        let c::ChipPhaseFacts { phase, .. } = task.chip.details.as_mut();
        *phase = c::ChipPhase::Active(c::RunStage::Due);
        let (mut root, _, watch) =
            fixtures::root_at(Address::Chat { task: fixtures::chat::TASK }, &fixtures::domain_limits());
        delivered(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
        let view = fixtures::page_view(&root);
        assert!(fixtures::find(view.tree(), Role::Status, b"Cannot start yet").is_some(), "{name}");
        assert!(fixtures::find(view.tree(), Role::Paragraph, expected.as_bytes()).is_some(), "{name}");
        assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_some(), "{name}");
        assert!(fixtures::find(view.tree(), Role::Button, b"Send").is_some(), "{name}");
        assert_eq!(view.next_change(), midnight, "{name}");
    }
}

#[test]
fn a_program_line_uses_its_first_usable_sources_name() {
    let (mut root, me_watch, task_watch, _) = line_fixture(
        Some(readiness(c::Readiness::Usable)),
        accounts(&[account(3, c::AccountKind::Program, None)]),
        Some(3),
        None,
        Some(b"high"),
        true,
    );
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let offer = me.providers.get_mut(0).unwrap();
    offer.name = c::DocumentText::complete(c::Text::new(Box::from(b"Claude".as_slice()), 32).unwrap());
    offer.key = false;
    offer.sign_in = false;
    offer.programs = List::with_capacity(1);
    offer.programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
    me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap().name =
        c::Text::new(Box::from(b"Opus".as_slice()), 32).unwrap();
    let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
    task.llm.as_mut().unwrap().main.model = c::Text::new(Box::from(b"Opus".as_slice()), 32).unwrap();
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    delivered(&mut root, task_watch.unwrap(), c::Delivery::Task(Box::new(task)));
    assert_eq!(using_text(&root), "your Claude subscription, through Claude Code · Opus · high");
    let rendered = text(&root);
    assert!(!rendered.contains("Last claimed account") && !rendered.contains("Requested for next run"));
}

#[test]
fn the_model_disclosure_holds_its_rows_and_folds_away() {
    let view = fixtures::page_view(&task_root());
    let nodes = chooser_nodes(&view);
    assert!(!nodes[0].states.as_slice().contains(&skein_ui::State::Expanded));
    for name in [b"chosen".as_slice(), b"second"] {
        assert!(nodes.iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(name)));
    }
    assert!(!nodes.iter().any(|n| n.name.as_deref() == Some(b"Apply to next run")));
}

#[test]
fn a_live_chat_without_message_keeps_the_line_and_chooser() {
    let (mut root, me_watch, watch, _, _) =
        fixtures::root_at_full(Address::Chat { task: 9 }, &fixtures::limits(), b"Shop");
    let template = task_root();
    let mut task = template.pages().chat().unwrap().task(template.client()).unwrap().clone();
    let mut acts = List::with_capacity(1);
    acts.push(c::Offered { act: c::ActKind::Amend, admission: c::Admission::Allowed, effect: None }).unwrap();
    task.acts = acts;
    // Reuse the model catalogue on a root with an observable task watch.
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models =
        d::pages::accounts::current_me(template.client()).unwrap().projects.get(0).unwrap().models.clone();
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    delivered(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_none());
    assert_eq!(using_text(&root), "chosen");
    assert!(fixtures::find(view.tree(), Role::Disclosure, b"Model").is_some());
    assert!(
        fixtures::browser_page_view(&root)
            .tree()
            .nodes()
            .iter()
            .any(|n| n.styles.as_slice().contains(&crate::look::BYLINE))
    );
}

#[test]
fn choice_feedback_is_in_the_menu_and_waits_stay_above() {
    let (mut root, me_watch, watch, _, _) =
        fixtures::root_at_full(Address::Chat { task: 9 }, &fixtures::limits(), b"Shop");
    let template = task_root();
    let me = d::pages::accounts::current_me(template.client()).unwrap().clone();
    let mut task = template.pages().chat().unwrap().task(template.client()).unwrap().clone();
    task.run_wait = Some(c::TaskRunWait::Credential);
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    delivered(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
    let mut out = start_model_press(&mut root);
    let pending = fixtures::page_view(&root);
    assert!(fixtures::find(pending.tree(), Role::Disclosure, b"Model").is_some());

    let nodes = pending.tree().nodes();
    let status = nodes.iter().position(|n| n.name.as_deref() == Some("Changing…".as_bytes())).unwrap();
    let wait =
        nodes.iter().position(|n| n.name.as_deref() == Some(b"This work needs a credential from its forge.")).unwrap();
    assert!(wait < status);
    answer_model_press(
        &mut root,
        &mut out,
        c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Amended { task: 9 })),
    );
    let view = fixtures::page_view(&root);
    let nodes = view.tree().nodes();
    let chooser = nodes.iter().position(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Model")).unwrap();
    let end = chooser + usize::try_from(nodes[chooser].size).unwrap();
    let outcome = nodes.iter().position(|n| n.role == Role::Status && n.name.as_deref() == Some(b"Applied")).unwrap();
    let wait =
        nodes.iter().position(|n| n.name.as_deref() == Some(b"This work needs a credential from its forge.")).unwrap();
    let composer = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::SESSION_COMPOSER)).unwrap();
    assert!(chooser < outcome && outcome < end && wait < composer && composer < chooser);
}

#[test]
fn the_model_choice_leaves_the_scripted_provider_out() {
    for address in [Address::ConnectForge { forge: None }, Address::Forges] {
        let (mut root, _) = playground_offers_root(address.clone(), true);
        fixtures::root_event(
            &mut root,
            d::Event::Act { action: d::Action::Pages(d::pages::Act::Project { project: fixtures::SECOND_PROJECT }) },
        );
        let check = |root: &d::Domain| {
            let mut view = fixtures::page_view(root);
            let tree = view.tree();
            assert!(fixtures::find(tree, Role::Paragraph, b"Agents need an LLM account to run.").is_some());
            assert!(fixtures::find(tree, Role::Link, b"Use a scripted API key").is_none());
            let sign_in = fixtures::find(tree, Role::Link, b"Continue with ChatGPT").unwrap();
            let key = fixtures::find(tree, Role::Link, b"Use an API key").unwrap();
            for (node, kind) in [(sign_in, d::WayKind::SignIn), (key, d::WayKind::Key)] {
                assert_eq!(
                    crate::act(&mut view, node, skein_ui::Act::Press),
                    Some(d::Action::Pages(d::pages::Act::Go {
                        address: Address::ConnectModel { way: Some(d::ModelWay { provider: 18, source: None, kind }) }
                    }))
                );
            }
        };
        check(&root);
        // A provider named scripted in a configuration without a playground is a legitimate key offer.
        let (root, _) = playground_offers_root(address, false);
        assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Link, b"Use a scripted API key").is_some());
    }
}

#[test]
fn chooser_groups_use_the_providers_words() {
    for (configured, shown) in [(b"openai".as_slice(), b"OpenAI".as_slice()), (b"anthropic", b"Claude")] {
        let (mut root, me_watch, _, _) = line_fixture(
            Some(readiness(c::Readiness::Usable)),
            accounts(&[account(3, c::AccountKind::Key, None)]),
            Some(3),
            None,
            None,
            true,
        );
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        me.providers.get_mut(0).unwrap().name =
            c::DocumentText::complete(c::Text::new(Box::from(configured), 32).unwrap());
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        assert!(
            String::from_utf8_lossy(&fixtures::text(fixtures::page_view(&root).tree()))
                .contains(core::str::from_utf8(shown).unwrap())
        );
    }
}

#[test]
fn the_no_model_line_keeps_the_tasks_wait_feedback() {
    assert!(text(&no_model_root(true)).contains("This work needs a credential from its forge."));
}

#[test]
fn the_no_model_composer_offers_connect_a_model_as_a_terminal_command() {
    for task_page in [false, true] {
        let mut view = fixtures::page_view(&no_model_root(task_page));
        let control = fixtures::find(view.tree(), Role::Link, b"Connect a model").unwrap();
        let (node, first) = fixtures::terminal_command(
            view.tree(),
            &view.limits().tree,
            control,
            b"Connect a model",
            "No model yet · ".as_bytes(),
        );
        assert_eq!(
            crate::act(&mut view, node, first),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::ConnectModel { way: None } }))
        );
    }
}

#[test]
fn a_choice_that_cant_change_is_words_not_a_button() {
    let (mut root, _, task_watch, _) = line_fixture(None, List::with_capacity(0), Some(3), None, None, true);
    let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
    task.acts.clear();
    delivered(&mut root, task_watch.unwrap(), c::Delivery::Task(Box::new(task)));
    let view = fixtures::page_view(&root);
    let node = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::MODEL_FIXED)).unwrap();
    assert_eq!(node.role, Role::Paragraph);
    assert_eq!(node.name.as_deref(), Some(b"chosen".as_slice()));
    assert!(fixtures::find(view.tree(), Role::Disclosure, b"Model").is_none());
}

#[test]
fn manage_usage_stays_with_the_agent_composers_line() {
    let root = line_facts(
        Some(readiness(c::Readiness::Usable)),
        accounts(&[account(3, c::AccountKind::SignIn, Some(b"https://provider.example/usage"))]),
        Some(3),
        None,
        None,
        true,
    );
    let view = fixtures::page_view(&root);
    assert!(!chooser_nodes(&view).iter().any(|node| node.name.as_deref() == Some(b"Manage usage")));
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::choice::using(&mut b, &root, root.pages().chat().unwrap().task(root.client()), &fixtures::words());
    assert!(fixtures::find(&b.finish(), Role::Link, b"Manage usage").is_some());
}

#[test]
fn a_tasks_byline_keeps_the_whole_bounded_model_and_effort() {
    let (mut root, _, task_watch, _) = line_fixture(None, List::with_capacity(0), None, None, None, true);
    let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
    let name = vec![b'm'; shown_bytes()];
    let effort = vec![b'e'; usize::try_from(EFFORT_BYTES).unwrap()];
    let choice = task.llm.as_mut().unwrap();
    choice.main.model = c::Text::new(name.clone().into_boxed_slice(), NAME_BYTES).unwrap();
    choice.main.effort = Some(c::Text::new(effort.clone().into_boxed_slice(), EFFORT_BYTES).unwrap());
    delivered(&mut root, task_watch.unwrap(), c::Delivery::Task(Box::new(task)));
    let view = catalogue_view(&root);
    let byline = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::MODEL)).unwrap();
    assert_eq!(byline.description.as_deref(), Some([&name[..], " · ".as_bytes(), &effort].concat().as_slice()));
}

fn chooser_nodes(view: &crate::View) -> &[skein_ui::Node] {
    let nodes = view.tree().nodes();
    let at =
        nodes.iter().position(|node| node.role == Role::Disclosure && node.name.as_deref() == Some(b"Model")).unwrap();
    let end = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    &nodes[at..end]
}

#[test]
fn the_panel_groups_offered_picker_rows_and_orders_efforts() {
    let (mut root, watch, _, _) = line_fixture(None, List::with_capacity(0), None, None, None, false);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::with_capacity(2);
    me.providers.push(provider_offer(18, b"Second provider")).unwrap();
    me.providers.push(provider_offer(17, b"First provider")).unwrap();
    let mut models = List::with_capacity(5);
    let mut displayed = offer_row(b"configured-name", Some(17));
    displayed.shown = Some(model_text(b"Shown name"));
    models.push(displayed).unwrap();
    let mut selected = offer_row(b"selected", Some(18));
    selected.efforts = List::with_capacity(2);
    selected.efforts.push(model_text(b"low")).unwrap();
    selected.efforts.push(model_text(b"high")).unwrap();
    selected.effort = Some(model_text(b"high"));
    models.push(selected).unwrap();
    let mut hidden = offer_row(b"hidden", Some(17));
    hidden.picker = false;
    models.push(hidden).unwrap();
    models.push(offer_row(b"later", Some(18))).unwrap();
    models.push(offer_row(b"unassigned", None)).unwrap();
    me.projects.get_mut(0).unwrap().models = models;
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    edit(&mut root, d::pages::choice::Act::Model { index: 1, account: None });
    let mut view = fixtures::page_view(&root);
    let rows: Vec<_> = chooser_nodes(&view)
        .iter()
        .filter(|node| node.role == Role::Button)
        .map(|node| node.name.as_deref().unwrap())
        .collect();
    assert_eq!(rows, [b"Shown name".as_slice(), b"selected", b"later", b"unassigned", b"low", b"high", b"All models"]);
    let shown = fixtures::find(view.tree(), Role::Button, b"Shown name").unwrap();
    assert_eq!(
        crate::act(&mut view, shown, skein_ui::Act::Press),
        Some(d::Action::Pages(d::pages::Act::Choice(d::pages::choice::Act::Model { index: 0, account: None })))
    );
}

fn empty_provider(account: Option<c::AccountLine>) -> d::Domain {
    let rows = match account {
        Some(account) => accounts(&[account]),
        None => List::with_capacity(0),
    };
    let (mut root, watch, _, _) = line_fixture(None, rows, None, None, None, true);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::with_capacity(2);
    me.providers.push(provider_offer(17, b"First provider")).unwrap();
    me.providers.push(provider_offer(18, b"OpenAI")).unwrap();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    root
}
#[test]
fn a_provider_with_no_account_is_one_line_and_no_model() {
    let mut view = fixtures::page_view(&empty_provider(None));
    let nodes = chooser_nodes(&view);
    let at = nodes
        .iter()
        .position(|node| {
            node.styles.as_slice().contains(&crate::look::MODEL_FROM)
                && nodes
                    .iter()
                    .skip_while(|n| n.id != node.id)
                    .take(usize::try_from(node.size).unwrap())
                    .any(|n| n.text.iter().any(|run| run.text.as_ref() == b"OpenAI"))
        })
        .unwrap();
    assert!(
        nodes[at..at + usize::try_from(nodes[at].size).unwrap()]
            .iter()
            .any(|n| n.name.as_deref() == Some("Connect a plan or key ›".as_bytes()))
    );
    assert!(!nodes.iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"gpt-6-luna")));
    let connect = fixtures::find(view.tree(), Role::Link, "Connect a plan or key ›".as_bytes()).unwrap();
    assert_eq!(
        crate::act(&mut view, connect, skein_ui::Act::Press),
        Some(d::Action::Pages(d::pages::Act::Go { address: Address::ConnectModel { way: None } }))
    );
}
#[test]
fn an_account_stopped_being_usable_keeps_its_fix_and_no_model() {
    for (kind, state, label) in [
        (c::AccountKind::SignIn, c::AccountState::SignInAgain, "Sign in again ›"),
        (c::AccountKind::Program, c::AccountState::SignInAgain, "Check again ›"),
        (c::AccountKind::Key, c::AccountState::SignInAgain, "Replace ›"),
        (c::AccountKind::Key, c::AccountState::Misconfigured, "Replace ›"),
        (c::AccountKind::SignIn, c::AccountState::Later { until: None }, "Resume ›"),
    ] {
        let mut row = account(7, kind, None);
        row.provider = Some(18);
        row.sources.clear();
        let mut source = source(kind, false);
        source.state = state;
        row.sources.push(source).unwrap();
        let mut view = fixtures::page_view(&empty_provider(Some(row)));
        let fix = fixtures::find(view.tree(), Role::Link, label.as_bytes()).unwrap();
        assert!(fixtures::find(view.tree(), Role::Link, "Connect a plan or key ›".as_bytes()).is_none());
        assert_eq!(
            crate::act(&mut view, fix, skein_ui::Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::Accounts }))
        );
    }
}
#[test]
fn an_empty_provider_without_a_fix_keeps_readiness_unknown() {
    let mut row = account(7, c::AccountKind::SignIn, None);
    row.provider = Some(18);
    row.sources.clear();
    let view = fixtures::page_view(&empty_provider(Some(row)));
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"Account readiness unknown").is_some());
    assert!(fixtures::find(view.tree(), Role::Link, "Connect a plan or key ›".as_bytes()).is_none());
}
#[test]
fn the_playground_provider_disappears_without_offered_rows() {
    let (mut root, watch, _, _, _) = fixtures::root_at_with_playground(
        Address::NewChat { draft: false },
        &fixtures::domain_limits(),
        b"Shop",
        Some((fixtures::PROJECT, true)),
    );
    let sample = fixtures::playground_root(Address::NewChat { draft: false }, fixtures::PROJECT, true);
    let mut me = d::pages::accounts::current_me(sample.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.clear();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    let view = fixtures::page_view(&root);
    assert!(!chooser_nodes(&view).iter().any(|node| node.name.as_deref() == Some(b"Playground")));
}
#[test]
fn the_inactive_playground_has_no_connect_line() {
    let (mut root, watch, _, _) = line_fixture(None, List::with_capacity(0), None, None, None, false);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::from_boxed(Box::new([provider_offer(17, b"First provider"), provider_offer(3, b"demo")]));
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    let view = fixtures::page_view(&root);
    assert!(chooser_nodes(&view).iter().all(|node| { node.text.iter().all(|run| run.text.as_ref() != b"demo") }));
}

#[test]
fn an_accounts_models_are_the_rows_that_name_its_number() {
    let (mut root, watch, _, _) =
        line_fixture(None, accounts(&[account(3, c::AccountKind::Key, None)]), None, None, None, true);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let project = me.projects.get_mut(0).unwrap();
    project.models = List::with_capacity(2);
    let mut same_provider = offer_row(b"wrong-account", Some(17));
    same_provider.accounts = List::with_capacity(1);
    same_provider.accounts.push(7).unwrap();
    project.models.push(same_provider).unwrap();
    let mut different_provider = offer_row(b"right-account", None);
    different_provider.accounts = List::with_capacity(1);
    different_provider.accounts.push(3).unwrap();
    project.models.push(different_provider).unwrap();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Accounts) });
    let view = fixtures::page_view(&root);
    let nodes = view.tree().nodes();
    let row = nodes.iter().position(|node| node.styles.as_slice().contains(&crate::look::SETTINGS_ROW)).unwrap();
    let summary = core::str::from_utf8(nodes[row].description.as_deref().unwrap()).unwrap();
    assert!(summary.contains("right-account"), "{summary}");
    assert!(!summary.contains("wrong-account"), "{summary}");
}

fn picker_line(view: &crate::View) -> &[u8] {
    chooser_nodes(view)[0].description.as_deref().unwrap()
}
#[test]
fn a_new_chats_picker_says_what_create_would_use() {
    let (root, _, _) = picker_fixture(false, true, false);
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        assert_eq!(picker_line(&view), "Claude Opus 5.5 · high".as_bytes());
        let model = fixtures::find(view.tree(), Role::Button, b"Opus 5.5").unwrap();
        assert!(
            view.tree()
                .nodes()
                .iter()
                .find(|n| n.id == model)
                .unwrap()
                .states
                .as_slice()
                .contains(&skein_ui::State::Pressed)
        );
    }
    let me = d::pages::accounts::current_me(root.client()).unwrap();
    let default = jig_client::shapes::resolved_default(me, fixtures::PROJECT, jig_client::shapes::CHAT).unwrap();
    assert_eq!(default.main.model.bytes(), b"claude-opus-5-5");
    assert!(root.pages().choice().requested.is_none());
}
#[test]
fn a_heading_says_what_pays() {
    for (provider, kind, held, expected) in [
        (b"anthropic".as_slice(), c::AccountKind::Program, true, "Claudeyour subscription"),
        (b"anthropic", c::AccountKind::SignIn, true, "Claudeyour subscription"),
        (b"openai", c::AccountKind::SignIn, true, "OpenAIyour ChatGPT plan"),
        (b"openai", c::AccountKind::Key, true, "OpenAIyour API key"),
        (b"openai", c::AccountKind::Key, false, "OpenAI"),
    ] {
        let (mut root, watch, accounts_watch) = picker_fixture(false, false, false);
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        me.providers.get_mut(0).unwrap().name = c::DocumentText::complete(model_text(provider));
        delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        doc.accounts.get_mut(0).unwrap().sources = List::from_boxed(Box::new([source(kind, true)]));
        if !held {
            doc.accounts.clear();
        }
        delivered(&mut root, accounts_watch, c::Delivery::Accounts(Box::new(doc)));
        let view = fixtures::page_view(&root);
        let nodes = chooser_nodes(&view);
        let at = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::MODEL_FROM)).unwrap();
        let bytes = fixtures::nodes_text(&nodes[at..at + usize::try_from(nodes[at].size).unwrap()]);
        assert_eq!(String::from_utf8_lossy(&bytes), expected);
    }
}
#[test]
fn two_accounts_of_one_provider_head_each_and_a_press_prefers_its_account() {
    const HEADINGS: usize = 3; // Two Claude accounts and OpenAI's connect line.
    for (first_email, other_email, first, other) in [
        (None, None, "ClaudePersonal", "ClaudeOther"),
        (
            Some(b"alice@example.com".as_slice()),
            Some(b"bob@example.com".as_slice()),
            "Claudealice@example.com",
            "Claudebob@example.com",
        ),
        (Some(b"alice@example.com".as_slice()), None, "Claudealice@example.com", "ClaudeOther"),
    ] {
        let (mut root, _, accounts_watch) = picker_fixture(true, false, false);
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        doc.accounts.get_mut(0).unwrap().facts = super::accounts::email_facts(first_email);
        doc.accounts.get_mut(1).unwrap().facts = super::accounts::email_facts(other_email);
        delivered(&mut root, accounts_watch, c::Delivery::Accounts(Box::new(doc)));
        let mut view = fixtures::page_view(&root);
        let nodes = chooser_nodes(&view);
        let headings: Vec<_> =
            nodes.iter().filter(|n| n.styles.as_slice().contains(&crate::look::MODEL_FROM)).collect();
        let labels = fixtures::nodes_text(nodes);
        assert!(String::from_utf8_lossy(&labels).contains(first));
        assert!(String::from_utf8_lossy(&labels).contains(other));
        assert_eq!(headings.len(), HEADINGS);
        let opus: Vec<_> =
            nodes.iter().filter(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Opus 5.5")).collect();
        assert!(opus[0].states.as_slice().contains(&skein_ui::State::Pressed));
        assert!(!opus[1].states.as_slice().contains(&skein_ui::State::Pressed));
        let id = opus[1].id;
        assert_eq!(
            crate::act(&mut view, id, skein_ui::Act::Press),
            Some(d::Action::Pages(d::pages::Act::Choice(d::pages::choice::Act::Model { index: 0, account: Some(3) })))
        );
    }
}
#[test]
fn efforts_follow_the_chosen_model_and_none_has_no_row() {
    let (mut root, watch, _) = picker_fixture(false, true, false);
    let view = fixtures::page_view(&root);
    let nodes = chooser_nodes(&view);
    let last_model =
        nodes.iter().rposition(|n| n.role == Role::Button && n.name.as_deref() == Some(b"GPT-6.1 Sol")).unwrap();
    let effort = nodes.iter().position(|n| n.role == Role::Group && n.name.as_deref() == Some(b"Effort")).unwrap();
    assert!(last_model < effort);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap().efforts.clear();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Group, b"Effort").is_none());
}
#[test]
fn all_models_shows_where_the_picker_shows_fewer() {
    for hidden in [false, true] {
        let (root, _, _) = picker_fixture(false, true, hidden);
        let mut view = fixtures::page_view(&root);
        let all = fixtures::find(view.tree(), Role::Button, b"All models");
        assert_eq!(all.is_some(), hidden);
        if let Some(all) = all {
            let at = view.tree().nodes().iter().position(|n| n.id == all).unwrap();
            assert_eq!(fixtures::node_text(view.tree(), at).as_ref(), "All models · 3".as_bytes());
            assert_eq!(
                crate::act(&mut view, all, skein_ui::Act::Press),
                Some(d::Action::Pages(d::pages::Act::Choice(d::pages::choice::Act::All)))
            );
        }
    }
}
#[test]
fn all_models_finds_by_shown_name_and_name() {
    let (mut root, _, _) = picker_fixture(false, true, true);
    edit(&mut root, d::pages::choice::Act::All);
    for words in [b"SoNnEt 5.5".as_slice(), b"SONNET-5-5"] {
        edit(&mut root, d::pages::choice::Act::Find { words: Box::from(words) });
        let mut view = fixtures::page_view(&root);
        assert!(fixtures::find(view.tree(), Role::Button, b"Sonnet 5.5").is_some());
        assert!(fixtures::find(view.tree(), Role::Button, b"Opus 5.5").is_none());
        let field = fixtures::find(view.tree(), Role::TextField, b"Find a model").unwrap();
        assert!(
            view.tree()
                .nodes()
                .iter()
                .find(|n| n.id == field)
                .unwrap()
                .states
                .as_slice()
                .contains(&skein_ui::State::SubmitsOnEnter)
        );
        assert_eq!(
            crate::act(&mut view, field, skein_ui::Act::Input { value: Box::from(b"opus".as_slice()) }),
            Some(d::Action::Pages(d::pages::Act::Choice(d::pages::choice::Act::Find {
                words: Box::from(b"opus".as_slice())
            })))
        );
        assert_eq!(crate::act(&mut view, field, skein_ui::Act::Submit), None);
    }
}
#[test]
fn all_models_links_to_the_projects_models_and_marks_new() {
    const WEEK: u64 = 604_800_000_000_000;
    let (mut root, watch, _) = picker_fixture(false, true, true);
    edit(&mut root, d::pages::choice::Act::All);
    let mut view = fixtures::page_view(&root);
    let link = fixtures::find(view.tree(), Role::Link, b"Choose what the list shows, in Deployment").unwrap();
    assert_eq!(
        crate::act(&mut view, link, skein_ui::Act::Press),
        Some(d::Action::Pages(d::pages::Act::Go { address: Address::Models { project: 1 } }))
    );
    assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/projects/1/settings/models".as_slice()));
    assert!(!chooser_nodes(&view).iter().any(|n| n.name.as_deref() == Some(b"new")));
    assert!(fixtures::find(view.tree(), Role::Button, "‹ Models".as_bytes()).is_some());
    for source in [c::ModelSource::List, c::ModelSource::Stock, c::ModelSource::Person] {
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        let model = me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap();
        model.source = source;
        model.new_until = Some(skein_lib::Wall::from_nanos(WEEK));
        delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
        for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            for now in [WEEK - 1, WEEK, WEEK + 1] {
                crate::build(&mut view, &root, skein_lib::Wall::from_nanos(now), &mut skein_lib::Queue::uncapped());
                let tags: Vec<_> = chooser_nodes(&view)
                    .iter()
                    .filter(|n| {
                        n.styles.as_slice().contains(&crate::look::MODEL_TAG) && {
                            let at = view.tree().nodes().iter().position(|node| node.id == n.id).unwrap();
                            let end = at + usize::try_from(n.size).unwrap();
                            fixtures::nodes_text(&view.tree().nodes()[at..end]).ends_with(b"new")
                        }
                    })
                    .collect();
                assert_eq!(tags.len(), usize::from(source == c::ModelSource::List && now < WEEK));
                if source == c::ModelSource::List && now < WEEK {
                    assert_eq!(view.next_change(), Some(skein_lib::Wall::from_nanos(WEEK)));
                }
            }
        }
    }
}
#[test]
fn a_started_chat_says_when_a_change_applies() {
    let view = fixtures::page_view(&task_root());
    assert!(chooser_nodes(&view).iter().any(|n| n.name.as_deref() == Some(b"A change applies from the next turn.")));
}
#[test]
fn no_model_of_a_provider_with_no_account_shows() {
    let (mut root, _, _) = picker_fixture(false, false, true);
    for all in [false, true] {
        if all {
            edit(&mut root, d::pages::choice::Act::All);
        }
        let view = fixtures::page_view(&root);
        assert_eq!(picker_line(&view), "Claude Opus 5.5 · high".as_bytes());
        let nodes = chooser_nodes(&view);
        assert!(!nodes.iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"GPT-6.1 Sol")));
        assert!(
            nodes
                .iter()
                .any(|n| n.role == Role::Link && n.name.as_deref() == Some("Connect a plan or key ›".as_bytes()))
        );
    }
}
#[test]
fn the_terminal_prints_the_list_as_lines() {
    let (root, _, _) = picker_fixture(false, true, false);
    let view = fixtures::page_view(&root);
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    let printed = String::from_utf8_lossy(print.text.as_slice());
    assert!(printed.contains("Model: Claude Opus 5.5 · high"), "{printed}");
    assert!(
        printed
            .lines()
            .any(|line| line.trim_start().starts_with("- [") && line.ends_with("Opus 5.5 · default (pressed)")),
        "{printed}"
    );
    assert!(printed.lines().any(|line| line.trim() == "Effort:"), "{printed}");
    assert!(printed.find("Opus 5.5 · default").unwrap() < printed.find("Effort:").unwrap());
}

fn start_model_press(root: &mut d::Domain) -> skein_lib::Queue<d::Request> {
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: fixtures::limits() },
        d::Event::Act {
            action: d::Action::Pages(d::pages::Act::Choice(d::pages::choice::Act::Model { index: 1, account: None })),
        },
        &mut out,
    );
    out
}

fn answer_model_press(root: &mut d::Domain, out: &mut skein_lib::Queue<d::Request>, answer: c::Answer) {
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: fixtures::limits() },
        d::Event::Client { event: jig_client::Event::Stored },
        out,
    );
    let attempt = out
        .iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } = request
            {
                Some(*attempt)
            } else {
                None
            }
        })
        .unwrap();
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(answer)),
                    why: None,
                },
            },
        },
    );
}

#[test]
fn a_refusal_says_why_in_the_menu() {
    let mut root = task_root();
    let mut out = start_model_press(&mut root);
    answer_model_press(&mut root, &mut out, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
    let view = fixtures::page_view(&root);
    assert_eq!(picker_line(&view), b"chosen");
    assert!(String::from_utf8_lossy(&fixtures::nodes_text(chooser_nodes(&view))).contains("not offered"));
}
#[test]
fn no_model_yet_has_no_picker() {
    for task in [false, true] {
        let view = fixtures::page_view(&no_model_root(task));
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::MODEL)));
        assert!(String::from_utf8_lossy(&fixtures::text(view.tree())).contains("No model yet"));
    }
}
#[test]
fn a_chats_bar_keeps_its_number_and_no_account() {
    let view = fixtures::browser_page_view(&named_account());
    let nodes = view.tree().nodes();
    let bar = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::SESSION_TOP)).unwrap();
    let bar = &nodes[bar..bar + usize::try_from(nodes[bar].size).unwrap()];
    assert!(bar.iter().any(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(b"T9")));
    assert!(!String::from_utf8_lossy(&fixtures::nodes_text(bar)).contains("API key"));
    assert!(!bar.iter().any(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Model")));
}

#[test]
fn the_playground_model_uses_its_words_over_a_shown_name() {
    let mut model = offer_row(crate::words::SCRIPTED_MODEL, None);
    model.shown = Some(model_text(b"Provider identifier"));
    let mut root = catalogue_rows(List::from_boxed(Box::new([model])));
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    assert_eq!(picker_line(&fixtures::page_view(&root)), b"Scripted model");
}

use super::fixtures::choice::{accounts, line_facts};
pub(super) use crate::parts_tests::choice::{
    account_readiness, banner, edit, model_words, named_account, prefix_bytes, readiness, shown_bytes, task_root,
    using_text, waiting_task,
};
