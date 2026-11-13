#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic trees and allocate maximal model-name fixtures"
)]
use super::fixtures;
pub(super) use super::fixtures::choice::{
    NAME_BYTES, account, delivered, line_fixture, model_text, source, wait_pointers,
};
use jig_client::contract as c;
use skein_lib::{List, Token};
use skein_ui::{Builder, Role};
use temper_client_domain::{self as d, Address};

pub(crate) fn banner(usable: Option<bool>) -> (d::Domain, Token) {
    let (mut root, watch) = fixtures::account_root();
    if let Some(usable) = usable {
        delivered(
            &mut root,
            watch,
            c::Delivery::Accounts(Box::new(c::Accounts {
                program_hosts: List::with_capacity(0),
                accounts: List::with_capacity(0),
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
            })),
        );
    }
    fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::NewChat { draft: false }) });
    (root, watch)
}

pub(crate) fn task_root() -> d::Domain {
    account_readiness(None)
}
pub(crate) fn account_readiness(ready: Option<c::PayerReadiness>) -> d::Domain {
    account_facts(ready, List::with_capacity(0))
}
fn account_facts(ready: Option<c::PayerReadiness>, accounts: List<c::AccountLine>) -> d::Domain {
    line_facts(ready, accounts, Some(3), Some(7), None, true)
}

#[test]
fn a_requested_model_is_labelled_for_the_next_run() {
    assert_eq!(using_text(&task_root()), "chosen");
}
#[test]
fn an_unknown_readiness_keeps_the_model_only() {
    assert_eq!(using_text(&task_root()), "chosen");
}
pub(crate) fn waiting_task(wait: Option<c::TaskRunWait>) -> c::Task {
    let mut task = fixtures::task::document();
    task.llm = None;
    task.run_wait = wait;
    task
}
fn wait(why: c::TaskWaitWhy) -> skein_ui::Tree {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::bindings::Bindings::new(32);
    b.open(Role::Page);
    let (account, source) = if why == c::TaskWaitWhy::Empty { (None, None) } else { (Some(7), Some(8)) };
    crate::choice::waiting(
        &mut b,
        &task_root(),
        &waiting_task(Some(c::TaskRunWait::Account { account, source, why })),
        &fixtures::words(),
        &mut bindings,
        &mut crate::words::Times::new(fixtures::NOW),
    );
    b.close();
    b.finish()
}
#[test]
fn an_empty_wait_connects_a_model_and_uses_waiting_style() {
    let tree = wait(c::TaskWaitWhy::Empty);
    assert!(fixtures::find(&tree, Role::Paragraph, b"No account can run this chat yet.").is_some());
    assert!(fixtures::find(&tree, Role::Link, b"Connect a model").is_some());
    let status = tree
        .nodes()
        .iter()
        .find(|node| node.role == Role::Status && node.name.as_deref() == Some(b"Waiting for an account"))
        .unwrap();
    let styles = status.styles.as_slice();
    assert!(styles.contains(&crate::look::STATUS_WAITING));
    assert!(!styles.contains(&crate::look::STATUS_ACTIVE));
}

#[test]
fn a_limited_wait_links_to_resume() {
    assert!(
        fixtures::find(
            &wait(c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until: None })),
            Role::Link,
            b"Resume"
        )
        .is_some()
    );
}
#[test]
fn a_sign_in_wait_links_to_sign_in() {
    assert!(
        fixtures::find(
            &wait(c::TaskWaitWhy::Source(c::TaskSourceState::NeedsSignIn)),
            Role::Link,
            b"Sign in with an LLM provider"
        )
        .is_some()
    );
}
#[test]
fn an_unknown_reset_has_no_time() {
    assert!(
        !String::from_utf8(
            fixtures::text(&wait(c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until: None }))).into_vec()
        )
        .unwrap()
        .contains("until")
    );
}
#[test]
fn a_known_reset_shows_its_utc_moment() {
    let until = Some(skein_lib::Wall::from_nanos(1_791_828_240_000_000_000));
    let shown = String::from_utf8(
        fixtures::text(&wait(c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until }))).into_vec(),
    )
    .unwrap();
    assert!(shown.contains("Limited until Oct 12, 18:04 UTC"), "{shown}");
}

pub(crate) fn model_words() -> crate::words::Words {
    crate::words::Words::english(fixtures::FORGE, 128, crate::words::Shell::Terminal).unwrap()
}

const CUT_MARK: &[u8] = "…".as_bytes();
pub(crate) fn shown_bytes() -> usize {
    usize::try_from(crate::choice::MODEL_SHOWN).unwrap()
}
pub(crate) fn prefix_bytes() -> usize {
    shown_bytes().checked_sub(CUT_MARK.len()).expect("display bound includes cut mark")
}

