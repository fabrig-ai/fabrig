//! A milestone borrows the goal page's look, not its agent's parts.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::arithmetic_side_effects,
    reason = "ordinary view tests build numbered fixtures and query semantic trees"
)]

use super::fixtures as f;
use crate::{
    bindings::Bindings,
    look,
    pages::task,
    words::{Shell, Times, Words},
};
use jig_client::{Down, Event as ClientEvent, Readable, Readout, UnlinkedRequest, Up, Watchable, contract as c};
use skein_lib::{Env, List, Queue, Time, Wall};
use skein_ui::{Builder, Role, Run, Style, Tree};
use temper_client_domain::{
    Address, Domain, Event, Request,
    pages::{Act, task::Shape},
};
use temper_client_protocol_wire::{self as wire, v1 as w};

const TASK: u64 = f::goal::TASK;
const NOW: Wall = f::goal::NOW;
const MINUTE: u64 = f::goal::MINUTE;

fn document() -> c::Task {
    let mut task = f::goal::document();
    task.chip.title = f::goal::text(b"Store shape");
    task.spec.specification.words = f::goal::text(b"Store shape\n\nThe store's new shape.");
    gathering(&mut task);
    task.acts = f::goal::offers(&[
        c::ActKind::Amend,
        c::ActKind::Message,
        c::ActKind::Stop,
        c::ActKind::Release,
        c::ActKind::Cancel,
    ]);
    task.budget.numbers.budget = 400;
    task.requester = c::Requester::Person(f::chat::PARTY);
    task
}
fn gathering(task: &mut c::Task) {
    task.chip.executor =
        c::ChipExecutor::Procedure { connector: jig_client::shapes::RECURRING, kind: jig_client::shapes::GATHERING };
    task.chip.details.details.executor =
        c::Executor::Procedure { connector: jig_client::shapes::RECURRING, code: jig_client::shapes::GATHERING };
    f::task::phase(task, c::ChipPhase::Active(c::RunStage::Idle));
}
fn line(number: u64, title: &[u8]) -> c::TaskLine {
    let mut row = f::goals::line();
    row.chip.task = number;
    row.chip.title = f::goal::text(title);
    row.changed = Wall::from_nanos(2);
    row.last_activity = Wall::from_nanos(number);
    row
}
fn ended(number: u64, title: &[u8], at: u64, ending: c::ChipEnding) -> c::TaskLine {
    let mut row = line(number, title);
    row.chip.phase = c::Phase::Ended;
    row.chip.details.phase = c::ChipPhase::Ended;
    row.chip.details.details.ending = Some(ending);
    row.ended_at = Some(Wall::from_nanos(at * MINUTE));
    row
}
fn done(number: u64, title: &[u8], at: u64) -> c::TaskLine {
    ended(number, title, at, c::ChipEnding::Done(c::ResultValue::Report(f::goal::text(b"Done."))))
}
fn goals() -> Vec<c::TaskLine> {
    (1..=8)
        .map(|n| {
            if n == 7 {
                line(n + 10, b"S7 history in task logs")
            } else {
                done(n + 10, format!("S{n} store step").as_bytes(), 400 + n)
            }
        })
        .collect()
}
fn page(rows: &[c::TaskLine], active: bool, more: bool) -> c::ReadValue {
    let rows = List::from_boxed(rows.into());
    c::ReadValue::Tasks(Box::new(c::TaskPage {
        ended_mark: 0,
        live: if active { rows.clone() } else { List::uncapped() },
        live_after: None,
        ended: if active { c::EndedGroup::None } else { c::EndedGroup::Rows { rows, after: None, cut: more } },
    }))
}
fn totals() -> Readout {
    let totals = w::MilestoneTotals {
            task: TASK,
            done: 7,
            failed: 0,
            live: 1,
            closed: 0,
            ended: None,
            last_landing: None,
            spent: 212,
        };
    let answer =
        w::MilestoneAnswer { result: w::MilestoneResult::Totals(totals) };
    Readout::Family { family: wire::FORGE, record: Some(wire::milestone_answer(&answer)) }
}
struct Harness {
    root: Domain,
    task: c::Task,
    rows: Vec<c::TaskLine>,
    cards: Vec<(c::ObjectKey, c::ReadValue)>,
    more: bool,
    totals: bool,
    out: Queue<Request>,
}
impl Harness {
    fn new(task: c::Task, mut rows: Vec<c::TaskLine>) -> Self {
        for row in &mut rows {
            row.parts = List::from_boxed(Box::new([f::documents::goal_part(
                Some((b"5 of its 7 changes landed; the other 2 are building.", NOW)),
                Some(TASK),
            )]));
        }
        let (root, _, watch, _, _, _, inbox) = f::root_at_with_dismissals_and_needs(
            Address::Task { task: TASK },
            &f::domain_limits(),
            b"Shop",
            None,
            None,
            None,
            Some(&[]),
        );
        let mut h = Self { root, task, rows, cards: Vec::new(), more: false, totals: true, out: Queue::uncapped() };
        h.up(Up::Watched {
            watch: watch.unwrap(),
            event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(h.task.clone()))),
        });
        h.up(Up::Watched {
            watch: inbox.unwrap(),
            event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(f::documents::inbox(
                List::uncapped(),
                0,
                c::Count { value: 0, limited: false },
                0,
                false,
            )))),
        });
        h
    }
    fn up(&mut self, up: Up) {
        temper_client_domain::step(
            &mut self.root,
            &Env { now: Time::ZERO, wall: NOW, limits: f::domain_limits() },
            Event::Client { event: ClientEvent::Up { up } },
            &mut self.out,
        );
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one fixture answers the milestone's documents, goals and inbox together"
    )]
    fn settle(&mut self) {
        while let Some(request) = self.out.pop() {
            let Request::Unlinked { request: UnlinkedRequest::Down { down } } = request else { continue };
            let answer = match down {
                Down::Read { read, what: Readable::Jig(what) } => {
                    let value = match what {
                        c::Read::Task { task } if task == TASK => Some(c::ReadValue::Task(Box::new(self.task.clone()))),
                        c::Read::Tasks { query, .. } if query.parent == Some(TASK) => {
                            let ended: Vec<_> = self
                                .rows
                                .iter()
                                .filter(|r| r.chip.details.phase == c::ChipPhase::Ended)
                                .cloned()
                                .collect();
                            Some(page(&ended, false, self.more))
                        }
                        c::Read::Tasks { query, .. } if query.tracked == Some(true) => {
                            let mut row = line(TASK, b"Store shape");
                            row.chip = self.task.chip.clone();
                            row.budget = self.task.budget.clone();
                            row.changed = self.task.changed;
                            row.last_activity = self.task.last_activity;
                            Some(page(&[row], true, false))
                        }
                        c::Read::Notes { scope: c::NoteScope::Task { .. }, .. } => panic!("no per-goal report read"),
                        c::Read::Inbox { group, goal: Some(TASK), .. } => {
                            let mut rows = List::uncapped();
                            let mut selection = List::uncapped();
                            if group == c::InboxReadGroup::Needs {
                                for (object, _) in &self.cards {
                                    let entry = f::documents::entry(*object, f::PROJECT, 1, NOW, None);
                                    selection
                                        .push(c::InboxSelection {
                                            cursor: entry.cursor,
                                            goals: List::from_boxed(Box::new([17, TASK])),
                                        })
                                        .unwrap();
                                    rows.push(entry).unwrap();
                                }
                            }
                            Some(c::ReadValue::Inbox(Box::new(c::InboxPage {
                                rows,
                                selection: Some(selection),
                                before: None,
                                after: None,
                                cut: false,
                            })))
                        }
                        c::Read::Question { task, message } => self
                            .cards
                            .iter()
                            .find(|(object, _)| *object == (c::ObjectKey::Question { task, message }))
                            .map(|(_, value)| value.clone()),
                        c::Read::Settings
                        | c::Read::Setting(_)
                        | c::Read::Revision(_)
                        | c::Read::Revisions(_)
                        | c::Read::Tasks { .. }
                        | c::Read::Me
                        | c::Read::Turn { .. }
                        | c::Read::TaskText { .. }
                        | c::Read::Task { .. }
                        | c::Read::Proposal { .. }
                        | c::Read::Escalation { .. }
                        | c::Read::PersonTask { .. }
                        | c::Read::Result { .. }
                        | c::Read::Ended { .. }
                        | c::Read::History { .. }
                        | c::Read::Activity { .. }
                        | c::Read::ActivityBeside { .. }
                        | c::Read::Inbox { .. }
                        | c::Read::Spend { .. }
                        | c::Read::Project { .. }
                        | c::Read::Notes { .. }
                        | c::Read::Deployment => None,
                    };
                    value.map(|value| Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(value)) })
                }
                Down::Read { read, what: Readable::Family { record, .. } } => {
                    if let Some(reading) = wire::decode_reading(&record.bytes, &w::CEILINGS)
                        && let w::Read::Milestone(_) = reading.read()
                    {
                        Some(Up::ReadDone {
                            read,
                            result: if self.totals {
                                totals()
                            } else {
                                Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown))
                            },
                        })
                    } else {
                        None
                    }
                }
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { query, .. }) }
                    if query.parent == Some(TASK) =>
                {
                    let rows: Vec<_> =
                        self.rows.iter().filter(|r| r.chip.details.phase != c::ChipPhase::Ended).cloned().collect();
                    Some(Up::Watched {
                        watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                            ended_mark: 0,
                            rows: List::from_boxed(rows.into_boxed_slice()),
                            more: false,
                        }))),
                    })
                }
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Inbox { .. }) } => Some(Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(f::documents::inbox(
                        List::uncapped(),
                        0,
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    )))),
                }),
                Down::Save { .. }
                | Down::Erase { .. }
                | Down::Send { .. }
                | Down::Open { .. }
                | Down::Continue { .. }
                | Down::Close { .. } => None,
            };
            if let Some(up) = answer {
                self.up(up);
            }
        }
    }
}
fn root() -> Domain {
    let mut h = Harness::new(document(), goals());
    h.settle();
    assert_eq!(h.root.pages().task().unwrap().shape, Shape::Milestone);
    h.root
}
fn draw(root: &Domain, words: &Words) -> (Tree, Bindings) {
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = Bindings::new(f::tree_limits().bindings);
    task::main(&mut b, root, words, &mut bindings, 8, &mut Times::new(NOW), &crate::forge_sign_in::Forges::new());
    (b.finish(), bindings)
}
fn styled(tree: &Tree, style: Style) -> usize {
    tree.nodes()
        .iter()
        .position(|n| n.styles.as_slice().contains(&style))
        .unwrap_or_else(|| panic!("style {style:?}: {}", String::from_utf8_lossy(&f::text(tree))))
}
fn styled_text(tree: &Tree, style: Style) -> Box<[u8]> {
    f::node_text(tree, styled(tree, style))
}
fn contains(tree: &Tree, text: &[u8]) -> bool {
    skein_lib::bytes::find(&f::text(tree), text).is_some()
}

