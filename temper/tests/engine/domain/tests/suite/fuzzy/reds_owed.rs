//! Random records and real triage keep the root's derived reads honest (red-findings.md, 11).
use crate::findings as f;
use jig_domain_people as people;
use skein_lib::{ReplyTo, Rng, Token, Wall};
use temper_engine_domain::{self as engine, Application, Delivery, Record};
use temper_engine_domain_forge::{self as forge, findings::FindingsFilter};
use temper_engine_domain_world::direct::Driver;

const MILESTONES: [u64; 2] = [812, 813];
const GOALS: [u64; 3] = [820, 821, 822];

fn records(driver: &Driver) -> Vec<forge::Finding> {
    driver
        .store
        .rows
        .values()
        .filter_map(|record| {
            if let Record::Forge { row } = record
                && let forge::Stored::Finding(row) = row.as_ref()
            {
                Some(row.clone())
            } else {
                None
            }
        })
        .collect()
}

fn blame(rng: &mut Rng) -> Option<forge::FindingBlame> {
    if rng.below(4) == 0 {
        return None;
    }
    Some(forge::FindingBlame {
        strength: if rng.below(2) == 0 { forge::BlameStrength::Owned } else { forge::BlameStrength::Suspected },
        landings: (0..rng.below(3))
            .map(|n| forge::BlamedLanding {
                commit: [u8::try_from(n).expect("three landings"); 32],
                goals: (0..rng.below(4))
                    .map(|_| forge::BlamedGoal {
                        goal: GOALS[usize::try_from(rng.below(3)).expect("three goals")],
                        milestone: match rng.below(3) {
                            0 => None,
                            1 => Some(812),
                            _ => Some(813),
                        },
                        person: None,
                    })
                    .collect(),
            })
            .collect(),
    })
}

fn totals(driver: &mut Driver, milestone: u64) -> forge::MilestoneTotals {
    let before = driver.delivered.len();
    driver.send(engine::Event::Party(engine::PartyInput::ForgeRead {
        reply: ReplyTo::new(Token::new(850)),
        sign_in: driver.session(),
        scope: people::ApplicationScope::Project(1),
        query: engine::ForgeRead::Milestone { project: 1, task: milestone },
    }));
    driver.settle();
    driver.delivered[before..]
        .iter()
        .find_map(|delivery| {
            if let Delivery::ForgeRead { answer: engine::ForgeReadAnswer::Milestone(Some(totals)), .. } = delivery {
                Some(totals.clone())
            } else {
                None
            }
        })
        .expect("milestone read")
}

fn check(driver: &mut Driver, seed: u64, step: u64) -> Vec<(u64, u32)> {
    let saved = records(driver);
    let mut counted = Vec::new();
    for milestone in MILESTONES {
        let owed = saved
            .iter()
            .filter(|row| {
                row.project == 1
                    && row.kind == forge::FindingKind::Red
                    && row.cleared.is_none()
                    && row.blame.as_ref().is_some_and(|blame| {
                        blame
                            .landings
                            .iter()
                            .any(|landing| landing.goals.iter().any(|goal| goal.milestone == Some(milestone)))
                    })
            })
            .count();
        let got = totals(driver, milestone);
        assert_eq!(
            usize::try_from(got.reds).expect("reds count"),
            owed,
            "seed {seed}, step {step}, milestone {milestone}"
        );
        counted.push((milestone, got.reds));
    }
    for filter in GOALS.map(FindingsFilter::Goal).into_iter().chain(MILESTONES.map(FindingsFilter::Milestone)) {
        let expected: Vec<_> = saved
            .iter()
            .rev()
            .filter(|row| {
                row.project == 1
                    && row.blame.as_ref().is_some_and(|blame| {
                        blame.landings.iter().any(|landing| {
                            landing.goals.iter().any(|goal| match filter {
                                FindingsFilter::Goal(number) => goal.goal == number,
                                FindingsFilter::Milestone(number) => goal.milestone == Some(number),
                                FindingsFilter::Triage { .. } => unreachable!("blamed filters"),
                            })
                        })
                    })
            })
            .map(|row| row.number)
            .collect();
        let document = engine::findings_document(&driver.root, 1, 1, filter);
        assert!(!document.cut);
        assert_eq!(
            document.rows.iter().map(|row| row.number).collect::<Vec<_>>(),
            expected,
            "seed {seed}, step {step}, {filter:?}"
        );
    }
    counted
}