#[test]
fn a_cut_model_name_ends_at_a_utf8_boundary() {
    let letter = "é";
    let name = letter.repeat(usize::try_from(NAME_BYTES).unwrap() / letter.len());
    let shown = crate::choice::shown(name.as_bytes(), &model_words());
    assert_eq!(
        String::from_utf8(shown.into_vec()).unwrap(),
        format!("{}…", letter.repeat(prefix_bytes() / letter.len()))
    );
}

pub(crate) fn readiness(state: c::Readiness) -> c::PayerReadiness {
    c::PayerReadiness { provider: 17, payer: c::AccountUse::Party(73), readiness: state, settled: Some(true) }
}

pub(crate) fn edit(root: &mut d::Domain, act: d::pages::choice::Act) {
    fixtures::root_event(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::Choice(act)) });
}

pub(crate) fn named_account() -> d::Domain {
    one_account(3, false)
}
fn one_account(number: u32, plan: bool) -> d::Domain {
    let mut accounts = List::with_capacity(1);
    accounts
        .push(c::AccountLine {
            list: None,
            facts: List::with_capacity(0),
            number,
            place: 1,
            uses: None,
            plan,
            provider: None,
            provider_name: None,
            manage_usage: None,
            display: c::DocumentText::complete(c::Text::new(Box::from(*b"Personal"), 32).unwrap()),
            holder: c::AccountHolder::Party(73),
            sources: List::with_capacity(0),
            cut: false,
        })
        .unwrap();
    account_facts(None, accounts)
}

#[test]
fn an_unread_readiness_withholds_the_accounts_words() {
    assert_eq!(using_text(&one_account(7, true)), "chosen");
}
#[test]
fn a_requested_account_waits_for_readiness() {
    let root = named_account();
    assert_eq!(using_text(&root), "chosen");
}
#[test]
fn a_waiting_account_shows_its_display() {
    let root = named_account();
    let mut b = Builder::new(&fixtures::tree_limits());
    b.open(Role::Page);
    crate::choice::waiting(
        &mut b,
        &root,
        &waiting_task(Some(c::TaskRunWait::Account {
            account: Some(3),
            source: None,
            why: c::TaskWaitWhy::Refreshing,
        })),
        &fixtures::words(),
        &mut crate::bindings::Bindings::new(32),
        &mut crate::words::Times::new(fixtures::NOW),
    );
    b.close();
    assert!(String::from_utf8(fixtures::text(&b.finish()).into_vec()).unwrap().contains("Waiting account: Personal"));
}

