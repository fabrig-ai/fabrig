use super::fixtures;
use crate::look::*;
use skein_ui::Role;

#[test]
fn the_frame_carries_its_tokens() {
    let view = fixtures::page_view(&fixtures::no_project_root());
    assert_eq!(fixtures::styles(view.tree(), Role::Banner).as_ref(), [Box::from([TOP])]);
}
#[test]
fn the_looks_names_pass_skeins_check() {
    assert!(skein_ui::look::check(&[NAMES]).is_none());
    for (token, name) in [
        (WHY_OFFERS, b"temper-why-offers".as_slice()),
        (TOP, b"temper-top".as_slice()),
        (TOP_END, b"temper-top-end".as_slice()),
        (OUTSIDE, b"temper-outside".as_slice()),
        (OUTSIDE_PRIMARY, b"temper-outside-primary".as_slice()),
        (OUTSIDE_SECONDARY, b"temper-outside-secondary".as_slice()),
        (OUTSIDE_ACTS, b"temper-outside-acts".as_slice()),
        (OUTSIDE_QUIET, b"temper-outside-quiet".as_slice()),
        (APPROVE_FIELD, b"temper-approve-field".as_slice()),
        (PUBLIC_PAGE, b"temper-public-page".as_slice()),
        (PUBLIC_INTRO, b"temper-public-intro".as_slice()),
        (PUBLIC_LEDE, b"temper-public-lede".as_slice()),
        (PUBLIC_ROWS, b"temper-public-rows".as_slice()),
        (PUBLIC_ROW, b"temper-public-row".as_slice()),
        (PUBLIC_LINE, b"temper-public-line".as_slice()),
        (PUBLIC_FACTS, b"temper-public-facts".as_slice()),
        (PUBLIC_SECTION, b"temper-public-section".as_slice()),
        (CHAT_GOAL, b"temper-chat-goal".as_slice()),
        (WRITING_END, b"temper-writing-end"),
        (WELCOME_COPY, b"temper-welcome-copy"),
        (WELCOME_CMD_BOX, b"temper-welcome-cmd-box"),
        (CONFIRM, b"temper-confirm"),
        (CONFIRM_FATES, b"temper-confirm-fates"),
        (CONFIRM_ACTS, b"temper-confirm-acts"),
        (CONFIRM_DO, b"temper-confirm-do"),
        (RAIL_SEP, b"temper-rail-sep"),
        (WORK_WAY, b"temper-work-way"),
    ] {
        assert_eq!(NAMES.style(token), Some(name));
    }
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary tests scan the browser stylesheet as UTF-8 text"
)]
fn every_class_the_stylesheet_selects_is_a_look_name() {
    let mut source = include_str!("../../../temper-client-shell-web/temper.css");
    let mut css = String::new();
    while let Some((before, comment)) = source.split_once("/*") {
        css.push_str(before);
        let (_, after) = comment.split_once("*/").expect("stylesheet comment ends");
        source = after;
    }
    css.push_str(source);
    let mut classes = alloc::collections::BTreeSet::new();
    for block in css.split('{') {
        let selector = block.rsplit('}').next().expect("selector before declarations");
        for class in selector.split(".temper-").skip(1) {
            let end = class.find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_').unwrap_or(class.len());
            let name = format!("temper-{}", &class[..end]);
            assert!(!name.starts_with("temper-reserved-"), "stylesheet selects reserved slot: {name}");
            assert!(NAMES.styles.contains(&name.as_bytes()), "stylesheet selects unknown look: {name}");
            classes.insert(name);
        }
    }
    assert!(!classes.is_empty(), "stylesheet selects look classes");
}
#[test]
fn model_line_styles_name_the_reason_and_acts() {
    for (style, name) in [
        (WELCOME_STATUS_WHY, b"temper-welcome-status-why".as_slice()),
        (WELCOME_STATUS_ACTS, b"temper-welcome-status-acts"),
    ] {
        assert_eq!(NAMES.style(style), Some(name));
    }
}
const FLOW_NAMES: [(skein_ui::Style, &[u8]); 25] = [
    (FLOW_BUSY, b"temper-flow-busy"),
    (FLOW_METER, b"temper-flow-meter"),
    (FLOW_PRIMARY, b"temper-flow-primary"),
    (FLOW_FAILED, b"temper-flow-failed"),
    (FLOW_PAIR, b"temper-flow-pair"),
    (WELCOME_INTRO, b"temper-welcome-intro"),
    (PROJECT_EMPTY, b"temper-project-empty"),
    (FLOW_SECONDARY, b"temper-flow-secondary"),
    (FLOW_QUIET, b"temper-flow-quiet"),
    (FLOW_CHOICES, b"temper-flow-choices"),
    (FLOW_CHOICE, b"temper-flow-choice"),
    (FLOW_ACTIONS, b"temper-flow-actions"),
    (FLOW_FIELD, b"temper-flow-field"),
    (FLOW_FACTS, b"temper-flow-facts"),
    (MODEL_CHOICES, b"temper-model-choices"),
    (FLOW_DETAILS, b"temper-flow-details"),
    (MODEL_KEY, b"temper-model-key"),
    (ADMIN_FIRST, b"temper-admin-first"),
    (ADMIN_LEGEND, b"temper-admin-legend"),
    (MODEL_FACTS, b"temper-model-facts"),
    (MODEL_ACTIONS, b"temper-model-actions"),
    (FLOW_SUBTITLE, b"temper-flow-subtitle"),
    (MODEL_DESCRIPTION, b"temper-model-description"),
    (MODEL_FACT, b"temper-model-fact"),
    (MODEL_SIGN_IN, b"temper-model-sign-in"),
];
const HOME_NAMES: [(skein_ui::Style, &[u8]); 18] = [
    (HOME_TAG, b"temper-home-tag"),
    (HOME_SKIP, b"temper-home-skip"),
    (HOME_NOTE, b"temper-home-note"),
    (HOME_NEEDS, b"temper-home-needs"),
    (HOME_HEAD, b"temper-home-head"),
    (HOME_SIGNAL, b"temper-home-signal"),
    (HOME_IN_A_ROW, b"temper-home-in-a-row"),
    (HOME_ROWS, b"temper-home-rows"),
    (HOME_WHEN, b"temper-home-when"),
    (HOME_EMPTY, b"temper-home-empty"),
    (HOME_CHATS, b"temper-home-chats"),
    (HOME_CTA, b"temper-home-cta"),
    (HOME_CTA_GO, b"temper-home-cta-go"),
    (ASK, b"temper-ask"),
    (ASK_META, b"temper-ask-meta"),
    (ASK_QUESTION, b"temper-ask-question"),
    (ASK_CHOICES, b"temper-ask-choices"),
    (ASK_QUIET, b"temper-ask-quiet"),
];
const GOALS_NAMES: [(skein_ui::Style, &[u8]); 65] = [
    (GOALS, b"temper-goals"),
    (GOALS_INTRO, b"temper-goals-intro"),
    (GOALS_KICKER, b"temper-goals-kicker"),
    (GOALS_HEADLINE, b"temper-goals-headline"),
    (GOALS_LEDE, b"temper-goals-lede"),
    (GOALS_BYLINE, b"temper-goals-byline"),
    (GOALS_SECTION, b"temper-goals-section"),
    (GOALS_SECTION_HEAD, b"temper-goals-section-head"),
    (GOALS_SECTION_TITLE, b"temper-goals-section-title"),
    (GOALS_SIGNAL, b"temper-goals-signal"),
    (GOALS_EMPTY, b"temper-goals-empty"),
    (GOALS_FACTS, b"temper-goals-facts"),
    (GOALS_ROWS, b"temper-goals-rows"),
    (GOALS_ROW, b"temper-goals-row"),
    (GOALS_ROW_ENDED, b"temper-goals-row-ended"),
    (GOALS_TOP, b"temper-goals-top"),
    (GOALS_LINE, b"temper-goals-line"),
    (GOALS_FILTERS, b"temper-goals-filters"),
    (GOALS_MORE, b"temper-goals-more"),
    (GOALS_STATE_NEEDS, b"temper-goals-state-needs"),
    (GOALS_STATE_RUNNING, b"temper-goals-state-running"),
    (GOALS_STATE_PLANNING, b"temper-goals-state-planning"),
    (GOALS_STATE_PAUSED, b"temper-goals-state-paused"),
    (GOALS_STATE_LANDED, b"temper-goals-state-landed"),
    (GOALS_STATE_CLOSED, b"temper-goals-state-closed"),
    (GOAL, b"temper-goal"),
    (GOAL_HEAD, b"temper-goal-head"),
    (GOAL_META, b"temper-goal-meta"),
    (GOAL_BYLINE, b"temper-goal-byline"),
    (GOAL_ACTS, b"temper-goal-acts"),
    (GOAL_BTN, b"temper-goal-btn"),
    (GOAL_SMALL, b"temper-goal-small"),
    (GOAL_QUIET, b"temper-goal-quiet"),
    (GOAL_PRIMARY, b"temper-goal-primary"),
    (GOAL_BAND, b"temper-goal-band"),
    (GOAL_BAND_TAG, b"temper-goal-band-tag"),
    (GOAL_CHOICES, b"temper-goal-choices"),
    (GOAL_TWO, b"temper-goal-two"),
    (GOAL_REPORT, b"temper-goal-report"),
    (GOAL_TITLE, b"temper-goal-title"),
    (GOAL_TYPING, b"temper-goal-typing"),
    (GOAL_SIDE, b"temper-goal-side"),
    (GOAL_SECTION, b"temper-goal-section"),
    (GOAL_FACTS, b"temper-goal-facts"),
    (GOAL_BAR, b"temper-goal-bar"),
    (GOAL_BAR_LANDED, b"temper-goal-bar-landed"),
    (GOAL_BAR_RUNNING, b"temper-goal-bar-running"),
    (GOAL_LINE, b"temper-goal-line"),
    (GOAL_LINE_RUNNING, b"temper-goal-line-running"),
    (GOAL_LINE_NEEDS, b"temper-goal-line-needs"),
    (GOAL_LANDED_FOLD, b"temper-goal-landed-fold"),
    (GOAL_SENT, b"temper-goal-sent"),
    (GOAL_YOU, b"temper-goal-you"),
    (GOAL_COMPOSER_BOX, b"temper-goal-composer-box"),
    (GOAL_COMPOSER, b"temper-goal-composer"),
    (GOAL_HINT, b"temper-goal-hint"),
    (GOAL_FIELD, b"temper-goal-field"),
    (GOAL_TEXT, b"temper-goal-text"),
    (SHEET, b"temper-sheet"),
    (SHEET_MUTED, b"temper-sheet-muted"),
    (SHEET_FIELDSET, b"temper-sheet-fieldset"),
    (SHEET_OPTION, b"temper-sheet-option"),
    (SHEET_LINK, b"temper-sheet-link"),
    (SHEET_FOOT, b"temper-sheet-foot"),
    (SHEET_ERROR, b"temper-sheet-error"),
];
const RAIL_NAMES: [(skein_ui::Style, &[u8]); 12] = [
    (RAIL, b"temper-rail"),
    (RAIL_NAV, b"temper-rail-nav"),
    (RAIL_NEW, b"temper-rail-new"),
    (RAIL_SEP, b"temper-rail-sep"),
    (RAIL_FOOT, b"temper-rail-foot"),
    (RAIL_PERSON, b"temper-rail-person"),
    (TABBAR, b"temper-tabbar"),
    (TABBAR_BEGINNING, b"temper-tabbar-beginning"),
    (TABBAR_ASK, b"temper-tabbar-ask"),
    (AWAY, b"temper-away"),
    (AWAY_CHIP, b"temper-away-chip"),
    (AWAY_TRACK, b"temper-away-track"),
];
#[test]
#[expect(clippy::too_many_lines, reason = "the table checks the closed vocabulary of view tokens")]
fn each_style_or_region_is_named_under_temper() {
    let named: [(skein_ui::Style, &[u8]); 110] = [
        (MODE_CHAT, b"temper-mode-chat"),
        (MODE_SPLIT, b"temper-mode-split"),
        (MODE_DRAFT, b"temper-mode-draft"),
        (DOC, b"temper-doc"),
        (DOC_INNER, b"temper-doc-inner"),
        (DOC_PATH, b"temper-doc-path"),
        (DOC_SEC, b"temper-doc-sec"),
        (DOC_CHANGED, b"temper-doc-changed"),
        (DOC_NOTE, b"temper-doc-note"),
        (DOC_FLASH, b"temper-doc-flash"),
        (DOC_EMPTY, b"temper-doc-empty"),
        (ADD_DRAFT, b"temper-add-draft"),
        (DOC_MUTED, b"temper-doc-muted"),
        (SEG, b"temper-seg"),
        (SEG_SPLIT, b"temper-seg-split"),
        (RAIL_UNFOLD, b"temper-rail-unfold"),
        (START_OVER, b"temper-start-over"),
        (PROPOSAL_FACTS, b"temper-proposal-facts"),
        (SESSION_RIGHT, b"temper-session-right"),
        (META_LINE, b"temper-meta-line"),
        (THREAD_INNER, b"temper-thread-inner"),
        (TYPING, b"temper-typing"),
        (TURN_TOOL, b"temper-turn-tool"),
        (TOOL_FILE, b"temper-tool-file"),
        (BY_DELEGATE, b"temper-by-delegate"),
        (NAV_COUNT, b"temper-nav-count"),
        (NAVIGATION, b"temper-navigation"),
        (MAIN, b"temper-main"),
        (CARD, b"temper-card"),
        (TURN, b"temper-turn"),
        (COMPOSER, b"temper-composer"),
        (BRAND, b"temper-brand"),
        (LIVE, b"temper-live"),
        (CONNECTING, b"temper-connecting"),
        (OFFLINE, b"temper-offline"),
        (BEHIND, b"temper-behind"),
        (ROWS, b"temper-rows"),
        (ROW, b"temper-row"),
        (ENTRY, b"temper-entry"),
        (STATUS_YOURS, b"temper-status-yours"),
        (STATUS_HELD, b"temper-status-held"),
        (STATUS_RETRYING, b"temper-status-retrying"),
        (STATUS_ACTIVE, b"temper-status-active"),
        (STATUS_QUIET, b"temper-status-quiet"),
        (TIME, b"temper-time"),
        (HINT, b"temper-hint"),
        (ACCOUNTS, b"temper-accounts"),
        (ACCOUNT_LINE, b"temper-account-line"),
        (AGENT_PAGE, b"temper-agent-page"),
        (TASKBAR, b"temper-taskbar"),
        (THREAD, b"temper-thread"),
        (MESSAGE, b"temper-message"),
        (BY_PERSON, b"temper-by-person"),
        (BY_AGENT, b"temper-by-agent"),
        (BY_TOOL, b"temper-by-tool"),
        (BY_SYSTEM, b"temper-by-system"),
        (WHO, b"temper-who"),
        (SAYS, b"temper-says"),
        (WORK, b"temper-work"),
        (MARKER, b"temper-marker"),
        (CARDS, b"temper-cards"),
        (COMPOSER_AREA, b"temper-composer-area"),
        (PAGE, b"temper-page"),
        (PAGEBAR, b"temper-pagebar"),
        (HEADING, b"temper-heading"),
        (FILTERS, b"temper-filters"),
        (UPDATES, b"temper-updates"),
        (META, b"temper-meta"),
        (TITLE, b"temper-title"),
        (DETAILS, b"temper-details"),
        (KIND_PROPOSAL, b"temper-kind-proposal"),
        (KIND_QUESTION, b"temper-kind-question"),
        (KIND_HELD, b"temper-kind-held"),
        (KIND_PERSON_TASK, b"temper-kind-person-task"),
        (KIND_RESULT, b"temper-kind-result"),
        (KIND_REPLY, b"temper-kind-reply"),
        (TASK_HEADING, b"temper-task-heading"),
        (TASK_CARD, b"temper-task-card"),
        (PLAN, b"temper-plan"),
        (PLAN_ROW, b"temper-plan-row"),
        (PLAN_HOLD, b"temper-plan-hold"),
        (STATUS_DONE, b"temper-status-done"),
        (CHOICE, b"temper-choice"),
        (ACCOUNTS_PAGE, b"temper-accounts-page"),
        (SETTINGS_COL, b"temper-settings-col"),
        (ADOPTION_PAGE, b"temper-adoption-page"),
        (REFUSAL, b"temper-refusal"),
        (STEPS, b"temper-steps"),
        (STEP, b"temper-step"),
        (STEP_DONE, b"temper-step-done"),
        (PLAN_SPEND, b"temper-plan-spend"),
        (DEPENDENCIES, b"temper-dependencies"),
        (STATUS_WAITING, b"temper-status-waiting"),
        (AGENT_LINEAGE, b"temper-agent-lineage"),
        (AGENT_BRIEF, b"temper-agent-brief"),
        (AGENT_RUN, b"temper-agent-run"),
        (AGENT_RESULT, b"temper-agent-result"),
        (AGENT_FOOT, b"temper-agent-foot"),
        (REPOSITORY, b"temper-repository"),
        (TASK_ACTS, b"temper-task-acts"),
        (AGENT_STEER, b"temper-agent-steer"),
        (TASK_AMEND, b"temper-task-amend"),
        (DAY, b"temper-day"),
        (CHANGE, b"temper-change"),
        (CHANGE_TAG, b"temper-change-tag"),
        (STEP_CURRENT, b"temper-step-current"),
        (CHANGE_HOLD, b"temper-change-hold"),
        (GATE, b"temper-gate"),
        (PLAN_DONE, b"temper-plan-done"),
        (ENDED, b"temper-ended"),
    ];
    let chats = [
        (CHATS_PAGE, b"temper-chats-page".as_slice()),
        (CHATS_INTRO, b"temper-chats-intro".as_slice()),
        (CHATS_KICKER, b"temper-chats-kicker".as_slice()),
        (CHATS_LEDE, b"temper-chats-lede".as_slice()),
        (CHATS_BAND, b"temper-chats-band".as_slice()),
        (CHATS_TAG, b"temper-chats-tag".as_slice()),
        (CHATS_SECTION, b"temper-chats-section".as_slice()),
        (CHATS_SECTION_HEAD, b"temper-chats-section-head".as_slice()),
        (CHATS_ACT, b"temper-chats-act".as_slice()),
        (CHATS_MORE, b"temper-chats-more".as_slice()),
        (CHATS_ROWS, b"temper-chats-rows".as_slice()),
        (CHATS_ROW, b"temper-chats-row".as_slice()),
        (CHATS_ROW_ENDED, b"temper-chats-row-ended".as_slice()),
        (CHATS_TOP, b"temper-chats-top".as_slice()),
        (CHATS_LINE, b"temper-chats-line".as_slice()),
        (STATE_NEEDS, b"temper-state-needs".as_slice()),
    ];
    for (style, name) in
        named.into_iter().chain(FLOW_NAMES).chain(RAIL_NAMES).chain(HOME_NAMES).chain(chats).chain(GOALS_NAMES)
    {
        assert_eq!(NAMES.style(style), Some(name));
    }
    assert_eq!(NAMES.region(crate::frame::link_region()), Some(&b"temper-link"[..]));
}
#[test]
fn each_pages_main_carries_the_main_token() {
    let builds: [fn() -> temper_client_domain::Domain; 2] = [fixtures::signed_out, fixtures::out_of_date];
    for build in builds {
        let view = fixtures::page_view(&build());
        assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([OUTSIDE])]);
    }
    let view = fixtures::page_view(&fixtures::chats_root());
    assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([MAIN, CHATS_PAGE])]);
    let view = fixtures::page_view(&fixtures::task::held_root());
    assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([MAIN, GOAL])]);
    let view = fixtures::page_view(&fixtures::inbox_root());
    assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([MAIN, PAGE])]);
    let view = fixtures::page_view(&fixtures::account_root().0);
    assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([MAIN, SETTINGS])]);
    let view = fixtures::page_view(&fixtures::adoption_root());
    assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([MAIN, SETTINGS])]);
    let view = fixtures::page_view(&fixtures::chat::chat_root());
    assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([MODE_CHAT, SESSION])]);
}
#[test]
fn the_composers_carry_the_composer_token() {
    let view = agent_composer();
    assert_eq!(fixtures::styles(view.tree(), Role::TextArea).as_ref(), [Box::from([COMPOSER])]);
}

