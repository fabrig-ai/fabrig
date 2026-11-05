use super::{chats, fixtures};
use crate::{
    frame::Bindings,
    look,
    pages::goals,
    words::{Times, Words},
};
use jig_client::contract as c;
use skein_lib::{List, Queue, Wall};
use skein_ui::{Builder, Role, Style, Tree};
use temper_client_domain::{
    Address, PlanFilter,
    pages::{goals::State, report::Report},
};

// The mock's row has six landed tasks out of nine (client/README.md, 3.5).
const DONE: u32 = 6;
const TOTAL: u32 = 9;
const EMPTY: u32 = 0;
const ONE: u32 = 1;
// One link, top, title, metadata, state and headline in the browser contract.
const ROW_NODES: usize = 6;
const TOP_NODES: u32 = 4;
const META_NODES: u32 = 2;
// Seven states, with the two needs words checked separately.
const STATE_CASES: usize = 8;
// The template fixture's bounds and identities, independent of its name.
const TEXT_BYTES: u32 = 64;
const CHARTER: u32 = 1;
const LABEL: u32 = 1;
const DEPTH: u32 = 1;
const TASK_WAY: u32 = 2;
const WHOLE_WAY: u32 = 4;
const UNTRACKED_WAY: u32 = 7;
const OTHER_WAY: u32 = 9;
// Nanoseconds and epoch days for Thursday 22 October 2026 and its Monday.
const MINUTE: u64 = 60_000_000_000;
const HOUR: u64 = 3_600_000_000_000;
const DAY: u64 = 86_400_000_000_000;
const THURSDAY: u64 = 20_748 * DAY;
const MONDAY: u64 = 20_745 * DAY;
const FRIDAY: u64 = 20_749 * DAY;
const NOON: u64 = THURSDAY + 12 * HOUR;
const TODAY_END: u64 = THURSDAY + HOUR + 12 * MINUTE;
const MONDAY_END: u64 = MONDAY + 18 * HOUR + 40 * MINUTE;