#[test]
fn a_task_waiting_on_a_moving_account_says_why() {
    let root = named_account();
    let mut b = Builder::new(&fixtures::tree_limits());
    b.open(Role::Page);
    crate::choice::waiting(
        &mut b,
        &root,
        &waiting_task(Some(c::TaskRunWait::Account {
            account: Some(3),
            source: Some(8),
            why: c::TaskWaitWhy::Source(c::TaskSourceState::Moving),
        })),
        &fixtures::words(),
        &mut crate::bindings::Bindings::new(32),
        &mut crate::words::Times::new(fixtures::NOW),
    );
    b.close();
    let tree = b.finish();
    assert!(
        fixtures::find(&tree, Role::Paragraph, b"Waiting because the account is moving to another deployment")
            .is_some()
    );
    assert!(!tree.nodes().iter().any(|node| node.role == Role::Button || node.role == Role::Link));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table retains each hello cause and all ten bound clauses; client/pages/accounts.md, 2"
)]
fn every_hello_refusal_keeps_its_cause_and_numbers() {
    use c::{AgentShortfall as S, Figures, HelloRefusal as F, HostKind as K, TaskWaitRefused as R};
    let hello = |why| R::Hello { kind: K::Worker, at: fixtures::NOW, why, figures: None };
    let bounds = |why, required, offered| R::Bounds {
        kind: K::Worker,
        why: Some(why),
        figures: Some(Figures { required, offered }),
    };
    let mut rows = Vec::new();
    for (why, cause) in [
        (F::Workers { limit: 7 }, "No room for another worker. Limit: 7."),
        (
            F::StopBound { grace: skein_lib::Duration::from_secs(30) },
            "It would not stop before the engine's grace ends. Allowed grace: 30.",
        ),
        (F::Agent { why: S::Model { charter: 3, model: 8 } }, "Agent model missing for charter 3. Model position 8."),
        (
            F::Agent { why: S::Window { charter: 3, model: 8 } },
            "Agent window too small for charter 3. Model position 8.",
        ),
        (
            F::Agent { why: S::Output { charter: 3, model: 8 } },
            "Agent output too small for charter 3. Model position 8.",
        ),
        (F::Agent { why: S::Turns { charter: 2 } }, "Agent turns too few."),
        (F::Agent { why: S::Spend }, "Agent spend too small."),
        (F::Agent { why: S::Time { charter: 2 } }, "Agent time too short."),
        (F::Agent { why: S::Waiting { charter: 2 } }, "Agent waiting too short."),
        (
            F::Agent { why: S::Checks { charter: 2 } },
            "Agent check timeout exceeds reserve_time for charter 2. See temper help settings for held revisions.",
        ),
        (F::Agent { why: S::Message }, "Agent message bound too small."),
        (F::Agent { why: S::Inbox }, "Agent inbox too small."),
        (F::Agent { why: S::Unacknowledged }, "Agent unacknowledged turns too few."),
    ] {
        rows.push((hello(why), format!("Next run refused · A worker was refused at 00:00. {cause}")));
    }
    for (why, required, offered, clause) in [
        (S::Turns { charter: 3 }, 100_000, 800, "its charter asks 100,000 turns, and the largest worker takes 800."),
        (S::Time { charter: 3 }, 7_200_000, 3_600_000, "its charter asks 2 h a run, and the largest worker takes 1 h."),
        (
            S::Waiting { charter: 3 },
            1_800_000,
            600_000,
            "its charter asks 30 min of waiting, and the largest worker takes 10 min.",
        ),
        (
            S::Checks { charter: 3 },
            300_000,
            600_000,
            "its charter keeps 5 min for checks, and the quickest worker's checks take 10 min.",
        ),
        (
            S::Window { charter: 3, model: 1 },
            200_000,
            128_000,
            "its charter's model at position 1 asks a window of 200,000 tokens, and the largest worker takes 128,000.",
        ),
        (
            S::Output { charter: 3, model: 1 },
            64000,
            32000,
            "its charter's model at position 1 asks 64,000 output tokens, and the largest worker takes 32,000.",
        ),
        (S::Spend, 5000, 2000, "a run here may spend 5,000, and the largest worker takes 2,000."),
        (S::Message, 65536, 16384, "a message here may be 65,536 bytes, and the largest worker takes 16,384."),
        (S::Inbox, 12, 8, "a start here sends 12 messages, and the largest worker takes 8."),
        (S::Unacknowledged, 64, 32, "a run here keeps 64 turns unacknowledged, and the largest worker takes 32."),
        (
            S::Time { charter: 3 },
            59000,
            0,
            "its charter asks under 1 min a run, and the largest worker takes under 1 min.",
        ),
        (
            S::Waiting { charter: 3 },
            5_400_000,
            3_600_000,
            "its charter asks 1 h 30 min of waiting, and the largest worker takes 1 h.",
        ),
    ] {
        rows.push((bounds(why, required, offered), format!("No worker can take this: {clause}")));
    }
    rows.extend([
        (
            R::Bounds { kind: K::Worker, why: None, figures: None },
            String::from("No worker can take this: each falls short of another bound its charter asks."),
        ),
        (
            R::Bounds { kind: K::Worker, why: Some(S::Turns { charter: 3 }), figures: None },
            String::from("No worker can take this. Agent turns too few."),
        ),
        (
            R::Bounds {
                kind: K::Engine,
                why: Some(S::Turns { charter: 3 }),
                figures: Some(Figures { required: 100_000, offered: 800 }),
            },
            String::from("The engine's own host can't take this: its charter asks 100,000 turns, and it takes 800."),
        ),
        (
            R::Hello {
                kind: K::Worker,
                at: skein_lib::Wall::from_nanos(38_520_000_000_000),
                why: F::Agent { why: S::Turns { charter: 3 } },
                figures: Some(Figures { required: 100_000, offered: 800 }),
            },
            String::from(
                "No worker can take this: charter 3 asks 100,000 turns, and the worker refused at 10:42 takes 800.",
            ),
        ),
    ]);
    // The widest count, model position and duration fit the bound words' holes.
    rows.extend([
        (bounds(S::Window { charter: u32::MAX, model: u32::MAX }, u64::MAX, u64::MAX), String::from("No worker can take this: its charter's model at position 4294967295 asks a window of 18,446,744,073,709,551,615 tokens, and the largest worker takes 18,446,744,073,709,551,615.")),
        (bounds(S::Time { charter: u32::MAX }, u64::MAX, u64::MAX), String::from("No worker can take this: its charter asks 213503982334 d a run, and the largest worker takes 213503982334 d.")),
    ]);
    let root = task_root();
    let words = [fixtures::words(), fixtures::browser_words()];
    let bound_words = [
        crate::words::Words::english(fixtures::FORGE, 1, crate::words::Shell::Terminal).unwrap(),
        crate::words::Words::english(fixtures::FORGE, 1, crate::words::Shell::Browser).unwrap(),
    ];
    for (why, expected) in rows {
        wait_pointers(expected.as_bytes());
        let vocabularies = match why {
            R::Bounds { .. } => &bound_words,
            R::Hello { .. } => &words,
        };
        for w in vocabularies {
            let mut builder = Builder::new(&fixtures::tree_limits());
            let at = match why {
                R::Hello { at, .. } => at,
                R::Bounds { .. } => fixtures::NOW,
            };
            crate::choice::waiting(
                &mut builder,
                &root,
                &waiting_task(Some(c::TaskRunWait::Refused(why))),
                w,
                &mut crate::bindings::Bindings::new(32),
                &mut crate::words::Times::new(at),
            );
            let text = fixtures::text(&builder.finish());
            wait_pointers(&text);
            assert_eq!(text.as_ref(), expected.as_bytes(), "{why:?}");
        }
    }
}

