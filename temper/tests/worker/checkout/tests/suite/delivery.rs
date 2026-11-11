//! Delivery ancestry through git's sequence and the shared fake's object graph.
use skein_lib::Token;
use temper_fake_checkout::git::{self as local, Remote};
use temper_worker_checkout_world::{
    LIMITS,
    hosted::{self, Scenario},
    translate,
};
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git::{
    invocation::Command,
    sequence::{self, Next},
};

fn deliver(scenario: &mut Scenario, unchecked: bool, saved: Option<u64>) -> git::Done {
    let mut op = git::Op::Commit {
        always: saved.is_some(),
        at: git::Place { workspace: Token::new(1), repository: b"specimen".as_slice().into(), git: None },
        parent: translate::commit(saved.unwrap_or(scenario.base)),
        merging: None,
        title: if unchecked {
            b"Unchecked: work in progress, saved before a question".as_slice().into()
        } else {
            b"Delivery".as_slice().into()
        },
        body: Box::default(),
        identity: 1,
        snapshot: Some(git::Snapshot { remote: scenario.remote.clone().into(), account: None }),
    };
    let mut next = sequence::first(&op);
    for _ in 0..16 {
        next = match next {
            Next::Run(command) => {
                let answer = hosted::apply(scenario, &command);
                sequence::after(&op, &command, answer.exit, &answer.stdout, &answer.stderr, 0, &LIMITS)
            }
            Next::Snapshot { parent, merging } => {
                let git::Op::Commit { parent: target, merging: second, always, .. } = &mut op else { unreachable!() };
                if *target != parent {
                    *always = false;
                }
                *target = parent;
                *second = merging;
                Next::Run(if merging.is_some() { Command::Unmerged } else { Command::Add })
            }
            Next::End(done) => return done,
            Next::Make | Next::Store | Next::InstallStore { .. } | Next::Alternates | Next::Scan { .. } => {
                panic!("these resolutions are staged")
            }
        };
    }
    panic!("delivery settles within its sequence");
}

#[test]
fn a_staged_merge_delivers_both_parents_and_a_committed_merge_keeps_its_id() {
    for unchecked in [false, true] {
        for committed in [false, true] {
            let mut scenario = hosted::scenario();
            let base = scenario.base;
            let topic = scenario.topic;
            let outcome = hosted::apply(&mut scenario, &Command::Merge { theirs: translate::commit(topic) });
            assert_eq!(outcome.exit, 0);
            hosted::apply(&mut scenario, &Command::Add);
            let existing = if committed {
                Some(
                    local::commit_merging(
                        &mut scenario.transport,
                        &mut scenario.checkout,
                        &scenario.at,
                        base,
                        topic,
                        b"Producer's merge",
                    )
                    .expect("staged merge"),
                )
            } else {
                None
            };
            let git::Done::Committed { commit } = deliver(&mut scenario, unchecked, None) else {
                panic!("delivered merge")
            };
            let delivered = translate::fake(commit);
            assert_eq!(scenario.transport.parent(delivered), Some(base));
            assert_eq!(scenario.transport.merge_parent(delivered), Some(topic));
            if let Some(existing) = existing {
                assert_eq!(delivered, existing);
                assert_eq!(scenario.transport.message(delivered), b"Producer's merge");
            }
            assert_eq!(scenario.checkout.content(&[scenario.at.as_slice(), b"/.git/MERGE_HEAD"].concat()), None);
        }
    }
}

#[test]
fn a_producers_local_commit_is_kept_when_its_tree_is_delivered() {
    for unchecked in [false, true] {
        for after_save in [false, true] {
            let mut scenario = hosted::scenario();
            let place = git::Place { workspace: Token::new(1), repository: b"specimen".as_slice().into(), git: None };
            let at = translate::path(&place);
            for (path, content) in scenario.checkout.tree(&scenario.at) {
                scenario.checkout.write(&[at.as_slice(), b"/", &path].concat(), &content);
            }
            scenario.at = at;
            let saved = if after_save {
                scenario.checkout.write(&[scenario.at.as_slice(), b"/saved"].concat(), b"work in progress");
                Some(
                    local::commit(
                        &mut scenario.transport,
                        &mut scenario.checkout,
                        &scenario.at,
                        scenario.base,
                        b"Unchecked: work in progress, saved before a question",
                    )
                    .expect("local parent")
                    .expect("saved work"),
                )
            } else {
                None
            };
            scenario.checkout.write(&[scenario.at.as_slice(), b"/new"].concat(), b"checked work");
            let existing = local::commit(
                &mut scenario.transport,
                &mut scenario.checkout,
                &scenario.at,
                saved.unwrap_or(scenario.base),
                b"Producer's commit",
            )
            .expect("local parent")
            .expect("new file");
            assert_eq!(
                deliver(&mut scenario, unchecked, saved),
                git::Done::Committed { commit: translate::commit(existing) }
            );
            assert_eq!(scenario.transport.message(existing), b"Producer's commit");
            let op = git::Op::Commit {
                always: saved.is_some(),
                at: git::Place { workspace: Token::new(1), repository: b"specimen".as_slice().into(), git: None },
                parent: translate::commit(saved.unwrap_or(scenario.base)),
                merging: None,
                title: b"Delivery".as_slice().into(),
                body: Box::default(),
                identity: 1,
                snapshot: Some(git::Snapshot { remote: scenario.remote.clone().into(), account: None }),
            };
            assert_eq!(
                translate::perform(&mut scenario.transport, &mut scenario.checkout, op),
                git::Done::Committed { commit: translate::commit(existing) }
            );
        }
    }
}

