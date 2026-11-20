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
                tiers: Box::new([]),
                commit: FIRST,
                branch: Box::from(b"main".as_slice()),
                at: env().wall,
                changes: Box::new([LoggedChange {
                    fixes: Box::new([]),
                    goal: None,
                    milestone: None,
                    person: None,
                    goal_title: None,
                    draft: None,
                    signed: Box::new([]),
                    task: CHANGE,
                    pull,
                    title,
                }]),
            }]),
        };
        assert_eq!(saved(&out).as_ref(), core::slice::from_ref(&expected));
        assert_eq!(d.landing_log(REPO), Some(&expected));
        assert_eq!(d.change(CHANGE).unwrap().change.state, change::State::Landed { merge: FIRST, since: env().wall });
    }
}

#[test]
fn body_finding_lines_are_exact_distinct_sorted_and_bounded() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"main");
    d.changes.get_mut(&CHANGE).unwrap().body = b"Fixes-finding: 12\nFixes-finding: 7\r\nFixes-finding: 12\n Fixes-finding: 1\nFixes-finding: +2\nFixes-finding: 3 extra\nFixes-finding: 4 \nFixes-finding: 4294967296\nFixes #5\nFixes-finding: \nFixes-finding:  6\nFixes-finding: 0007".as_slice().into();
    merged(&mut d, CHANGE, FIRST);
    assert_eq!(d.landing_log(REPO).unwrap().landings[0].changes[0].fixes.as_ref(), [7, 12]);
    // Distinct entries fill the eight slots; repeats do not consume one.
    let mut body = skein_lib::Buffer::with_capacity(512);
    for n in 1..=9 {
        for _ in 0_u8..2 {
            body.put(b"Fixes-finding: ").unwrap();
            body.put(skein_lib::Decimal::of(n).as_bytes()).unwrap();
            body.put(b"\n").unwrap();
        }
    }
    d.changes.get_mut(&CHANGE).unwrap().body = body.to_boxed();
    merged(&mut d, CHANGE, LAST);
    assert_eq!(d.landing_log(REPO).unwrap().landings[1].changes[0].fixes.as_ref(), [1, 2, 3, 4, 5, 6, 7, 8]);
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
    // A prior admitted merge may have failed before someone else merged the pull.
    signed(&mut d, CHANGE, SignOff { person: 7, head: HEAD, at: env().wall });
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
        progress: List::uncapped(),
        failed: change_ci::Failed::new(Box::new([])),
        stall: None,
        execution: None,
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
        base_read: Some(FIRST),
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
        retry_at: None,
        read: None,
    };
    // A fresh read of a merged pull reaches the same step for external and forge-queue merges.
    decide_step(&mut d, &env(), pending, &mut out);
    assert_eq!(d.change(CHANGE).unwrap().change.state, change::State::Landed { merge: FIRST, since: env().wall });
    assert_eq!(saved(&out).len(), 1);
    let row = d.landing_log(REPO).unwrap();
    assert_eq!(row.landings[0].commit, FIRST);
    assert_eq!(row.landings[0].changes[0].pull, Some(PULL));
    assert!(row.landings[0].changes[0].signed.is_empty());
}