#[test]
fn the_meta_says_title_state_count_spend_and_since() {
    let root = root();
    for words in [f::browser_words(), f::words()] {
        let (tree, _) = draw(&root, &words);
        let expected = if words.shell == Shell::Browser {
            "Store shape·running·7 of 8 goals done·212 spent of 400·since 05:40"
        } else {
            "Store shape · running · 7 of 8 goals done · 212 spent of 400 · since 05:40"
        };
        assert_eq!(styled_text(&tree, look::GOAL_META).as_ref(), expected.as_bytes());
        let heading = tree.nodes().iter().position(|n| n.role == (Role::Heading { level: 1 })).unwrap();
        assert_eq!(
            f::node_text(&tree, heading).as_ref(),
            b"7 of 8 goals done. S7: 5 of its 7 changes landed; the other 2 are building."
        );
        assert_eq!(tree.nodes()[0].styles.as_slice(), [look::MAIN, look::GOAL]);
        if words.shell == Shell::Browser {
            let meta = styled(&tree, look::GOAL_META);
            assert_eq!(tree.nodes()[meta + 1].text.as_slice()[0].run, Run::Strong);
            assert_eq!(styled_text(&tree, look::GOALS_STATE_RUNNING).as_ref(), b"running");
            assert_eq!(tree.nodes()[styled(&tree, look::MS_STRIP)].name.as_deref(), Some(b"7 done of 8".as_slice()));
        }
        assert!(!contains(&tree, b"Task details"));
        assert!(!contains(&tree, b"Written by"));
        assert!(!contains(&tree, b"A report body"));
        assert!(!tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::GOAL_COMPOSER)));
    }
    let mut h = Harness::new(document(), goals());
    let tree = draw(&h.root, &f::words()).0;
    assert!(!contains(&tree, b"goals done"));
    assert!(!tree.nodes().iter().any(|n| n.role == (Role::Heading { level: 1 })));
    assert!(
        !draw(&h.root, &f::browser_words())
            .0
            .nodes()
            .iter()
            .any(|n| n.styles.as_slice().contains(&look::GOALS_STATE_PLANNING))
    );
    h.totals = false;
    h.settle();
    assert!(!contains(&draw(&h.root, &f::words()).0, b"spent of"));
    let mut task = document();
    task.budget.numbers.budget = 0;
    let mut h = Harness::new(task, goals());
    h.settle();
    assert!(!contains(&draw(&h.root, &f::words()).0, b"spent of"));
}