pub(crate) fn using_text(root: &d::Domain) -> String {
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::choice::using(
        &mut b,
        root,
        root.pages().chat().and_then(|chat| chat.task(root.client())),
        &fixtures::words(),
    );
    let text = String::from_utf8(fixtures::text(&b.finish()).into_vec()).unwrap();
    let text = text.split(" · Manage usage").next().unwrap();
    if text == "Using the default model" {
        return text.to_owned();
    }
    if text.starts_with("Using your ") && text.contains("API key") {
        text.strip_prefix("Using your ").unwrap().to_owned()
    } else {
        text.strip_prefix("Using ").unwrap_or(text).to_owned()
    }
}
#[test]
fn a_keys_line_uses_the_configured_provider_name_and_appends_effort() {
    let root = line_facts(
        Some(readiness(c::Readiness::Usable)),
        accounts(&[account(3, c::AccountKind::Key, None)]),
        Some(3),
        Some(7),
        Some(b"high"),
        true,
    );
    assert_eq!(using_text(&root), "Supplied Provider API key · chosen · high");
}
#[test]
fn task_lines_take_their_own_templates_preview_and_offered_effort() {
    use super::fixtures::chat_helpers::{model_choice, model_line_root};
    for (own, preview, charter, using, closed) in [
        (None, Some(1), 1, "GPT-6 Astra · high", "GPT-6 Astra · high"),
        (None, Some(5), 37, "GPT-6 Astra · high", "GPT-6 Astra · high"),
        (Some(model_choice("gpt-6-astra", None)), None, 1, "GPT-6 Astra · high", "GPT-6 Astra · high"),
        (Some(model_choice("gpt-6-astra", Some("low"))), None, 1, "GPT-6 Astra · low", "GPT-6 Astra · low"),
        (None, None, 37, "Using the default model", "the default model"),
    ] {
        let mut task = fixtures::chat_helpers::asking_task();
        task.chip.executor = c::ChipExecutor::Agent { charter };
        task.activity = 1;
        task.llm = own;
        let root = model_line_root(task, None, preview);
        assert_eq!(using_text(&root), using);
        let task = root.pages().chat().unwrap().task(root.client()).unwrap();
        assert_eq!(crate::choice::closed_line(&root, Some(task), &fixtures::words()).as_ref(), closed.as_bytes());
    }
}
#[test]
fn a_plan_has_a_manage_usage_link_only_for_a_safe_address() {
    for address in [
        None,
        Some(b"https://provider.example/usage".as_slice()),
        Some(b"javascript:bad"),
        Some(b"https://provider.example/usage?secret"),
    ] {
        let root = line_facts(
            Some(readiness(c::Readiness::Usable)),
            accounts(&[account(3, c::AccountKind::SignIn, address)]),
            Some(3),
            None,
            None,
            true,
        );
        let mut b = Builder::new(&fixtures::tree_limits());
        crate::choice::using(&mut b, &root, root.pages().chat().unwrap().task(root.client()), &fixtures::words());
        let tree = b.finish();
        let link = tree.nodes().iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Manage usage"));
        let safe = address == Some(b"https://provider.example/usage".as_slice());
        assert_eq!(link.is_some(), safe);
        if let Some(link) = link {
            assert_eq!(link.external.as_deref(), address);
        }
        assert_eq!(using_text(&root), "Supplied Provider plan · chosen");
    }
}