#[test]
fn the_ring_keeps_the_last_landings_kept() {
    let mut d = configured_domain(&[(TIER, None)]);
    add_change(&mut d, CHANGE, b"main");
    landing_log::landed(&mut d, env().wall, [99; 32], CHANGE, Box::new([]), &mut Queue::uncapped());
    d.landing.projects.get_mut(&PROJECT).unwrap().1[0].tiers = Box::new([]);
    // No tier follows these landings, so each is judged as it is logged.
    for number in 1..=17 {
        let out = merged(&mut d, CHANGE, [number; 32]);
        assert_eq!(saved(&out).len(), 1);
    }
    let row = d.landing_log(REPO).unwrap();
    assert_eq!(row.landings.len(), 17, "the pending landing stays beside the last 16 judged");
    assert_eq!(row.landings[0].commit, [99; 32]);
    assert_eq!(row.landings[1].commit, [2; 32]);
    assert_eq!(row.landings[16].commit, [17; 32]);
    green(&mut d, 1, &[[99; 32]]);
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
            assert!(erase_removed_repositories(&mut d, &env(), REPO.forge, &mut out));
            release::first_page(&mut d, &env(), &mut out);
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

#[test]
fn a_default_branch_merge_keeps_only_a_newer_milestone_landing() {
    let mut d = configured_domain(&[]);
    let milestone =
        MilestoneRow { project: 5, task: 40, name: b"Store shape".as_slice().into(), last_landing: None, ended: None };
    step(&mut d, &env(), Event::Milestone { row: milestone }, &mut Queue::uncapped());
    add_change(&mut d, CHANGE, b"main");
    let row = d.changes.get_mut(&CHANGE).unwrap();
    row.goal = Some(42);
    row.milestone = Some(40);
    let out = merged(&mut d, CHANGE, FIRST);
    let expected = MilestoneLanding { at: env().wall, goal: 42, change: CHANGE };
    assert_eq!(d.milestone(40).unwrap().last_landing, Some(expected.clone()));
    let mut saved = false;
    for request in &out {
        if let Request::Save { record: Stored::Milestone(row), .. } = request {
            saved |= row.last_landing == Some(expected.clone());
        }
    }
    assert!(saved);
    add_change(&mut d, SECOND, b"main");
    let row = d.changes.get_mut(&SECOND).unwrap();
    row.goal = Some(43);
    row.milestone = Some(40);
    let mut older = env();
    older.wall = Wall::EPOCH;
    outcome_facts(
        &mut d,
        &older,
        OPEN,
        SECOND,
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged([9; 32]), found: false },
        &mut Queue::uncapped(),
    );
    assert_eq!(d.milestone(40).unwrap().last_landing, Some(expected));
}

fn signed(d: &mut Domain, task: u64, signed: SignOff) -> Queue<Request> {
    let mut out = Queue::uncapped();
    step(d, &env(), Event::Signed { task, signed: Box::new([signed]) }, &mut out);
    out
}

#[test]
fn a_new_admission_replaces_sign_offs_and_joins_an_existing_commit() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"main");
    add_change(&mut d, SECOND, b"main");
    merged(&mut d, CHANGE, FIRST);
    let first = SignOff { person: 7, head: HEAD, at: Wall::from_nanos(10) };
    let second = SignOff { person: 8, head: FIRST, at: Wall::from_nanos(11) };
    signed(&mut d, SECOND, first);
    let out = signed(&mut d, SECOND, second);
    let mut kept = false;
    for request in &out {
        if let Request::Save { record: Stored::ChangeHot(row), .. } = request {
            kept |= row.task == SECOND && row.signed.as_ref() == [second];
        }
    }
    assert!(kept, "replacement saved in the admission decision");
    let out = merged(&mut d, SECOND, FIRST);
    let log = saved(&out);
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].landings.len(), 1);
    assert_eq!(log[0].landings[0].changes[1].signed.as_ref(), &[second]);
}

#[test]
fn a_merge_logs_its_goal_and_milestone() {
    for (goal, milestone) in [(Some(42), Some(40)), (None, None)] {
        for already_logged in [false, true] {
            let mut d = configured_domain(&[]);
            if already_logged {
                add_change(&mut d, SECOND, b"main");
                merged(&mut d, SECOND, FIRST);
            }
            add_change(&mut d, CHANGE, b"main");
            let row = d.changes.get_mut(&CHANGE).unwrap();
            row.goal = goal;
            row.milestone = milestone;
            if goal.is_some() {
                row.person = Some(7);
                row.goal_title = Some(Box::from(b"R2 Reds blamed".as_slice()));
                row.draft = Some(Box::from(b"temper/docs/design/draft/after-landing-repairs.md".as_slice()));
            }
            let expected = row.clone();
            let out = merged(&mut d, CHANGE, FIRST);
            let log = d.landing_log(REPO).unwrap();
            assert_eq!(saved(&out).as_ref(), core::slice::from_ref(log));
            assert_eq!(log.landings.len(), 1);
            let member = log.landings[0].changes.last().unwrap();
            assert_eq!((member.goal, member.milestone), (goal, milestone));
            assert_eq!(member.person, expected.person);
            assert_eq!(member.goal_title, expected.goal_title);
            assert_eq!(member.draft, expected.draft);
        }
    }
}