#[test]
fn the_chat_structure_tokens_stay_on_their_roles() {
    let view = fixtures::page_view(&super::chats::listed_root());
    for (token, role) in [
        (BRAND, Role::Link),
        (LIVE, Role::Status),
        (CHATS_ROWS, Role::List),
        (CHATS_ROW, Role::Item),
        (RAIL_FOOT, Role::Slot),
        (TIME, Role::Text),
    ] {
        let mut found = false;
        for node in view.tree().nodes() {
            if node.styles.as_slice().contains(&token) {
                assert_eq!(node.role, role);
                found = true;
            }
        }
        assert!(found, "{token:?}");
    }
    for (line, token) in super::chats::status_rows() {
        let (tree, _) = super::chats::tree(&line);
        assert_eq!(fixtures::styles(&tree, Role::Status).as_ref(), [super::chats::row_status_styles(token)]);
    }
}
#[test]
fn the_connection_tokens_stay_on_statuses() {
    let (mut root, watch) = fixtures::root_with_watch(None);
    let connecting = fixtures::built();
    assert_status_token(&connecting, CONNECTING);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
        },
    );
    let view = fixtures::page_view(&root);
    for token in [OFFLINE, BEHIND] {
        assert_status_token(&view, token);
    }
}

fn assert_status_token(view: &crate::View, token: skein_ui::Style) {
    for node in view.tree().nodes() {
        if node.role == Role::Status && node.styles.as_slice().contains(&token) {
            return;
        }
    }
    panic!("status token {token:?} absent");
}

