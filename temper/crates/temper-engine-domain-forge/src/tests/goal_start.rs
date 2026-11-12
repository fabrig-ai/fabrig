//! A goal's design writer (temper/docs/design/domain/forge/branches.md, 3).
use super::*;

const GOAL: u64 = 7;
const G: &[u8] = b"temper/7/g7";
const R: &[u8] = b"temper/7/r7";
const S: &[u8] = b"temper/7/s7";
const TIP: skein_forge::api::Commit = [4; 32];
const RUN_TIP: skein_forge::api::Commit = [5; 32];

fn name(branch: &[u8]) -> Name {
    Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([
            b"temper".as_slice().into(),
            b"7".as_slice().into(),
            branch.strip_prefix(b"temper/7/").expect("the fixture's goal root").into(),
        ])),
    }
}

fn held_goal() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                design_writers: Box::new([]),
                name: name(G),
                task: GOAL,
                writer: None,
                drift: None,
            }),
        },
    );
    head(&mut d, G, TIP);
    d
}

fn head(d: &mut Domain, branch: &[u8], commit: skein_forge::api::Commit) {
    outputs(d, Event::Restore { record: Stored::BranchHead(BranchHead { name: name(branch), commit, owned: true }) });
}

fn selection(d: &Domain, modify: bool, saved: bool) -> Option<items::Item> {
    d.workspace_item(items::Seed {
        project: 5,
        modify,
        task: GOAL,
        parent: None,
        repository: REPO,
        run_branch: R.into(),
        saved_branch: if saved { Some(S.into()) } else { None },
    })
}

fn item(d: &Domain, modify: bool, saved: bool) -> items::Item {
    selection(d, modify, saved).expect("goal workspace")
}

fn assert_selection(
    item: &items::Item,
    start: items::Start,
    push: Option<&[u8]>,
    landing: Option<&[u8]>,
    delivers: bool,
) {
    assert_eq!(item.start, start);
    assert_eq!(item.push.as_deref(), push);
    assert_eq!(item.landing.as_deref(), landing);
    assert_eq!(item.delivers, delivers);
    assert_eq!(item.holder, None, "the goal holds its own design branch");
}

#[test]
fn a_modifying_goal_starts_at_g_and_pushes_design_to_g() {
    assert_selection(&item(&held_goal(), true, false), items::Start::Base(G.into()), Some(G), Some(G), true);
}

#[test]
fn a_modifying_goals_later_run_merges_r_into_g_and_pushes_g() {
    let mut d = held_goal();
    head(&mut d, R, RUN_TIP);
    assert_selection(
        &item(&d, true, false),
        items::Start::Merge { branch: G.into(), base: RUN_TIP },
        Some(G),
        Some(G),
        true,
    );
}

#[test]
fn a_modifying_goal_resumes_saved_work_and_pushes_design_to_g() {
    let mut d = held_goal();
    head(&mut d, R, RUN_TIP);
    assert_selection(&item(&d, true, true), items::Start::Saved(S.into()), Some(G), Some(G), true);
}

#[test]
fn a_goal_without_modify_inspects_its_confirmed_branch() {
    let mut d = held_goal();
    head(&mut d, R, RUN_TIP);
    assert_selection(&item(&d, false, false), items::Start::Branch(G.into()), None, None, false);
}

#[test]
fn a_modifying_goal_waits_for_its_drifted_branch_to_be_released() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    outputs(&mut d, Event::Hold { entry: 0, task: GOAL, resource: name(G), from: None });
    fixtures::answer_next(&mut d, Ok(skein_forge::api::Answer::Commit(TIP)));
    d.holds.get_mut(&name(G)).expect("held goal").drift =
        Some(Drift { at: env().wall, change: DriftChange::Branch { expected: TIP, observed: Some(RUN_TIP) } });
    assert!(selection(&d, true, false).is_none(), "a drifted goal must not fall back to the base");
    outputs(&mut d, Event::GoalReleased { task: GOAL });
    assert!(selection(&d, true, false).is_none(), "the release reads g before starting its writer");
    fixtures::answer_next(&mut d, Ok(skein_forge::api::Answer::Commit(RUN_TIP)));
    assert_eq!(d.hold(&name(G)).expect("released goal hold").drift, None);
    assert_eq!(d.branch_head(&name(G)), Some(RUN_TIP));
    assert_selection(&item(&d, true, false), items::Start::Base(G.into()), Some(G), Some(G), true);
}

#[test]
fn a_modifying_goal_waits_until_its_branch_is_confirmed_owned() {
    let mut d = held_goal();
    d.heads.remove(&name(G));
    assert!(selection(&d, true, false).is_none(), "an unconfirmed goal must not fall back to the base");
    outputs(
        &mut d,
        Event::Restore { record: Stored::BranchHead(BranchHead { name: name(G), commit: TIP, owned: false }) },
    );
    assert!(selection(&d, true, false).is_none(), "an observed head is not an owned head");
    head(&mut d, G, TIP);
    assert_selection(&item(&d, true, false), items::Start::Base(G.into()), Some(G), Some(G), true);
}

#[test]
fn a_modifying_goal_waits_while_its_branch_is_being_made() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    head(&mut d, G, TIP);
    outputs(&mut d, Event::Hold { entry: 0, task: GOAL, resource: name(G), from: None });
    assert!(selection(&d, true, false).is_none(), "the taking's read must finish before the workspace");
    fixtures::answer_next(&mut d, Ok(skein_forge::api::Answer::Commit(TIP)));
    assert_selection(&item(&d, true, false), items::Start::Base(G.into()), Some(G), Some(G), true);
}

#[test]
fn a_task_without_a_goal_hold_keeps_its_default_start_without_delivery() {
    let mut d = held_goal();
    d.holds.remove(&name(G));
    assert_selection(&item(&d, true, false), items::Start::Base(b"main".as_slice().into()), Some(R), None, false);
}
