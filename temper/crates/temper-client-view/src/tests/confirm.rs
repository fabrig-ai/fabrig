use super::fixtures;
pub(super) use super::fixtures::confirm::{PARTY, PROPOSAL, root};
use jig_client::contract::{self as c, ActKind, Admission, ObjectKey};
use skein_lib::List;
use skein_ui::Role;
use temper_client_domain::{Action, Event, pages::Act};

const ACCEPT_WORDS: [&[u8]; 4] = [b"Why", b"Funding needed: 5", b"Funding left: 20", b"Funding after: 15"];
#[test]
fn the_accept_confirmation_says_what_it_grants_costs_and_leaves_in_the_pool() {
    let mut root = root(ActKind::Accept, Admission::Allowed);
    open(&mut root, ActKind::Accept);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Accept this proposal?").is_some());
    let said = fixtures::text(tree);
    for expected in ACCEPT_WORDS {
        assert!(contains(&said, expected), "{expected:?}");
    }
}
#[test]
fn rejecting_says_so_with_its_reason_before_it_is_sent() {
    let mut root = root(ActKind::Reject, Admission::Allowed);
    press(&mut root, Act::CardWords { object: ObjectKey::Proposal(PROPOSAL), words: Box::from(*b"Too costly") });
    open(&mut root, ActKind::Reject);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Reject this proposal?").is_some());
    assert!(contains(&fixtures::text(tree), b"Too costly"));
}
#[test]
fn an_act_only_proposable_says_so_and_to_whom() {
    let root = root(ActKind::Accept, Admission::Proposed { to: c::Recipient::Party(PARTY) });
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Button, b"Accept").is_none());
    assert!(contains(&fixtures::text(tree), b"This needs a proposal to"));
}

// A three-place table, with room at 01:02 after the epoch; wall words use seconds.

#[test]
fn the_inbox_keeps_proposal_and_decision_facts_before_its_controls() {
    let view = fixtures::page_view(&root(ActKind::Accept, Admission::Allowed));
    let tree = view.tree();
    let details = super::inbox::styled(tree, crate::look::PROPOSAL_FACTS);
    let facts = super::inbox::subtree(tree, details);
    for expected in ACCEPT_WORDS {
        let mut found = false;
        for node in facts {
            for run in &node.text {
                found |= contains(&run.text, expected);
            }
        }
        assert!(found, "{expected:?}");
    }
    let accept = fixtures::find(tree, Role::Button, b"Accept").expect("accept");
    let mut control = None;
    let mut reason = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.id == accept {
            control = Some(index);
        }
        for run in &node.text {
            if run.text.as_ref() == b"Make a goal" {
                reason = Some(index);
            }
        }
    }
    assert!(control.expect("control") >= details.checked_add(facts.len()).expect("facts end"));
    for node in facts {
        assert_ne!(node.role, Role::Button);
        assert_ne!(node.role, Role::TextArea);
    }

    assert!(reason.expect("reason") > details);
}

#[test]
fn proposal_rejection_starts_in_a_closed_disclosure_in_both_pages() {
    for root in fixtures::pair(fixtures::chat::proposal_root(), root(ActKind::Reject, Admission::Allowed)) {
        let view = fixtures::page_view(&root);
        let nodes = view.tree().nodes();
        let at = super::chat::named_position(nodes, Role::Disclosure, Some(b"Reject with reason"))
            .expect("reject disclosure");
        assert!(!nodes[at].states.as_slice().contains(&skein_ui::State::Expanded));
        let children = super::inbox::subtree(view.tree(), at);
        assert!(super::chat::named_position(children, Role::TextArea, Some(b"Reason")).is_some());
        assert!(super::chat::named_position(children, Role::Button, Some(b"Reject")).is_some());
        assert!(super::chat::named_position(children, Role::Button, Some(b"Accept")).is_none());
    }
}
#[test]
fn a_rejecting_chat_keeps_its_reason_disclosure_open() {
    let view = fixtures::page_view(&fixtures::chat::rejecting_chat());
    let disclosure = fixtures::find(view.tree(), Role::Disclosure, b"Reject with reason").expect("reject disclosure");
    assert!(view.tree().find(disclosure).unwrap().states.as_slice().contains(&skein_ui::State::Expanded));
}