#[test]
fn the_conversation_tokens_stay_on_their_roles() {
    let message_tree = fixtures::chat::tree(&super::chat::structured_thread());
    let proposal = fixtures::page_view(&fixtures::chat::proposal_root());
    let composer = fixtures::page_view(&fixtures::chat::messaging_root());
    let agent = agent_composer();
    for token in [AGENT_PAGE, TASKBAR, THREAD, MESSAGE, BY_PERSON, BY_AGENT, WHO, SAYS, WORK, CARDS, COMPOSER_AREA] {
        let role = match token {
            AGENT_PAGE => Role::Main,
            CARDS => Role::List,
            _ => Role::Slot,
        };
        let mut found = false;
        for tree in [&message_tree, proposal.tree(), composer.tree(), agent.tree()] {
            for node in tree.nodes() {
                if node.styles.as_slice().contains(&token) {
                    if token == WORK && node.role == Role::Disclosure {
                        assert!(!node.states.as_slice().contains(&skein_ui::State::Expanded));
                    } else {
                        assert_eq!(node.role, role);
                    }
                    found = true;
                }
            }
        }
        assert!(found, "{token:?}");
    }
}

#[test]
fn the_inbox_tokens_stay_on_their_roles() {
    let mut roots = skein_lib::List::with_capacity(3);
    roots.push(fixtures::inbox_root()).expect("inbox");
    roots
        .push(super::confirm::root(jig_client::contract::ActKind::Accept, jig_client::contract::Admission::Allowed))
        .expect("proposal");
    roots.push(super::inbox::question_root()).expect("question");
    for (token, role) in [
        (PAGE, Role::Main),
        (PAGEBAR, Role::Slot),
        (HEADING, Role::Slot),
        (FILTERS, Role::Slot),
        (UPDATES, Role::List),
        (META, Role::Slot),
        (TITLE, Role::Heading { level: 3 }),
        (PROPOSAL_FACTS, Role::Slot),
        (SUBTLE_COUNT, Role::Paragraph),
        (KIND_MENU, Role::Disclosure),
        (INBOX_TAG, Role::Paragraph),
        (KIND_PROPOSAL, Role::Item),
        (KIND_QUESTION, Role::Item),
        (KIND_HELD, Role::Item),
        (KIND_PERSON_TASK, Role::Item),
        (KIND_RESULT, Role::Item),
        (KIND_REPLY, Role::Item),
    ] {
        let mut found = false;
        for root in &roots {
            let view = fixtures::page_view(root);
            for node in view.tree().nodes() {
                if node.styles.as_slice().contains(&token) {
                    assert_eq!(node.role, role);
                    found = true;
                }
            }
        }
        assert!(found, "{token:?}");
    }
}