#[test]
fn preference_then_kept_then_the_first_usable_account_decides_the_line() {
    let rows = accounts(&[
        account(1, c::AccountKind::Key, None),
        account(7, c::AccountKind::SignIn, None),
        account(3, c::AccountKind::Key, None),
    ]);
    for (preference, kept, expected) in [
        (Some(3), Some(7), "Supplied Provider API key · chosen"),
        (None, Some(7), "Supplied Provider plan · chosen"),
        (None, None, "Supplied Provider API key · chosen"),
    ] {
        let root = line_facts(Some(readiness(c::Readiness::Usable)), rows.clone(), preference, kept, None, true);
        assert_eq!(using_text(&root), expected);
    }
}
#[test]
fn automatic_accounts_skip_other_providers_payers_and_unusable_sources() {
    let mut other_provider = account(1, c::AccountKind::SignIn, None);
    other_provider.provider = Some(18);
    let mut other_payer = account(2, c::AccountKind::Program, None);
    other_payer.uses = Some(List::with_capacity(0));
    let mut unusable = account(3, c::AccountKind::SignIn, None);
    unusable.sources.get_mut(1).unwrap().usable = false;
    let rows = accounts(&[other_provider, other_payer, unusable, account(4, c::AccountKind::Key, None)]);
    let root = line_facts(Some(readiness(c::Readiness::Usable)), rows, None, None, None, true);
    assert_eq!(using_text(&root), "Supplied Provider API key · chosen");
}
#[test]
fn a_new_chats_default_comes_from_its_supplied_preview() {
    let mut ready = readiness(c::Readiness::Usable);
    ready.payer = c::AccountUse::Party(73);
    let (mut root, watch, _, _) =
        line_fixture(Some(ready), accounts(&[account(3, c::AccountKind::Key, None)]), None, None, None, false);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let project = me.projects.get_mut(0).unwrap();
    let model = project.models.get_mut(2).unwrap();
    model.efforts = List::with_capacity(2);
    model.efforts.push(model_text(b"low")).unwrap();
    model.efforts.push(model_text(b"high")).unwrap();
    model.effort = Some(model_text(b"high"));
    project.defaults = List::with_capacity(2);
    for (template, name, effort) in [
        (jig_client::shapes::CHAT + 1, b"catalogue-first".as_slice(), None),
        (jig_client::shapes::CHAT, b"second".as_slice(), Some(model_text(b"high"))),
    ] {
        project
            .defaults
            .push(c::MeDefault {
                template,
                choice: c::LlmChoice {
                    later: None,
                    account: None,
                    main: c::ModelEffort { model: model_text(name), effort },
                    subagents: List::with_capacity(0),
                },
            })
            .unwrap();
    }
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    assert_eq!(using_text(&root), "second · high");
}
#[test]
fn the_three_unsettled_lines_name_only_known_facts() {
    assert_eq!(using_text(&task_root()), "chosen");
    assert_eq!(using_text(&banner(None).0), "Using the default model");
    assert_eq!(using_text(&account_readiness(Some(readiness(c::Readiness::Unavailable)))), "chosen");
}