#[test]
fn goal_words_are_counted_in_change_and_log_storage() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"main");
    let row = d.change(CHANGE).unwrap().clone();
    let (cold, _) = row.clone().split();
    let mut with_words = row.clone();
    with_words.goal_title = Some(Box::from(b"title".as_slice()));
    with_words.draft = Some(Box::from(b"path".as_slice()));
    let (cold_with_words, _) = with_words.clone().split();
    let payload = 9;
    for (before, after) in [
        (Stored::Change(Box::new(row)), Stored::Change(Box::new(with_words))),
        (Stored::ChangeCold(Box::new(cold)), Stored::ChangeCold(Box::new(cold_with_words))),
    ] {
        assert_eq!(stored_bytes(&after), Some(stored_bytes(&before).unwrap() + payload));
    }
    merged(&mut d, CHANGE, FIRST);
    let log = d.landing_log(REPO).unwrap().clone();
    let mut with_words = log.clone();
    with_words.landings[0].changes[0].goal_title = Some(Box::from(b"title".as_slice()));
    with_words.landings[0].changes[0].draft = Some(Box::from(b"path".as_slice()));
    assert_eq!(
        stored_bytes(&Stored::LandingLog(with_words)),
        Some(stored_bytes(&Stored::LandingLog(log)).unwrap() + payload)
    );
}

#[test]
fn only_tiers_following_the_branch_are_logged_and_an_added_member_keeps_them() {
    for follows in [Follows::Landing, Follows::Branch(b"main".as_slice().into())] {
        let mut d = configured_domain(&[(TIER, None), (b"goals", None)]);
        let tiers = &mut d.landing.projects.get_mut(&PROJECT).unwrap().1[0].tiers;
        tiers[0].follows = follows;
        tiers[1].follows = Follows::Goals;
        for (task, branch, commit) in [(CHANGE, b"main".as_slice(), FIRST), (SECOND, b"goal", LAST)] {
            add_change(&mut d, task, branch);
            merged(&mut d, task, commit);
        }
        let entries = Box::new([LoggedTier { tier: TIER.into(), verdict: LoggedVerdict::Verifying }]);
        assert_eq!(d.landing_log(REPO).unwrap().landings[0].tiers.as_ref(), entries.as_ref());
        assert!(d.landing_log(REPO).unwrap().landings[1].tiers.is_empty());
        d.landing.projects.get_mut(&PROJECT).unwrap().1[0].tiers[0].follows =
            Follows::Branch(b"other".as_slice().into());
        add_change(&mut d, OLDER_REPAIR, b"main");
        merged(&mut d, OLDER_REPAIR, FIRST);
        let row = d.landing_log(REPO).unwrap();
        assert_eq!(row.landings[0].tiers.as_ref(), entries.as_ref());
        assert_eq!(row.landings[0].changes.len(), 2);
        merged(&mut d, CHANGE, [99; 32]);
        assert!(d.landing_log(REPO).unwrap().landings[2].tiers.is_empty(), "another branch adds no entry");
    }
}

#[test]
fn a_full_landing_replaces_only_judged_entries_of_removed_tiers() {
    for (still_named, verifying) in [(false, false), (true, false), (false, true)] {
        let names: &[(&[u8], Option<&[u8]>)] =
            if still_named { &[(b"a", None), (b"b", None)] } else { &[(b"b", None), (b"c", None)] };
        let mut d = configured_domain(names);
        d.landing.limits.tiers.tiers = 2;
        add_change(&mut d, CHANGE, b"main");
        merged(&mut d, CHANGE, FIRST);
        let entries = Box::new([
            LoggedTier {
                tier: b"a".as_slice().into(),
                verdict: if verifying {
                    LoggedVerdict::Verifying
                } else {
                    LoggedVerdict::Red { run: 3, failing: None, tests: Box::new([]), many: None }
                },
            },
            LoggedTier { tier: b"b".as_slice().into(), verdict: LoggedVerdict::Cleared { run: 3 } },
        ]);
        d.landing_logs.get_mut(&REPO).unwrap().landings[0].tiers = entries.clone();
        let verdict = policy::Verdict {
            number: 7,
            tip: FIRST,
            key: FIRST,
            failing: policy::Failures::Tests(Box::new([])),
            landings: Box::new([policy::Blame::Cleared]),
            flakes: Box::new([]),
        };
        let mut out = Queue::uncapped();
        landing_log::judged(
            &mut d,
            REPO,
            b"c",
            &verdict,
            &[policy::Landing { commit: FIRST, key: FIRST }],
            None,
            &mut out,
        );
        let log = d.landing_log(REPO).unwrap();
        if still_named || verifying {
            assert_eq!(log.landings[0].tiers.as_ref(), entries.as_ref());
            assert!(saved(&out).is_empty());
        } else {
            assert_eq!(
                log.landings[0].tiers.as_ref(),
                &[
                    entries[1].clone(),
                    LoggedTier { tier: b"c".as_slice().into(), verdict: LoggedVerdict::Cleared { run: 7 } },
                ]
            );
            assert_eq!(saved(&out).as_ref(), core::slice::from_ref(log));
        }
    }
}