#[test]
fn the_acts_in_order_by_offer() {
    for paused in [false, true] {
        let mut task = if paused {
            let mut t = f::goal::paused_document();
            gathering(&mut t);
            f::task::phase(&mut t, c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 1 });
            t
        } else {
            document()
        };
        task.acts = f::goal::offers(&[
            c::ActKind::Amend,
            c::ActKind::Message,
            c::ActKind::Stop,
            c::ActKind::Release,
            c::ActKind::Cancel,
        ]);
        let mut h = Harness::new(task, goals());
        h.settle();
        let (tree, _) = draw(&h.root, &f::browser_words());
        let buttons: Vec<_> =
            tree.nodes().iter().filter(|n| n.role == Role::Button).map(|n| n.name.as_deref().unwrap()).collect();
        assert_eq!(buttons, [if paused { b"Resume".as_slice() } else { b"Pause" }, b"Close"]);
        let sent = f::goal::press(&mut h.root, if paused { Act::Resume } else { Act::Pause });
        let _attempt = f::goal::sent_attempt(sent);
        f::goal::press(&mut h.root, Act::Close);
        let (tree, _) = draw(&h.root, &f::browser_words());
        assert!(contains(&tree, b"Close this task?"));
        assert!(contains(&tree, "You can still read it, but it can’t be reopened.".as_bytes()));
    }
    for acts in [
        List::uncapped(),
        List::from_boxed(Box::new([
            c::Offered {
                act: c::ActKind::Stop,
                admission: c::Admission::Refused(c::Refusal::NotOffered),
                effect: None,
            },
            c::Offered {
                act: c::ActKind::Cancel,
                admission: c::Admission::Proposed { to: c::Recipient::Party(73) },
                effect: None,
            },
        ])),
    ] {
        let mut task = document();
        task.acts = acts;
        let mut h = Harness::new(task, goals());
        h.settle();
        assert!(!draw(&h.root, &f::browser_words()).0.nodes().iter().any(|n| n.role == Role::Button));
    }
    let mut task = document();
    f::task::phase(&mut task, c::ChipPhase::Ended);
    let mut h = Harness::new(task, goals());
    h.settle();
    assert!(!draw(&h.root, &f::words()).0.nodes().iter().any(|n| n.role == Role::Button));
}

