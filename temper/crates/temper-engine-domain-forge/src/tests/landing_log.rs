//! Landing history survives its changes (temper/docs/design/domain/after-landing.md, 10).
use super::*;

fn saved(out: &Queue<Request>) -> Box<[LandingLog]> {
    let mut rows = List::uncapped();
    for request in out {
        if let Request::Save { record: Stored::LandingLog(row), .. } = request {
            rows.push(row.clone()).expect("saved logs");
        }
    }
    rows.into_boxed()
}

#[test]
fn a_merge_logs_its_change_with_its_title() {
    for pull in [None, Some(PULL)] {
        let mut d = configured_domain(&[]);
        add_change(&mut d, CHANGE, b"main");
        d.changes.get_mut(&CHANGE).unwrap().pull = pull;
        let title = d.change(CHANGE).unwrap().title.clone();
        let out = merged(&mut d, CHANGE, FIRST);
        let expected = LandingLog {
            repository: REPO,
            landings: Box::new([LoggedLanding {
                commit: FIRST,
                branch: Box::from(b"main".as_slice()),
                at: env().wall,
                changes: Box::new([LoggedChange { task: CHANGE, pull, title }]),
            }]),
        };
        assert_eq!(saved(&out).as_ref(), core::slice::from_ref(&expected));
        assert_eq!(d.landing_log(REPO), Some(&expected));
        assert_eq!(d.change(CHANGE).unwrap().change.state, change::State::Landed { merge: FIRST, since: env().wall });
    }
}

#[test]
fn the_log_outlives_the_changes_close() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"main");
    let expected = saved(&merged(&mut d, CHANGE, FIRST))[0].clone();
    end_change(&mut d, &env(), CHANGE, &mut Queue::uncapped());
    d.changes.remove(&CHANGE);
    assert_eq!(d.landing_log(REPO), Some(&expected));
}

#[test]
fn a_pull_seen_merged_is_logged() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"main");
    let pull = skein_forge::api::Pull {
        number: PULL,
        head: Box::from(b"temper/51".as_slice()),
        commit: HEAD,
        base_commit: Some(FIRST),
        base: Box::from(b"main".as_slice()),
        state: skein_forge::api::State::Closed,
        merged: Some(FIRST),
        mergeable: true,
        reviewers: Box::new([]),
        ci: skein_forge::api::Ci::Passed,
        queued: None,
        dequeued: None,
        size: None,
    };
    let mut out = Queue::uncapped();
    let pending = PendingStep {
        failed: change_ci::Failed::new(Box::new([])),
        pull_ci: false,
        task: CHANGE,
        entry: OPEN,
        follow_tree: List::with_capacity(LIMITS.key_entries),
        stage: StepStage::Branch,
        heard: heard(false),
        gates: Box::new([]),
        queue_repair_active: false,
        kept: false,
        merge_job: false,
        pending: Box::new([]),
        branch: Some(HEAD),
        branch_fresh: false,
        base_tip: Some(FIRST),
        pull: Some(pull),
        ci: change::Status::Passed,
        base_ci: change::Status::Passed,
        contains_base: change::Status::Passed,
        contains_head: change::Status::Unknown,
        contains_catch_up: change::Status::Passed,
        reviews: Box::new([]),
        reviews_complete: true,
        paths: Box::new([]),
        paths_whole: false,
        woken: false,
    };
    // A fresh read of a merged pull reaches the same step for external and forge-queue merges.
    decide_step(&mut d, &env(), pending, &mut out);
    assert_eq!(d.change(CHANGE).unwrap().change.state, change::State::Landed { merge: FIRST, since: env().wall });
    assert_eq!(saved(&out).len(), 1);
    let row = d.landing_log(REPO).unwrap();
    assert_eq!(row.landings[0].commit, FIRST);
    assert_eq!(row.landings[0].changes[0].pull, Some(PULL));
}

#[test]
fn the_ring_keeps_the_last_landings_kept() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"main");
    for number in 1..=17 {
        let out = merged(&mut d, CHANGE, [number; 32]);
        assert_eq!(saved(&out).len(), 1);
    }
    let row = d.landing_log(REPO).unwrap();
    assert_eq!(row.landings.len(), 16);
    assert_eq!(row.landings[0].commit, [2; 32]);
    assert_eq!(row.landings[15].commit, [17; 32]);
}

#[test]
fn a_commit_logged_already_gains_the_change() {
    let mut d = configured_domain(&[]);
    for task in [CHANGE, SECOND, OLDER_REPAIR] {
        add_change(&mut d, task, b"main");
    }
    merged(&mut d, CHANGE, FIRST);
    let mut later = env();
    later.wall = Wall::from_nanos(env().wall.as_nanos() + 1);
    let mut out = Queue::uncapped();
    outcome_facts(
        &mut d,
        &later,
        OPEN,
        SECOND,
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged(FIRST), found: false },
        &mut out,
    );
    assert_eq!(saved(&out).len(), 1);
    assert!(saved(&merged(&mut d, SECOND, FIRST)).is_empty());
    assert!(saved(&merged(&mut d, OLDER_REPAIR, FIRST)).is_empty());
    let row = d.landing_log(REPO).unwrap();
    assert_eq!(row.landings.len(), 1);
    assert_eq!(row.landings[0].at, env().wall);
    assert_eq!(row.landings[0].changes.len(), 2);
    assert_eq!(row.landings[0].changes[0].task, CHANGE);
    assert_eq!(row.landings[0].changes[1].task, SECOND);
}

#[test]
fn a_long_title_is_fitted_at_a_character() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"main");
    d.changes.get_mut(&CHANGE).unwrap().title = "é".repeat(130).into_bytes().into_boxed_slice();
    merged(&mut d, CHANGE, FIRST);
    let title = &d.landing_log(REPO).unwrap().landings[0].changes[0].title;
    assert_eq!(title.get(..252), Some("é".repeat(126).as_bytes()));
    assert_eq!(title.get(252..), Some(b"...".as_slice()));
    assert!(title.len() <= LOGGED_TITLE_BYTES);
}

#[test]
fn its_repositorys_removal_erases_the_log() {
    for forget in [false, true] {
        let mut d = configured_domain(&[]);
        add_change(&mut d, CHANGE, b"main");
        merged(&mut d, CHANGE, FIRST);
        let mut out = Queue::uncapped();
        if forget {
            step(&mut d, &env(), Event::ForgetAdoption { repository: REPO, restore: None }, &mut out);
        } else {
            assert!(erase_removed_repositories(&mut d, REPO.forge, &mut out));
        }
        assert!(contains(&out, &Request::Erase { key: Key::Repository(REPO) }));
        assert!(contains(&out, &Request::Erase { key: Key::LandingLog { repository: REPO } }));
        assert!(d.landing_log(REPO).is_none());
    }
}

#[test]
fn a_merge_during_release_is_still_logged() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"main");
    d.releases.insert(CHANGE, release_row(CHANGE, CHANGE, ReleaseEnding::Cancelled, Some(PULL), true, false)).unwrap();
    assert_eq!(saved(&merged(&mut d, CHANGE, FIRST)).len(), 1);
    assert!(d.verifications.is_empty());
}