#[test]
fn the_task_tokens_stay_on_their_roles() {
    let view = fixtures::page_view(&super::task::held_agent_root());
    let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
    let mut task = fixtures::task::document();
    task.chip.phase = jig_client::contract::Phase::Done;
    crate::pages::task::header(
        &mut builder,
        &task,
        None,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        None,
    );
    let done = builder.finish();
    let plan = super::task::dependency_plan();
    for (token, role) in [
        (TASK_HEADING, Role::Region),
        (TASK_CARD, Role::Region),
        (PLAN, Role::Tree),
        (PLAN_ROW, Role::Slot),
        (PLAN_SPEND, Role::Paragraph),
        (DEPENDENCIES, Role::List),
        (PLAN_HOLD, Role::List),
        (STATUS_DONE, Role::Status),
        (STATUS_WAITING, Role::Status),
    ] {
        let mut found = false;
        for tree in [view.tree(), &done, &plan] {
            for node in tree.nodes() {
                if node.styles.as_slice().contains(&token) {
                    assert_eq!(node.role, role);
                    found = true;
                }
            }
        }
        assert!(found, "{token:?}");
    }
}

#[test]
fn the_choice_token_stays_on_the_composer_slot() {
    let mut roots = skein_lib::List::with_capacity(3);
    roots
        .push(fixtures::playground_root(
            temper_client_domain::Address::NewChat { draft: false },
            fixtures::PROJECT,
            true,
        ))
        .expect("new chat");
    roots.push(fixtures::chat::messaging_root()).expect("chat");
    roots.push(fixtures::task::root(jig_client::contract::ChipExecutor::Agent { charter: 1 }, None)).expect("task");
    for root in &roots {
        let view = fixtures::page_view(root);
        let mut found = false;
        for node in view.tree().nodes() {
            if node.styles.as_slice().contains(&CHOICE) {
                assert_eq!(node.role, Role::Slot);
                found = true;
            }
        }
        assert!(found);
    }
}