#[test]
fn ended_goals_fold_newest_first_and_live_goals_keep_their_order() {
    let mut rows = goals();
    rows.push(ended(20, b"S9 failed", 500, c::ChipEnding::Failed(f::goal::text(b"No."))));
    rows.push(ended(
        21,
        b"S10 closed",
        510,
        c::ChipEnding::Cancelled {
            reason: f::goal::text(b"Closed."),
            result: None,
            by: c::Requester::Person(73),
            at: NOW,
        },
    ));
    let mut waits = line(22, b"S11 waiting");
    waits.chip.phase = c::Phase::Waiting;
    waits.chip.details.phase =
        c::ChipPhase::Waiting { dependencies: List::from_boxed(Box::new([16])), more: false, holds: false };
    rows.push(waits);
    let mut h = Harness::new(document(), rows);
    h.more = true;
    h.settle();
    for words in [f::browser_words(), f::words()] {
        let (tree, _) = draw(&h.root, &words);
        let folds: Vec<_> =
            tree.nodes().iter().filter(|n| n.role == Role::Disclosure).map(|n| n.name.as_deref().unwrap()).collect();
        assert_eq!(folds, [b"7 done".as_slice(), b"1 failed", b"1 closed"]);
        let titles: Vec<_> = tree
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, n)| n.styles.as_slice().contains(&look::MS_GOAL))
            .map(|(i, _)| f::node_text(&tree, i))
            .collect();
        assert!(titles[0].starts_with(b"S7 history"));
        assert!(titles[1].starts_with(b"S11 waiting"));
        assert!(titles[1].windows(b"waits on S6".len()).any(|w| w == b"waits on S6"));
        assert!(titles[2].starts_with(b"S8 store step"));
        assert!(titles[8].starts_with(b"S1 store step"));
        assert!(contains(&tree, b"landed 06:48"));
        assert!(contains(&tree, b"More of its goals are not shown."));
        if words.shell == Shell::Terminal {
            let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
            skein_ui_text::render(&tree, b"temper", None, &mut print);
            let print = String::from_utf8_lossy(print.text.as_slice());
            let mut previous = 0;
            for part in [
                "Store shape",
                "Goals",
                "S7 history",
                "7 done",
                "S8 store step",
                "1 failed",
                "S9 failed",
                "1 closed",
                "S10 closed",
                "What it is for",
                "Landing",
                "Its goals",
                "■■■■■■▒■×□ 7 done of 8",
            ] {
                let at = print.find(part).unwrap_or_else(|| panic!("{part}: {print}"));
                assert!(at >= previous, "{part}: {print}");
                previous = at;
            }
        }
    }
}