fn line() -> c::TaskLine {
    let mut line = chats::line();
    line.delegates_done = DONE;
    line.delegates_total = TOTAL;
    line
}
fn tree(line: &c::TaskLine, state: State, report: Option<&Report>, words: &Words) -> (Tree, Bindings) {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = Bindings::new(fixtures::tree_limits().bindings);
    goals::row(&mut b, words, &mut bindings, &mut Times::new(fixtures::NOW), line, state, report);
    (b.finish(), bindings)
}
fn states() -> [(State, &'static [u8], Style, bool); STATE_CASES] {
    [
        (State::Needs { funds: true }, b"needs funds", look::GOALS_STATE_NEEDS, false),
        (State::Needs { funds: false }, b"needs you", look::GOALS_STATE_NEEDS, false),
        (State::Paused, b"paused by you", look::GOALS_STATE_PAUSED, false),
        (State::Planning, b"planning", look::GOALS_STATE_PLANNING, false),
        (State::Running, b"running", look::GOALS_STATE_RUNNING, false),
        (State::Landed, b"landed", look::GOALS_STATE_LANDED, true),
        (State::Stopped, b"stopped", look::GOALS_STATE_STOPPED, true),
        (State::Failed, b"failed", look::GOALS_STATE_STOPPED, true),
    ]
}
fn styled_text(tree: &Tree, style: Style) -> Box<[u8]> {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&style) {
            return fixtures::node_text(tree, index);
        }
    }
    panic!("styled node")
}
#[test]
fn a_row_is_one_link_to_its_task() {
    let (tree, bindings) = tree(&line(), State::Running, None, &fixtures::browser_words());
    let nodes = tree.nodes();
    let mut links = EMPTY;
    for node in nodes {
        if node.role == Role::Link {
            links = links.checked_add(ONE).unwrap();
        }
    }
    assert_eq!(links, ONE);
    let link = &nodes[0];
    assert_eq!(link.role, Role::Link);
    assert_eq!(usize::try_from(link.size).unwrap(), nodes.len());
    assert_eq!(link.styles.as_slice(), [look::GOALS_ROW]);
    let binding = link.binding.expect("row binding");
    assert_eq!(link.link, Some(binding));
    assert_eq!(
        bindings.decode(binding.token, skein_ui::Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: Address::Task { task: line().chip.task, filter: PlanFilter::All }
        }))
    );
    // Text roles map to spans in the browser: top(title, count(state)), then line.
    assert_eq!(nodes.len(), ROW_NODES);
    assert_eq!(nodes[1].role, Role::Text);
    assert_eq!(nodes[1].styles.as_slice(), [look::GOALS_TOP]);
    assert_eq!(nodes[1].size, TOP_NODES);
    assert_eq!(nodes[2].role, Role::Text);
    assert_eq!(fixtures::node_text(&tree, 2).as_ref(), b"Hello");
    assert_eq!(nodes[3].role, Role::Text);
    assert_eq!(nodes[3].size, META_NODES);
    assert_eq!(nodes[5].role, Role::Text);
    assert_eq!(nodes[5].styles.as_slice(), [look::GOALS_LINE]);
}
#[test]
fn each_state_says_its_word_and_carries_its_token() {
    for (state, word, token, _) in states() {
        let (tree, _) = tree(&line(), state, None, &fixtures::browser_words());
        assert_eq!(styled_text(&tree, token).as_ref(), word, "{state:?}");
    }
    // 22 October 2026, and the Monday in the same week.
    let now = Wall::from_nanos(NOON);
    for (at, expected) in [(TODAY_END, b"landed 01:12".as_slice()), (MONDAY_END, b"landed Mon 18:40".as_slice())] {
        let mut line = line();
        line.ended_at = Some(Wall::from_nanos(at));
        let mut b = Builder::new(&fixtures::tree_limits());
        let mut times = Times::new(now);
        goals::row(
            &mut b,
            &fixtures::browser_words(),
            &mut Bindings::new(fixtures::tree_limits().bindings),
            &mut times,
            &line,
            State::Landed,
            None,
        );
        assert_eq!(styled_text(&b.finish(), look::GOALS_STATE_LANDED).as_ref(), expected);
        assert_eq!(times.next, Some(Wall::from_nanos(FRIDAY)));
    }
}
#[test]
fn its_count_is_left_out_with_no_delegates() {
    for (total, expected) in [(EMPTY, b"Hellorunning".as_slice()), (TOTAL, "Hello6 / 9 · running".as_bytes())] {
        let mut line = line();
        line.delegates_total = total;
        let (tree, _) = tree(&line, State::Running, None, &fixtures::browser_words());
        assert_eq!(styled_text(&tree, look::GOALS_TOP).as_ref(), expected);
    }
}
#[test]
fn its_headline_is_the_reports_or_its_states() {
    let report = Report {
        headline: Box::from(*b"The store is split in two."),
        paragraphs: List::uncapped(),
        left: None,
        risk: None,
        decided: List::uncapped(),
        at: None,
    };
    for (state, fallback) in [
        (State::Planning, b"Planning from the draft.".as_slice()),
        (State::Running, b"6 of 9 tasks landed.".as_slice()),
        (State::Paused, b"6 of 9 tasks landed.".as_slice()),
        (State::Needs { funds: true }, b"6 of 9 tasks landed.".as_slice()),
        (State::Needs { funds: false }, b"6 of 9 tasks landed.".as_slice()),
        (State::Landed, b"Landed.".as_slice()),
        (State::Stopped, b"Stopped by you.".as_slice()),
        (State::Failed, b"Failed.".as_slice()),
    ] {
        for (given, expected) in [(None, fallback), (Some(&report), report.headline.as_ref())] {
            let (tree, _) = tree(&line(), state, given, &fixtures::browser_words());
            assert_eq!(styled_text(&tree, look::GOALS_LINE).as_ref(), expected, "{state:?}");
        }
    }
    for state in [State::Needs { funds: true }, State::Needs { funds: false }] {
        let mut line = line();
        line.delegates_total = EMPTY;
        let (tree, _) = tree(&line, state, None, &fixtures::browser_words());
        assert_eq!(styled_text(&tree, look::GOALS_LINE).as_ref(), b"Planning from the draft.");
    }
}
#[test]
fn an_ended_row_is_marked() {
    for (state, _, _, ended) in states() {
        let (tree, _) = tree(&line(), state, None, &fixtures::browser_words());
        assert_eq!(tree.nodes()[0].styles.as_slice().contains(&look::GOALS_ROW_ENDED), ended, "{state:?}");
    }
}
#[test]
fn a_row_reads_in_the_terminal() {
    for (total, name, headline) in [
        (EMPTY, "Hello · needs funds", b"Planning from the draft.".as_slice()),
        (TOTAL, "Hello · 6 / 9 · needs funds", b"6 of 9 tasks landed.".as_slice()),
    ] {
        let mut line = line();
        line.delegates_total = total;
        let (mut tree, bindings) = tree(&line, State::Needs { funds: true }, None, &fixtures::words());
        let limits = fixtures::tree_limits();
        skein_ui::diff(&Tree::new(limits.nodes), &mut tree, &limits, &mut 0, &mut Queue::with_capacity(limits.patches));
        let name = name.as_bytes();
        let control = fixtures::find(&tree, Role::Link, name).expect("one goal link");
        let (node, act) = fixtures::terminal_command(&tree, &limits, control, name, headline);
        let mut bound = EMPTY;
        let mut binding = None;
        for item in tree.nodes() {
            if item.binding.is_some() {
                bound = bound.checked_add(ONE).unwrap();
            }
            if item.id == node {
                binding = item.binding;
            }
        }
        assert_eq!(bound, ONE);
        let binding = binding.expect("goal link binding");
        assert_eq!(
            bindings.decode(binding.token, act),
            Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                address: Address::Task { task: line.chip.task, filter: PlanFilter::All }
            }))
        );
    }
}
fn template(number: u32, name: &[u8], tracked: bool) -> c::Template {
    c::Template {
        resources: List::uncapped(),
        number,
        name: c::DocumentText::complete(c::Text::new(name.into(), TEXT_BYTES).unwrap()),
        executor: c::TemplateExecutor::Charter(CHARTER),
        contract: c::TaskContractDocument::Report { words: TEXT_BYTES },
        shallower: DEPTH,
        budget: c::TemplateBudget::AsAllowed { cap: None },
        deadline: c::TemplateDeadline::None,
        tracked: if tracked { Some(LABEL) } else { None },
        flags: line().flags,
        label: LABEL,
        grace: None,
        llm: None,
    }
}
#[test]
fn ways_name_their_templates_in_number_order() {
    let root = fixtures::root_in_project(Some(fixtures::PROJECT));
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let project = me.projects.get_mut(EMPTY).unwrap();
    project.templates = List::uncapped();
    for template in [
        template(OTHER_WAY, b"other way", true),
        template(WHOLE_WAY, b"goal for a draft", true),
        template(UNTRACKED_WAY, b"untracked", false),
        template(TASK_WAY, b"goal", true),
    ] {
        project.templates.push(template).unwrap();
    }
    for words in [fixtures::words(), fixtures::browser_words()] {
        let ways = goals::ways(&me, fixtures::PROJECT, &words);
        let mut actual = List::uncapped();
        for way in &ways {
            actual.push((way.name.as_ref(), way.line.as_deref())).unwrap();
        }
        assert_eq!(actual.as_slice(), [
            (b"Task by task".as_slice(), Some(b"The same plan; each task lands on its own once checked.".as_slice())),
            (b"Plan, tasks, land whole".as_slice(), Some(b"A plan from the draft, its tasks on a goal branch, landed as one change once all are checked.".as_slice())),
            (b"other way".as_slice(), None),
        ]);
        assert!(goals::ways(&me, u32::MAX, &words).is_empty());
    }
}