fn reopen(driver: &mut Driver, run: &engine::Assignment) {
    // The client retains its valid session when the server reopens.
    let index = driver
        .delivered
        .iter()
        .position(|delivery| matches!(delivery, Delivery::WebReply { reply: people::Reply::SignedIn { .. }, .. }))
        .expect("client session");
    let signed_in = driver.delivered.remove(index);
    driver.reopen(f::configuration());
    driver.send(temper_engine_domain_world::protocol::hello(
        Token::new(7),
        Box::new([temper_engine_domain_world::protocol::hosting(
            run.task,
            run.attempt,
            jig_domain_link::Phase::Active,
        )]),
    ));
    driver.settle();
    driver.delivered.push(signed_in);
}

fn run(seed: u64) -> (Vec<forge::Finding>, Vec<Vec<(u64, u32)>>) {
    let (mut driver, run, repair, _) = super::findings::scenario();
    let milestone_rows: Vec<_> = MILESTONES
        .into_iter()
        .map(|task| Record::Forge {
            row: Box::new(forge::Stored::Milestone(forge::MilestoneRow {
                project: 1,
                task,
                name: b"Complete milestone".as_slice().into(),
                last_landing: None,
                ended: Some(forge::MilestoneEnd {
                    at: Wall::EPOCH,
                    outcome: forge::MilestoneOutcome::Complete,
                    done: 1,
                    failed: 0,
                    closed: 0,
                    spent: 0,
                }),
            })),
        })
        .collect();
    driver.store.seed(milestone_rows.into_boxed_slice(), Box::new([]));
    reopen(&mut driver, &run);
    let mut rng = Rng::new(seed);
    let mut history = Vec::new();
    for step in 0..80 {
        let saved = records(&driver);
        let act = if saved.is_empty() { 0 } else { rng.below(5) };
        if act == 0 {
            let mut file = f::file(None);
            file.kind = if rng.below(4) == 0 { forge::FindingKind::Flake } else { forge::FindingKind::Red };
            f::application(&mut driver, Application::FileFinding(file));
        } else {
            let mut row = saved[usize::try_from(rng.below(u64::try_from(saved.len()).expect("finding count")))
                .expect("finding index")]
            .clone();
            match act {
                1 | 2 => {
                    // Blame moves and clearings enter as stored facts; the root must derive afresh after reopening.
                    if act == 1 {
                        row.blame = blame(&mut rng);
                    } else {
                        row.cleared = if rng.below(2) == 0 {
                            None
                        } else {
                            Some(forge::FindingCleared { run: step, at: driver.env.wall })
                        };
                    }
                    driver
                        .store
                        .seed(Box::new([Record::Forge { row: Box::new(forge::Stored::Finding(row)) }]), Box::new([]));
                    reopen(&mut driver, &run);
                }
                3 | 4 => {
                    f::application(
                        &mut driver,
                        Application::TriageFinding(forge::TriageFinding {
                            project: 1,
                            number: row.number,
                            act: match rng.below(3) {
                                0 => forge::Triage::Route(repair),
                                1 => forge::Triage::Backlog,
                                _ => forge::Triage::Drop(b"drawn drop".as_slice().into()),
                            },
                        }),
                    );
                }
                _ => unreachable!("five acts"),
            }
        }
        history.push(check(&mut driver, seed, step));
    }
    (records(&driver), history)
}

#[test]
fn every_milestones_reds_and_filtered_findings_follow_random_blames_clearings_and_triage() {
    for seed in [0x0bed_8121, 0x0bed_8132] {
        assert_eq!(run(seed), run(seed), "seed {seed}");
    }
}