#[test]
fn the_side_says_what_it_is_for_and_who_made_it() {
    for (maker, expected) in [
        (c::Requester::Person(f::chat::PARTY), b"Made by you.".as_slice()),
        (c::Requester::Person(91), b"Made by person 91."),
        (c::Requester::Task(44), b"Made by task 44."),
        (c::Requester::Deployment { project: f::PROJECT }, b"Made by the deployment."),
    ] {
        let mut task = document();
        task.requester = maker;
        let mut h = Harness::new(task, Vec::new());
        h.totals = false;
        h.settle();
        let (tree, _) = draw(&h.root, &f::words());
        assert!(!tree.nodes().iter().any(|n| n.role == Role::Disclosure));
        for text in [
            b"What it is for".as_slice(),
            b"The store's new shape.",
            b"Each goal lands on its own, once done.",
            b"Open: more goals may join.",
            b"No goals yet.",
            expected,
        ] {
            assert!(contains(&tree, text), "{text:?}");
        }
        let mut task = h.task.clone();
        task.concluded = Some(c::Concluded { by: f::chat::PARTY, at: NOW });
        task.spec.specification.words = f::goal::text(b"Store shape");
        let mut h = Harness::new(task, Vec::new());
        h.totals = false;
        h.settle();
        let (tree, _) = draw(&h.root, &f::words());
        assert!(contains(&tree, b"Whole: all its goals are in."));
        assert!(contains(&tree, b"No goals."));
        assert!(!contains(&tree, b"What it is for"));
    }
}

#[test]
fn needs_bands_name_the_goal_and_only_the_first_question_is_whole() {
    let mut h = Harness::new(document(), goals());
    for message in [1, 2] {
        h.cards.push((
            c::ObjectKey::Question { task: 50, message },
            f::goal::question(50, message, b"Which way?\n\nThe context of the question."),
        ));
    }
    h.settle();
    for words in [f::browser_words(), f::words()] {
        let (tree, _) = draw(&h.root, &words);
        let bands: Vec<_> = tree
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, n)| n.styles.as_slice().contains(&look::GOAL_BAND))
            .map(|(i, _)| f::node_text(&tree, i))
            .collect();
        assert_eq!(bands.len(), 2);
        assert!(
            bands[0]
                .windows(b"S7 history in task logs: Which way?".len())
                .any(|w| w == b"S7 history in task logs: Which way?")
        );
        assert!(bands[0].windows(b"The context".len()).any(|w| w == b"The context"));
        assert!(!bands[1].windows(b"The context".len()).any(|w| w == b"The context"));
        assert!(styled(&tree, look::GOAL_HEAD) < styled(&tree, look::GOAL_BAND));
        assert!(styled(&tree, look::GOAL_BAND) < styled(&tree, look::GOAL_REPORT));
        if words.shell == Shell::Browser {
            assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::MS_NEEDS)));
        }
    }
}