#[test]
fn an_unpushed_second_parent_refuses_a_staged_or_committed_merge() {
    for unchecked in [false, true] {
        for committed in [false, true] {
            let mut scenario = hosted::scenario();
            scenario.checkout.write(&[scenario.at.as_slice(), b"/local"].concat(), b"local only");
            let second = local::commit(
                &mut scenario.transport,
                &mut scenario.checkout,
                &scenario.at,
                scenario.base,
                b"Unpushed",
            )
            .expect("local parent")
            .expect("new file");
            let base = scenario.base;
            hosted::apply(&mut scenario, &Command::CheckOut { commit: translate::commit(base) });
            hosted::apply(&mut scenario, &Command::Merge { theirs: translate::commit(second) });
            hosted::apply(&mut scenario, &Command::Add);
            if committed {
                local::commit_merging(
                    &mut scenario.transport,
                    &mut scenario.checkout,
                    &scenario.at,
                    base,
                    second,
                    b"Local merge",
                )
                .expect("staged merge");
            }
            let git::Done::FailedWithOutput { fault, diagnostic } = deliver(&mut scenario, unchecked, None) else {
                panic!("refused merge")
            };
            assert_eq!(fault, git::Fault::Refused);
            assert_eq!(diagnostic.output(), b"delivery second parent is not reachable from the branch's remote");
        }
    }
}

#[test]
fn a_design_delivery_fast_forwards_g() {
    temper_worker_checkout_world::merges::design_fast_forward();
}

#[test]
fn a_design_delivery_merges_gs_moved_tip_before_pushing() {
    temper_worker_checkout_world::merges::design_clean_move();
}

#[test]
fn a_design_delivery_leaves_a_conflict_to_settle_and_deliver_again() {
    temper_worker_checkout_world::merges::design_conflicting_move();
}

#[test]
fn a_design_delivery_refuses_non_markdown_additions_and_deletions() {
    temper_worker_checkout_world::merges::design_markdown_only();
}

#[test]
fn a_design_delivery_retries_a_leased_push_up_to_three_times() {
    temper_worker_checkout_world::merges::design_moving_again(2);
}

#[test]
fn a_design_delivery_refuses_a_goal_that_keeps_moving_without_ending_stale() {
    temper_worker_checkout_world::merges::design_moving_again(3);
}

#[test]
fn a_design_repository_refuses_unchecked_save_but_keeps_saved_work() {
    temper_worker_checkout_world::merges::design_no_unchecked_save();
}

#[test]
fn a_changed_tree_lists_the_deleted_non_markdown_name_even_when_renamed_to_markdown() {
    let mut scenario = hosted::scenario();
    let tree = scenario.transport.tree(scenario.base);
    let (deleted, content) = tree.iter().next().expect("base has files");
    scenario.checkout.remove(&[scenario.at.as_slice(), b"/", deleted].concat());
    scenario.checkout.write(&[scenario.at.as_slice(), b"/renamed.md"].concat(), content);
    let git::Done::Committed { commit } = deliver(&mut scenario, false, None) else { panic!("checked tree committed") };
    let command = Command::DiffPaths { before: translate::commit(scenario.base), after: commit };
    let answer = hosted::apply(&mut scenario, &command);
    assert_eq!(answer.exit, 0);
    let expected = [deleted.as_slice(), b"\0renamed.md\0"].concat();
    assert_eq!(answer.stdout, expected, "no rename detection: deletion is listed in git order");
}