#[test]
fn the_adoption_page_keeps_its_heading() {
    let view = fixtures::page_view(&fixtures::adoption_root());
    let heading: Box<[skein_ui::Style]> = Box::from([SETTINGS_HEADLINE]);
    assert_eq!(fixtures::styles(view.tree(), Role::Heading { level: 1 }).as_ref(), [heading]);
    let mut found = false;
    for node in view.tree().nodes() {
        if node.styles.as_slice().contains(&SETTINGS_COL) {
            assert_eq!(node.role, Role::Slot);
        }
        if node.styles.as_slice().contains(&PAGEBAR) {
            assert_eq!(node.role, Role::Slot);
            found = true;
        }
    }
    assert!(!found);
}

#[test]
fn the_delegation_steps_keep_their_roles() {
    super::delegation::step_roles();
}

#[test]
fn account_rows_and_sign_ins_have_distinct_style_names() {
    assert_eq!(NAMES.style(SETTINGS_ROW), Some(b"temper-settings-row".as_slice()));
    assert_eq!(NAMES.style(MODEL_SIGN_IN), Some(b"temper-model-sign-in".as_slice()));
    assert_ne!(SETTINGS_ROW, MODEL_SIGN_IN);
}

fn agent_composer() -> crate::View {
    use jig_client::contract as c;
    let mut task = fixtures::task::delegate();
    task.acts = skein_lib::List::with_capacity(1);
    task.acts.push(c::Offered { act: c::ActKind::Message, admission: c::Admission::Allowed, effect: None }).unwrap();
    fixtures::page_view(&fixtures::task::agent_root(task, None, true))
}