#[test]
fn a_restore_retires_judged_removed_tiers_to_the_entry_bound() {
    let mut d = configured_domain(&[(b"b", None), (b"c", None)]);
    d.landing.limits.tiers.tiers = 2;
    add_change(&mut d, CHANGE, b"main");
    merged(&mut d, CHANGE, FIRST);
    let expected = Box::new([
        LoggedTier { tier: b"b".as_slice().into(), verdict: LoggedVerdict::Verifying },
        LoggedTier { tier: b"c".as_slice().into(), verdict: LoggedVerdict::Cleared { run: 3 } },
    ]);
    d.landing_logs.get_mut(&REPO).unwrap().landings[0].tiers = Box::new([
        LoggedTier { tier: b"a".as_slice().into(), verdict: LoggedVerdict::Cleared { run: 3 } },
        expected[0].clone(),
        expected[1].clone(),
    ]);
    let mut out = Queue::uncapped();
    landing_log::restored(&mut d, &mut out);
    let log = d.landing_log(REPO).unwrap();
    assert_eq!(log.landings[0].tiers.as_ref(), expected.as_ref());
    assert_eq!(saved(&out).as_ref(), core::slice::from_ref(log));
}

#[test]
fn a_restore_infers_verifying_from_ranges_and_drops_removed_tiers() {
    for (removed, posted) in [(false, false), (true, false), (false, true)] {
        let mut d = configured_domain(&[(TIER, None)]);
        for (task, commit) in [(CHANGE, FIRST), (SECOND, LAST), (OLDER_REPAIR, REPAIR)] {
            add_change(&mut d, task, b"main");
            merged(&mut d, task, commit);
        }
        let mut log = d.landing_log(REPO).unwrap().clone();
        if !removed {
            for landing in &mut log.landings {
                landing.tiers = Box::new([]);
            }
        }
        if posted {
            green(&mut d, 1, &[FIRST]);
        } else {
            tier_run(&mut d, TIER, 1, &[FIRST]);
        }
        d.tiers.get_mut(&(REPO, TIER.into())).unwrap().pending = Box::new([TierLanding { commit: LAST, key: None }]);
        let tier = d.tiers.get(&(REPO, TIER.into())).unwrap().clone();
        let mut restored = configured_domain(&[(TIER, None)]);
        restored.restore_begin(Wall::EPOCH);
        step(&mut restored, &env(), Event::Restore { record: Stored::Tier(tier) }, &mut Queue::uncapped());
        step(&mut restored, &env(), Event::Restore { record: Stored::LandingLog(log) }, &mut Queue::uncapped());
        if removed {
            restored.landing.projects.get_mut(&PROJECT).unwrap().1[0].tiers = Box::new([]);
        }
        let mut out = Queue::uncapped();
        step(&mut restored, &env(), Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }, &mut out);
        let row = restored.landing_log(REPO).unwrap();
        for landing in &row.landings[..2] {
            if removed || (posted && landing.commit == FIRST) {
                assert!(landing.tiers.is_empty(), "a verdict awaiting its posts is already judged");
            } else {
                assert_eq!(
                    landing.tiers.as_ref(),
                    &[LoggedTier { tier: TIER.into(), verdict: LoggedVerdict::Verifying }]
                );
            }
        }
        assert!(row.landings[2].tiers.is_empty(), "a tier no longer holding this commit leaves it judged");
        assert_eq!(saved(&out).as_ref(), core::slice::from_ref(row));
        let out = judge(
            &mut restored,
            TIER,
            1,
            &[FIRST],
            Box::new([policy::Blame::Cleared]),
            policy::Failures::Tests(Box::new([])),
        );
        let row = restored.landing_log(REPO).unwrap();
        assert_eq!(
            row.landings[0].tiers.as_ref(),
            &[LoggedTier { tier: TIER.into(), verdict: LoggedVerdict::Cleared { run: 1 } }],
            "a removed tier's in-flight run still judges"
        );
        assert_eq!(saved(&out).as_ref(), core::slice::from_ref(row));
    }
}