#[test]
fn own_key_and_plan_of_openai_follow_place_order() {
    for (first, second, expected) in [
        (c::AccountKind::Key, c::AccountKind::SignIn, "OpenAI API key · chosen"),
        (c::AccountKind::SignIn, c::AccountKind::Key, "ChatGPT plan · chosen"),
    ] {
        let mut rows = accounts(&[account(9, first, None), account(3, second, None)]);
        rows.get_mut(0).unwrap().place = 1;
        rows.get_mut(1).unwrap().place = 2;
        let (mut root, me_watch, _, _) =
            line_fixture(Some(readiness(c::Readiness::Usable)), rows, None, None, None, true);
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        me.providers.get_mut(0).unwrap().name =
            c::DocumentText::complete(c::Text::new(Box::from(b"openai".as_slice()), 32).unwrap());
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        assert_eq!(using_text(&root), expected);
    }
}
#[test]
fn another_members_chat_and_project_work_never_name_the_viewers_plan() {
    for funder in [
        c::Funder::Pool { project: fixtures::PROJECT, person: 74, period: 1 },
        c::Funder::Period { project: fixtures::PROJECT, period: 1 },
    ] {
        let (mut root, _, task_watch, accounts_watch) = line_fixture(
            Some(readiness(c::Readiness::Usable)),
            accounts(&[account(3, c::AccountKind::SignIn, None)]),
            Some(3),
            Some(3),
            None,
            true,
        );
        let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
        task.budget.funder = funder;
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        doc.readiness.get_mut(0).unwrap().payer = match funder {
            c::Funder::Pool { person, .. } => c::AccountUse::Party(person),
            c::Funder::Period { project, .. } => c::AccountUse::Project(project),
            c::Funder::Recurring { .. } | c::Funder::Task(_) => unreachable!("fixture funder"),
        };
        delivered(&mut root, task_watch.unwrap(), c::Delivery::Task(Box::new(task)));
        delivered(&mut root, accounts_watch, c::Delivery::Accounts(Box::new(doc)));
        let line = using_text(&root);
        assert_eq!(line, "chosen");
        assert!(!line.contains("plan") && !line.contains("API key"));
    }
}
#[test]
fn a_held_pin_stays_pinned_while_fresh_selection_passes_to_the_next_account() {
    for state in [
        c::AccountState::Later { until: None },
        c::AccountState::SignInAgain,
        c::AccountState::Misconfigured,
        c::AccountState::Unloaded { at: c::AccountPlace::File, why: c::AccountUnread::Unreadable },
        c::AccountState::Cancelled,
        c::AccountState::Closing,
    ] {
        let mut held = account(3, c::AccountKind::SignIn, None);
        held.sources.get_mut(1).unwrap().state = state;
        if let c::AccountState::Later { .. } = state {
            // An unusable Later source waits in both possible underlying states.
            held.sources.get_mut(1).unwrap().usable = false;
        } else {
            // Even contradictory usable evidence cannot override a held deciding source.
            assert!(held.sources.get(1).unwrap().usable);
        }
        let rows = accounts(&[held, account(7, c::AccountKind::Key, None)]);
        for (pin, expected) in [(Some(3), "chosen"), (None, "Supplied Provider API key · chosen")] {
            let root = line_facts(Some(readiness(c::Readiness::Usable)), rows.clone(), pin, None, None, true);
            assert_eq!(using_text(&root), expected);
        }
    }
}
#[test]
fn a_moving_source_is_held_even_if_supplied_as_usable() {
    let mut moving = account(3, c::AccountKind::SignIn, None);
    moving.sources.get_mut(1).unwrap().state = c::AccountState::Moving;
    assert!(moving.sources.get(1).unwrap().usable);
    let rows = accounts(&[moving, account(7, c::AccountKind::Key, None)]);
    for (pin, expected) in [(Some(3), "chosen"), (None, "Supplied Provider API key · chosen")] {
        let root = line_facts(Some(readiness(c::Readiness::Usable)), rows.clone(), pin, None, None, true);
        assert_eq!(using_text(&root), expected);
    }
}
#[test]
fn a_waiting_direct_source_does_not_jump_to_a_later_direct_source() {
    let mut waiting = account(3, c::AccountKind::Key, None);
    waiting.sources.get_mut(0).unwrap().kind = c::AccountKind::SignIn;
    let rows = accounts(&[waiting, account(7, c::AccountKind::SignIn, None)]);
    for (pin, expected) in [(Some(3), "chosen"), (None, "Supplied Provider plan · chosen")] {
        let root = line_facts(Some(readiness(c::Readiness::Usable)), rows.clone(), pin, None, None, true);
        assert_eq!(using_text(&root), expected);
    }
}
#[test]
fn ineligible_preference_passes_to_the_kept_account() {
    let mut wrong = account(3, c::AccountKind::Key, None);
    wrong.uses = Some(List::with_capacity(0));
    let root = line_facts(
        Some(readiness(c::Readiness::Usable)),
        accounts(&[wrong, account(7, c::AccountKind::SignIn, None)]),
        Some(3),
        Some(7),
        None,
        true,
    );
    assert_eq!(using_text(&root), "Supplied Provider plan · chosen");
}
#[test]
fn truncated_documents_and_accounts_name_only_the_model() {
    for cut_document in [true, false] {
        let (mut root, _, _, watch) = line_fixture(
            Some(readiness(c::Readiness::Usable)),
            accounts(&[account(3, c::AccountKind::Key, None)]),
            Some(3),
            None,
            None,
            true,
        );
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        if cut_document {
            doc.cut = true;
        } else {
            doc.accounts.get_mut(0).unwrap().cut = true;
        }
        delivered(&mut root, watch, c::Delivery::Accounts(Box::new(doc)));
        assert_eq!(using_text(&root), "chosen");
    }
}
#[test]
fn the_offer_cross_checks_the_translated_account_across_expiry() {
    for ready in [c::Readiness::Unknown, c::Readiness::Unavailable] {
        let root = line_facts(
            Some(readiness(ready)),
            accounts(&[account(3, c::AccountKind::Key, None)]),
            Some(3),
            None,
            None,
            true,
        );
        assert_eq!(using_text(&root), "Supplied Provider API key · chosen");
    }
}

