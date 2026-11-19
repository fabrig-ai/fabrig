//! Milestone tools through the root's fake documents and requests (client/ctl/milestones.md).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary fixtures and UTF-8 output assertions in step tests"
)]
use crate::{
    jig::{
        call::{Value, Values},
        rows,
    },
    tests::fixtures as f,
};
use jig_client::{contract as c, shapes};
use skein_lib::{List, Wall};
use temper_client_protocol_wire::{self as wire, v1 as w};

const NEW: &[&str] = &["milestone", "new", "--project", "1", "--budget", "4.00", "Store shape", "--key", "store-shape"];
const WHOLE: &[&str] = &["milestone", "whole", "812", "--key", "whole"];
pub(super) const KEYED: &[&[&str]] = &[NEW, WHOLE];
fn me() -> c::Me {
    let mut me = super::goal_template::goal_me();
    let project = me.projects.get_mut(0).unwrap();
    let mut template = project.templates.get(0).unwrap().clone();
    template.number = 8;
    template.name = f::document_text(b"milestone");
    template.deadline = c::TemplateDeadline::None;
    template.tracked = Some(1);
    template.flags.ended_by = c::EndedBy::Requester;
    template.shallower = 1;
    template.label = 4;
    project.templates = List::from_boxed(Box::new([template]));
    let creating = project.creating.as_mut().unwrap();
    creating.within.budget = 400;
    creating.within.procedures =
        List::from_boxed(Box::new([c::CreateProcedure { connector: shapes::RECURRING, code: shapes::GATHERING }]));
    me
}
fn milestone() -> c::Task {
    let mut task = f::goal(812, c::WaitsOn::Delegates);
    task.chip.executor = c::ChipExecutor::Procedure { connector: shapes::RECURRING, kind: shapes::GATHERING };
    task.chip.details.details.executor =
        c::Executor::Procedure { connector: shapes::RECURRING, code: shapes::GATHERING };
    task.chip.title = f::document_text(b"Store shape");
    task.spec.specification.words = f::document_text(b"Store shape\n\nThe store's new shape");
    task.requester = c::Requester::Person(1);
    task.concluded = Some(c::Concluded { by: 1, at: Wall::from_nanos(60_000_000_000) });
    task.budget.numbers.budget = 400;
    task
}
fn created(root: &mut f::Harness, call: u32) {
    root.done(call, c::Outcome::Document(c::DocumentOutcome::Created { first: 812, last: 812, given: None }));
}
#[test]
fn milestone_new_fills_a_gathering_with_name_words_and_controls() {
    for (words, expected) in
        [(None, b"Store shape".as_slice()), (Some("The store's new shape"), b"Store shape\n\nThe store's new shape")]
    {
        for stdin in [false, true] {
            let mut root = f::choice_root(me());
            let mut argv = f::extended(NEW, &["--priority", "7", "--deadline", "1970-01-01T00:01Z"]);
            if let Some(words) = words {
                argv.push("--words").unwrap();
                argv.push(if stdin { "-" } else { words }).unwrap();
            }
            let mut call = f::call(argv.as_slice());
            if stdin && let Some(words) = words {
                assert_eq!(call.values.get(2), Some(&Some(Values::One(Value::Stdin))));
                *call.values.get_mut(2).unwrap() = Some(Values::One(Value::Text(Box::from(words.as_bytes()))));
            }
            let number = root.submit_prepared(call.clone(), false, None);
            let c::Ask::Create { project, tasks } = f::act_ask(&root, number) else { panic!("create") };
            assert_eq!((*project, tasks.len()), (1, 1));
            let task = tasks.get(0).unwrap();
            assert_eq!(task.executor, c::Executor::Procedure { connector: u16::MAX, code: 2 });
            assert_eq!(task.spec.words.bytes(), expected);
            assert_eq!(
                (task.authority.spend, task.authority.deadline, task.tracked, task.label),
                (400, Some(Wall::from_nanos(60_000_000_000)), Some(7), 4)
            );
            assert_eq!(task.flags.ended_by, c::EndedBy::Requester);
            let ask = f::act_ask(&root, number).clone();
            created(&mut root, number);
            assert_eq!(root.answer(number).unwrap().stdout.as_ref(), b"done: task 812, milestone\n");
            let replay = root.submit_prepared(call, false, None);
            assert_eq!(f::act_ask(&root, replay), &ask);
            assert_eq!(root.sent_name(replay), Some(f::named("store-shape")));
            created(&mut root, replay);
            assert_eq!(root.answer(replay).unwrap().stdout.as_ref(), b"done: task 812, milestone\n");
        }
    }
    assert_eq!(rows::MILESTONE_NEW.result, b"milestone_create");
}
#[test]
fn milestone_names_are_checked_before_any_input_or_request() {
    for name in [String::new(), "a".repeat(61), "two\nlines".to_owned(), "a\tb".to_owned(), "a\u{7f}b".to_owned()] {
        let argv = ["milestone", "new", "--project", "1", "--words", "-", name.as_str(), "--key", "bad-name"];
        let why = f::parse(&argv).expect_err("name refused before reading stdin");
        let mut out = skein_lib::Writer::new(b"milestone new: NAME is 1 to 60 characters on one line".len());
        let bytes: Vec<&[u8]> = argv.iter().map(|word| word.as_bytes()).collect();
        crate::table::refusal(why, &bytes, &mut out).unwrap();
        assert_eq!(out.finish().as_ref(), b"milestone new: NAME is 1 to 60 characters on one line");
    }
    for name in ["é".repeat(60), "a".to_owned()] {
        f::parse(&["milestone", "new", "--project", "1", &name, "--key", "valid"]).unwrap();
    }
}
#[test]
fn milestone_controls_keep_required_budget_and_jigs_proposal_judgement() {
    let mut root = f::choice_root(me());
    let call = root.submit(&["milestone", "new", "--project", "1", "Store shape", "--key", "missing"]);
    assert!(root.sends.is_empty());
    assert_eq!(
        root.answer(call).unwrap().stderr.as_ref(),
        b"template 8 needs: --budget AMOUNT or --as-allowed; help milestone new\n"
    );
    for json in [false, true] {
        let mut offered = me();
        offered.projects.get_mut(0).unwrap().creating.as_mut().unwrap().within.budget = 399;
        let mut root = f::choice_root(offered);
        let call = root.submit_prepared(f::call(NEW), json, None);
        let c::Ask::Propose { project, action: c::ProposeAction::Batch { tasks }, .. } = f::act_ask(&root, call) else {
            panic!("proposal")
        };
        assert_eq!((*project, tasks.get(0).unwrap().authority.spend), (1, 400));
        root.done(call, c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 17, to: 1 }));
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        if !json {
            assert!(answer.stdout.starts_with(b"proposed: "));
        }
    }
    let mut root = f::choice_root(me());
    let call = root.submit(&["milestone", "new", "--project", "1", "--as-allowed", "Store shape", "--key", "allowed"]);
    assert_eq!(f::chat_task(&root, call).budget, c::NewBudget::AsAllowed { cap: None });
    let mut root = f::choice_root(me());
    let call =
        root.submit(&["milestone", "new", "--project", "1", "--budget", "5.01", "Store shape", "--key", "ceiling"]);
    assert!(root.sends.is_empty());
    assert_eq!(root.answer(call).unwrap().status, 1);
    f::parse(f::extended(NEW, &["--as-allowed"]).as_slice()).unwrap_err();
    f::parse(f::extended(NEW, &["--deadline", "2026-02-30T00:00Z"]).as_slice()).unwrap_err();
}
#[test]
fn milestone_whole_sends_conclude_under_the_same_key_on_replay() {
    let mut root = f::choice_root(me());
    for json in [false, true] {
        let call = root.submit_prepared(f::call(WHOLE), json, None);
        assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Task { task: 812 });
        root.serve(call, f::task_result(milestone()));
        assert_eq!(f::act_ask(&root, call), &c::Ask::Conclude { project: 1, task: 812 });
        assert_eq!(root.sent_name(call), Some(f::named("whole")));
        root.done(call, c::Outcome::Document(c::DocumentOutcome::Concluded { task: 812 }));
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        if !json {
            assert_eq!(answer.stdout.as_ref(), b"done: task 812, whole\n");
        }
    }
    assert_eq!(rows::MILESTONE_WHOLE.result, b"milestone_whole");
}
#[test]
fn milestone_whole_prints_jigs_refusals_as_cancel_does() {
    for ended in [false, true] {
        let mut root = f::choice_root(me());
        let call = root.submit(WHOLE);
        let mut task = if ended { milestone() } else { f::goal(812, c::WaitsOn::Nothing) };
        if ended {
            task.chip.phase = c::Phase::Cancelled;
        }
        root.serve(call, if ended { f::ended_task_result(task, c::Ending::Done) } else { f::task_result(task) });
        root.answered(
            call,
            c::Answer::Refused {
                refusal: if ended {
                    c::Refusal::Unknown
                } else {
                    c::Refusal::Malformed { at: 0, why: c::Malformed::EndedBy }
                },
                at: None,
            },
        );
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 2);
        if ended {
            assert_eq!(answer.stderr.as_ref(), b"task 812 has ended: cancelled\n");
        } else {
            assert!(answer.stderr.starts_with(b"milestone whole: "));
        }
    }
}
fn row(number: u64, name: &[u8], phase: c::Phase) -> c::TaskLine {
    let mut row = f::task_line();
    row.chip.task = number;
    row.chip.title = f::document_text(name);
    row.chip.phase = phase;
    row.delegates_done = 7;
    row.delegates_total = 8;
    row
}
fn tasks(live: Vec<c::TaskLine>, ended: Vec<c::TaskLine>, more: bool) -> c::ReadResult {
    c::ReadResult::Value(c::ReadValue::Tasks(Box::new(c::TaskPage {
        ended_mark: 0,
        live: List::from_boxed(live.into_boxed_slice()),
        live_after: if more { Some(f::cursor()) } else { None },
        ended: c::EndedGroup::Rows {
            rows: List::from_boxed(ended.into_boxed_slice()),
            after: if more { Some(f::cursor()) } else { None },
            cut: false,
        },
    })))
}
#[test]
fn milestone_list_reads_the_offered_label_and_one_group_with_a_cursor() {
    for ended in [false, true] {
        for json in [false, true] {
            let mut offered = me();
            offered.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap().label = 40;
            let mut root = f::choice_root(offered);
            let mut argv = f::extended(&["milestone", "list", "--project", "1", "--after", "1122"], &[]);
            if ended {
                argv.push("--ended").unwrap();
            }
            let call = root.submit_prepared(f::call(argv.as_slice()), json, None);
            let c::Read::Tasks { query, live, ended: after } = &root.reads.get(&call).unwrap().1 else {
                panic!("query")
            };
            assert_eq!(
                (query.label, query.tracked, query.phase),
                (Some(40), Some(true), if ended { Some(c::PhaseClass::Ended) } else { None })
            );
            assert_eq!((live.is_some(), after.is_some()), (!ended, ended));
            let mut complete = row(790, b"Search", c::Phase::Ended);
            complete.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(f::document_text(
                b"Concluded: 5 done, 0 failed, 1 cancelled.",
            ))));
            let mut failed = row(791, b"Moves", c::Phase::Done);
            failed.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(f::document_text(
                b"Concluded: 4 done, 2 failed, 0 cancelled.",
            ))));
            root.serve(
                call,
                tasks(
                    vec![row(812, b"Store shape", c::Phase::Held)],
                    vec![complete, failed, row(792, b"Closed", c::Phase::Cancelled)],
                    true,
                ),
            );
            let answer = root.answer(call).unwrap();
            assert_eq!(answer.status, 0);
            let text = String::from_utf8(answer.stdout.to_vec()).unwrap();
            if json {
                assert!(
                    text.contains(if ended {
                        "\"state\":\"ended\""
                    } else {
                        "\"name\":\"Store shape\",\"state\":\"live\""
                    }),
                    "{text}"
                );
                assert!(text.contains("\"delegates_done\":7,\"delegates_total\":8"));
            } else if ended {
                assert!(text.starts_with("790  Search  7 / 8 done  complete\n791  Moves  7 / 8 done  ended\n792  Closed  7 / 8 done  closed\n"), "{text}");
            } else {
                assert!(text.starts_with("812  Store shape  7 / 8 done  live\n"), "{text}");
            }
            assert!(text.contains("milestone list --project 1"));
            assert!(text.contains("--after 1122"));
        }
    }
    assert_eq!(rows::MILESTONE_LIST.result, b"milestone_list");
}
fn totals(ended: bool) -> w::MilestoneResult {
    w::MilestoneResult::Totals(
        w::MilestoneTotals::new(
            &w::CEILINGS,
            w::MilestoneTotalsParts {
                task: 812,
                done: 7,
                failed: 0,
                closed: 1,
                live: u32::from(!ended),
                ended: if ended { Some(120_000_000_000) } else { None },
                last_landing: Some(
                    w::MilestoneLanding::new(
                        &w::CEILINGS,
                        w::MilestoneLandingParts { at: 90_000_000_000, goal: 820, change: 821 },
                    )
                    .unwrap(),
                ),
                spent: 212,
                reds: 0,
            },
        )
        .unwrap(),
    )
}
fn answer(result: w::MilestoneResult) -> jig_client::Readout {
    let answer = w::MilestoneAnswer::new(&w::CEILINGS, w::MilestoneAnswerParts { result }).unwrap();
    jig_client::Readout::Family { family: wire::FORGE, record: Some(wire::milestone_answer(&answer)) }
}
fn show(json: bool, task: c::Task, result: w::MilestoneResult) -> crate::jig::Printed {
    let mut root = f::choice_root(me());
    let call = root.submit_prepared(f::call(&["milestone", "show", "812"]), json, None);
    root.serve(call, f::task_result(task));
    let c::Read::Tasks { query, .. } = &root.reads.get(&call).unwrap().1 else { panic!("goals query") };
    assert_eq!((query.parent, query.order), (Some(812), c::QueryOrder::Number));
    root.serve(
        call,
        tasks(
            vec![row(820, b"S7 history in task logs", c::Phase::Running)],
            vec![row(813, b"S1 Draft the layout", c::Phase::Done), row(820, b"old duplicate", c::Phase::Done)],
            true,
        ),
    );
    let c::Read::Tasks { query, .. } = &root.reads.get(&call).unwrap().1 else { panic!("ended goals query") };
    assert_eq!((query.parent, query.phase), (Some(812), Some(c::PhaseClass::Ended)));
    root.serve(
        call,
        tasks(
            vec![],
            vec![row(813, b"S1 Draft the layout", c::Phase::Done), row(820, b"old duplicate", c::Phase::Done)],
            true,
        ),
    );
    let read = &root.family_reads.get(&call).unwrap().1;
    let expected = temper_client_domain_forge::forge::milestones::read(1, 812, &root.limits.domain.forge).unwrap();
    assert_eq!(*read, expected);
    root.serve_readout(call, answer(result));
    assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Project { project: 1 });
    root.serve(call, c::ReadResult::Value(c::ReadValue::Project(Box::new(super::projects::project()))));
    root.answer(call).expect("all milestone reads answered")
}
#[test]
fn milestone_show_joins_facts_totals_and_goals_in_task_order() {
    let printed = show(false, milestone(), totals(false));
    assert_eq!(printed.status, 0);
    assert_eq!(printed.stdout.as_ref(), b"milestone 812 \xc2\xb7 Store shape \xc2\xb7 whole since 1970-01-01T00:01Z\nThe store's new shape\nmade by person 1, Alice \xc2\xb7 1970-01-01T00:00Z\ngoals: 7 / 8 done \xc2\xb7 0 failed \xc2\xb7 1 closed \xc2\xb7 1 live\nspent: 2.12 of 4.00\nlast landing: 1970-01-01T00:01Z, S7 history in task logs (task 820)\ngoals:\n  813  S1 Draft the layout  done\n  820  S7 history in task logs  live\nread on with: task list --project 1 --after 1122 --parent 812\nread on with: task list --project 1 --ended --after 1122 --parent 812\n");
    let printed = show(true, milestone(), totals(false));
    assert_eq!(printed.status, 0);
    let text = String::from_utf8(printed.stdout.into_vec()).unwrap();
    assert!(text.starts_with("{\"facts\":{\"name\":\"Store shape\",\"words\":\"The store's new shape\""), "{text}");
    assert!(text.contains("\"totals\":{\"task\":812,\"done\":7,\"failed\":0,\"closed\":1,\"live\":1"));
    assert_eq!(text.matches("\"title\":{\"bytes\":\"S7 history in task logs\"").count(), 1);
    assert!(!text.contains("old duplicate"));
    assert!(text.contains("\"read_on\":[\"task list --project 1 --after 1122 --parent 812\""));
    assert_eq!(rows::MILESTONE_SHOW.result, b"milestone_show");
}
#[test]
fn milestone_show_prints_absent_totals_open_facts_and_an_ended_outcome() {
    let mut task = milestone();
    task.concluded = None;
    task.spec.specification.words = f::document_text(b"Store shape");
    let printed = show(false, task.clone(), w::MilestoneResult::NotMilestone);
    let text = String::from_utf8(printed.stdout.into_vec()).unwrap();
    assert!(text.starts_with("milestone 812 · Store shape · open\nmade by "), "{text}");
    assert!(text.contains("\ntotals: none\ngoals:\n"));
    let printed = show(true, task.clone(), w::MilestoneResult::NotMilestone);
    assert!(has(&printed.stdout, b"\"totals\":null"));
    task.chip.phase = c::Phase::Done;
    task.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(f::document_text(
        b"Concluded: 7 done, 1 failed, 0 cancelled.",
    ))));
    let printed = show(false, task, totals(true));
    assert!(has(&printed.stdout, b"ended: 1970-01-01T00:02Z, ended\n"));
    assert!(!has(&printed.stdout, b"last landing:"));
}

