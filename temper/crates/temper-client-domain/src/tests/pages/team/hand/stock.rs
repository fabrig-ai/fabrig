//! Stock starts follow the settings answer, never undoing it (client/pages/team.md, 1).
use super::*;
use hand::Stock;

const REV: u32 = 2;
fn stock_fixture(owner: bool, offered: bool) -> Reads {
    let mut f = fixture(owner, &[(b"tests", owner_role())]);
    let mut me = reader();
    let project = me.projects.get_mut(0).unwrap();
    project.name = text(b"vsr");
    project.role = if owner { c::HeldRole::Owner } else { c::HeldRole::Member };
    let template = project.templates.get_mut(0).unwrap();
    template.number = 7;
    template.executor = c::TemplateExecutor::Chosen;
    template.budget = c::TemplateBudget::Given;
    template.flags = c::TaskFlags {
        ended_by: c::EndedBy::Standing,
        conversation: false,
        listed: true,
        retention: c::Retention::Kept,
    };
    template.label = 3;
    template.grace = None;
    let creating = project.creating.as_mut().unwrap();
    creating.within.charters = List::from_boxed(if offered { Box::new([15, 16]) } else { Box::new([]) });
    creating.within.budget = 100;
    creating.within.maximum_budget = 100;
    creating.within.pool_left = 100;
    creating.within.authority.authority.spend = 100;
    if !owner {
        creating.within.budget = 1;
        creating.beyond = c::Beyond::Proposed { to: c::Recipient::Party(YOU) };
    }
    let watch = f.me;
    crate::tests::pages::welcome::deliver(&mut f.root, watch, c::Delivery::Me(Box::new(me)));
    let settings = f.settings;
    crate::tests::pages::welcome::deliver(
        &mut f.root,
        settings,
        c::Delivery::Settings(Box::new(c::SettingsDocument {
            newest: REV,
            settings: List::with_capacity(0),
            held: List::with_capacity(0),
            pending: List::with_capacity(0),
        })),
    );
    act(&mut f, Act::Open);
    f
}
fn settings(f: &mut Reads, stock: Stock) -> (jig_client::Key, c::Ask) {
    act(f, Act::Stock(stock));
    act(f, Act::Budget(b"40".as_slice().into()));
    let out = act(f, Act::Send);
    let read = reading(&out, REV);
    let out = deliver_record(f, read, REV, Some(record(&[(b"tests", owner_role())])));
    let sent = saves(&out);
    assert_eq!(sent.len(), 1, "settings alone precede the start");
    sent[0].clone()
}
fn settings_done(owner: bool) -> c::Answer {
    c::Answer::Done(if owner {
        c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 3 })
    } else {
        c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 9, to: 1 })
    })
}
fn expected_toast(owner: bool, name: &[u8]) -> needs::Toasted {
    if owner { needs::Toasted::AreaHanded { title: name.into(), revision: 3 } } else { needs::Toasted::AreasProposed }
}
#[test]
fn stock_budgets_are_positive_whole_units_judged_only_on_press() {
    for stock in [Stock::Prover, Stock::Porter] {
        let mut f = stock_fixture(true, true);
        act(&mut f, Act::Stock(stock));
        let open = f.root.pages.hand_area().unwrap();
        assert_eq!((open.role, open.stock), (None, Some(stock)));
        assert!(!open.judged);
        for budget in [
            b"".as_slice(),
            b"0",
            b"-1",
            b"+1",
            b"1.5",
            b" 40",
            b"40x",
            b"18446744073709551616",
            b"000000000000000000000001x",
        ] {
            act(&mut f, Act::Budget(budget.into()));
            let out = act(&mut f, Act::Send);
            assert!(out.is_empty(), "invalid budget sends neither a read nor a request");
            let open = f.root.pages.hand_area().unwrap();
            assert!(open.judged && !open.busy());
            assert_eq!(open.budget_value(), None);
        }
        for (budget, expected) in [(b"1".as_slice(), 1), (b"18446744073709551615", u64::MAX)] {
            act(&mut f, Act::Budget(budget.into()));
            assert_eq!(f.root.pages.hand_area().unwrap().budget_value(), Some(expected));
        }
        act(&mut f, Act::Role(Some(52)));
        assert_eq!(f.root.pages.hand_area().unwrap().stock, None);
        act(&mut f, Act::Stock(stock));
        act(&mut f, Act::Role(None));
        assert_eq!(f.root.pages.hand_area().unwrap().stock, None);
    }
}
#[test]
fn stock_handoffs_save_or_propose_before_starting_template_seven() {
    for (stock, number, name, words) in [
        (Stock::Prover, 15, b"Prover".as_slice(), b"Prover\n\nOwns the area tests on vsr.".as_slice()),
        (Stock::Porter, 16, b"Porter", b"Porter\n\nOwns the area tests on vsr."),
    ] {
        for owner in [true, false] {
            let mut f = stock_fixture(owner, true);
            let (key, ask) = settings(&mut f, stock);
            let items = if owner {
                let c::Ask::ChangeSettings(c::ChangeSettings { against, items, running }) = &ask else {
                    panic!("saves")
                };
                assert_eq!((*against, running), (REV, &c::Running::Stays));
                items
            } else {
                let c::Ask::Propose { project, action: c::ProposeAction::Settings(settings), reason } = &ask else {
                    panic!("proposes settings")
                };
                assert_eq!((*project, settings.against), (0, REV));
                assert_eq!(reason.bytes(), skein_lib::bytes::concat(&[b"Hand tests to ", name]).as_ref());
                &settings.items
            };
            assert_eq!(item(items), record(&[(b"tests", charter(number))]));
            assert!(f.root.pages.needs().toast().is_none());
            assert!(act(&mut f, Act::Send).is_empty(), "a second press while pending starts nothing");
            let out = answer(&mut f, key, settings_done(owner));
            let sent = saves(&out);
            assert_eq!(sent.len(), 1);
            let (key, ask) = &sent[0];
            let tasks = if owner {
                let c::Ask::Create { project, tasks } = ask else { panic!("creates role") };
                assert_eq!(*project, PROJECT);
                tasks
            } else {
                let c::Ask::Propose { project, action: c::ProposeAction::Batch { tasks }, .. } = ask else {
                    panic!("proposes role")
                };
                assert_eq!(*project, PROJECT);
                tasks
            };
            assert_eq!(tasks.len(), 1);
            let task = tasks.get(0).unwrap();
            assert_eq!(
                (task.executor, task.authority.spend, task.label),
                (c::Executor::Agent { charter: number }, 40, 3)
            );
            assert_eq!(task.flags.ended_by, c::EndedBy::Standing);
            assert_eq!(task.spec.words.bytes(), words);
            assert_eq!((task.authority.deadline, task.tracked, &task.llm), (None, None, &None));
            assert!(f.root.pages.hand_area().unwrap().busy());
            assert!(f.root.pages.needs().toast().is_none());
            let done = if owner {
                c::DocumentOutcome::Created { first: 60, last: 60, given: None }
            } else {
                c::DocumentOutcome::Proposed { proposal: 10, to: 1 }
            };
            let out = answer(&mut f, *key, c::Answer::Done(c::Outcome::Document(done)));
            assert!(saves(&out).is_empty());
            assert!(f.root.pages.hand_area().is_none());
            assert_eq!(f.root.pages.needs().toast(), Some(&expected_toast(owner, name)));
        }
    }
}
#[test]
fn a_refused_stock_start_keeps_the_settings_and_sheet_without_rollback() {
    for offered in [true, false] {
        let mut f = stock_fixture(true, offered);
        let (key, _) = settings(&mut f, Stock::Prover);
        let out = answer(&mut f, key, settings_done(true));
        let refusal = if offered {
            let sent = saves(&out);
            assert_eq!(sent.len(), 1);
            let refusal = c::Refusal::Judged {
                at: 0,
                setting: c::Setting::Charter(c::SettingNumber { number: 15 }),
                why: c::Text::new(b"The prover cannot start here.".as_slice().into(), 128).unwrap(),
            };
            let out = answer(&mut f, sent[0].0, c::Answer::Refused { refusal: refusal.clone(), at: None });
            assert!(out.is_empty(), "start refusal sends no rollback");
            refusal
        } else {
            assert!(saves(&out).is_empty(), "a local refusal sends no start or rollback");
            c::Refusal::NotOffered
        };
        let open = f.root.pages.hand_area().unwrap();
        assert!(!open.busy() && open.locked());
        let Some(hand::Failure::Outcome(Outcome::Answered(Answered::Jig(c::Answer::Refused { refusal: why, .. })))) =
            &open.failure
        else {
            panic!("refusal stays in sheet")
        };
        assert_eq!(why, &refusal);
        assert!(f.root.pages.needs().toast().is_none());
        act(&mut f, Act::Role(Some(52)));
        assert_eq!(f.root.pages.hand_area().unwrap().stock, Some(Stock::Prover));
        act(&mut f, Act::Budget(b"0".as_slice().into()));
        assert!(act(&mut f, Act::Send).is_empty(), "a retry also judges its budget");
        act(&mut f, Act::Budget(b"30".as_slice().into()));
        let out = act(&mut f, Act::Send);
        if offered {
            let sent = saves(&out);
            assert_eq!(sent.len(), 1);
            let c::Ask::Create { tasks, .. } = &sent[0].1 else { panic!("retry starts only the role, not settings") };
            assert_eq!(tasks.get(0).unwrap().authority.spend, 30);
        } else {
            assert!(out.is_empty());
        }
    }
}