#[test]
fn mixed_or_cut_program_offers_do_not_guess_a_models_endpoint() {
    for cut in [false, true] {
        let (mut root, me_watch, _, _) = line_fixture(
            Some(readiness(c::Readiness::Usable)),
            accounts(&[account(3, c::AccountKind::Key, None)]),
            Some(3),
            None,
            None,
            true,
        );
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        let offer = me.providers.get_mut(0).unwrap();
        offer.programs = List::with_capacity(1);
        offer.programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
        offer.programs_cut = cut;
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        assert_eq!(using_text(&root), "chosen");
    }
}

#[test]
fn a_usable_later_source_names_no_account() {
    for (ready, pin) in [
        (c::Readiness::Usable, None),
        (c::Readiness::Unavailable, None),
        (c::Readiness::Usable, Some(3)),
        (c::Readiness::Unavailable, Some(3)),
    ] {
        let mut first = account(3, c::AccountKind::Key, None);
        first.sources.get_mut(1).unwrap().state = c::AccountState::Later { until: None };
        let rows = accounts(&[first, account(7, c::AccountKind::SignIn, None)]);
        let (mut root, me_watch, _, _) = line_fixture(Some(readiness(ready)), rows, pin, None, None, true);
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        me.providers.get_mut(0).unwrap().name =
            c::DocumentText::complete(c::Text::new(Box::from(b"openai".as_slice()), 32).unwrap());
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        assert_eq!(using_text(&root), "chosen");
    }
}
#[test]
fn an_unusable_later_program_source_names_no_account_when_a_program_source_after_it_would_be_chosen() {
    for (ready, pin, after_usable) in [
        (c::Readiness::Usable, None, true),
        (c::Readiness::Unavailable, None, true),
        (c::Readiness::Usable, Some(3), true),
        (c::Readiness::Unavailable, Some(3), true),
        (c::Readiness::Usable, None, false),
    ] {
        // jig's Later goes on to the account's next program source; its Limited waits.
        let mut first = account(3, c::AccountKind::Program, None);
        first.sources.get_mut(0).unwrap().state = c::AccountState::Later { until: None };
        first.sources.get_mut(1).unwrap().usable = after_usable;
        let rows = accounts(&[first, account(7, c::AccountKind::Program, None)]);
        let (mut root, me_watch, _, _) = line_fixture(Some(readiness(ready)), rows, pin, None, None, true);
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        let offer = me.providers.get_mut(0).unwrap();
        offer.name = c::DocumentText::complete(c::Text::new(Box::from(b"Claude".as_slice()), 32).unwrap());
        offer.key = false;
        offer.sign_in = false;
        offer.programs = List::with_capacity(1);
        offer.programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        let expected = match (after_usable, ready) {
            (true, _) => "chosen",
            // Both of jig's states wait on this account, so the next one is chosen.
            (false, _) => "your Claude subscription, through Claude Code · chosen",
        };
        assert_eq!(using_text(&root), expected, "ready {ready:?}, pin {pin:?}");
    }
}

#[test]
fn configured_provider_words_name_the_whole_byline() {
    for (provider, kind, address, expected) in [
        (b"openai".as_slice(), c::AccountKind::Key, None, "OpenAI API key · chosen"),
        (b"openai", c::AccountKind::SignIn, None, "ChatGPT plan · chosen"),
        (
            b"openai",
            c::AccountKind::SignIn,
            Some(b"https://provider.example/usage".as_slice()),
            "ChatGPT plan · chosen",
        ),
        (b"anthropic", c::AccountKind::Program, None, "your Claude subscription, through Claude Code · chosen"),
        (b"local", c::AccountKind::Key, None, "local API key · chosen"),
    ] {
        let mut row = account(3, kind, address);
        row.sources.clear();
        let mut first = source(kind, true);
        first.name = Some(Box::from(b"a-login-not-a-program-name".as_slice()));
        row.sources.push(first).unwrap();
        let (mut root, me_watch, _, _) =
            line_fixture(Some(readiness(c::Readiness::Usable)), accounts(&[row]), Some(3), None, None, true);
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        let offer = me.providers.get_mut(0).unwrap();
        offer.name = c::DocumentText::complete(c::Text::new(Box::from(provider), 32).unwrap());
        if kind == c::AccountKind::Program {
            offer.key = false;
            offer.sign_in = false;
            offer.programs = List::with_capacity(1);
            offer.programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
        }
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        assert_eq!(using_text(&root), expected);
    }
}