#[test]
fn the_settings_controls_have_their_own_names_and_small_element() {
    for (style, name) in [
        (SETTINGS_CHECKS, b"temper-settings-checks".as_slice()),
        (SETTINGS_SWITCH, b"temper-settings-switch".as_slice()),
        (SETTINGS_TOGGLE_ROW, b"temper-settings-toggle-row".as_slice()),
        (SETTINGS_TEXT, b"temper-settings-text".as_slice()),
        (SETTINGS_FACTS, b"temper-settings-facts".as_slice()),
        (SETTINGS_REFUSAL, b"temper-settings-refusal".as_slice()),
    ] {
        assert_eq!(NAMES.style(style), Some(name));
    }
    assert!(ELEMENTS.contains(&(crate::frame::owner(), Role::Text, SETTINGS_DETAIL, "small")));
}

#[test]
fn asking_opening_and_needs_have_their_css_names() {
    for (style, name) in [
        (STARTER_LEDE, b"temper-starter-lede".as_slice()),
        (STARTER_WAYS, b"temper-starter-ways"),
        (WAYS_LEDE, b"temper-ways-lede"),
        (WAYS, b"temper-ways"),
        (WAYS_TABS, b"temper-ways-tabs"),
        (WAYS_NOW, b"temper-ways-now"),
        (WAYS_AXIS, b"temper-ways-axis"),
        (WAYS_PANEL, b"temper-ways-panel"),
        (WAYS_SAYS, b"temper-ways-says"),
        (WAYS_ZOOM, b"temper-ways-zoom"),
        (WAYS_ZOOM_HEAD, b"temper-ways-zoom-head"),
        (WAYS_FOOT, b"temper-ways-foot"),
        (WAYS_LEGEND, b"temper-ways-legend"),
        (WAYS_KIND_AGENT, b"temper-ways-kind-agent"),
        (WAYS_KIND_PERSON, b"temper-ways-kind-person"),
        (WAYS_KIND_CODE, b"temper-ways-kind-code"),
        (WAYS_KIND_LINE, b"temper-ways-kind-line"),
        (WAYS_ACTS, b"temper-ways-acts"),
        (WAYS_USE, b"temper-ways-use"),
        (WAYS_REFUSED, b"temper-ways-refused"),
        (TOAST, b"temper-toast"),
        (NEEDS, b"temper-needs"),
        (WHY, b"temper-why"),
        (NEEDS_BAR, b"temper-needs-bar"),
        (NEEDS_BAR_AT, b"temper-needs-bar-at"),
    ] {
        assert_eq!(NAMES.style(style), Some(name));
    }
}