fn has(bytes: &[u8], text: &[u8]) -> bool {
    skein_lib::bytes::find(bytes, text).is_some()
}

#[test]
fn milestone_budget_argv_and_totals_share_the_amount() {
    for (amount, spend) in [("130", 13000), ("1.25", 125), ("0.5", 50), ("0", 0), ("130.00", 13000)] {
        let mut offered = me();
        let bounds = &mut offered.projects.get_mut(0).unwrap().creating.as_mut().unwrap().within;
        bounds.budget = 13000;
        bounds.maximum_budget = 13000;
        bounds.pool_left = 13000;
        bounds.authority.authority.spend = 13000;
        let mut root = f::choice_root(offered);
        let call = root.submit(&["milestone", "new", "--project", "1", "--budget", amount, "Work", "--key", "amount"]);
        assert_eq!(f::chat_task(&root, call).authority.spend, spend, "{amount}");
    }
    let mut task = milestone();
    task.budget.numbers.budget = 13000;
    let w::MilestoneResult::Totals(totals) = totals(false) else { panic!("totals") };
    let mut parts = totals.into_parts();
    parts.spent = 0;
    let result = w::MilestoneResult::Totals(w::MilestoneTotals::new(&w::CEILINGS, parts).unwrap());
    let printed = show(false, task, result);
    assert!(skein_lib::bytes::find(&printed.stdout, b"spent: 0.00 of 130.00").is_some());
}
