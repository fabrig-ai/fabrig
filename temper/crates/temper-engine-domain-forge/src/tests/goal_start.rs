//! A goal writes through its run branch (temper/docs/design/domain/forge.md, 11.1).
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
        Event::Restore { record: Stored::Hold(Hold { name: name(G), task: GOAL, writer: None, drift: None }) },
    );
    head(&mut d, G, TIP);
    d
}

fn head(d: &mut Domain, branch: &[u8], commit: skein_forge::api::Commit) {
    outputs(d, Event::Restore { record: Stored::BranchHead(BranchHead { name: name(branch), commit, owned: true }) });
}

fn item(d: &Domain, modify: bool, saved: bool) -> items::Item {
    d.workspace_item(items::Seed {
        modify,
        task: GOAL,
        parent: None,
        repository: REPO,
        run_branch: R.into(),
        saved_branch: if saved { Some(S.into()) } else { None },
    })
    .expect("goal workspace")
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
    assert_eq!(item.holder, None, "the run takes its own writer slot");
}

#[test]
fn a_modifying_goal_starts_at_g_and_pushes_to_its_new_run_branch() {
    assert_selection(&item(&held_goal(), true, false), items::Start::Base(G.into()), Some(R), None, true);
}

#[test]
fn a_modifying_goals_later_run_merges_r_into_g_and_pushes_r() {
    let mut d = held_goal();
    head(&mut d, R, RUN_TIP);
    assert_selection(
        &item(&d, true, false),
        items::Start::Merge { branch: G.into(), base: RUN_TIP },
        Some(R),
        Some(G),
        true,
    );
}

#[test]
fn a_modifying_goal_resumes_saved_work_without_a_landing_comparison() {
    let mut d = held_goal();
    head(&mut d, R, RUN_TIP);
    assert_selection(&item(&d, true, true), items::Start::Saved(S.into()), Some(R), None, true);
}

#[test]
fn a_goal_without_modify_inspects_its_confirmed_branch() {
    let mut d = held_goal();
    head(&mut d, R, RUN_TIP);
    assert_selection(&item(&d, false, false), items::Start::Branch(G.into()), None, None, false);
}

#[test]
fn a_goal_with_a_drifted_branch_starts_from_the_default_without_delivery() {
    let mut d = held_goal();
    d.holds.get_mut(&name(G)).expect("held goal").drift =
        Some(Drift { at: env().wall, change: DriftChange::Branch { expected: TIP, observed: Some(RUN_TIP) } });
    assert_selection(&item(&d, true, false), items::Start::Base(b"main".as_slice().into()), Some(R), None, false);
}

#[test]
fn a_goal_with_an_unconfirmed_branch_starts_from_the_default_without_delivery() {
    let mut d = held_goal();
    d.heads.remove(&name(G));
    assert_selection(&item(&d, true, false), items::Start::Base(b"main".as_slice().into()), Some(R), None, false);
}

#[test]
fn a_task_without_a_goal_hold_keeps_its_default_start_without_delivery() {
    let mut d = held_goal();
    d.holds.remove(&name(G));
    assert_selection(&item(&d, true, false), items::Start::Base(b"main".as_slice().into()), Some(R), None, false);
}