#[test]
fn path_styles_keep_their_css_names() {
    for (token, name) in [
        (WAYS_MAYBE, b"temper-ways-maybe".as_slice()),
        (WAYS_WHY, b"temper-ways-why"),
        (WAYS_RISE_PERSON, b"temper-ways-rise-person"),
        (WAYS_RISE_AGENT, b"temper-ways-rise-agent"),
        (WAYS_FADE_PERSON, b"temper-ways-fade-person"),
        (WAYS_FADE_PERSON_FAINT, b"temper-ways-fade-person-faint"),
        (WAYS_FADE_AGENT, b"temper-ways-fade-agent"),
        (WAYS_FADE_AGENT_FAINT, b"temper-ways-fade-agent-faint"),
        (WAYS_FADE_BASE, b"temper-ways-fade-base"),
        (WAYS_KIND_MAYBE, b"temper-ways-kind-maybe"),
    ] {
        assert_eq!(NAMES.style(token), Some(name));
    }
}

#[test]
fn settings_styles_keep_their_css_names_beside_main_pages() {
    for (style, name) in [
        (SETTINGS_COL, b"temper-settings-col".as_slice()),
        (SETTINGS_INTRO, b"temper-settings-intro".as_slice()),
        (SETTINGS_KICKER, b"temper-settings-kicker".as_slice()),
        (SETTINGS_HEADLINE, b"temper-settings-headline".as_slice()),
        (SETTINGS_LEDE, b"temper-settings-lede".as_slice()),
        (SETTINGS_TABS, b"temper-settings-tabs".as_slice()),
        (SETTINGS_BODY, b"temper-settings-body".as_slice()),
        (SETTINGS_SETTING, b"temper-settings-setting".as_slice()),
        (SETTINGS_SIGN_OUT, b"temper-settings-sign-out"),
        (SETTINGS_SECTION, b"temper-settings-section".as_slice()),
        (SETTINGS_SECTION_HEAD, b"temper-settings-section-head".as_slice()),
        (SETTINGS_SECTION_TITLE, b"temper-settings-section-title".as_slice()),
        (SETTINGS_LINES, b"temper-settings-lines".as_slice()),
        (SETTINGS_WHEN, b"temper-settings-when".as_slice()),
        (SETTINGS_BAND, b"temper-settings-band".as_slice()),
        (SETTINGS_QUIET_BAND, b"temper-settings-quiet-band".as_slice()),
        (SETTINGS_TAG, b"temper-settings-tag".as_slice()),
        (SETTINGS_BTN, b"temper-settings-btn".as_slice()),
        (SETTINGS_PRIMARY, b"temper-settings-primary".as_slice()),
        (SETTINGS_SMALL, b"temper-settings-small".as_slice()),
        (SETTINGS_DETAIL, b"temper-settings-detail".as_slice()),
        (SETTINGS_END, b"temper-settings-end".as_slice()),
        (SETTINGS_ROW, b"temper-settings-row".as_slice()),
        (SETTINGS_PATH, b"temper-settings-path".as_slice()),
        (SETTINGS_TEXT, b"temper-settings-text".as_slice()),
    ] {
        assert_eq!(NAMES.style(style), Some(name));
    }
}

#[test]
fn welcome_and_repository_styles_keep_their_css_names() {
    for (style, name) in [
        (WELCOME_FORGE_STATUS, b"temper-welcome-forge-status".as_slice()),
        (WELCOME_SPIN, b"temper-welcome-spin"),
        (WELCOME_ACTS, b"temper-welcome-acts"),
        (WELCOME_OK, b"temper-welcome-ok"),
        (WELCOME_SEP, b"temper-welcome-sep"),
        (WELCOME_PANEL, b"temper-welcome-panel"),
        (WELCOME_AWAY, b"temper-welcome-away"),
        (WELCOME_AGAIN, b"temper-welcome-again"),
        (WELCOME_PASTE, b"temper-welcome-paste"),
        (WELCOME_FIELD, b"temper-welcome-field"),
        (WELCOME_CMD, b"temper-welcome-cmd"),
        (HOME_COPY, b"temper-home-copy"),
        (HOME_SMALL, b"temper-home-small"),
    ] {
        assert_eq!(NAMES.style(style), Some(name));
    }
}

#[test]
fn decision_tokens_name_the_away_rows() {
    for (token, name) in [
        (DECIDED, b"temper-decided".as_slice()),
        (DECISION, b"temper-decision"),
        (DECISION_ALLOWED, b"temper-decision-allowed".as_slice()),
        (DECISION_TOOK, b"temper-decision-took"),
        (DECISION_WHY, b"temper-decision-why"),
        (DECISION_LATE, b"temper-decision-late"),
        (DECISION_OVERTURNED, b"temper-decision-overturned"),
    ] {
        assert_eq!(NAMES.style(token), Some(name));
    }
}