// One row for each confirmation in client/README.md, 3.6.

#[test]
fn escape_cancels_a_confirmation_without_sending_its_action() {
    let mut root = close_root(c::RunStage::Idle, 0, 0, false);
    let mut view = fixtures::browser_page_view(&root);
    let keep = fixtures::find(view.tree(), Role::Button, b"Keep it").expect("keep act");
    let mut shortcut = None;
    for node in view.tree().nodes() {
        if node.id == keep {
            shortcut = node.shortcut;
        }
    }
    assert_eq!(shortcut, Some(skein_ui::Shortcut::ESCAPE));
    let action = crate::act(&mut view, keep, skein_ui::Act::Press).expect("Escape's act");
    assert_eq!(action, Action::Pages(Act::Cancel));
    let mut out = skein_lib::Queue::uncapped();
    temper_client_domain::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: fixtures::domain_limits() },
        Event::Act { action },
        &mut out,
    );
    assert!(root.pages().confirmation().is_none());
    assert!(out.is_empty(), "cancelling a sheet sends no request");
    assert!(root.pages().chat().unwrap().task(root.client()).is_some());
}

#[test]
fn a_why_arriving_keeps_the_acts_elements() {
    let mut root = restart_sheet();
    let limits = fixtures::tree_limits();
    let mut old = sheet(&root);
    let acts = super::inbox::styled(&old, crate::look::CONFIRM_ACTS);
    assert!(old.nodes()[acts].key.is_some(), "the acts group keeps its own key");
    let mut next = 0;
    let mut patches = skein_lib::Queue::with_capacity(limits.patches);
    skein_ui::diff(&skein_ui::Tree::new(limits.nodes), &mut old, &limits, &mut next, &mut patches);
    while patches.pop().is_some() {}
    failed_sheet(&mut root);
    let mut after = sheet(&root);
    skein_ui::diff(&old, &mut after, &limits, &mut next, &mut patches);
    assert_kept_buttons(&old, &after, &mut patches);
    let mut view = fixtures::page_view(&root);
    let old_buttons = action_elements(view.tree());
    assert!(!has_style(view.tree(), crate::look::DOC));
    press(&mut root, Act::Mode { mode: temper_client_domain::pages::chat::document::Mode::Split });
    crate::build(&mut view, &root, fixtures::NOW, &mut patches);
    while let Some(patch) = patches.pop() {
        if let skein_ui::Patch::Remove { node } = patch {
            assert!(!old_buttons.as_slice().contains(&node), "button ancestor removed");
        }
    }
    assert!(has_style(view.tree(), crate::look::DOC));
    let after = action_elements(view.tree());
    for id in &old_buttons {
        assert!(after.as_slice().contains(id), "button or ancestor lost");
    }
}

fn action_elements(tree: &skein_ui::Tree) -> List<skein_ui::NodeId> {
    let acts = super::inbox::styled(tree, crate::look::CONFIRM_ACTS);
    let acts_end = acts.checked_add(usize::try_from(tree.nodes().get(acts).unwrap().size).unwrap()).unwrap();
    let mut elements = List::with_capacity(tree.len());
    for (index, node) in tree.nodes().iter().enumerate() {
        let end = index.checked_add(usize::try_from(node.size).unwrap()).unwrap();
        if index <= acts && end > acts || index > acts && index < acts_end {
            elements.push(node.id).expect("tree bounds button ancestors");
        }
    }
    elements
}
fn assert_kept_buttons(old: &skein_ui::Tree, new: &skein_ui::Tree, patches: &mut skein_lib::Queue<skein_ui::Patch>) {
    let kept = action_elements(old);
    while let Some(patch) = patches.pop() {
        if let skein_ui::Patch::Remove { node } = patch {
            assert!(!kept.as_slice().contains(&node), "button ancestor removed");
        }
    }
    let after = action_elements(new);
    for id in &kept {
        assert!(after.as_slice().contains(id), "button or ancestor lost");
    }
}