#[test]
fn the_agent_composers_using_words_stay_whole() {
    let root = line_facts(
        Some(readiness(c::Readiness::Usable)),
        accounts(&[account(3, c::AccountKind::SignIn, Some(b"https://provider.example/usage"))]),
        Some(3),
        None,
        None,
        true,
    );
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::choice::using(&mut b, &root, root.pages().chat().unwrap().task(root.client()), &fixtures::words());
    assert_eq!(
        core::str::from_utf8(&fixtures::text(&b.finish())).unwrap(),
        "Using Supplied Provider plan · chosen · Manage usage"
    );
}

#[test]
fn a_host_wait_names_the_account_and_its_program_command() {
    let mut row = account(3, c::AccountKind::Program, None);
    row.sources.clear();
    let mut program = source(c::AccountKind::Program, false);
    program.name = Some(Box::from(b"claude-alice".as_slice()));
    row.sources.push(program).unwrap();
    let root = account_facts(None, accounts(&[row]));
    let mut b = Builder::new(&fixtures::tree_limits());
    b.open(Role::Page);
    crate::choice::waiting(
        &mut b,
        &root,
        &waiting_task(Some(c::TaskRunWait::Account { account: Some(3), source: Some(1), why: c::TaskWaitWhy::Host })),
        &fixtures::words(),
        &mut crate::bindings::Bindings::new(32),
        &mut crate::words::Times::new(fixtures::NOW),
    );
    b.close();
    let tree = b.finish();
    assert!(
        fixtures::find(&tree, Role::Paragraph, b"Personal's login isn't signed in on a host that can run this.")
            .is_some()
    );
    assert!(
        String::from_utf8(fixtures::text(&tree).into_vec())
            .unwrap()
            .contains("temper program-login --config CONFIG --data DATA --cache CACHE claude-alice")
    );
    assert!(fixtures::find(&tree, Role::Link, b"Accounts").is_some());
}

#[test]
fn a_choice_outside_the_offer_names_its_missing_account() {
    let (mut root, watch, _, _) =
        line_fixture(Some(readiness(c::Readiness::Usable)), List::with_capacity(0), None, None, None, true);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.clear();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    assert_eq!(using_text(&root), "chosen");
}
#[test]
fn a_draft_outside_the_offer_names_its_missing_account() {
    let (mut root, watch, _, _) = line_fixture(None, List::with_capacity(0), None, None, None, false);
    edit(&mut root, d::pages::choice::Act::Model { index: 2, account: None });
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.clear();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    assert_eq!(using_text(&root), "second");
}
#[test]
fn the_playgrounds_offered_model_ignores_document_readiness() {
    let root = fixtures::hidden_playground_root(Some(c::Readiness::Unavailable), 73);
    assert_eq!(using_text(&root), "Scripted model · Default");
}
#[test]
fn a_missing_preview_never_uses_the_catalogues_first_model() {
    let root = line_facts(Some(readiness(c::Readiness::Usable)), List::with_capacity(0), None, None, None, false);
    assert_eq!(using_text(&root), "Using the default model");
}

#[test]
fn an_empty_wait_names_only_a_choice_the_task_holds() {
    for (choice, expected) in [
        (None, "No account can run this chat yet."),
        (
            Some(c::LlmChoice {
                later: None,
                account: None,
                main: c::ModelEffort { model: model_text(b"explicit"), effort: None },
                subagents: List::with_capacity(0),
            }),
            "No account can run explicit yet.",
        ),
    ] {
        let root = task_root();
        let mut task =
            waiting_task(Some(c::TaskRunWait::Account { account: None, source: None, why: c::TaskWaitWhy::Empty }));
        task.llm = choice;
        let mut b = Builder::new(&fixtures::tree_limits());
        let mut bindings = crate::bindings::Bindings::new(32);
        crate::choice::waiting(
            &mut b,
            &root,
            &task,
            &fixtures::words(),
            &mut bindings,
            &mut crate::words::Times::new(fixtures::NOW),
        );
        let tree = b.finish();
        assert!(fixtures::find(&tree, Role::Paragraph, expected.as_bytes()).is_some());
        assert!(!String::from_utf8_lossy(&fixtures::text(&tree)).contains("chosen"));
        assert!(fixtures::find(&tree, Role::Link, b"Connect a model").is_some());
    }
}

#[test]
fn an_offered_display_name_is_used_on_the_line() {
    let (mut root, watch, _, _) = line_fixture(None, List::with_capacity(0), None, None, None, true);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap().shown = Some(model_text(b"Offered name"));
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    assert_eq!(using_text(&root), "Offered name");
}

use super::fixtures::choice::{accounts, line_facts};