#[test]
fn work_tokens_name_the_line_and_its_rows() {
    for (token, name) in [
        (WORK_RUN, b"temper-work-run".as_slice()),
        (WORK_RUNNING, b"temper-work-running"),
        (WORK_FAILED, b"temper-work-failed"),
        (WORK_CALLS, b"temper-work-calls"),
        (WORK_OUT, b"temper-work-out"),
    ] {
        assert_eq!(NAMES.style(token), Some(name));
    }
}

#[test]
fn the_writing_end_and_reading_controls_have_their_css_names() {
    for (token, name) in [
        (WRITING_END, b"temper-writing-end".as_slice()),
        (LATEST, b"temper-latest"),
        (THREAD_LOADING, b"temper-thread-loading"),
    ] {
        assert_eq!(NAMES.style(token), Some(name));
    }
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests parse the phone stylesheet's selectors and pixel heights"
)]
fn each_phone_target_stands_44_pixels() {
    let mut source = include_str!("../../../temper-client-shell-web/temper.css");
    let mut rules = Vec::new();
    while let Some((_, after)) = source.split_once("@media (max-width: 760px)") {
        let (_, body) = after.split_once('{').expect("phone media body");
        let mut depth = 1_u32;
        let end = body
            .char_indices()
            .find_map(|(at, ch)| {
                match ch {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                (depth == 0).then_some(at)
            })
            .expect("phone media ends");
        for rule in body[..end].split('}') {
            if let Some((selectors, declarations)) = rule.split_once('{') {
                // A section comment is not part of its first selector.
                let selectors = selectors.rsplit("*/").next().unwrap();
                rules.push((selectors.trim(), declarations));
            }
        }
        source = &body[end + 1..];
    }
    for target in [
        "#app > .temper-rail .temper-projects > summary",
        "#app > .temper-rail > .temper-away-chip::after",
        "#app > .temper-rail > .temper-night-switch::after",
        "#app > .temper-tabbar > a",
        "#app > .temper-session .temper-session-composer > button::after",
        "#app > .temper-session .temper-model > summary::before",
        "#app > .temper-session .temper-model-menu button::before",
        "#app > .temper-session .temper-model-menu a::before",
        "#app > .temper-session .temper-model-find",
        "#app > .temper-session .temper-ways-tabs > [role=tab]::after",
        "#app > .temper-session .temper-ways-acts > button::after",
        "#app > .temper-session .temper-ways-zoom-head > a::after",
        "#app > .temper-session .temper-crumbs > .temper-back::after",
        "#app > .temper-session .temper-session-right > button::after",
        "#app > .temper-home .temper-need-acts .temper-recommended",
        "#app > .temper-home .temper-need-card > .temper-need-more",
        "#app > .temper-home .temper-home-acts > :is(a, button)::after",
        "#app > .temper-home .temper-home-small::after",
        "#app > .temper-home .temper-home-skip::after",
        "#app > .temper-home .temper-home-head > :is(a, button)::after",
        "#app > .temper-home > .temper-home-phone > a",
        "#app > .temper-home .temper-home-rows > li > a",
        "#app > .temper-needs .temper-ask-choices button",
        "#app > .temper-needs .temper-ask-choices a.temper-recommended",
        "#app > .temper-needs .temper-ask-choices > details > summary",
        "#app > .temper-needs .temper-need-tell > button::after",
        "#app > .temper-needs .temper-ask-quiet::after",
        "#app > .temper-needs .temper-ask-saw > summary::after",
        "#app > .temper-needs .temper-ask-context > a::after",
        "#app > .temper-needs .temper-needs-home::after",
        "#app > .temper-needs .temper-decision > button::after",
        "#app > :is(.temper-home, .temper-needs) .temper-answered-undo::after",
        "#app > .temper-needs .temper-needs-bar > a::after",
        "#app > .temper-work-page .temper-work-chip::after",
        "#app > .temper-work-page .temper-work-card a::after",
        "#app > .temper-work-page .temper-work-row > a::after",
        "#app > .temper-work-page .temper-work-section-head > a::after",
    ] {
        assert!(
            rules.iter().any(|(selectors, declarations)| {
                selectors.split(",\n").any(|selector| selector.trim() == target)
                    && declarations.split(';').any(|declaration| {
                        let Some((property, value)) = declaration.split_once(':') else { return false };
                        property.trim() == "min-height"
                            && value
                                .trim()
                                .strip_suffix("px")
                                .and_then(|n| n.parse::<u32>().ok())
                                .is_some_and(|height| height >= 44)
                    })
            }),
            "phone target needs at least 44 px: {target}"
        );
    }
}

#[test]
fn team_and_how_resolve_to_distinct_style_names() {
    assert_eq!(NAMES.style(TEAM), Some(b"temper-team".as_slice()));
    assert_eq!(NAMES.style(HOW_WIDE), Some(b"temper-how-wide".as_slice()));
}