pub(crate) fn press(root: &mut temper_client_domain::Domain, act: Act) {
    fixtures::root_event(root, Event::Act { action: Action::Pages(act) });
}

pub(crate) fn open(root: &mut temper_client_domain::Domain, act: ActKind) {
    press(root, Act::Decide { object: ObjectKey::Proposal(PROPOSAL), revision: None, act });
}

pub(crate) fn contains(text: &[u8], expected: &[u8]) -> bool {
    for words in text.windows(expected.len()) {
        if words == expected {
            return true;
        }
    }
    false
}

pub(crate) fn sheet(root: &temper_client_domain::Domain) -> skein_ui::Tree {
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    crate::confirm::main(
        &mut b,
        root,
        &fixtures::words(),
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
        fixtures::domain_limits().client.domain.rows,
    );
    b.finish()
}

pub(crate) fn close_root(
    stage: c::RunStage,
    below: u32,
    proposals: u32,
    delegates: bool,
) -> temper_client_domain::Domain {
    let source = fixtures::chat::chat_root();
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    task.chip.details.phase = c::ChipPhase::Active(stage);
    let Some(c::Effect::Cancel(impact)) = &mut task.acts.get_mut(0).unwrap().effect else { panic!("cancel facts") };
    impact.tasks = c::ImpactCounts {
        waiting: below,
        running: 0,
        held: 0,
        closing: 0,
        ended: 0,
        done: 17,
        failed: 18,
        cancelled: 19,
    };
    impact.proposals = proposals;
    task.delegates = List::with_capacity(2);
    if delegates {
        for (number, title) in [(4, b"Fix the parser".as_slice()), (5, b"Review the parser")] {
            let mut chip = task.chip.clone();
            chip.task = number;
            chip.title = c::DocumentText::complete(c::Text::new(Box::from(title), 32).unwrap());
            task.delegates.push(c::LiveDelegate { chip, takeover_budget: 0 }).unwrap();
        }
    }
    let (mut root, me_watch, watch) =
        fixtures::root_at(temper_client_domain::Address::Chat { task: task.chip.task }, &fixtures::domain_limits());
    let me = source.pages().me(source.client()).unwrap().clone();
    for (watch, delivery) in
        [(me_watch, c::Delivery::Me(Box::new(me))), (watch.unwrap(), c::Delivery::Task(Box::new(task)))]
    {
        fixtures::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) },
                },
            },
        );
    }
    press(&mut root, Act::Close);
    root
}

pub(crate) fn restart_sheet() -> temper_client_domain::Domain {
    let mut root = fixtures::document::document_root(
        Some(b"# How we work\n"),
        temper_client_domain::pages::chat::document::Mode::Chat,
        &[],
        fixtures::document::ProposalState::NotProposed,
    );
    press(&mut root, Act::StartOver);
    root
}

pub(crate) fn failed_sheet(root: &mut temper_client_domain::Domain) {
    let mut out = skein_lib::Queue::uncapped();
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: fixtures::domain_limits() },
        Event::Act { action: Action::Pages(Act::Confirm) },
        &mut out,
    );
    let mut sent = None;
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            sent = Some(attempt);
        }
    }
    fixtures::root_event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: sent.expect("ask sent"),
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                        refusal: c::Refusal::NotOffered,
                        at: None,
                    })),
                    why: None,
                },
            },
        },
    );
}

pub(crate) fn has_style(tree: &skein_ui::Tree, style: skein_ui::Style) -> bool {
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&style) {
            return true;
        }
    }
    false
}
